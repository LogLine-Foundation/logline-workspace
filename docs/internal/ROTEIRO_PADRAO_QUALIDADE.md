# 🎯 Roteiro Padrão de Qualidade — Publicação de Crates

Este documento define o padrão mínimo e completo para publicação de crates no ecossistema LogLine Foundation.

> **🔍 Verificação Automatizada**: Use os scripts em `scripts/` para verificar automaticamente se uma crate atende a este padrão:
> - `bash scripts/verify_quality.sh <crate_dir>` - Verificação completa de uma crate
> - `bash scripts/verify_all_crates.sh` - Verifica todas as crates do repositório
> - `python3 scripts/verify_quality_python.py <crate_dir>` - Versão Python (mais robusta)
> - Ver documentação completa: `scripts/README.md`

---

## 📋 Fase 1: Estrutura Básica do Projeto

### 1.1 Arquivos Obrigatórios na Raiz

```bash
Cargo.toml          # Configuração do pacote
README.md           # Documentação principal
LICENSE             # MIT (padrão)
.gitignore          # Ignora target/, .env, etc.
```

### 1.2 Arquivos Recomendados

```bash
CHANGELOG.md        # Histórico de mudanças (Keep a Changelog)
CITATION.cff        # Citação acadêmica (se aplicável)
SECURITY.md         # Política de segurança
CODE_OF_CONDUCT.md  # Código de conduta (Contributor Covenant v2.1)
deny.toml           # Configuração cargo-deny
```

---

## 📋 Fase 2: Configuração do Cargo.toml

### 2.1 Metadados Mínimos

```toml
[package]
name = "nome-da-crate"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Descrição clara e concisa"
repository = "https://github.com/LogLine-Foundation/nome-da-crate"
homepage = "https://logline.foundation"
readme = "README.md"
keywords = ["keyword1", "keyword2", "keyword3"]
categories = ["cryptography", "encoding"]  # Escolher do crates.io
rust-version = "1.75"
resolver = "2"
documentation = "https://docs.rs/nome-da-crate"
exclude = [".github/**", "deny.toml", "SECURITY.md", "CODE_OF_CONDUCT.md", "CHANGELOG.md"]
```

### 2.2 Features

```toml
[features]
default = ["std"]  # ou ["std", "manifest"] se tiver manifest
std = []
alloc = []         # Se suportar no_std
# outras features específicas
```

### 2.3 Docs.rs

**Configuração para documentação automática:**

```toml
[package.metadata.docs.rs]
features = ["std"]  # ou ["std", "manifest"] - features a usar na docs.rs
no-default-features = false  # se false, usa default features
all-features = false  # se true, documenta todas as features (pode ser lento)
```

**Notas importantes:**
- A docs.rs compila automaticamente sua crate após publicação no crates.io
- Use `features` para especificar quais features documentar (evita builds muito longos)
- Se usar `no_std`, configure adequadamente para documentação correta
- Documentação inline (`///`) é automaticamente incluída
- Exemplos em `examples/` aparecem na documentação

---

## 📋 Fase 3: Estrutura de Código

### 3.1 Diretórios Mínimos

```
crate-name/
├── src/           # Código fonte
├── tests/         # Testes de integração (mínimo 2 arquivos)
├── examples/      # Exemplos de uso (mínimo 1 arquivo)
└── benches/       # Benchmarks (opcional, mas recomendado)
```

### 3.2 Testes

**Mínimo:**
- 2 arquivos de teste em `tests/`
- Testes unitários no código (`#[cfg(test)]`)

**Recomendado:**
- Testes de integração
- Testes de ataque/segurança (se aplicável)
- Testes de edge cases

### 3.3 Exemplos

**Mínimo:**
- 1 exemplo funcional em `examples/`
- Documentado no README

**Recomendado:**
- Múltiplos exemplos cobrindo casos de uso principais
- Exemplo mínimo e exemplo completo

---

## 📋 Fase 4: Documentação

### 4.1 README.md

