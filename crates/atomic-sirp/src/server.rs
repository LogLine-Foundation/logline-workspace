//! Minimal Axum server for SIRP capsules with optional HMAC and SQLite idempotency.
#[cfg(feature = "server")]
use crate::receipt::sign_receipt;
#[cfg(feature = "server")]
use anyhow::Result;
#[cfg(feature = "server")]
use atomic_crypto::{b64_encode, blake3_hex, hmac_verify, key_id_v1, SecretKey};
#[cfg(feature = "server")]
use axum::http::{HeaderMap, StatusCode};
#[cfg(feature = "server")]
use axum::{body::Bytes, extract::State, routing::post, Router};
#[cfg(feature = "server")]
use ed25519_dalek::VerifyingKey;
#[cfg(feature = "server")]
use std::sync::Arc;

/// Processador de cápsulas.
#[cfg(feature = "server")]
pub trait Processor: Send + Sync + 'static {
    /// Processa uma cápsula e retorna o payload de resposta.
    ///
    /// # Errors
    ///
    /// - Implementações podem retornar erros específicos de negócio ou parsing
    fn process(&self, capsule: &[u8]) -> Result<Vec<u8>>;
}
#[cfg(feature = "server")]
/// Implementação de conveniência com closure.
pub struct FnProcessor<T>(pub T);
#[cfg(feature = "server")]
impl<T> Processor for FnProcessor<T>
where
    T: Send + Sync + 'static + Fn(&[u8]) -> Result<Vec<u8>>,
{
    fn process(&self, capsule: &[u8]) -> Result<Vec<u8>> {
        (self.0)(capsule)
    }
}

#[cfg(feature = "server")]
#[derive(Clone)]
/// Estado compartilhado do servidor SIRP.
pub struct AppState {
    proc: Arc<dyn Processor>,
    sk: SecretKey,
    vk: VerifyingKey,
    kid: String,
    hmac_key: Option<Vec<u8>>,
    #[cfg(feature = "sqlite")]
    idem: Option<Arc<crate::idempotency::SqliteIdem>>,
    ttl_seconds: i64,
}

/// Constrói router.
#[cfg(feature = "server")]
pub fn router<P: Processor>(
    proc: P,
    sk: SecretKey,
    ttl_seconds: i64,
    idem: Option<crate::idempotency::SqliteIdem>,
    hmac_key: Option<Vec<u8>>,
) -> Router {
    let vk = sk.verifying_key();
    let kid = key_id_v1(&vk);
    let idem = idem.map(Arc::new);
    let state = AppState {
        proc: Arc::new(proc),
        sk,
        vk,
        kid,
        idem,
        ttl_seconds,
        hmac_key,
    };
    Router::new()
        .route("/sirp/capsule", post(handle_capsule))
        .with_state(state)
}

#[cfg(feature = "server")]
async fn handle_capsule(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<axum::response::Response, StatusCode> {
    let capsule = body.to_vec();
    let cid = blake3_hex(&capsule);

    // HMAC opcional
    if let Some(key) = &state.hmac_key {
        let got = headers
            .get("x-sirp-hmac")
            .ok_or(StatusCode::UNAUTHORIZED)
            .and_then(|v| v.to_str().map_err(|_| StatusCode::UNAUTHORIZED))?;
        hmac_verify(key, &capsule, got).map_err(|_| StatusCode::UNAUTHORIZED)?;
    }

    #[cfg(feature = "sqlite")]
    if let Some(store) = &state.idem {
        if store
            .already(&cid)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            let r = crate::receipt::Receipt {
                capsule_cid_hex: cid.clone(),
                ok: true,
                msg: Some("already_done".into()),
                sig_b64: b64_encode(&atomic_crypto::sign_cid_hex(&state.sk, &cid)),
                pk_b64: b64_encode(state.vk.as_bytes()),
                kid: state.kid.clone(),
                already_done: true,
            };
            let rj = serde_json::to_string(&r).unwrap();
            return Ok(axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("x-sirp-receipt", rj)
                .header("content-type", "application/octet-stream")
                .body(axum::body::Body::from(Vec::<u8>::new()))
                .unwrap());
        }
        store.cleanup_ttl_seconds(state.ttl_seconds).ok();
    }

    let out = state
        .proc
        .process(&capsule)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    #[cfg(feature = "sqlite")]
    if let Some(store) = &state.idem {
        store
            .mark(&cid)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let r = sign_receipt(&state.sk, &state.vk, &state.kid, &cid, true, None, false);
    let rj = serde_json::to_string(&r).unwrap();
    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("x-sirp-receipt", rj)
        .header("content-type", "application/octet-stream")
        .body(axum::body::Body::from(out))
        .unwrap())
}
