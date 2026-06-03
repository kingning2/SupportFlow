//! Binary license key file codec.

const MAGIC: &[u8; 4] = b"SFLK";
const VERSION: u8 = 1;
const OBFUSCATION_KEY: &[u8] = b"supportflow-license-key-v1";

fn obfuscate(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ OBFUSCATION_KEY[i % OBFUSCATION_KEY.len()] ^ (i as u8))
        .collect()
}

pub fn encode_token_to_key_bytes(token: &str) -> Result<Vec<u8>, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("activation token is empty".to_string());
    }
    let payload = obfuscate(token.as_bytes());
    let len = u32::try_from(payload.len()).map_err(|_| "license payload too large".to_string())?;

    let mut out = Vec::with_capacity(4 + 1 + 4 + payload.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

pub fn decode_token_from_key_bytes(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() < 9 {
        return Err("invalid license key file: too short".to_string());
    }
    if &bytes[0..4] != MAGIC {
        return Err("invalid license key file: bad magic".to_string());
    }
    let version = bytes[4];
    if version != VERSION {
        return Err(format!("unsupported license key version: {version}"));
    }

    let mut len_buf = [0u8; 4];
    len_buf.copy_from_slice(&bytes[5..9]);
    let payload_len = u32::from_le_bytes(len_buf) as usize;
    if bytes.len() != 9 + payload_len {
        return Err("invalid license key file: payload length mismatch".to_string());
    }

    let payload = &bytes[9..];
    let decoded = obfuscate(payload);
    let token =
        String::from_utf8(decoded).map_err(|e| format!("invalid license key payload utf8: {e}"))?;
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err("invalid license key file: empty token".to_string());
    }
    Ok(token)
}
