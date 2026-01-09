# 🎯 Tasklist Completa — Padronização das 4 Crates

Este documento lista todas as tarefas necessárias para elevar as 4 crates publicadas ao **padrão completo de qualidade** estabelecido.

---

## 🔍 Resultados da Verificação Automatizada

> **Última verificação**: Executada com `scripts/verify_quality.sh` (incluindo **Fase 9: Anti-Padrões** e **Fase 10: Dependências**) em todas as 4 crates
> 
> **Última atualização**: Correções aplicadas - `json_atomic` (rust-version), `lllv-core` (exclude + example), `logline-core` (deny.toml)
> 
> **Status**: ✅ **TODAS AS 4 CRATES ATENDEM AO PADRÃO MÍNIMO!** (0 erros, apenas warnings não críticos)

### 📊 Resumo Geral

| Crate | ✅ Passou | ⚠️ Warnings | ❌ Erros | Status |
|-------|----------|-------------|----------|--------|
| **logline-core** | 27 | 1 | 0 | ✅ Atende padrão mínimo |
| **json_atomic** | 15 | 2 | 0 | ✅ **CORRIGIDO** - Atende padrão mínimo |
| **lllv-core** | 50 | 3 | 0 | ✅ **CORRIGIDO** - Atende padrão mínimo |
| **lllv-index** | 52 | 4 | 0 | ✅ Atende padrão mínimo |

### ❌ Erros Críticos Encontrados

#### json_atomic
- ~~**❌ Campo 'rust-version' no Cargo.toml** (OBRIGATÓRIO - padrão não encontrado)~~
  - ✅ **CORRIGIDO**: `rust-version = "1.75"` adicionado no `[package]`

### ⚠️ Warnings Encontrados

#### logline-core
- ~~**⚠️ deny.toml** (recomendado - faltando)~~
  - ✅ **CORRIGIDO**: `deny.toml` criado com configuração padrão

#### lllv-core
- ~~**⚠️ Campo 'exclude' no Cargo.toml** (recomendado - padrão não encontrado)~~
  - ✅ **CORRIGIDO**: `exclude` adicionado no `Cargo.toml`

#### lllv-index
- **⚠️ Campo 'exclude' no Cargo.toml** (recomendado - padrão não encontrado)
  - **Ação**: Verificar se `exclude` está presente (pode estar em formato diferente)

### 📋 Detalhamento por Fase

#### Fase 1: Estrutura Básica
- ✅ Todas as crates têm: Cargo.toml, README.md, LICENSE, .gitignore
- ✅ Todas têm CHANGELOG.md e CITATION.cff (recomendados)

#### Fase 2: Configuração Cargo.toml
- ✅ Todas têm: name, version, edition, license, description, repository, readme, documentation
- ✅ **json_atomic**: `rust-version` adicionado ✅ **CORRIGIDO**
- ✅ **lllv-core**: `exclude` adicionado ✅ **CORRIGIDO**
- ⚠️ **lllv-index**: `exclude` presente (verificação pode estar com padrão diferente)

#### Fase 3: Estrutura de Código
- ✅ Todas têm: src/ (com arquivos .rs), tests/ (≥2 arquivos), examples/ (≥1 arquivo)
- ⚠️ Nenhuma tem benches/ (opcional, mas recomendado)

#### Fase 4: Segurança e Qualidade
- ✅ Todas têm: `#![forbid(unsafe_code)]` no lib.rs
- ✅ **logline-core**: `deny.toml` criado ✅ **CORRIGIDO**
- ✅ Todas têm SECURITY.md e CODE_OF_CONDUCT.md (onde aplicável)

#### Fase 5: CI/CD e Workflows
- ✅ **lllv-core** e **lllv-index**: 4 workflows completos (CI, audit, deny, SBOM)
- ⚠️ **logline-core**: Apenas CI workflow (faltam: audit, deny, SBOM)
- ⚠️ **json_atomic**: Apenas CI + release (faltam: audit, deny, SBOM)

