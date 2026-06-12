//! Modal 子窗口管理：支持预热、打开、定位与隐藏复用。

use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};

use crate::events::{self, payloads::ModalOpenPanelPayload, MAIN_WINDOW_LABEL};

/// 唯一 modal 子窗口 label。
pub const MODAL_WINDOW_LABEL: &str = "modal";

const MODAL_PRELOAD_PATH: &str = "/modal-window";
const MODAL_DEFAULT_WIDTH: f64 = 720.0;
const MODAL_DEFAULT_HEIGHT: f64 = 640.0;

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn resolve_modal_url(app: &AppHandle, path: &str) -> Result<WebviewUrl, String> {
    let path = normalize_path(path);

    #[cfg(debug_assertions)]
    {
        let dev_url = app
            .config()
            .build
            .dev_url
            .as_ref()
            .ok_or_else(|| "build.devUrl is missing in tauri.conf.json".to_string())?;
        let base = dev_url.as_str().trim_end_matches('/');
        let full = format!("{base}{path}");
        full.parse::<url::Url>()
            .map(WebviewUrl::External)
            .map_err(|e| format!("invalid modal url `{full}`: {e}"))
    }

    #[cfg(not(debug_assertions))]
    {
        Ok(WebviewUrl::App(path.into()))
    }
}

fn panel_name_from_path(path: &str) -> String {
    let normalized = normalize_path(path);
    let fake = format!("http://local{normalized}");
    let Ok(parsed) = url::Url::parse(&fake) else {
        return String::new();
    };
    for (key, value) in parsed.query_pairs() {
        if key == "name" {
            return value.into_owned();
        }
    }
    String::new()
}

fn register_modal_destroy_listener(app: &AppHandle, label: &str) {
    let Some(window) = app.get_webview_window(label) else {
        return;
    };
    let app_handle = app.clone();
    let label_owned = label.to_string();
    window.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            let _ = events::modal_closed(&app_handle, label_owned.clone());
        }
    });
}

fn emit_open_panel(
    app: &AppHandle,
    label: &str,
    panel_name: String,
    title: Option<String>,
) -> Result<(), String> {
    app.emit_to(
        label,
        events::names::MODAL_OPEN_PANEL,
        ModalOpenPanelPayload {
            name: panel_name,
            title,
        },
    )
    .map_err(|e| e.to_string())
}

fn center_modal_on_parent(
    app: &AppHandle,
    modal: &tauri::WebviewWindow,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let parent = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| format!("{MAIN_WINDOW_LABEL} window not found"))?;

    let scale = parent.scale_factor().map_err(|e| e.to_string())?;
    let modal_size = LogicalSize::new(width, height).to_physical(scale);
    let parent_pos = parent.outer_position().map_err(|e| e.to_string())?;
    let parent_size = parent.outer_size().map_err(|e| e.to_string())?;

    let offset_x = (parent_size.width.saturating_sub(modal_size.width) / 2) as i32;
    let offset_y = (parent_size.height.saturating_sub(modal_size.height) / 2) as i32;
    let x = parent_pos.x + offset_x;
    let y = parent_pos.y + offset_y;

    modal
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn apply_modal_geometry(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    window.set_title(title).map_err(|e| e.to_string())?;
    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    center_modal_on_parent(app, window, width, height)?;
    Ok(())
}

fn create_modal_webview(
    app: &AppHandle,
    path: &str,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let label = MODAL_WINDOW_LABEL;

    if app.get_webview_window(label).is_some() {
        return Ok(());
    }

    let parent = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| format!("{MAIN_WINDOW_LABEL} window not found"))?;

    let url = resolve_modal_url(app, path)?;

    WebviewWindowBuilder::new(app, label, url)
        .title(title)
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .shadow(false)
        .visible(false)
        .parent(&parent)
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(modal) = app.get_webview_window(label) {
        center_modal_on_parent(app, &modal, width, height)?;
    }

    register_modal_destroy_listener(app, label);
    Ok(())
}

/// 预热 modal 窗口，避免首次打开卡顿。
pub fn preload_modal_window(app: &AppHandle) -> Result<(), String> {
    create_modal_webview(
        app,
        MODAL_PRELOAD_PATH,
        "Modal",
        MODAL_DEFAULT_WIDTH,
        MODAL_DEFAULT_HEIGHT,
    )
}

/// 打开 modal 窗口并切换到指定面板。
pub fn open_modal_window(
    app: &AppHandle,
    path: String,
    title: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    _label: Option<String>,
) -> Result<String, String> {
    let label = MODAL_WINDOW_LABEL.to_string();
    let panel_name = panel_name_from_path(&path);
    let width = width.unwrap_or(MODAL_DEFAULT_WIDTH);
    let height = height.unwrap_or(MODAL_DEFAULT_HEIGHT);
    let title = title.unwrap_or_else(|| "Modal".to_string());

    if app.get_webview_window(&label).is_none() {
        create_modal_webview(app, &path, &title, width, height)?;
    }

    let existing = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("modal window not found: {label}"))?;

    let was_visible = existing.is_visible().map_err(|e| e.to_string())?;

    apply_modal_geometry(app, &existing, &title, width, height)?;

    if !was_visible {
        events::modal_opened(app, &label)?;
    }

    emit_open_panel(app, &label, panel_name, Some(title.clone()))?;

    if was_visible {
        existing.set_focus().map_err(|e| e.to_string())?;
        let _ = events::modal_opened(app, &label);
    }

    Ok(label)
}

/// 前端内容就绪后显示 modal。
pub fn modal_window_ready(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| format!("modal window not found: {label}"))?;

    if !window.is_visible().map_err(|e| e.to_string())? {
        window.show().map_err(|e| e.to_string())?;
    }
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// 隐藏 modal 窗口并同步关闭事件。
pub fn close_modal_window(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| format!("modal window not found: {label}"))?;

    if window.is_visible().map_err(|e| e.to_string())? {
        window.hide().map_err(|e| e.to_string())?;
    }
    let _ = events::modal_closed(app, label);
    Ok(())
}
