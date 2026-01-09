# 🔍 Scripts de Verificação de Qualidade

Scripts automatizados para verificar se crates atendem ao **padrão completo de qualidade** estabelecido.

## 📋 Scripts Disponíveis

### 1. `verify_quality.sh` (Bash)
Script shell portável que verifica todos os aspectos do padrão de qualidade.

**Uso:**
```bash
# Verificar crate específica
bash scripts/verify_quality.sh logline-core

# Verificar crate atual
bash scripts/verify_quality.sh .

# Verificar todas as crates
for dir in logline-core json_atomic lllv-core lllv-index; do
    echo "=== $dir ==="
    bash scripts/verify_quality.sh "$dir"
    echo ""
done
```

**Requisitos:**
- Bash 4+
- `cargo` instalado (para validações de código)
- `find`, `grep`, `wc` (comandos padrão Unix)

### 2. `verify_quality_python.py` (Python)
Versão Python do verificador, mais robusta e com melhor tratamento de erros.

**Uso:**
```bash
# Verificar crate específica
python3 scripts/verify_quality_python.py logline-core

# Verificar crate atual
python3 scripts/verify_quality_python.py .
```

**Requisitos:**
- Python 3.7+
- `cargo` instalado

### 3. `verify_quality.rs` (Rust)
Versão Rust do verificador (requer compilação).

**Uso:**
```bash
# Compilar
cargo build --release --manifest-path scripts/Cargo.toml

# Executar
./target/release/verify_quality logline-core
```

**Requisitos:**
- Rust toolchain
- Dependência: `walkdir`

## ✅ O que é Verificado

> **⭐ NOVO**: Fase 9 adicionada para verificar **anti-padrões** — coisas que **não devem** estar nas crates!

### Fase 1: Estrutura Básica
- ✅ Cargo.toml (obrigatório)
- ✅ README.md (obrigatório)
- ✅ LICENSE (obrigatório)
- ✅ .gitignore (obrigatório)
- ⚠️ CHANGELOG.md (recomendado)
- ⚠️ CITATION.cff (recomendado)

### Fase 2: Configuração Cargo.toml
- ✅ Metadados obrigatórios (name, version, edition, license, etc.)
- ⚠️ Campo `exclude` (recomendado)
- ⚠️ Seção `[package.metadata.docs.rs]` (recomendado)

### Fase 3: Estrutura de Código
- ✅ Diretório `src/` com arquivos .rs (mínimo: 1)
- ✅ Diretório `tests/` com arquivos .rs (mínimo: 2)
- ✅ Diretório `examples/` com arquivos .rs (mínimo: 1)
- ⚠️ Diretório `benches/` com arquivos .rs (opcional, recomendado: 1)

### Fase 4: Segurança e Qualidade
- ⚠️ SECURITY.md (recomendado)
- ⚠️ CODE_OF_CONDUCT.md (recomendado)
- ⚠️ deny.toml (recomendado)
- ⚠️ `#![forbid(unsafe_code)]` no lib.rs (recomendado)

### Fase 5: CI/CD e Workflows
- ⚠️ `.github/workflows/ci.yml` (recomendado)
- ⚠️ `.github/workflows/audit.yml` (recomendado)
- ⚠️ `.github/workflows/deny.yml` (recomendado)
- ⚠️ `.github/workflows/sbom.yml` (recomendado)

### Fase 6: Templates GitHub
- ⚠️ `.github/ISSUE_TEMPLATE/bug_report.md` (recomendado)
- ⚠️ `.github/ISSUE_TEMPLATE/feature_request.md` (recomendado)
- ⚠️ `.github/ISSUE_TEMPLATE/config.yml` (recomendado)
- ⚠️ `.github/pull_request_template.md` (recomendado)

### Fase 7: Documentação
- ✅ Qualidade do README.md (badges, seções)
- ⚠️ RELEASE_NOTES.md (recomendado)

### Fase 8: Validação de Código
- ✅ `cargo fmt --all -- --check` (obrigatório)
- ⚠️ `cargo clippy --all-targets --all-features -- -D warnings` (recomendado)
- ✅ `cargo test --all-features` (obrigatório)

### Fase 9: Anti-Padrões (O Que NÃO Deve Estar) ⭐ NOVO
- ❌ Arquivos proibidos: `target/`, `.env`, `.env.local` (ERRO CRÍTICO)
- ⚠️ Arquivos não recomendados: `.DS_Store`, `Thumbs.db`, `.idea/`, `.vscode/`, `*.iml`
- ⚠️ Arquivos grandes (>1MB, exceto docs/imagens)
- ❌ Secrets/credenciais hardcoded (detecção de padrões: `password=`, `api_key=`, etc.)
- ⚠️ Dependências não utilizadas (cargo-udeps)

### Fase 10: Dependências Crescentes e Acumulativas ⭐ NOVO
- ✅ Verifica ordem crescente de dependências (Paper I → II → III → ...)
- ✅ Detecta dependências circulares
- ✅ Valida versões corretas (não `path =` em produção)
- ✅ Verifica dependências esperadas vs encontradas
- 📝 **Script dedicado**: `scripts/verify_dependencies.sh` para verificação completa

