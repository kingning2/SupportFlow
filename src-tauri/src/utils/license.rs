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
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Deserialize;
use sha2::Sha256;

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