#### Fase 6: Templates GitHub
- ✅ **json_atomic** e **lllv-index**: Templates completos
- ⚠️ **logline-core** e **lllv-core**: Faltam templates

#### Fase 7: Documentação
- ✅ Todas têm README.md com badges (≥3 badges)
- ✅ Todas têm seções de Instalação e Quickstart/Exemplo
- ✅ Todas têm RELEASE_NOTES.md (onde aplicável)

#### Fase 8: Validação de Código
- ✅ Todas passam: `cargo fmt --all -- --check`
- ⚠️ Algumas têm warnings no `cargo clippy` (não crítico)
- ✅ Todas passam: `cargo test --all-features`

#### Fase 9: Anti-Padrões (O Que NÃO Deve Estar) ⭐ NOVO
- ✅ **Todas as crates**: Nenhum arquivo proibido encontrado (target/, .env, etc.)
- ✅ **Todas as crates**: Nenhum secret/credencial hardcoded detectado
- ✅ **Todas as crates**: Nenhum arquivo grande desnecessário encontrado
- ⚠️ **Todas as crates**: cargo-udeps não instalado (recomendado para verificar dependências não utilizadas)

#### Fase 10: Dependências Crescentes e Acumulativas ⭐ NOVO
- ✅ **Todas as crates**: Dependências na ordem crescente correta
  - `logline-core` (BASE) → sem dependências internas
  - `json_atomic` → depende de `logline-core`
  - `lllv-core` → depende de `logline-core`, `json_atomic`
  - `lllv-index` → depende de `lllv-core`, `json_atomic`
- ✅ **Todas as crates**: Nenhuma dependência circular detectada
- ✅ **Todas as crates**: Versões corretas (não `path =` em produção)
- 📝 **Verificação completa**: Execute `bash scripts/verify_dependencies.sh .`

---

## 📊 Status Atual vs Padrão Completo

| Item | logline-core | json_atomic | lllv-core | lllv-index | Padrão |
|------|--------------|-------------|-----------|------------|--------|
| **Workflows** | 1 (CI) | 2 (CI+release) | 4 (completo) | 4 (completo) | 4 (CI+audit+deny+SBOM) |
| **Templates** | ❌ | ✅ | ❌ | ✅ | ✅ |
| **deny.toml** | ❌ | ❌ | ✅ | ✅ | ✅ |
| **SECURITY.md** | ✅ | ❌ | ✅ | ✅ | ✅ |
| **CODE_OF_CONDUCT.md** | ✅ | ❌ | ✅ | ✅ | ✅ |
| **RELEASE_NOTES.md** | ❌ | ❌ | ✅ | ✅ | ✅ |
| **Examples** | ✅ (2) | ✅ (3) | ❌ | ✅ (1) | ✅ (1+) |
| **Badges README** | ⚠️ (básico) | ⚠️ (básico) | ⚠️ (básico) | ✅ (completo) | ✅ (completo) |
| **Cargo.toml exclude** | ✅ | ❌ | ⚠️ | ✅ | ✅ |
| **package.metadata.docs.rs** | ✅ | ❌ | ✅ | ✅ | ✅ |
| **#![forbid(unsafe_code)]** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **rust-version** | ✅ | ❌ | ✅ | ✅ | ✅ |

---

## 🔧 TASKLIST POR CRATE

---

## 1. logline-core v0.1.1

### ✅ Já possui
- Cargo.toml completo com exclude e docs.rs
- README.md, CHANGELOG.md, LICENSE
- SECURITY.md, CODE_OF_CONDUCT.md, CONTRIBUTING.md
- 4 testes, 2 exemplos, 1 benchmark
- CI workflow básico

### ❌ Faltando (Padrão Completo)

#### 1.1 Workflows GitHub
- [ ] `.github/workflows/audit.yml` — Security Audit
- [ ] `.github/workflows/deny.yml` — License/Advisory Deny
- [ ] `.github/workflows/sbom.yml` — SBOM Generation