## 📊 Códigos de Saída

- `0` - Sucesso (sem erros, pode ter warnings)
- `1` - Falha (erros encontrados)

## 🔧 Integração com CI/CD

O workflow `.github/workflows/quality-check.yml` executa automaticamente em:
- Push para `main`
- Pull requests
- Manualmente via `workflow_dispatch`

## 📝 Exemplo de Saída

```
🔍 Verificando qualidade da crate: lllv-index
📁 Diretório: /path/to/lllv-index

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 FASE 1: ESTRUTURA BÁSICA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Cargo.toml
✅ README.md
✅ LICENSE
...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 RESUMO FINAL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  ATENÇÃO: 5 warning(s) encontrado(s)
✅ Nenhum erro crítico. Crate atende ao padrão mínimo.
```

## 🚀 Uso em Pipeline

```bash
# Em CI/CD, falhar se houver erros
if ! bash scripts/verify_quality.sh "$CRATE_DIR"; then
    echo "❌ Verificação de qualidade falhou!"
    exit 1
fi
```

## 🔗 Verificação de Dependências Crescentes

Para verificar a ordem crescente e acumulativa de dependências entre todas as crates:

```bash
# Verificar todas as crates do repositório
bash scripts/verify_dependencies.sh .

# O script verifica:
# - Ordem crescente: logline-core → json_atomic → lllv-core → lllv-index
# - Dependências circulares
# - Versões corretas (não path em produção)
# - Dependências faltando ou inesperadas
```

### Hierarquia de Dependências Esperada

A ordem crescente e acumulativa das dependências segue a estrutura dos papers:

| Paper | Crate | Dependências Diretas |
|-------|-------|----------------------|
| **I** | `logline-core` | *(BASE - sem dependências internas)* |
| **II** | `json_atomic` | `logline-core` |
| **III** | `lllv-core` | `json_atomic` *(logline-core vem transitivamente)* |
| **III** | `lllv-index` | `lllv-core`, `json_atomic` *(opcional)* |
| **IV+** | *[futuras crates]* | *[dependências acumulativas]* |

**Regras**:
- Cada crate só pode depender de crates anteriores na hierarquia
- Dependências transitivas não precisam ser declaradas diretamente (ex: `lllv-index` não precisa declarar `logline-core` se já depende de `lllv-core`)
- O Rust dará erro se uma dependência for declarada mas não usada (exceto se for opcional/feature)

### ⚠️ Dependências Diretas vs Transitivas

**IMPORTANTE**: Há uma diferença crucial entre dependências **diretas** e **transitivas**:

#### Dependência Direta (Obrigatória)
Uma crate **deve** declarar uma dependência diretamente se:
- Usa tipos/funções diretamente no código (ex: `json_atomic::SignedFact`)
- Precisa de uma versão específica independente da transitiva
- A dependência é opcional (feature) e precisa estar disponível quando a feature está ativa

**Exemplo**: `lllv-index` declara `json_atomic` diretamente porque usa `json_atomic::SignedFact` e `json_atomic::seal_value()` no código.

#### Dependência Transitiva (Opcional)
Uma crate **não precisa** declarar uma dependência diretamente se:
- A dependência já vem transitivamente via outra dependência
- A crate não usa tipos/funções diretamente da dependência transitiva
- A versão transitiva é suficiente

**Exemplo**: `lllv-index` não precisa declarar `logline-core` diretamente porque:
- `lllv-core` já traz `logline-core` transitivamente
- `lllv-index` não usa `logline-core` diretamente no código

#### Quando Declarar Diretamente?
- ✅ **SIM**: Se usa tipos/funções diretamente (`crate::Type`, `crate::function()`)
- ✅ **SIM**: Se precisa de versão específica diferente da transitiva
- ✅ **SIM**: Se é opcional (feature) e precisa estar disponível quando ativa
- ❌ **NÃO**: Se só precisa da dependência transitivamente
- ❌ **NÃO**: Se não usa diretamente no código

#### Verificação Automatizada
O script `verify_dependencies.sh` verifica:
- ✅ Dependências diretas esperadas estão presentes
- ⚠️ Dependências inesperadas (mas podem ser válidas se usadas diretamente)
- ⚠️ Dependências transitivas declaradas diretamente (redundantes, mas não erros)

**Os testes de qualidade NÃO acusam ERRO** por dependências transitivas, apenas **ALERTAS (warnings)** para revisão manual.

## 📚 Ver Também

- `ROTEIRO_PADRAO_QUALIDADE.md` - Padrão completo de qualidade
- `TASKLIST_PADRONIZACAO_CRATES.md` - Tasklist de padronização
- `docs/DEPENDENCIAS_DIRETAS_VS_TRANSITIVAS.md` - **Guia completo sobre dependências diretas vs transitivas** ⭐
