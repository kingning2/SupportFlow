use std::fs;
use std::path::Path;

use chrono::Utc;

fn main() {
    /* ===== env ===== */
    let dotenv_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    if dotenv_path.exists() {
        println!("cargo:rerun-if-changed={}", dotenv_path.display());

        if let Ok(contents) = fs::read_to_string(&dotenv_path) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    if key.is_empty() {
                        continue;
                    }

                    let mut value = value.trim().to_string();
                    if (value.starts_with('"') && value.ends_with('"'))
                        || (value.starts_with('\'') && value.ends_with('\''))
                    {
                        value = value[1..value.len() - 1].to_string();
                    }

                    println!("cargo:rustc-env={}={}", key, value);
                }
            }
        }
    }

    println!("cargo:rustc-env=APP_VERSION={}", env!("CARGO_PKG_VERSION"));

    let now = Utc::now();
    let build_number = now.format("%Y%m%d%H%M%S").to_string();
    println!("cargo:rustc-env=APP_BUILD_NUMBER={}", build_number);

    let last_update = now.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("cargo:rustc-env=APP_LAST_UPDATE={}", last_update);

    tauri_build::build()
}
