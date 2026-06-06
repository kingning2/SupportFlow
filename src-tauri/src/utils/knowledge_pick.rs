//! Knowledge picker helpers.
//!
//! Keeps `cmd/` layer thin: the Tauri command should delegate picker + local file IO
//! to this module.

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath};

type PickedKnowledgeFile = (String, Vec<u8>);

/// Pick supported knowledge files from the native OS file dialog, then read them from disk.
///
/// # Arguments
///
/// * `app` - Tauri app handle (used to open the dialog)
///
/// # Returns
///
/// * `Ok(None)` - user cancelled the dialog
/// * `Ok(Some(vec))` - picked files read from disk (may be empty if all reads failed)
pub fn pick_and_read_supported_knowledge_files(
    app: &AppHandle,
) -> Result<Option<Vec<PickedKnowledgeFile>>, String> {
    // Collect supported extensions (strip the leading dot for dialog filter API).
    let raw_exts = crate::agent::knowledge::document_parser::all_doc_suffixes();
    let exts: Vec<&str> = raw_exts
        .iter()
        .copied()
        .map(|s| s.trim_start_matches('.'))
        .collect();

    let selected = app
        .dialog()
        .file()
        .add_filter("Supported Documents", &exts)
        .blocking_pick_files();

    let Some(paths) = selected else {
        return Ok(None);
    };

    // Read bytes on Rust side only. Frontend never sends content over IPC.
    let files: Vec<PickedKnowledgeFile> = paths
        .into_iter()
        .filter_map(|fp| {
            let path = match fp {
                FilePath::Path(pb) => pb,
                FilePath::Url(u) => {
                    // Some picker backends may return `Url` values that aren't usable as files.
                    if let Ok(p) = u.to_file_path() {
                        p
                    } else {
                        crate::log_warn!(
                            "[KnowledgePick] skipping non-file URL from picker: {}",
                            u
                        );
                        return None;
                    }
                }
            };

            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown-file".into());

            match crate::utils::fs::read(&path) {
                Ok(data) => Some((filename, data)),
                Err(e) => {
                    crate::log_warn!("[KnowledgePick] failed to read {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect();

    Ok(Some(files))
}
