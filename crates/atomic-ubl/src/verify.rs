//! Verification helpers for UBL files and chains (signature, CID, prev-link).
use crate::event::UblEvent;
use anyhow::{anyhow, Result};
use ubl_crypto::{b64_decode, blake3_hex, verify_cid_hex};
use ed25519_dalek::VerifyingKey;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

/// Verifica um evento isolado (e opcionalmente a cadeia).
///
/// # Errors
///
/// - Retorna erros de decoding/assinatura ou inconsistências de cadeia
pub fn verify_event(ev: &UblEvent, expect_prev: Option<&str>, strict_chain: bool) -> Result<()> {
    let canon = b64_decode(&ev.canon_b64)?;
    let cid = blake3_hex(&canon);
    if cid != ev.cid_hex {
        return Err(anyhow!("CID mismatch"));
    }
    let pk = b64_decode(&ev.pk_b64)?;
    let vk = VerifyingKey::from_bytes(&pk.as_slice().try_into().map_err(|_| anyhow!("bad pk"))?)?;
    let sig = b64_decode(&ev.sig_b64)?;
    if !verify_cid_hex(&vk, &ev.cid_hex, &sig) {
        return Err(anyhow!("bad signature"));
    }
    if strict_chain {
        if let Some(exp) = expect_prev {
            let got = ev.prev_cid_hex.as_deref().unwrap_or("");
            if got != exp {
                return Err(anyhow!("prev mismatch"));
            }
        }
    }
    Ok(())
}
/// Verifica arquivo (sem cadeia).
///
/// # Errors
///
/// - Propaga erros de I/O, parsing ou verificação de eventos
pub fn verify_file<P: AsRef<Path>>(path: P) -> Result<usize> {
    verify_file_with_chain(path, false)
}
/// Verifica arquivo (cadeia estrita).
///
/// # Errors
///
/// - Propaga erros de I/O, parsing ou verificação de cadeia
pub fn verify_file_with_chain<P: AsRef<Path>>(path: P, strict: bool) -> Result<usize> {
    let f = File::open(path)?;
    let r = BufReader::new(f);
    let mut n = 0usize;
    let mut last = None;
    for line in r.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let mut ev: UblEvent = serde_json::from_str(&line)?;
        if strict {
            ev.prev_cid_hex.get_or_insert(String::new());
        }
        verify_event(&ev, last.as_deref(), strict)?;
        last = Some(ev.cid_hex.clone());
        n += 1;
    }
    Ok(n)
}
