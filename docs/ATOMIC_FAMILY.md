# Atomic Family (v0.3.0)

- **atomic-types** — common identifiers, DIM helpers, time utilities, and validation errors shared across the stack.
- **atomic-crypto** — crypto helpers (BLAKE3, Ed25519, HMAC, key IDs) with safe key handling.
- **atomic-codec** — JSON✯Atomic canonical encode/decode helpers plus guard/reader utilities.
- **ubl-ledger** — UBL writer (NDJSON), daily rotation, signing, and path/event helpers.
- **atomic-sirp** — network atom: DIM capsule + payload parsing, receipts, idempotency; HTTP transport support.
- **atomic-runtime** — router/handler runtime that dispatches DIM to handlers with hooks and UBL logging.
- **atomic-cli** — lightweight CLI for sending capsules, validating, operating UBL, and dev/server helpers.