**Estrutura Mínima:**
```markdown
# Nome da Crate

[![crates.io](https://img.shields.io/crates/v/nome-da-crate.svg)](https://crates.io/crates/nome-da-crate)
[![docs.rs](https://docs.rs/nome-da-crate/badge.svg)](https://docs.rs/nome-da-crate)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)

## Descrição
Breve descrição do que a crate faz.

## Instalação
```toml
[dependencies]
nome-da-crate = "0.1.0"
```

## Quickstart
Código de exemplo mínimo funcional.

## API
Lista das principais APIs públicas.

## Licença
MIT © LogLine Foundation
```

**Badges Recomendados (Shields.io):**

```markdown
[![crates.io](https://img.shields.io/crates/v/nome-da-crate.svg)](https://crates.io/crates/nome-da-crate)
[![docs.rs](https://docs.rs/nome-da-crate/badge.svg)](https://docs.rs/nome-da-crate)
![CI](https://img.shields.io/github/actions/workflow/status/LogLine-Foundation/nome-da-crate/ci.yml?label=CI)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![no_std](https://img.shields.io/badge/no__std-ready-success)  # Se suportar no_std
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![downloads](https://img.shields.io/crates/d/nome-da-crate)  # Opcional
```

**Recomendado:**
- Badges adicionais (CI, no_std, downloads)
- Seção de segurança
- Seção de supply-chain
- Links para documentação
- Exemplos de uso mais completos
- Seção de "Contribuindo" (link para CONTRIBUTING.md)

### 4.2 CHANGELOG.md

