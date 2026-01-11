#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64URL, Engine as _};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub iss: Option<String>,
    pub sub: String,
    pub aud: Option<Json>,
    pub iat: Option<i64>,
    pub exp: Option<i64>,
    pub jti: Option<String>,
    #[serde(flatten)]
    pub extra: Json,
}

fn b64url_decode_to_vec(s: &str) -> Result<Vec<u8>, String> {
    B64URL.decode(s.as_bytes()).map_err(|_| "bad b64".into())
}

/// Verify a JWT signed with Ed25519 against a JWKS json (EdDSA/OKP keys).
/// Returns decoded claims if signature verifies and issuer/audience (if provided) match.
pub fn verify_ed25519_jwt_with_jwks(token: &str, jwks_json: &str, expected_iss: Option<&str>, expected_aud: Option<&str>) -> Result<Claims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 { return Err("bad jwt".into()); }
    let header_json = String::from_utf8(b64url_decode_to_vec(parts[0]).map_err(|_| "bad header b64")?).map_err(|_| "utf8 header")?;
    let payload_json = String::from_utf8(b64url_decode_to_vec(parts[1]).map_err(|_| "bad payload b64")?).map_err(|_| "utf8 payload")?;
    let header: Json = serde_json::from_str(&header_json).map_err(|_| "json header")?;
    let payload: Claims = serde_json::from_str(&payload_json).map_err(|_| "json payload")?;

    if let Some(iss) = expected_iss {
        if payload.iss.as_deref() != Some(iss) { return Err("iss mismatch".into()); }
    }
    if let Some(aud) = expected_aud {
        let ok = match &payload.aud {
            Some(Json::String(a)) => a == aud,
            Some(Json::Array(arr)) => arr.iter().any(|v| v.as_str() == Some(aud)),
            _ => false
        };
        if !ok { return Err("aud mismatch".into()); }
    }

    // Resolve Ed25519 key from JWKS by kid (or thumbprint)
    let kid = header.get("kid").and_then(|v| v.as_str()).ok_or_else(|| "missing kid".to_string())?;
    let jwks: Json = serde_json::from_str(jwks_json).map_err(|_| "bad jwks")?;
    let keys = jwks.get("keys").and_then(|v| v.as_array()).ok_or_else(|| "no keys".to_string())?;
    let mut x_b64: Option<&str> = None;
    for k in keys {
        if k.get("kty").and_then(|v| v.as_str()) != Some("OKP") { continue; }
        if k.get("crv").and_then(|v| v.as_str()) != Some("Ed25519") { continue; }
        let x = k.get("x").and_then(|v| v.as_str()).unwrap_or("");
        let this_kid = k.get("kid").and_then(|v| v.as_str()).unwrap_or("");
        if this_kid == kid || this_kid.is_empty() {
            x_b64 = Some(x);
            break;
        }
    }
    let x = x_b64.ok_or_else(|| "kid not found".to_string())?;
    let pk_bytes = b64url_decode_to_vec(x).map_err(|_| "bad x b64")?;
    if pk_bytes.len() != 32 { return Err("bad ed25519 pk".into()); }

    // Verify signature
    use ed25519_dalek::{VerifyingKey, Signature};
    let vk = VerifyingKey::from_bytes(pk_bytes.as_slice().try_into().map_err(|_| "pk size")?).map_err(|_| "pk parse")?;
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let sig_bytes = b64url_decode_to_vec(parts[2]).map_err(|_| "bad sig b64")?;
    let sig = Signature::from_bytes(sig_bytes.as_slice().try_into().map_err(|_| "sig size")?);
    vk.verify_strict(signing_input.as_bytes(), &sig).map_err(|_| "sig verify")?;

    Ok(payload)
}

#[cfg(feature = "assets")]
pub mod assets {
    pub fn sqlite_migration_001() -> &'static str { include_str!("../assets/sql/001_add_did.sql") }
    pub fn postgres_migration_001() -> &'static str { include_str!("../assets/sql/001_add_did.pg.sql") }
    pub fn receipt_action_v1_schema() -> &'static str { include_str!("../assets/receipts/action.v1.schema.json") }
}
