//! DOM snapshot for the browser tool (ported from SupportFlow Agent `browser_service.py`).

use serde_json::Value;

/// In-page snapshot script; returns `{ tree, refCount }` and sets `window.__agentRefMap`.
pub const SNAPSHOT_JS: &str = r#"(() => {
    const KEEP = new Set(["a","button","input","textarea","select","option","label","details","summary","h1","h2","h3","h4","h5","h6","p","li","td","th","caption","figcaption","blockquote","pre","code","nav","main","article","section","header","footer","form","table","img","video","audio"]);
    const INTERACTIVE = new Set(["a","button","input","textarea","select","option","label","details","summary"]);
    const SKIP = new Set(["script","style","noscript","svg","path","meta","link","br","hr"]);
    const CLICKABLE_ROLES = new Set(["button","link","tab","menuitem","menuitemcheckbox","menuitemradio","option","switch","checkbox","radio","combobox","searchbox","slider","spinbutton","textbox","treeitem"]);
    let refCounter = 0;
    const refMap = {};
    function visible(el) {
        if (!(el instanceof HTMLElement)) return true;
        const st = window.getComputedStyle(el);
        if (st.display === "none" || st.visibility === "hidden") return false;
        if (parseFloat(st.opacity) === 0) return false;
        return true;
    }
    function hasStrongInteractiveSignal(el) {
        const role = el.getAttribute("role");
        if (role && CLICKABLE_ROLES.has(role)) return true;
        if (el.hasAttribute("onclick") || el.hasAttribute("tabindex")) return true;
        if (el.hasAttribute("data-click") || el.hasAttribute("data-action")) return true;
        if (el.getAttribute("contenteditable") === "true") return true;
        return false;
    }
    function hasOwnPointerCursor(el) {
        try {
            const st = window.getComputedStyle(el);
            if (st.cursor !== "pointer") return false;
            const parent = el.parentElement;
            if (parent && window.getComputedStyle(parent).cursor === "pointer") return false;
            return true;
        } catch(e) { return false; }
    }
    function hasTextOrContent(el) {
        const t = el.textContent || "";
        if (t.trim().length > 0) return true;
        if (el.querySelector("img,video,audio,canvas")) return true;
        if ((el.getAttribute("aria-label") || "").trim()) return true;
        if ((el.getAttribute("title") || "").trim()) return true;
        return false;
    }
    function isImplicitInteractive(el) {
        return hasStrongInteractiveSignal(el) || (hasOwnPointerCursor(el) && hasTextOrContent(el));
    }
    function getTextContent(el) {
        let text = "";
        for (const ch of el.childNodes) {
            if (ch.nodeType === Node.TEXT_NODE) text += ch.textContent;
        }
        return text.trim();
    }
    function walk(node) {
        if (node.nodeType === Node.TEXT_NODE) {
            const t = node.textContent.trim();
            return t ? t : null;
        }
        if (node.nodeType !== Node.ELEMENT_NODE) return null;
        const tag = node.tagName.toLowerCase();
        if (SKIP.has(tag) || !visible(node)) return null;
        const children = [];
        for (const ch of node.childNodes) {
            const r = walk(ch);
            if (r !== null) {
                if (typeof r === "string") children.push(r);
                else children.push(r);
            }
        }
        const nativeInteractive = INTERACTIVE.has(tag);
        const implicitInteractive = !nativeInteractive && (node instanceof HTMLElement) && isImplicitInteractive(node);
        const keep = KEEP.has(tag) || implicitInteractive;
        if (!keep) {
            if (children.length === 0) return null;
            if (children.length === 1) return children[0];
            return children;
        }
        const obj = { tag };
        if (nativeInteractive || implicitInteractive) {
            refCounter++;
            obj.ref = refCounter;
            refMap[refCounter] = node;
        }
        if (tag === "a" && node.href) obj.href = node.getAttribute("href");
        if (tag === "img") { obj.alt = node.alt || ""; obj.src = node.getAttribute("src") || ""; }
        if (tag === "input" || tag === "textarea" || tag === "select") {
            obj.type = node.type || "text";
            if (node.name) obj.name = node.name;
            if (node.value) obj.value = node.value;
            if (node.placeholder) obj.placeholder = node.placeholder;
            if (node.disabled) obj.disabled = true;
            if (tag === "input" && node.type === "checkbox") obj.checked = node.checked;
        }
        if (tag === "button" && node.disabled) obj.disabled = true;
        if (tag === "option") { obj.value = node.value; if (node.selected) obj.selected = true; }
        const role = node.getAttribute("role");
        if (role) obj.role = role;
        const ariaLabel = node.getAttribute("aria-label");
        if (ariaLabel) obj.ariaLabel = ariaLabel;
        if (children.length === 1 && typeof children[0] === "string") obj.text = children[0];
        else if (children.length > 0) obj.children = children;
        return obj;
    }
    const tree = walk(document.body);
    window.__agentRefMap = refMap;
    return { tree, refCount: refCounter };
})()"#;

pub fn format_snapshot(tree: &Value, ref_count: u64, title: &str, url: &str, max_chars: usize) -> String {
    let lines = flatten_tree(tree, 0);
    let body = lines.join("\n");
    let body = if body.len() > max_chars {
        format!("{}\n... [snapshot truncated]", &body[..max_chars])
    } else {
        body
    };
    format!(
        "Page: {title}  ({url})\nInteractive elements: {ref_count}\n---\n{body}"
    )
}

pub fn flatten_tree(node: &Value, indent: usize) -> Vec<String> {
    match node {
        Value::Null => vec![],
        Value::String(s) => vec![format!("{}{}", " ".repeat(indent), s)],
        Value::Array(arr) => arr
            .iter()
            .flat_map(|c| flatten_tree(c, indent))
            .collect(),
        Value::Object(obj) => {
            let tag = obj.get("tag").and_then(|v| v.as_str()).unwrap_or("?");
            let mut parts = vec![if let Some(r) = obj.get("ref").and_then(|v| v.as_u64()) {
                format!("[{r}] {tag}")
            } else {
                tag.to_string()
            }];
            for attr in [
                "type", "name", "href", "alt", "role", "ariaLabel", "placeholder", "value",
            ] {
                if let Some(val) = obj.get(attr).and_then(|v| v.as_str()) {
                    let s = if val.len() > 80 {
                        format!("{}...", &val[..77])
                    } else {
                        val.to_string()
                    };
                    parts.push(format!("{attr}=\"{s}\""));
                }
            }
            for flag in ["disabled", "checked", "selected"] {
                if obj.get(flag).and_then(|v| v.as_bool()) == Some(true) {
                    parts.push(flag.into());
                }
            }
            let mut lines = vec![format!("{}{}", " ".repeat(indent), parts.join(" "))];
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                let t = if text.len() > 120 {
                    format!("{}...", &text[..117])
                } else {
                    text.to_string()
                };
                if let Some(last) = lines.last_mut() {
                    *last = format!("{last}: {t}");
                }
            }
            if let Some(children) = obj.get("children") {
                for child in children.as_array().into_iter().flatten() {
                    lines.extend(flatten_tree(child, indent + 2));
                }
            }
            lines
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flatten_simple_tree() {
        let tree = json!({"tag": "button", "ref": 1, "text": "OK"});
        let lines = flatten_tree(&tree, 0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("[1] button"));
    }
}
