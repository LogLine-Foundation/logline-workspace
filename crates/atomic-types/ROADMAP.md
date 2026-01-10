# Roadmap
## P3 (0.3.0) — Feito
- `Dim::{from_hex,to_hex}`
- Macro `newtype_id!` com validação opcional
- `gen::new_ulid_*` sob feature `ulid`

## P4 (0.4.0) — Planejado
- Exportar schemas (`schemars`) e `ts-rs` p/ tipos TS
- Tabela canônica de Dims reservados com gerador
- PoD: `cargo test -F schemars,ts-rs` gera `types.d.ts`