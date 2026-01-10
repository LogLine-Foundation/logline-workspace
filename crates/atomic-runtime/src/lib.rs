#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::multiple_crate_versions)]
//! Runtime/router for DIM dispatch with middleware and UBL logging.
/// Application context and logging.
pub mod ctx;
/// Events emitted to UBL.
pub mod events;
/// Middleware and budget helpers.
pub mod mw;
/// DIM router and middleware wiring.
pub mod router;
/// Typed handler helper.
pub mod typed;
/// HTTP octet parsing helpers.
pub mod web;
pub use ctx::AppCtx;
pub use mw::{Budgets, Middleware};
pub use router::{FnHandler, HandlerFn, Router};
pub use typed::handle_typed;
pub use web::parse_http_octets;

use anyhow::{anyhow, Result};
use atomic_crypto::blake3_hex;
use atomic_types::{ActorId, Dim};
use events::{IntentCompleted, IntentReceived};

/// Processa cápsula com middlewares e budgets.
///
/// # Errors
///
/// - `anyhow::Error` se budgets estourarem, não houver handler ou handlers/middlewares falharem
pub fn process(
    dim: Dim,
    actor: &ActorId,
    capsule_bytes: &[u8],
    router: &Router,
    ctx: &AppCtx,
    budgets: &mut Budgets,
) -> Result<Vec<u8>> {
    let cid = blake3_hex(capsule_bytes);
    let received_ev = IntentReceived {
        dim: dim.0,
        capsule_cid_hex: cid.clone(),
        size: capsule_bytes.len() as u64,
    };
    ctx.received(&received_ev)?;

    if let Some(rem) = budgets.consume(actor, 1) {
        if rem == 0 {
            ctx.log(
                "budget.limit_hit",
                &serde_json::json!({"actor":actor, "dim":dim.0}),
            )?;
            return Err(anyhow!("budget exceeded"));
        }
    }

    for m in &router.mw_before {
        m.before(dim, actor, capsule_bytes, ctx)?;
    }

    let h = router
        .get(dim)
        .ok_or_else(|| anyhow!(format!("no handler for dim=0x{:04x}", dim.0)))?;
    let out = h(capsule_bytes);

    for m in &router.mw_after {
        m.after(
            dim,
            actor,
            capsule_bytes,
            out.as_deref().unwrap_or(&[]),
            ctx,
        )?;
    }

    match out {
        Ok(ref b) => {
            let completed_ev = IntentCompleted {
                dim: dim.0,
                capsule_cid_hex: cid,
                ok: true,
                result_size: b.len() as u64,
            };
            ctx.completed(&completed_ev)?;
        }
        Err(ref e) => {
            let completed_ev = IntentCompleted {
                dim: dim.0,
                capsule_cid_hex: cid,
                ok: false,
                result_size: 0,
            };
            ctx.completed(&completed_ev)?;
            return Err(anyhow!(format!("handler failed: {e}")));
        }
    }
    out
}