#### 1.2 Templates GitHub
- [ ] `.github/ISSUE_TEMPLATE/bug_report.md`
- [ ] `.github/ISSUE_TEMPLATE/feature_request.md`
- [ ] `.github/ISSUE_TEMPLATE/config.yml`
- [ ] `.github/pull_request_template.md`

#### 1.3 Arquivos de Segurança
- [ ] `deny.toml` — Configuração cargo-deny

#### 1.4 Documentação
- [ ] `RELEASE_NOTES.md` — Notas de release (para GitHub Releases)
- [ ] Atualizar `README.md` com badges completos (CI, no_std se aplicável)

#### 1.5 Código
- [x] Verificar `#![forbid(unsafe_code)]` em `src/lib.rs` ✅ (já presente)
- [ ] Adicionar documentação inline se faltar

#### 1.6 Cargo.toml
- [ ] Verificar se `exclude` está completo
- [ ] Verificar `[package.metadata.docs.rs]` (já tem `all-features = true`)

---

## 2. json_atomic v0.1.0

### ✅ Já possui
- Cargo.toml (mas falta exclude e docs.rs)
- README.md, CHANGELOG.md, LICENSE
- 5 testes, 3 exemplos, 2 benchmarks
- CI workflow + release-drafter
- Templates GitHub completos

### ❌ Faltando (Padrão Completo)

#### 2.1 Cargo.toml
- [ ] Adicionar `exclude = [".github/**", "deny.toml", "SECURITY.md", "CODE_OF_CONDUCT.md", "CHANGELOG.md"]`
- [ ] Adicionar `[package.metadata.docs.rs]` com features corretas

#### 2.2 Workflows GitHub
- [ ] `.github/workflows/audit.yml` — Security Audit
- [ ] `.github/workflows/deny.yml` — License/Advisory Deny
- [ ] `.github/workflows/sbom.yml` — SBOM Generation

#### 2.3 Arquivos de Segurança
- [ ] `deny.toml` — Configuração cargo-deny
- [ ] `SECURITY.md` — Política de segurança
- [ ] `CODE_OF_CONDUCT.md` — Código de conduta

#### 2.4 Documentação
- [ ] `RELEASE_NOTES.md` — Notas de release
- [ ] Atualizar `README.md` com badges completos (CI, no_std, etc)

#### 2.5 Código
- [x] Verificar `#![forbid(unsafe_code)]` em `src/lib.rs` ✅ (já presente)
- [ ] Adicionar documentação inline se faltar

---

## 3. lllv-core v0.1.0

### ✅ Já possui
- Cargo.toml completo (mas falta verificar exclude)
- README.md, CHANGELOG.md, LICENSE
- SECURITY.md, CODE_OF_CONDUCT.md, RELEASE_NOTES.md
- deny.toml
- 3 testes (incluindo ataque), 1 benchmark
- 4 workflows completos (CI, audit, deny, SBOM)

### ❌ Faltando (Padrão Completo)

#### 3.1 Templates GitHub
- [ ] `.github/ISSUE_TEMPLATE/bug_report.md`
- [ ] `.github/ISSUE_TEMPLATE/feature_request.md`
- [ ] `.github/ISSUE_TEMPLATE/config.yml`
- [ ] `.github/pull_request_template.md`

#### 3.2 Exemplos
- [x] Adicionar pelo menos 1 exemplo em `examples/` ✅ **CORRIGIDO** (create_capsule.rs)

#### 3.3 Cargo.toml
- [ ] Verificar se `exclude` está completo e correto

#### 3.4 Documentação
- [ ] Atualizar `README.md` com badges completos (verificar se tem todos)

#### 3.5 Código
- [x] Verificar `#![forbid(unsafe_code)]` em `src/lib.rs` ✅ (já presente)
- [ ] Verificar documentação inline completa

---

## 4. lllv-index v0.1.0

### ✅ Já possui
- Cargo.toml completo
- README.md, CHANGELOG.md, LICENSE
- SECURITY.md, CODE_OF_CONDUCT.md, RELEASE_NOTES.md
- deny.toml
- 2 testes, 1 exemplo
- 4 workflows completos (CI, audit, deny, SBOM)
- Templates GitHub completos

