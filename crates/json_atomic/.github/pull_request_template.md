## Descrição

Descrição clara e concisa das mudanças propostas.

## Tipo de mudança

Marque o que se aplica:

- [ ] Bug fix (mudança que corrige um problema)
- [ ] Nova feature (mudança que adiciona funcionalidade)
- [ ] Breaking change (mudança que quebra compatibilidade)
- [ ] Documentação (mudança apenas em docs)
- [ ] Refatoração (mudança de código sem mudança de comportamento)

## Checklist

- [ ] Código segue o estilo do projeto (`cargo fmt --all`)
- [ ] Clippy sem warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Testes adicionados/atualizados (`cargo test --all-features`)
- [ ] Documentação atualizada (doc comments, README se necessário)
- [ ] CHANGELOG.md atualizado (se aplicável)
- [ ] Compatibilidade `no_std` mantida (se aplicável)

## Testes

Como foi testado? Inclua comandos relevantes:

```bash
cargo test --all-features
cargo build --no-default-features --features alloc
```

## Breaking Changes

Se houver breaking changes, descreva aqui e inclua instruções de migração.

## Screenshots/Exemplos

Se aplicável, inclua screenshots ou exemplos de código.
