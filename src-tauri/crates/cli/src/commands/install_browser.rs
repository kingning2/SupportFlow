use anyhow::Result;
use clap::Args;

use crate::runtime::CliRuntime;
use agent::tools::browser::{resolve_chrome_executable, BrowserSettings};

#[derive(Args)]
pub struct InstallBrowserArgs {}

pub fn run(_args: InstallBrowserArgs) -> Result<()> {
    let rt = CliRuntime::load()?;
    let settings = BrowserSettings::from_models(rt.config.as_ref());

    println!("\n  Browser tool setup (Rust agent)\n");
    println!(
        "  The Rust browser tool uses Chrome/Chromium via CDP (chromiumoxide), not Playwright.\n"
    );

    match resolve_chrome_executable(settings.chrome_executable.as_deref()) {
        Ok(exe) => println!("  Chrome/Chromium: {}", exe.display()),
        Err(e) => {
            println!("  {e}");
        }
    }

    if !settings.cdp_endpoint.is_empty() {
        println!("  CDP endpoint: {}", settings.cdp_endpoint);
    }

    println!("\n  Optional: set tools.browser.headless / user_data_dir in config.json.");
    println!("  Legacy SupportFlow Agent Playwright install: pip install playwright && playwright install chromium\n");
    Ok(())
}
