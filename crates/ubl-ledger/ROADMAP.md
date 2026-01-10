## P3 (0.3.0) — Feito
- `UblWriterAsync` com canal e backpressure
- WAL: escreve `*.wal` e renomeia atômico
- `verify_file_with_chain(strict=true)`
- Rotação por tamanho/hora; zstd opcional

## P4 (0.4.0) — Planejado
- Mirror R2 (S3) com checkpoints
- Merkle por janela + `ubl.merkle_root`
- Políticas de retenção/compaction
- Reader HTTP de `.ndjson.zst`