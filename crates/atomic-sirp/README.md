# ubl-sirp 0.3.1

*P3 entregue*: **HMAC Edge↔Lab**, idempotência SQLite c/ TTL, recibo assinado, client com timeout/backoff, cabeçalhos de métricas.  
*P4 (roadmap)*: libp2p QUIC/Noise + Kademlia, TTL/PoW na cápsula, HTTP2 fallback.

## Segurança & Testes

- **Limites DoS**: Frames > 1 MiB rejeitados; varints > 10 bytes rejeitados
- **Testes adversariais**: 27 testes cobrindo headers malformados, TLV truncado, assinaturas inválidas
- **Property tests**: invariantes de round-trip com proptest
- **Fuzzing**: target `fuzz_sirp_decode` integrado ao CI

```bash
cargo test --test wire_adversarial
```