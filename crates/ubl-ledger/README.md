# ubl-ledger 0.3.1

*P3 entregue*: `UblWriterAsync` (tokio, mpsc), WAL simples (`*.wal`), verificação com `strict_chain`, zstd feature.  
*P4 (roadmap)*: mirror R2, Merkle root/anchor, compaction, tail HTTP.

## Segurança & Testes

- **Limites DoS**: Entradas com CID/sig inválidos rejeitadas; frames > 1 MiB rejeitados
- **Property tests**: 21 testes de invariantes (CID integrity, signature validation, stress tests)
- **Fuzzing**: target `fuzz_ubl_entry` integrado ao CI

```bash
cargo test --test ledger_prop
```