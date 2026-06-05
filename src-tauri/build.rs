use std::fs;
use std::path::Path;

use chrono::Utc;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(channel_sidecar_embedded)");

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=BUILD_TARGET={target}");

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sidecar = manifest.join(format!(
        "binaries/channel-sidecar-{target}{}",
        std::env::consts::EXE_SUFFIX
    ));
    if sidecar.is_file() {
        println!("cargo:rerun-if-changed={}", sidecar.display());
        let profile = std::env::var("PROFILE").unwrap_or_default();
        if profile == "release" {
            println!("cargo:rustc-cfg=channel_sidecar_embedded");
        }
    }
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("../channel_agent/channel/__main__.py")
            .display()
    );

    /* ===== env ===== */
    let dotenv_path = manifest.join("../.env");
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

    ensure_channel_sidecar_exe(manifest);
    ensure_license_verifier_exe(manifest);

    if std::env::var("CARGO_FEATURE_DESKTOP").is_ok() {
        tauri_build::build();
    }
}

/// PyInstaller sidecar must exist before `tauri_build` validates `externalBin`.
fn ensure_channel_sidecar_exe(manifest: &Path) {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    let sidecar = manifest.join(format!(
        "binaries/channel-sidecar-{target}{}",
        std::env::consts::EXE_SUFFIX
    ));

    if sidecar.is_file() {
        return;
    }

    if std::env::var("SKIP_CHANNEL_SIDECAR_BUILD").is_ok() {
        println!(
            "cargo:warning=channel sidecar missing at {}; set SKIP_CHANNEL_SIDECAR_BUILD only if you patch tauri.conf externalBin",
            sidecar.display()
        );
        return;
    }

    println!(
        "cargo:warning=channel sidecar not found, running build-channel-sidecar.ps1 (first build may take several minutes)..."
    );

    let script = manifest.join("scripts/build-channel-sidecar.ps1");
    let status = std::process::Command::new("powershell")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script)
        .current_dir(manifest)
        .status();

    match status {
        Ok(s) if s.success() && sidecar.is_file() => {
            println!("cargo:rerun-if-changed={}", sidecar.display());
        }
        Ok(s) => {
            println!(
                "cargo:warning=build-channel-sidecar.ps1 exit {:?}; dev will use Python source fallback if exe still missing",
                s.code()
            );
        }
        Err(e) => {
            println!("cargo:warning=failed to run build-channel-sidecar.ps1: {e}");
        }
    }
}

fn ensure_license_verifier_exe(manifest: &Path) {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    let verifier = manifest.join(format!(
        "binaries/license-verifier-{target}{}",
        std::env::consts::EXE_SUFFIX
    ));

    if verifier.is_file() {
        println!("cargo:rerun-if-changed={}", verifier.display());
        return;
    }

    if std::env::var("SKIP_LICENSE_VERIFIER_BUILD").is_ok() {
        println!(
            "cargo:warning=license-verifier missing at {}; set SKIP_LICENSE_VERIFIER_BUILD to skip auto-build",
            verifier.display()
        );
        return;
    }

    println!("cargo:warning=license-verifier not found, running build-license-verifier.ps1 ...");

    let script = manifest.join("scripts/build-license-verifier.ps1");
    let status = std::process::Command::new("powershell")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script)
        .current_dir(manifest)
        .status();

    match status {
        Ok(s) if s.success() && verifier.is_file() => {
            println!("cargo:rerun-if-changed={}", verifier.display());
        }
        Ok(s) => {
            println!(
                "cargo:warning=build-license-verifier.ps1 exit {:?}; license checks need the verifier exe",
                s.code()
            );
        }
        Err(e) => {
            println!("cargo:warning=failed to run build-license-verifier.ps1: {e}");
        }
    }
}
