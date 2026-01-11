# ubl-auth (mono-crate)

**One crate** to run UBL ID everywhere:

- ✅ Pure-Rust **JWT/JWKS verification** (Ed25519) — works server & WASM
- ✅ Optional **issuer server** (feature `server`): did:web, JWKS, OIDC discovery, `/oauth/token` (client_credentials, EdDSA)
- ✅ Embedded **rollout assets** (feature `assets`): SQL migrations & `action.v1` schema

## Install
```toml
# Cargo.toml
ubl-auth = { version = "0.7", default-features = false }
# or with the server binary:
ubl-auth = { version = "0.7", features = ["server","assets"] }
```

## Verify tokens (library)
```rust
use ubl_auth::{verify_ed25519_jwt_with_jwks, Claims};

let jwks_json = include_str!("./tests/jwks.example.json");
let token = std::env::var("ACCESS_TOKEN")?;
let claims: Claims = verify_ed25519_jwt_with_jwks(&token, jwks_json, Some("https://id.ubl.agency"), Some("https://your-app"))?;
assert!(claims.sub.starts_with("did:key:"));
```

## Run the issuer server (did:web + JWKS + discovery + /oauth/token)
```bash
# Ed25519 private key (PKCS8 PEM)
export UBL_JWK_PRIVATE_PEM="$(cat ed25519-private.pk8.pem)"
# Clients registry (id -> secret)
export OIDC_CLIENTS_JSON='{"demo-client":"demo-secret"}'
# Optional overrides
export OIDC_ISSUER="https://id.ubl.agency"
export SUBJECT_DID="did:key:z6Mkj..."    # sub for service tokens

# Run
cargo run --release --features server --bin ubl-auth-issuer
```

## Embedded assets (migrations + receipt schema)
```rust
#[cfg(feature = "assets")]
{
    let sql = ubl_auth::assets::sqlite_migration_001();
    let schema = ubl_auth::assets::receipt_action_v1_schema();
    println!("{}", &sql[..60]);
    println!("{}", &schema[..60]);
}
```

## License
Dual-licensed under **MIT** or **Apache-2.0**.


**Note:** the simple `verify_ed25519_jwt_with_jwks` example checks issuer/audience and signature.
For production, also enforce `exp`/`nbf`/clock-skew per your threat model.