### ❌ Faltando (Padrão Completo)

#### 4.1 Benchmarks
- [ ] Adicionar pelo menos 1 benchmark em `benches/`

#### 4.2 Testes
- [ ] Considerar adicionar mais testes (opcional, já tem 2)

#### 4.3 Código
- [x] Verificar `#![forbid(unsafe_code)]` em `src/lib.rs` ✅ (já presente)
- [ ] Verificar documentação inline completa

---

## 🔄 TASKS COMUNS A TODAS AS CRATES

### Verificações de Código
- [x] Verificar `#![forbid(unsafe_code)]` em todas ✅ (todas já têm)
- [ ] Executar `cargo audit` em todas
- [ ] Executar `cargo deny check all` em todas (onde tiver deny.toml)
- [ ] Verificar documentação inline (`///`) completa
- [ ] Executar `cargo doc --no-deps` para verificar docs.rs

### Badges README
- [ ] Adicionar badge de CI (se não tiver)
- [ ] Adicionar badge no_std (se aplicável)
- [ ] Verificar se todos os badges estão funcionando

### Cargo.toml
- [ ] Verificar `exclude` em todas (não incluir .git, target, etc)
- [ ] Verificar `[package.metadata.docs.rs]` em todas
- [ ] Verificar se `rust-version = "1.75"` está presente

---

## 📋 TASKLIST CONSOLIDADA (ORDEM DE PRIORIDADE)

### Prioridade Alta (Segurança e Qualidade)

#### Para logline-core:
1. [x] Criar `deny.toml` ✅ **CORRIGIDO**
2. [ ] Criar workflows: `audit.yml`, `deny.yml`, `sbom.yml`
3. [ ] Criar templates GitHub (4 arquivos)
4. [ ] Criar `RELEASE_NOTES.md`
5. [ ] Atualizar README com badges completos
6. [ ] ⭐ NOVO: Instalar `cargo-udeps` para verificar dependências não utilizadas (Fase 9)

#### Para json_atomic:
1. [x] **URGENTE**: Adicionar `rust-version = "1.75"` no Cargo.toml ✅ **CORRIGIDO**
2. [ ] Adicionar `exclude` no Cargo.toml
3. [ ] Adicionar `[package.metadata.docs.rs]` no Cargo.toml
4. [ ] Criar `deny.toml`
5. [ ] Criar `SECURITY.md`
6. [ ] Criar `CODE_OF_CONDUCT.md`
7. [ ] Criar workflows: `audit.yml`, `deny.yml`, `sbom.yml`
8. [ ] Criar `RELEASE_NOTES.md`
9. [ ] Atualizar README com badges completos
10. [ ] ⭐ NOVO: Instalar `cargo-udeps` para verificar dependências não utilizadas (Fase 9)

#### Para lllv-core:
1. [ ] Criar templates GitHub (4 arquivos)
2. [ ] Adicionar exemplo em `examples/`
3. [x] Adicionar `exclude` no Cargo.toml ✅ **CORRIGIDO**
4. [ ] Atualizar README com badges completos (se faltar)
5. [ ] ⭐ NOVO: Instalar `cargo-udeps` para verificar dependências não utilizadas (Fase 9)

#### Para lllv-index:
1. [ ] Adicionar benchmark em `benches/`
2. [ ] Verificar `exclude` no Cargo.toml (⚠️ WARNING - pode estar em formato diferente)
3. [ ] Verificar código e documentação
4. [ ] ⭐ NOVO: Instalar `cargo-udeps` para verificar dependências não utilizadas (Fase 9)

### Prioridade Média (Melhorias)

#### Para todas:
1. [x] Verificar `#![forbid(unsafe_code)]` ✅ (todas já têm)
2. [ ] Executar `cargo audit` e corrigir vulnerabilidades
3. [ ] Executar `cargo deny check all` e corrigir problemas
4. [ ] Verificar documentação inline completa
5. [ ] Testar `cargo doc --no-deps` localmente
6. [ ] ⭐ NOVO: Instalar e executar `cargo-udeps` para verificar dependências não utilizadas (Fase 9)

