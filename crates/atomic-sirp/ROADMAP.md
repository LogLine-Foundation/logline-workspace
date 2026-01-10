## P3 (0.3.0) — Feito
- `client::post_capsule_hmac(url, capsule, hmac_key)` com retry/backoff
- `server::router(...).with_hmac(key)`
- Idempotência SQLite (TTL) + recibo assinado

## P4 (0.4.0) — Planejado
- Transporte libp2p real (QUIC/Noise, provider records)
- Fallback HTTP2 e cabeçalho TTL/PoW opcional