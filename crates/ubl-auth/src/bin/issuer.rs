#[cfg(feature = "server")]
mod server {
    use axum::{routing::{get, post}, Router, extract::{State, Host}, response::IntoResponse, http::{StatusCode, HeaderMap}};
    use serde_json::json;
    use std::{sync::Arc, net::SocketAddr};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64URL, Engine as _};
    use ed25519_dalek::{SigningKey, VerifyingKey, pkcs8::DecodePrivateKey, Signer};
    use uuid::Uuid;
    use time::{OffsetDateTime, Duration};
    use tower_http::cors::CorsLayer;
    use sha2::{Sha256, Digest};

    #[derive(Clone)]
    struct AppState {
        sk_pem: String,
        clients_json: Option<String>,
        issuer: Option<String>,
        subject_did: Option<String>,
    }

    fn parse_signing_key(sk_pem: &str) -> SigningKey {
        SigningKey::from_pkcs8_pem(sk_pem).expect("invalid PKCS8 Ed25519 PEM")
    }
    fn jwk_thumbprint_b64url(vk: &VerifyingKey) -> String {
        let x = B64URL.encode(vk.to_bytes());
        let s = format!(r#"{{"crv":"Ed25519","kty":"OKP","x":"{}"}}"#, x);
        let mut h = Sha256::new(); h.update(s.as_bytes());
        B64URL.encode(h.finalize())
    }
    fn to_pub_jwk(vk: &VerifyingKey, kid: &str) -> serde_json::Value {
        json!({"kty":"OKP","crv":"Ed25519","x":B64URL.encode(vk.to_bytes()),"alg":"EdDSA","kid":kid})
    }
    fn infer_issuer(host: &str, state: &AppState) -> String {
        state.issuer.clone().unwrap_or_else(|| format!("https://{}", host))
    }
    fn with_json_headers(body: serde_json::Value) -> (HeaderMap, String) {
        let mut headers = HeaderMap::new();
        headers.insert(axum::http::header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(axum::http::header::CACHE_CONTROL, "public, max-age=300".parse().unwrap());
        (headers, serde_json::to_string_pretty(&body).unwrap())
    }

    async fn jwks(axum::extract::Host(_host): Host, State(state): State<Arc<AppState>>) -> impl IntoResponse {
        let sk = parse_signing_key(&state.sk_pem);
        let vk = sk.verifying_key();
        let kid = jwk_thumbprint_b64url(&vk);
        (StatusCode::OK, with_json_headers(json!({"keys":[to_pub_jwk(&vk,&kid)]})))
    }
    async fn did_web(axum::extract::Host(host): Host, State(state): State<Arc<AppState>>) -> impl IntoResponse {
        let sk = parse_signing_key(&state.sk_pem);
        let vk = sk.verifying_key();
        let _kid = jwk_thumbprint_b64url(&vk);
        let did = format!("did:web:{}", host.replace(':', "%3A"));
        let key_id = format!("{}#key-1", did);
        let vm = json!({
            "id": key_id,
            "type": "JsonWebKey2020",
            "controller": did,
            "publicKeyJwk": {"kty":"OKP","crv":"Ed25519","x": B64URL.encode(vk.to_bytes())}
        });
        (StatusCode::OK, with_json_headers(json!({
            "@context": ["https://www.w3.org/ns/did/v1","https://w3id.org/security/jws/v1"],
            "id": did,
            "verificationMethod": [vm],
            "authentication": [key_id],
            "assertionMethod": [key_id]
        })))
    }
    async fn discovery(axum::extract::Host(host): Host, State(state): State<Arc<AppState>>) -> impl IntoResponse {
        let iss = infer_issuer(&host, &state);
        (StatusCode::OK, with_json_headers(json!({
            "issuer": iss,
            "jwks_uri": format!("{}/.well-known/jwks.json", iss),
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["EdDSA"],
            "token_endpoint": format!("{}/oauth/token", iss),
            "scopes_supported": ["openid","profile","email"],
            "response_types_supported": ["code","token","id_token"]
        })))
    }
    #[derive(serde::Deserialize)]
    struct TokenForm { grant_type:String, client_id:Option<String>, client_secret:Option<String>, audience:Option<String>, scope:Option<String> }
    async fn token(axum::extract::Host(host): Host, State(state): State<Arc<AppState>>, axum::Form(f): axum::Form<TokenForm>) -> impl IntoResponse {
        if f.grant_type != "client_credentials" {
            return (StatusCode::BAD_REQUEST, with_json_headers(json!({"error":"unsupported_grant_type"})));
        }
        // validate client
        let mut ok = false;
        if let Some(cfg) = &state.clients_json {
            if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(cfg) {
                if let (Some(id), Some(secret)) = (f.client_id.clone(), f.client_secret.clone()) {
                    if let Some(v) = map.get(&id) { if v.as_str() == Some(&secret) { ok = true; } }
                }
            }
        }
        if !ok {
            return (StatusCode::UNAUTHORIZED, with_json_headers(json!({"error":"invalid_client"})));
        }
        let iss = infer_issuer(&host, &state);
        let sub = state.subject_did.clone().unwrap_or_else(|| format!("did:web:{}:issuer", host.replace(':', "%3A")));
        let aud = f.audience.unwrap_or_else(|| "self".to_string());
        let now = OffsetDateTime::now_utc();
        let iat = now.unix_timestamp();
        let exp = (now + Duration::hours(1)).unix_timestamp();
        let jti = Uuid::new_v4().to_string();

        let sk = parse_signing_key(&state.sk_pem);
        let vk = sk.verifying_key();
        let kid = jwk_thumbprint_b64url(&vk);

        let header = json!({"alg":"EdDSA","typ":"JWT","kid":kid});
        let payload = json!({"iss":iss,"sub":sub,"aud":aud,"iat":iat,"exp":exp,"jti":jti,"scope":f.scope.unwrap_or_default()});

        let h = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let p = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        let msg = format!("{}.{}", h, p);
        let sig = sk.sign(msg.as_bytes());
        let s = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig.to_bytes());
        (StatusCode::OK, with_json_headers(json!({"access_token": format!("{}.{}.{}",h,p,s), "token_type":"Bearer", "expires_in":3600, "issuer":payload["iss"], "sub":payload["sub"], "scope":payload["scope"]})))
    }

    pub async fn run() {
        let state = AppState {
            sk_pem: std::env::var("UBL_JWK_PRIVATE_PEM").expect("set UBL_JWK_PRIVATE_PEM (PKCS8 Ed25519 PEM)"),
            clients_json: std::env::var("OIDC_CLIENTS_JSON").ok(),
            issuer: std::env::var("OIDC_ISSUER").ok(),
            subject_did: std::env::var("SUBJECT_DID").ok(),
        };
        let app = Router::new()
            .route("/.well-known/jwks.json", get(jwks))
            .route("/.well-known/did.json", get(did_web))
            .route("/.well-known/openid-configuration", get(discovery))
            .route("/oauth/token", post(token))
            .with_state(Arc::new(state))
            .layer(CorsLayer::very_permissive());

        let addr: SocketAddr = ([0,0,0,0], 3000).into();
        println!("listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!("Enable the `server` feature to build this binary: cargo run --features server --bin ubl-auth-issuer");
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    server::run().await;
}
