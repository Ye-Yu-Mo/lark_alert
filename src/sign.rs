use crate::error::LarkAlertError;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Generate the Feishu custom bot signature for a given timestamp and secret.
///
/// Official algorithm:
/// `sign = base64(hmac_sha256(key = "{timestamp}\n{secret}", message = ""))`
pub fn sign(timestamp: &str, secret: &str) -> Result<String, LarkAlertError> {
    let key = format!("{timestamp}\n{secret}");
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .map_err(|e| LarkAlertError::Validation(format!("invalid HMAC key: {e}")))?;
    mac.update(b"");
    Ok(BASE64.encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_feishu_example_vector() {
        // This vector is generated from the official Python sample:
        // hmac.new(f"{timestamp}\n{secret}".encode(), digestmod=hashlib.sha256)
        assert_eq!(
            sign("1627114387000", "test_secret").unwrap(),
            "E0zT2kmkJtx1JzyO80J6oXudmfFrYa+qhe/zgjOApR4="
        );
    }

    #[test]
    fn sign_changes_with_timestamp_or_secret() {
        let a = sign("1", "s").unwrap();
        let b = sign("2", "s").unwrap();
        let c = sign("1", "t").unwrap();
        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}