Formato: [Keep a Changelog](https://keepachangelog.com/)

```markdown
# Changelog
Todas as mudanças notáveis deste projeto serão documentadas aqui.
Formato: [Keep a Changelog](https://keepachangelog.com/) — SemVer.

## [Unreleased]
- Itens planejados para próxima versão

## [0.1.0] - YYYY-MM-DD
### Adicionado
- Feature 1
- Feature 2
```

---

## 📋 Fase 5: Segurança e Qualidade

### 5.1 Segurança no Código

**Evitar código unsafe:**
```rust
#![forbid(unsafe_code)]  // Adicionar no topo de lib.rs
```

**Auditoria de dependências:**
```bash
# Instalar cargo-audit
cargo install cargo-audit

# Verificar vulnerabilidades
cargo audit
```

**Verificar código inseguro nas dependências:**
```bash
# Verificar uso de unsafe nas dependências
cargo geiger  # Requer instalação: cargo install cargo-geiger
```

### 5.2 deny.toml

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
ignore = []

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC", "Unicode-DFS-2016", "Zlib", "CC0-1.0"]
deny  = []
copyleft = "warn"
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "deny"
```

### 5.2 SECURITY.md

```markdown
# Security Policy

- Reporte vulnerabilidades por **issue privada** ou e-mail da organização.
- Evite PoCs destrutivas em produção.
- Releases incluem **cargo-audit**, **cargo-deny** e **SBOM** (CycloneDX).
```

### 5.3 CODE_OF_CONDUCT.md

Usar Contributor Covenant v2.1 (copiar de uma crate existente).

---

## 📋 Fase 6: CI/CD (GitHub Actions)

### 6.1 Workflow Mínimo: CI

`.github/workflows/ci.yml`:
```yaml
name: CI
on:
  push: { branches: ["main"] }
  pull_request: {}
jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all-features
```

### 6.2 Workflows Recomendados

**audit.yml** (Security Audit):
```yaml
name: Security Audit
on:
  push: {}
  pull_request: {}
  schedule:
    - cron: "0 5 * * 1"
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit
      - run: cargo audit
```

**deny.yml** (License/Advisory):
```yaml
name: License/Advisory Deny
on:
  push: {}
  pull_request: {}
jobs:
  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-deny
      - run: cargo deny check all
```

**sbom.yml** (SBOM Generation):
```yaml
name: SBOM
on:
  release:
    types: [published]
jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cyclonedx
      - run: cargo cyclonedx -o sbom.json
      - uses: softprops/action-gh-release@v2
        with:
          files: sbom.json
```

---

## 📋 Fase 7: Templates GitHub

### 7.1 Issue Templates

`.github/ISSUE_TEMPLATE/bug_report.md`:
```markdown
---
name: Bug report
about: Reportar um bug
labels: bug
---

**Descrição**
O que aconteceu?

**Passos para reproduzir**
1.
2.

**Ambiente**
- OS / Rust:
- Versão do crate:

**Logs/Stacktrace**
```
```
```

`.github/ISSUE_TEMPLATE/feature_request.md`:
```markdown
---
name: Feature request
about: Sugerir melhoria/feature
labels: enhancement
---

**Motivação**
Por que isso é útil?

**Proposta**
O que mudar/adicionar?

**Impacto**
Breakings? Compatibilidade?
```

`.github/ISSUE_TEMPLATE/config.yml`:
```yaml
blank_issues_enabled: false
contact_links:
  - name: Docs
    url: https://docs.rs/nome-da-crate
    about: Documentação do crate
```

### 7.2 Pull Request Template

`.github/pull_request_template.md`:
```markdown
## Resumo

## Tipo
- [ ] Feature
- [ ] Fix
- [ ] Docs
- [ ] Maintenance

## Checklist
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-features`
- [ ] Atualizei o CHANGELOG (se aplicável)
```

---

## 📋 Fase 8: Validação Pré-Publicação

### 8.1 Checklist de Validação

```bash
# 1. Formatação
cargo fmt --all -- --check

# 2. Linting (com warnings como erros)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Testes (todas as features)
cargo test --all-features

# 4. Build no_std (se aplicável)
cargo build --no-default-features --features alloc

# 5. Auditoria de segurança
cargo audit  # Verificar vulnerabilidades conhecidas

# 6. Verificar licenças e dependências
cargo deny check all  # Se usar cargo-deny

# 7. Empacotamento
cargo package --list  # Verificar arquivos incluídos (não deve incluir .git, target, etc)
cargo package         # Testar empacotamento

# 8. Dry-run publicação (OBRIGATÓRIO)
cargo publish --dry-run

# 9. Verificar documentação (opcional mas recomendado)
cargo doc --no-deps --open  # Ver como ficará na docs.rs
```

### 8.2 Verificações de Segurança Adicionais

```bash
# Verificar uso de unsafe no código
grep -r "unsafe" src/  # Deve ser mínimo ou zero

# Verificar dependências desnecessárias
cargo tree  # Visualizar árvore de dependências

# Verificar tamanho do pacote
cargo package --list | wc -l  # Número de arquivos
du -sh target/package/nome-da-crate-0.1.0/  # Tamanho do pacote
```

### 8.3 Verificações Manuais

- [ ] README.md está completo e atualizado
- [ ] CHANGELOG.md tem entrada para a versão
- [ ] Todos os exemplos compilam e funcionam
- [ ] Testes passam (incluindo testes de integração)
- [ ] Cargo.toml tem todos os metadados obrigatórios
- [ ] exclude está configurado corretamente (não incluir .git, target, etc)
- [ ] Workflows CI estão configurados e passando
- [ ] Documentação inline (`///`) está completa
- [ ] Licença está correta e presente
- [ ] Repository URL está correto
- [ ] Keywords e categories são relevantes
- [ ] Sem código `unsafe` desnecessário
- [ ] Dependências são mínimas e confiáveis
- [ ] Versão segue SemVer corretamente

---

## 📋 Fase 9: Publicação

### 9.1 Políticas do crates.io

**Importante saber:**
- ✅ **Sem curadoria**: crates.io não revisa crates antes da publicação
- ✅ **Versões permanentes**: versões publicadas não podem ser removidas
- ✅ **Yank disponível**: versões problemáticas podem ser "yanked" (não removidas, mas marcadas)
- ✅ **Nomes únicos**: nomes de crates são únicos e permanentes
- ✅ **Sem revisão de código**: responsabilidade do mantenedor

**Requisitos mínimos:**
- Crate deve compilar (`cargo build` passa)
- `Cargo.toml` válido com metadados mínimos
- Licença especificada
- Descrição presente

### 9.2 Publicação no crates.io

**Método 1: Manual (tradicional)**

```bash
# 1. Verificar login
cargo login  # Se necessário (gera token em https://crates.io/settings/tokens)

# 2. Dry-run (OBRIGATÓRIO antes de publicar)
cargo publish --dry-run

# 3. Verificar arquivos incluídos
cargo package --list

# 4. Publicar
cargo publish
```

**Método 2: Trusted Publishing (Recomendado - 2024+)**

Use GitHub Actions com OIDC para publicação automática e segura:

```yaml
name: Publish to crates.io

on:
  push:
    tags: ['v*']  # Publica quando tag v* é criada

jobs:
  publish:
    runs-on: ubuntu-latest
    environment: release  # Configurar no GitHub com permissões crates.io
    permissions:
      id-token: write  # Necessário para OIDC
      contents: read
    steps:
      - uses: actions/checkout@v4
      - uses: rust-lang/crates-io-auth-action@v1
        id: auth
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

**Configuração do Trusted Publishing:**
1. Vá para https://crates.io/settings/publishing
2. Adicione GitHub como publisher
3. Configure o environment no GitHub com permissões crates.io
4. Use o workflow acima

**Vantagens:**
- ✅ Sem tokens de longa duração
- ✅ Publicação automática via CI
- ✅ Mais seguro (OIDC)
- ✅ Auditável

### 9.3 Gerenciamento de Versões

**Yank (remover versão problemática):**
```bash
cargo yank --version 0.1.0 nome-da-crate
# ou para desfazer:
cargo yank --undo --version 0.1.0 nome-da-crate
```

**Nota:** Versões yanked não podem ser baixadas por novos projetos, mas projetos existentes continuam funcionando.

**Versionamento Semântico (SemVer):**
- `MAJOR.MINOR.PATCH` (ex: 1.2.3)
- MAJOR: breaking changes
- MINOR: novas features compatíveis
- PATCH: correções de bugs compatíveis

### 9.2 GitHub

```bash
# 1. Commit final
git add -A
git commit -m "nome-da-crate v0.1.0 — descrição"

# 2. Tag
git tag -a v0.1.0 -m "nome-da-crate v0.1.0"

# 3. Push
git push origin main --tags

# 4. Criar Release (via gh CLI)
gh release create v0.1.0 \
  --title "nome-da-crate v0.1.0 — Título" \
  --notes-file RELEASE_NOTES.md \
  --repo LogLine-Foundation/nome-da-crate
```

### 9.3 RELEASE_NOTES.md

```markdown
# nome-da-crate v0.1.0 — Título

## Highlights
- Feature 1
- Feature 2

## Security
- Notas de segurança relevantes

## Dependencies
- Lista de dependências principais
```

---

## 📊 Níveis de Qualidade

### 🥉 Nível Mínimo (Básico)
- ✅ Cargo.toml completo
- ✅ README.md básico
- ✅ LICENSE
- ✅ 2+ testes
- ✅ 1+ exemplo
- ✅ CI básico (fmt, clippy, test)

### 🥈 Nível Recomendado (Padrão)
- ✅ Tudo do nível mínimo
- ✅ CHANGELOG.md
- ✅ SECURITY.md
- ✅ CODE_OF_CONDUCT.md
- ✅ deny.toml
- ✅ CI + audit + deny
- ✅ Templates GitHub

### 🥇 Nível Completo (Ideal)
- ✅ Tudo do nível recomendado
- ✅ SBOM workflow
- ✅ Benchmarks
- ✅ Testes de ataque/segurança
- ✅ Documentação expandida
- ✅ Múltiplos exemplos
- ✅ RELEASE_NOTES.md

---

## 🎯 Checklist Rápido por Crate

Copie e cole este checklist para cada nova crate:

### Estrutura e Configuração
```
[ ] Cargo.toml completo (metadados, features, docs.rs, exclude)
[ ] README.md com badges e quickstart
[ ] CHANGELOG.md (Keep a Changelog)
[ ] LICENSE (MIT)
[ ] SECURITY.md
[ ] CODE_OF_CONDUCT.md
[ ] deny.toml
[ ] .gitignore
[ ] CITATION.cff (se aplicável)
```

### Código e Testes
```
[ ] 2+ arquivos de teste em tests/
[ ] 1+ exemplo funcional em examples/
[ ] Código sem unsafe desnecessário (#![forbid(unsafe_code)])
[ ] Documentação inline completa (///)
[ ] Benchmarks (opcional mas recomendado)
```

### CI/CD e Workflows
```
[ ] CI workflow (ci.yml) - fmt, clippy, test
[ ] Audit workflow (audit.yml) - cargo-audit
[ ] Deny workflow (deny.yml) - cargo-deny
[ ] SBOM workflow (sbom.yml) - cargo-cyclonedx
[ ] Trusted Publishing configurado (opcional mas recomendado)
```

### Templates GitHub
```
[ ] Issue templates (bug_report.md, feature_request.md, config.yml)
[ ] PR template (pull_request_template.md)
```

### Validação
```
[ ] cargo fmt --all -- --check ✓
[ ] cargo clippy --all-targets --all-features -- -D warnings ✓
[ ] cargo test --all-features ✓
[ ] cargo audit ✓
[ ] cargo deny check all ✓
[ ] cargo build --no-default-features --features alloc ✓ (se aplicável)
[ ] cargo package --list (verificar arquivos) ✓
[ ] cargo publish --dry-run ✓
[ ] cargo doc --no-deps (verificar documentação) ✓
```

### Publicação
```
[ ] Publicado no crates.io ✓
[ ] Verificado em https://crates.io/crates/nome-da-crate ✓
[ ] Docs.rs compilou (verificar após 10-30 min) ✓
[ ] Tag criada no Git ✓
[ ] Release criado no GitHub ✓
[ ] RELEASE_NOTES.md anexado ao release ✓
```

### Pós-Publicação
```
[ ] Monitorar downloads e dependentes
[ ] Responder issues/PRs
[ ] Manter dependências atualizadas
```

---

## 📋 Fase 10: Pós-Publicação

### 10.1 Verificações Pós-Publicação

```bash
# 1. Verificar se apareceu no crates.io (pode levar alguns minutos)
# Visitar: https://crates.io/crates/nome-da-crate

# 2. Verificar se docs.rs compilou (pode levar 10-30 minutos)
# Visitar: https://docs.rs/nome-da-crate

# 3. Verificar se dependentes podem usar
cargo search nome-da-crate  # Deve aparecer na busca
```

### 10.2 Monitoramento

**Métricas importantes:**
- Downloads (disponível em crates.io)
- Dependents (quem usa sua crate)
- Issues e PRs no GitHub
- Vulnerabilidades reportadas (via cargo-audit)

**Ferramentas úteis:**
```bash
# Ver dependentes da sua crate
# Visitar: https://crates.io/crates/nome-da-crate/reverse_dependencies

# Monitorar downloads
# Dashboard em: https://crates.io/me
```

### 10.3 Manutenção Contínua

- ✅ Responder issues e PRs prontamente
- ✅ Manter dependências atualizadas
- ✅ Executar `cargo audit` regularmente
- ✅ Atualizar CHANGELOG.md em cada release
- ✅ Manter documentação atualizada
- ✅ Monitorar vulnerabilidades (RustSec)

---

## 🚫 Fase 10: Anti-Padrões — O Que NÃO Deve Estar

Esta fase verifica itens que **não devem** estar em crates publicadas.

### 10.1 Arquivos Proibidos no Repositório

**❌ ERRO CRÍTICO** (bloqueia publicação):
- `target/` — Diretório de build (deve estar no `.gitignore`)
- `.env`, `.env.local` — Variáveis de ambiente com secrets
- Arquivos com secrets/credenciais hardcoded

**⚠️ WARNING** (não recomendado):
- `.DS_Store` (macOS)
- `Thumbs.db` (Windows)
- `.idea/`, `.vscode/` — Configurações de IDE (devem estar no `.gitignore`)
- `*.iml` — Arquivos de configuração do IntelliJ

### 10.2 Arquivos Grandes Desnecessários

- Arquivos >1MB que não sejam documentação (`.md`, `.pdf`) ou imagens (`.png`, `.jpg`)
- Binários desnecessários
- Arquivos de cache ou temporários

### 10.3 Dependências Não Utilizadas

- Usar `cargo-udeps` para detectar dependências não utilizadas
- Remover dependências órfãs do `Cargo.toml`

### 10.4 Secrets e Credenciais

**NUNCA** incluir:
- Passwords hardcoded
- API keys hardcoded
- Tokens de acesso
- Chaves privadas

**Verificação automática**: O script procura por padrões comuns como:
- `password = "..."`
- `api_key = "..."`
- `secret = "..."`
- `token = "..."`

### 10.5 Código Comentado em Excesso

- Remover código comentado extenso (exceto documentação `///`)
- Manter apenas comentários relevantes

### 10.6 Features Não Utilizadas

- Verificar se todas as features declaradas são utilizadas
- Remover features órfãs

### 10.7 Checklist de Anti-Padrões

```bash
# Verificar manualmente
find . -name "target" -type d
find . -name ".env*" -type f
find . -name ".DS_Store" -type f
find . -name ".idea" -type d
find . -name ".vscode" -type d
find . -size +1M ! -name "*.md" ! -name "*.pdf"

# Verificar dependências não utilizadas
cargo install cargo-udeps
cargo udeps --all-targets --all-features
```

### 10.8 .gitignore Recomendado

```gitignore
# Rust
/target
**/*.rs.bk
Cargo.lock  # Para bibliotecas (não para bins)

# IDEs
.idea/
.vscode/
*.iml

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.*
!.env.example

# Build artifacts
*.o
*.so
*.dylib
*.dll
*.exe

# Logs
*.log
```

---

## 🔗 Fase 11: Dependências Crescentes e Acumulativas

Esta fase verifica que as dependências entre crates seguem uma ordem **crescente e acumulativa**, garantindo que a ordem de publicação está correta.

### 11.1 Hierarquia de Dependências Esperada

Para o projeto LogLine Foundation (7 papers, múltiplas crates):

| Paper | Crate | Dependências Diretas |
|-------|-------|----------------------|
| **I** | `logline-core` | *(BASE - sem dependências internas)* |
| **II** | `json_atomic` | `logline-core` |
| **III** | `lllv-core` | `json_atomic` *(logline-core vem transitivamente)* |
| **III** | `lllv-index` | `lllv-core`, `json_atomic` *(opcional)* |
| **IV+** | *[futuras crates]* | *[dependências acumulativas]* |

**Nota**: `lllv-index` não precisa declarar `logline-core` diretamente, pois já o obtém transitivamente via `lllv-core`. O Rust dará erro se uma dependência for declarada mas não usada.

### 11.2 Regras de Dependências

1. **Ordem Crescente**: Cada crate só pode depender de crates anteriores na hierarquia
2. **Acumulativa**: Dependências se acumulam (crate N depende de todas anteriores)
3. **Sem Ciclos**: Não pode haver dependências circulares
4. **Versões Corretas**: Dependências devem usar versões publicadas (não `path =` em produção)

### 11.2.1 Dependências Diretas vs Transitivas ⚠️ IMPORTANTE

**Diferença Crucial**: Há uma distinção importante entre dependências **diretas** e **transitivas**:

#### Quando Declarar Dependência Direta (Obrigatório)

Uma crate **deve** declarar uma dependência diretamente se:

1. **Usa tipos/funções diretamente no código**:
   ```rust
   // Se você usa assim no código:
   use json_atomic::SignedFact;
   json_atomic::seal_value(...)
   // Então PRECISA declarar json_atomic diretamente
   ```

2. **Precisa de versão específica** independente da transitiva:
   ```toml
   # Se precisa de versão diferente da que vem transitivamente
   logline-core = { version = "0.2.0" }  # mas transitivo traz 0.1.0
   ```

3. **Dependência opcional (feature)** que precisa estar disponível quando ativa:
   ```toml
   [features]
   manifest = ["json_atomic"]  # precisa declarar json_atomic diretamente
   ```

#### Quando NÃO Declarar (Transitiva é Suficiente)

Uma crate **não precisa** declarar uma dependência diretamente se:

1. **Dependência já vem transitivamente** via outra dependência
2. **Não usa tipos/funções diretamente** da dependência transitiva
3. **Versão transitiva é suficiente** para as necessidades

**Exemplo Prático**:
- `lllv-index` declara `json_atomic` diretamente ✅ (usa `json_atomic::SignedFact`)
- `lllv-index` NÃO declara `logline-core` diretamente ✅ (vem via `lllv-core`)
- `lllv-core` NÃO precisa declarar `logline-core` diretamente ✅ (vem via `json_atomic`)

#### Verificação e Alertas

**⚠️ IMPORTANTE**: Os testes de qualidade:
- ✅ **NÃO acusam ERRO** por dependências transitivas
- ⚠️ **Emitem ALERTAS (warnings)** para revisão manual
- ⚠️ Alertam sobre dependências inesperadas (mas podem ser válidas)
- ⚠️ Alertam sobre dependências transitivas declaradas diretamente (redundantes)

**Regra de Ouro**: Se você usa `crate::Type` ou `crate::function()` no código, declare diretamente. Caso contrário, a transitiva é suficiente.

### 11.3 Verificação Automatizada

Execute o script dedicado:

```bash
# Verificar todas as crates do repositório
bash scripts/verify_dependencies.sh .

# O script verifica:
# - Ordem crescente de dependências
# - Dependências circulares
# - Versões corretas (não path em produção)
# - Dependências faltando ou inesperadas
```

### 11.4 Checklist de Dependências

Para cada crate:

```bash
[ ] Dependências internas estão na ordem crescente correta
[ ] Não há dependências circulares
[ ] Versões das dependências estão corretas (não path)
[ ] Todas as dependências esperadas estão presentes
[ ] Não há dependências inesperadas (que quebram a ordem)
```

### 11.5 Exemplo de Cargo.toml Correto

```toml
# json_atomic (Paper II) - depende de logline-core
[dependencies]
logline-core = { version = "0.1.0", features = ["serde"] }

# lllv-core (Paper III) - depende de logline-core e json_atomic
[dependencies]
logline-core = { version = "0.1.1", features = ["serde"] }
json_atomic = { version = "0.1.1", optional = true }

# lllv-index (Paper III) - depende de lllv-core e json_atomic
[dependencies]
lllv-core = "0.1.0"
json_atomic = { version = "0.1.1", optional = true }
```

### 11.6 Ordem de Publicação

A ordem de publicação **deve seguir** a hierarquia de dependências:

1. **logline-core** (base) → publicar primeiro
2. **json_atomic** → publicar depois (depende de logline-core)
3. **lllv-core** → publicar depois (depende de logline-core e json_atomic)
4. **lllv-index** → publicar depois (depende de lllv-core e json_atomic)
5. **Futuras crates** → seguir ordem crescente

### 11.7 Erros Comuns

❌ **Dependência Circular**: `crate-a` depende de `crate-b` e `crate-b` depende de `crate-a`
❌ **Ordem Invertida**: `crate-base` depende de `crate-avancada`
❌ **Path em Produção**: Usar `path = "../crate"` em vez de `version = "x.y.z"` em publicação
❌ **Dependência Faltando**: Crate não declara dependência que usa

---

## 📚 Referências e Recursos

### Documentação Oficial
- [crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [docs.rs Documentation](https://docs.rs/about)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Contributor Covenant](https://www.contributor-covenant.org/)

### Ferramentas de Segurança
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) - Auditoria de vulnerabilidades
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) - Verificação de licenças e advisories
- [cargo-cyclonedx](https://github.com/CycloneDX/cargo-cyclonedx) - Geração de SBOM
- [cargo-geiger](https://github.com/rust-secure-code/cargo-geiger) - Detecção de unsafe

### Badges e Shields
- [Shields.io](https://shields.io/) - Gerador de badges
- [crates.io Badge](https://shields.io/category/version)
- [docs.rs Badge](https://docs.rs/badge.svg)

### CI/CD
- [Trusted Publishing](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/)
- [GitHub Actions for Rust](https://github.com/actions-rs)

### Políticas e Boas Práticas
- [crates.io Usage Policy](https://blog.rust-lang.org/2023/09/22/crates-io-usage-policy-rfc/)
- [Rust Security Best Practices](https://crates.guide/article/Rust_package_security_Best_practices.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
