//! Offline subscription activation verification.
//!
//! License token format (base64url(JSON)):
//! {
//!   "product": string,
//!   "v": string,
//!   "machineCode": string,
//!   "exp": number,   // unix seconds
//!   "iat": number,
//!   "sig": string    // base64url(RSA-PSS(SHA256) signature of signingMessage)
//! }
//!
//! signingMessage = "${product}|${v}|${machineCode}|${exp}"
//!
//! This module also computes machineCode on Windows by running a PowerShell
//! snippet that mirrors the client generator's hw-id normalization rules.

use std::process::Command;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use rsa::pkcs8::DecodePublicKey;
use rsa::{
    pss::{Signature as PssSignature, VerifyingKey},
    RsaPublicKey,
};
use signature::Verifier as _;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseToken {
    pub product: String,
    pub v: String,
    #[serde(rename = "machineCode")]
    pub machine_code: String,
    pub exp: i64,
    pub sig: String,
}

#[derive(Debug, Clone)]
pub struct LicenseVerificationResult {
    pub valid: bool,
    pub reason: Option<String>,
}

fn normalize_mac(mac: &str) -> String {
    mac.trim().to_uppercase().replace([':', '-', ' '], "")
}

fn normalize_str(s: &str) -> String {
    // collapse repeated whitespace and trim
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
    // MACs
    let ps_macs = r#"
$ErrorActionPreference = 'Stop';
$macs = Get-NetAdapter -Physical -ErrorAction Stop |
  Where-Object { $_.Status -eq 'Up' -and $_.MacAddress -ne $null } |
  Select-Object -ExpandProperty MacAddress;
($macs | ForEach-Object { $_.ToString() }) -join "`n"
"#;

    let mac_out = match run_powershell(ps_macs) {
        Ok(s) => s,
        Err(_) => String::new(),
    };

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
        // Fallback: pick first available MAC even if adapters are not "Up".
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

    // Disk serials
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
        let s = normalize_str(line);
        if s.is_empty() {
            continue;
        }
        let sl = s.to_lowercase();
        if sl == "to be filled by o.e.m." || sl == "none" || sl == "unknown" {
            continue;
        }
        if !disk_serials.contains(&s) {
            disk_serials.push(s);
        }
    }

    // CPU ProcessorId
    let ps_cpu = r#"
$ErrorActionPreference = 'Stop';
$cpu = Get-CimInstance Win32_Processor -ErrorAction Stop |
  Select-Object -First 1 -ExpandProperty ProcessorId;
if ($cpu -eq $null) { "" } else { $cpu.ToString() }
"#;
    let cpu_id = run_powershell(ps_cpu).unwrap_or_default();
    let cpu_id = normalize_str(&cpu_id);

    Ok((macs, disk_serials, cpu_id))
}

pub fn compute_machine_code_windows() -> Result<String, String> {
    let (mut macs, mut disk_serials, cpu_id) = get_windows_hw_ids()?;
    macs.sort();
    macs.dedup();
    disk_serials.sort();
    disk_serials.dedup();

    let mac_list = macs.join(",");
    let disk_list = disk_serials.join(",");
    let hw_text = format!("mac={}|disk={}|cpu={}", mac_list, disk_list, cpu_id);

    let digest = Sha256::digest(hw_text.as_bytes());
    Ok(format!("{:x}", digest))
}

pub fn verify_license_token(
    token_b64url: &str,
    now_unix_seconds: i64,
    expected_machine_code: &str,
    public_key_pem: &str,
) -> LicenseVerificationResult {
    let decoded = match URL_SAFE_NO_PAD.decode(token_b64url.trim()) {
        Ok(v) => v,
        Err(e) => {
            return LicenseVerificationResult {
                valid: false,
                reason: Some(format!("token base64url decode failed: {e}")),
            }
        }
    };

    let token: LicenseToken = match serde_json::from_slice(&decoded) {
        Ok(v) => v,
        Err(e) => {
            return LicenseVerificationResult {
                valid: false,
                reason: Some(format!("token json decode failed: {e}")),
            }
        }
    };

    if token.machine_code != expected_machine_code {
        return LicenseVerificationResult {
            valid: false,
            reason: Some("machineCode mismatch".to_string()),
        };
    }

    if now_unix_seconds > token.exp {
        return LicenseVerificationResult {
            valid: false,
            reason: Some("license expired".to_string()),
        };
    }

    // signingMessage = product|v|machineCode|exp
    let signing_message = format!(
        "{}|{}|{}|{}",
        token.product, token.v, token.machine_code, token.exp
    );

    let sig_bytes = match URL_SAFE_NO_PAD.decode(token.sig.trim()) {
        Ok(v) => v,
        Err(e) => {
            return LicenseVerificationResult {
                valid: false,
                reason: Some(format!("sig base64url decode failed: {e}")),
            }
        }
    };

    let pub_key = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(v) => v,
        Err(e) => {
            return LicenseVerificationResult {
                valid: false,
                reason: Some(format!("public key parse failed: {e}")),
            };
        }
    };

    // RSA-PSS + SHA256 (saltlen must match generator openssl 'auto' behavior).
    // The rsa crate exposes salt length through VerifyingKey; we default to MAX.
    let verifying_key = VerifyingKey::<Sha256>::new(pub_key);

    let sig = match PssSignature::try_from(sig_bytes.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            return LicenseVerificationResult {
                valid: false,
                reason: Some(format!("sig parse failed: {e}")),
            }
        }
    };

    let res = verifying_key.verify(signing_message.as_bytes(), &sig);
    match res {
        Ok(()) => LicenseVerificationResult {
            valid: true,
            reason: None,
        },
        Err(e) => LicenseVerificationResult {
            valid: false,
            reason: Some(format!("signature verify error: {e}")),
        },
    }
}