### Prioridade Baixa (Opcional)

1. [ ] Configurar Trusted Publishing (opcional, mas recomendado)
2. [ ] Adicionar mais exemplos (se necessário)
3. [ ] Adicionar mais testes (se necessário)
4. [ ] Melhorar documentação (se necessário)

---

## 🚀 PLANO DE EXECUÇÃO SUGERIDO

### Fase 1: Segurança (Todas as crates)
1. Criar `deny.toml` onde faltar
2. Criar workflows `audit.yml` e `deny.yml` onde faltar
3. Executar `cargo audit` e `cargo deny` em todas
4. Corrigir problemas encontrados

### Fase 2: Documentação (Todas as crates)
1. Criar `SECURITY.md` onde faltar
2. Criar `CODE_OF_CONDUCT.md` onde faltar
3. Criar `RELEASE_NOTES.md` onde faltar
4. Atualizar README com badges completos

### Fase 3: Templates e Workflows (Onde faltar)
1. Criar templates GitHub onde faltar
2. Criar workflow `sbom.yml` onde faltar
3. Verificar CI workflows

### Fase 4: Código e Exemplos
1. Adicionar exemplos onde faltar
2. Adicionar benchmarks onde faltar
3. [x] Verificar `#![forbid(unsafe_code)]` ✅ (todas já têm)
4. Verificar documentação inline

### Fase 5: Validação Final
1. Executar checklist completo em todas
2. Testar publicação (dry-run)
3. Atualizar CHANGELOG se necessário

---

## 📊 RESUMO POR CRATE

### logline-core
- **Faltam**: 3 workflows, 4 templates, deny.toml, RELEASE_NOTES.md
- **Total de tasks**: ~12

### json_atomic
- **Faltam**: exclude/docs.rs no Cargo.toml, 3 workflows, deny.toml, SECURITY.md, CODE_OF_CONDUCT.md, RELEASE_NOTES.md
- **Total de tasks**: ~15

### lllv-core
- **Faltam**: 4 templates, 1 exemplo
- **Total de tasks**: ~6

### lllv-index
- **Faltam**: 1 benchmark
- **Total de tasks**: ~2

**TOTAL GERAL**: ~35 tasks

---

## ✅ CHECKLIST FINAL (Após completar todas as tasks)

Para cada crate, verificar:

```
[ ] deny.toml presente e configurado
[ ] SECURITY.md presente
[ ] CODE_OF_CONDUCT.md presente
[ ] RELEASE_NOTES.md presente (ou preparado para próximo release)
[ ] 4 workflows GitHub (CI, audit, deny, SBOM)
[ ] Templates GitHub completos (3 issue + 1 PR)
[ ] README.md com badges completos
[ ] Cargo.toml com exclude e docs.rs
[x] #![forbid(unsafe_code)] no lib.rs ✅ (todas já têm)
[ ] cargo audit passa
[ ] cargo deny check all passa
[ ] cargo test --all-features passa
[ ] Exemplos funcionam
[ ] Documentação inline completa
```

---

## 📝 NOTAS

- **Prioridade**: 
  1. **URGENTE**: Corrigir erro crítico em `json_atomic` (rust-version)
  2. Segurança (deny.toml, workflows) 
  3. Documentação 
  4. Templates 
  5. Melhorias
- **Ordem sugerida**: 
  1. **json_atomic** (erro crítico) → 
  2. **logline-core** (mais faltando) → 
  3. **lllv-core** → 
  4. **lllv-index**
- **Templates**: Podem ser copiados de lllv-index ou json_atomic
- **Workflows**: Podem ser copiados de lllv-core ou lllv-index
- **deny.toml**: Pode ser copiado de lllv-core ou lllv-index
- **Verificação automatizada**: Execute `bash scripts/verify_quality.sh <crate_dir>` para verificar qualquer crate
