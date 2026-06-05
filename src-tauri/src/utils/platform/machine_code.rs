//! Windows-only machine code collection.

use std::process::Command;

use sha2::{Digest, Sha256};

fn normalize_mac(mac: &str) -> String {
    mac.trim().to_uppercase().replace([':', '-', ' '], "")
}

fn normalize_str(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn run_powershell(script: &str) -> Result<String, String> {
    let out = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|e| format!("powershell spawn failed: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return Err(format!(
            "powershell failed (code={}) stderr={stderr} stdout={stdout}",
            out.status.code().unwrap_or(-1)
        ));
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn get_windows_hw_ids() -> Result<(Vec<String>, Vec<String>, String), String> {
    let ps_macs = r#"
$ErrorActionPreference = 'Stop';
$macs = Get-NetAdapter -Physical -ErrorAction Stop |
  Where-Object { $_.Status -eq 'Up' -and $_.MacAddress -ne $null } |
  Select-Object -ExpandProperty MacAddress;
($macs | ForEach-Object { $_.ToString() }) -join "`n"
"#;

    let mac_out = run_powershell(ps_macs).unwrap_or_default();
    let mut macs: Vec<String> = Vec::new();
    if !mac_out.is_empty() {
        for line in mac_out.lines() {
            let mac = normalize_mac(line);
            if !mac.is_empty() && mac != "000000000000" && !macs.contains(&mac) {
                macs.push(mac);
            }
        }
    }

    if macs.is_empty() {
        let ps_any_mac = r#"
$ErrorActionPreference = 'Stop';
$mac_any = Get-NetAdapter -ErrorAction Stop |
  Where-Object { $_.MacAddress -ne $null } |
  Select-Object -First 1 -ExpandProperty MacAddress;
if ($mac_any -eq $null) { "" } else { $mac_any.ToString() }
"#;
        let any = run_powershell(ps_any_mac).unwrap_or_default();
        let mac = normalize_mac(&any);
        if !mac.is_empty() && mac != "000000000000" {
            macs.push(mac);
        }
    }

    let ps_disks = r#"
$ErrorActionPreference = 'Stop';
$serials = Get-CimInstance Win32_DiskDrive -ErrorAction Stop |
  Where-Object { $_.SerialNumber -ne $null } |
  Select-Object -ExpandProperty SerialNumber;
($serials | ForEach-Object { $_.ToString() }) -join "`n"
"#;
    let disk_out = run_powershell(ps_disks)?;
    let mut disk_serials: Vec<String> = Vec::new();
    for line in disk_out.lines() {
        let serial = normalize_str(line);
        if serial.is_empty() {
            continue;
        }
        let lower = serial.to_lowercase();
        if lower == "to be filled by o.e.m." || lower == "none" || lower == "unknown" {
            continue;
        }
        if !disk_serials.contains(&serial) {
            disk_serials.push(serial);
        }
    }

    let ps_cpu = r#"
$ErrorActionPreference = 'Stop';
$cpu = Get-CimInstance Win32_Processor -ErrorAction Stop |
  Select-Object -First 1 -ExpandProperty ProcessorId;
if ($cpu -eq $null) { "" } else { $cpu.ToString() }
"#;
    let cpu_id = normalize_str(&run_powershell(ps_cpu).unwrap_or_default());

    Ok((macs, disk_serials, cpu_id))
}

/// Compute the local machine code for Windows subscription verification.
///
/// # Returns
///
/// * `String` - Deterministic machine code for the current Windows host
pub fn compute_machine_code() -> Result<String, String> {
    let (mut macs, mut disk_serials, cpu_id) = get_windows_hw_ids()?;
    macs.sort();
    macs.dedup();
    disk_serials.sort();
    disk_serials.dedup();

    let hw_text = format!(
        "mac={}|disk={}|cpu={}",
        macs.join(","),
        disk_serials.join(","),
        cpu_id
    );

    let digest = Sha256::digest(hw_text.as_bytes());
    Ok(format!("{:x}", digest))
}
