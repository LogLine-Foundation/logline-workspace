//! HTTP client helper to send SIRP capsules with optional HMAC and retry/backoff.
#[cfg(feature = "http")]
use anyhow::{anyhow, Result};
#[cfg(feature = "http")]
use reqwest::{Client, StatusCode};
#[cfg(feature = "http")]
use std::{thread::sleep, time::Duration};

#[cfg(feature = "http")]
fn backoff_ms(attempt: usize) -> u64 {
    let b = (attempt as u64 + 1) * 50;
    b.min(1500)
}

/// Envia cápsula com HMAC opcional (header `x-sirp-hmac`).
#[cfg(feature = "http")]
///
/// # Errors
///
/// - Erros de rede ou HTTP retornados por `reqwest`
pub async fn post_capsule_hmac(
    url: &str,
    capsule_bytes: &[u8],
    hmac_key: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let cli = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..6 {
        let mut req = cli
            .post(url)
            .header("content-type", "application/octet-stream")
            .body(capsule_bytes.to_vec());
        if let Some(k) = hmac_key {
            let tag = atomic_crypto::hmac_sign(k, capsule_bytes);
            req = req.header("x-sirp-hmac", tag);
        }
        match req.send().await {
            Ok(resp) => {
                if resp.status() == StatusCode::OK {
                    let bytes = resp.bytes().await?;
                    return Ok(bytes.to_vec());
                }
                last_err = Some(anyhow!("status {}", resp.status()));
            }
            Err(e) => last_err = Some(anyhow!(e)),
        }
        sleep(Duration::from_millis(backoff_ms(attempt)));
    }
    Err(last_err.unwrap_or_else(|| anyhow!("send failed")))
}
