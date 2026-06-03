//! Chromium automation via `chromiumoxide` (pure Rust CDP, no Node Playwright).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::Page;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::info;

use super::config::{resolve_chrome_executable, BrowserSettings};
use super::snapshot::{format_snapshot, SNAPSHOT_JS};

struct LiveSession {
    #[allow(dead_code)]
    browser: Browser,
    page: Page,
    _handler: JoinHandle<()>,
}

pub struct BrowserService {
    settings: BrowserSettings,
    session: Mutex<Option<LiveSession>>,
}

impl BrowserService {
    pub fn new(settings: BrowserSettings) -> Arc<Self> {
        Arc::new(Self {
            settings,
            session: Mutex::new(None),
        })
    }

    async fn ensure(&self) -> Result<(), String> {
        let mut guard = self.session.lock().await;
        if guard.is_some() {
            return Ok(());
        }
        *guard = Some(launch_session(&self.settings).await?);
        info!("Browser session started (chromiumoxide)");
        Ok(())
    }

    async fn page(&self) -> Result<Page, String> {
        self.ensure().await?;
        let guard = self.session.lock().await;
        guard
            .as_ref()
            .map(|s| s.page.clone())
            .ok_or_else(|| "Browser session not available".to_string())
    }

    pub async fn navigate(&self, url: &str, _timeout_ms: u64) -> Result<String, String> {
        let page = self.page().await?;
        page.goto(url)
            .await
            .map_err(|e| format!("Navigation failed: {e}"))?;
        let _ =
            tokio::time::timeout(Duration::from_millis(8_000), page.wait_for_navigation()).await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let title = page_title(&page).await;
        let current = page_url(&page).await?;
        let snapshot = self.snapshot_text(&page).await?;

        Ok(format!(
            "Navigated to: {current}\nTitle: {title}\n\n--- Page Snapshot ---\n{snapshot}"
        ))
    }

    pub async fn snapshot(&self) -> Result<String, String> {
        let page = self.page().await?;
        self.snapshot_text(&page).await
    }

    async fn snapshot_text(&self, page: &Page) -> Result<String, String> {
        let eval = page
            .evaluate(SNAPSHOT_JS)
            .await
            .map_err(|e| format!("Snapshot error: {e}"))?;
        let value: Value = eval.into_value().map_err(|e| e.to_string())?;
        let tree = value.get("tree").cloned().unwrap_or(Value::Null);
        let ref_count = value.get("refCount").and_then(|v| v.as_u64()).unwrap_or(0);
        let title = page_title(page).await;
        let url = page_url(page).await?;
        Ok(format_snapshot(
            &tree,
            ref_count,
            &title,
            &url,
            self.settings.snapshot_max_chars,
        ))
    }

    pub async fn click(
        &self,
        ref_id: Option<u64>,
        selector: Option<&str>,
        _timeout_ms: u64,
    ) -> Result<String, String> {
        let page = self.page().await?;
        if let Some(r) = ref_id {
            let script = format!(
                r#"() => {{
                    const el = window.__agentRefMap && window.__agentRefMap[{r}];
                    if (!el) return {{ error: "ref {r} not found. Run snapshot first." }};
                    el.click();
                    return {{ clicked: true }};
                }}"#
            );
            let v = eval_object(&page, &script).await?;
            if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
                return Err(err.to_string());
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
            return Ok("Clicked successfully. Use 'snapshot' to see updated page.".into());
        }
        if let Some(sel) = selector.filter(|s| !s.is_empty()) {
            page.find_element(sel)
                .await
                .map_err(|e| format!("Click failed: {e}"))?
                .click()
                .await
                .map_err(|e| format!("Click failed: {e}"))?;
            return Ok("Clicked successfully. Use 'snapshot' to see updated page.".into());
        }
        Err("Provide either ref (from snapshot) or selector".into())
    }

    pub async fn fill(
        &self,
        text: &str,
        ref_id: Option<u64>,
        selector: Option<&str>,
        _timeout_ms: u64,
    ) -> Result<String, String> {
        let page = self.page().await?;
        if let Some(r) = ref_id {
            let text_json = serde_json::to_string(text).map_err(|e| e.to_string())?;
            let script = format!(
                r#"() => {{
                    const el = window.__agentRefMap && window.__agentRefMap[{r}];
                    if (!el) return {{ error: "ref {r} not found. Run snapshot first." }};
                    el.focus();
                    if ("value" in el) el.value = {text_json};
                    el.dispatchEvent(new Event("input", {{ bubbles: true }}));
                    return {{ filled: true }};
                }}"#
            );
            let v = eval_object(&page, &script).await?;
            if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
                return Err(err.to_string());
            }
            return Ok("Filled text into element. Use 'snapshot' to verify.".into());
        }
        if let Some(sel) = selector.filter(|s| !s.is_empty()) {
            page.find_element(sel)
                .await
                .map_err(|e| format!("Fill failed: {e}"))?
                .click()
                .await
                .map_err(|e| format!("Fill failed: {e}"))?
                .type_str(text)
                .await
                .map_err(|e| format!("Fill failed: {e}"))?;
            return Ok("Filled text into element. Use 'snapshot' to verify.".into());
        }
        Err("Provide either ref (from snapshot) or selector".into())
    }

    pub async fn screenshot(&self, full_page: bool, cwd: &PathBuf) -> Result<String, String> {
        let page = self.page().await?;
        let dir = cwd.join("tmp");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("screenshot_{}.png", uuid::Uuid::new_v4().simple()));
        let bytes = page
            .screenshot(ScreenshotParams::builder().full_page(full_page).build())
            .await
            .map_err(|e| format!("Screenshot failed: {e}"))?;
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        Ok(format!("Screenshot saved to: {}", path.display()))
    }

    pub async fn scroll(&self, direction: &str, amount: i32) -> Result<String, String> {
        let page = self.page().await?;
        let (dx, dy) = match direction {
            "up" => (0, -amount),
            "right" => (amount, 0),
            "left" => (-amount, 0),
            _ => (0, amount),
        };
        page.evaluate(format!(
            "() => {{ window.scrollBy({dx}, {dy}); return {{ scrollY: window.scrollY }}; }}"
        ))
        .await
        .map_err(|e| format!("Scroll failed: {e}"))?;
        Ok(format!("Scrolled {direction}."))
    }

    pub async fn wait(&self, selector: Option<&str>, timeout_ms: u64) -> Result<String, String> {
        let page = self.page().await?;
        if let Some(sel) = selector.filter(|s| !s.is_empty()) {
            page.find_element(sel)
                .await
                .map_err(|e| format!("Wait failed: {e}"))?;
            return Ok("Wait completed.".into());
        }
        tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
        Ok("Wait completed.".into())
    }

    pub async fn back(&self) -> Result<String, String> {
        let page = self.page().await?;
        page.evaluate("() => { history.back(); }")
            .await
            .map_err(|e| format!("Go back failed: {e}"))?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let url = page_url(&page).await?;
        Ok(format!("Navigated back to: {url}"))
    }

    pub async fn forward(&self) -> Result<String, String> {
        let page = self.page().await?;
        page.evaluate("() => { history.forward(); }")
            .await
            .map_err(|e| format!("Go forward failed: {e}"))?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let url = page_url(&page).await?;
        Ok(format!("Navigated forward to: {url}"))
    }

    pub async fn get_text(&self, selector: &str) -> Result<String, String> {
        let page = self.page().await?;
        let text = page
            .find_element(selector)
            .await
            .map_err(|e| format!("Get text failed: {e}"))?
            .inner_text()
            .await
            .map_err(|e| format!("Get text failed: {e}"))?
            .unwrap_or_default();
        Ok(text)
    }

    pub async fn press(&self, key: &str) -> Result<String, String> {
        let page = self.page().await?;
        let key_json = serde_json::to_string(key).map_err(|e| e.to_string())?;
        let script = format!(
            r#"() => {{
                const k = {key_json};
                const opts = {{ key: k, code: k, bubbles: true }};
                document.dispatchEvent(new KeyboardEvent("keydown", opts));
                document.dispatchEvent(new KeyboardEvent("keyup", opts));
            }}"#
        );
        page.evaluate(script)
            .await
            .map_err(|e| format!("Press failed: {e}"))?;
        Ok(format!("Pressed key: {key}"))
    }

    pub async fn evaluate(&self, script: &str) -> Result<String, String> {
        let page = self.page().await?;
        let wrapped = if script.trim_start().starts_with('(')
            || script.trim_start().starts_with("function")
        {
            script.to_string()
        } else {
            format!("() => {{ {script} }}")
        };
        let eval = page
            .evaluate(wrapped)
            .await
            .map_err(|e| format!("Evaluate failed: {e}"))?;
        let value: Value = eval.into_value().map_err(|e| e.to_string())?;
        Ok(serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()))
    }
}

async fn page_url(page: &Page) -> Result<String, String> {
    Ok(page
        .url()
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_default())
}

async fn page_title(page: &Page) -> String {
    page.get_title().await.ok().flatten().unwrap_or_default()
}

async fn eval_object(page: &Page, script: &str) -> Result<Value, String> {
    let eval = page.evaluate(script).await.map_err(|e| e.to_string())?;
    eval.into_value().map_err(|e| e.to_string())
}

async fn launch_session(settings: &BrowserSettings) -> Result<LiveSession, String> {
    let (browser, mut handler) = if !settings.cdp_endpoint.is_empty() {
        Browser::connect(&settings.cdp_endpoint)
            .await
            .map_err(|e| format!("CDP connect failed: {e}"))?
    } else {
        let chrome = resolve_chrome_executable(settings.chrome_executable.as_deref())?;
        tracing::info!(path = %chrome.display(), headless = settings.headless, "Launching system browser");

        let mut builder = BrowserConfig::builder()
            .chrome_executable(&chrome)
            .new_headless_mode()
            .args(vec!["--disable-dev-shm-usage".to_string()]);

        if !settings.headless {
            builder = builder.with_head();
        }

        if settings.persistent {
            std::fs::create_dir_all(&settings.user_data_dir).map_err(|e| e.to_string())?;
            builder = builder.user_data_dir(settings.user_data_dir.clone());
        }

        let config = builder.build().map_err(|e| e.to_string())?;
        Browser::launch(config).await.map_err(|e| {
            format!(
                "Browser launch failed: {e}. Ensure Chrome/Edge is installed or set tools.browser.chrome_executable."
            )
        })?
    };

    let handler_task = tokio::spawn(async move {
        while let Some(ev) = handler.next().await {
            if ev.is_err() {
                break;
            }
        }
    });

    let page = browser
        .new_page("about:blank")
        .await
        .map_err(|e| format!("Failed to open page: {e}"))?;

    Ok(LiveSession {
        browser,
        page,
        _handler: handler_task,
    })
}
