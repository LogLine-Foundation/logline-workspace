# 🔗 Dependências Diretas vs Transitivas — Guia Completo

Este documento explica a diferença crucial entre dependências **diretas** e **transitivas** no ecossistema LogLine Foundation.

---

## 📋 Conceitos Fundamentais

### Dependência Direta
Uma dependência declarada explicitamente no `Cargo.toml` da crate:
```toml
[dependencies]
json_atomic = { version = "0.1.1", optional = true }
```

### Dependência Transitiva
Uma dependência que vem indiretamente através de outra dependência:
```
lllv-index → lllv-core → json_atomic → logline-core
                    ↑
         logline-core vem transitivamente via json_atomic
```

---

## ✅ Quando Declarar Dependência Direta (Obrigatório)

### 1. Usa Tipos/Funções Diretamente no Código

**Se você usa assim no código:**
```rust
use json_atomic::SignedFact;

pub struct IndexPack {
    pub manifest: Option<json_atomic::SignedFact>,  // ← usa tipo diretamente
}

fn seal(data: &impl Serialize, sk: &SigningKey) {
    json_atomic::seal_value(data, sk)  // ← usa função diretamente
}
```

**Então PRECISA declarar diretamente:**
```toml
[dependencies]
json_atomic = { version = "0.1.1", optional = true }
```

**Exemplo Real**: `lllv-index` declara `json_atomic` diretamente porque usa `json_atomic::SignedFact` e `json_atomic::seal_value()` no código.

### 2. Precisa de Versão Específica Diferente

Se você precisa de uma versão diferente da que vem transitivamente:

```toml
# Transitivo traz logline-core 0.1.0
# Mas você precisa de 0.2.0
[dependencies]
logline-core = { version = "0.2.0" }  # declare diretamente
```

### 3. Dependência Opcional (Feature)

Se a dependência é opcional e precisa estar disponível quando a feature está ativa:

```toml
[features]
manifest = ["json_atomic"]  # precisa declarar json_atomic diretamente

[dependencies]
json_atomic = { version = "0.1.1", optional = true }
```

---

## ❌ Quando NÃO Declarar (Transitiva é Suficiente)

### 1. Dependência Já Vem Transitivamente

**Se você não usa diretamente no código**, a dependência transitiva é suficiente:

```toml
# lllv-index não precisa declarar logline-core diretamente
# porque lllv-core já traz logline-core transitivamente
[dependencies]
lllv-core = "0.1.0"  # já traz logline-core transitivamente
# logline-core = "..."  ← NÃO precisa declarar
```

**Exemplo Real**: `lllv-index` não declara `logline-core` diretamente porque:
- `lllv-core` já traz `logline-core` transitivamente
- `lllv-index` não usa `logline-core` diretamente no código

### 2. Não Usa Tipos/Funções Diretamente

Se você só precisa da dependência transitivamente (sem usar diretamente):

```rust
// Se você NÃO usa assim:
// use logline_core::LogLine;  ← não usa
// logline_core::something()   ← não usa

// Então NÃO precisa declarar diretamente
```

---

## 📊 Exemplos Práticos do Ecossistema LogLine

### Hierarquia Completa

| Crate | Dependências Diretas | Transitivas (via) |
|-------|---------------------|-------------------|
| `logline-core` | *(BASE)* | - |
| `json_atomic` | `logline-core` | - |
| `lllv-core` | `json_atomic` | `logline-core` (via `json_atomic`) |
| `lllv-index` | `lllv-core`, `json_atomic` | `logline-core` (via `lllv-core` → `json_atomic`) |

### Análise Detalhada

#### `lllv-core`
- ✅ Declara `json_atomic` diretamente (usa `json_atomic::seal_value()`)
- ❌ NÃO declara `logline-core` diretamente (vem transitivamente via `json_atomic`)
- ⚠️ **Nota**: Se `lllv-core` declarar `logline-core` diretamente mas não usar, o Rust não dará erro (pois vem transitivamente), mas é redundante.

#### `lllv-index`
- ✅ Declara `lllv-core` diretamente (usa `Capsule` de `lllv-core`)
- ✅ Declara `json_atomic` diretamente (usa `json_atomic::SignedFact` e `json_atomic::seal_value()`)
- ❌ NÃO declara `logline-core` diretamente (vem transitivamente via `lllv-core` → `json_atomic`)

---

## 🔍 Verificação Automatizada

### Scripts de Verificação

```bash
# Verificar dependências de todas as crates
bash scripts/verify_dependencies.sh .

# Verificação completa de qualidade (inclui dependências)
bash scripts/verify_quality.sh <crate_dir>
```

### Comportamento dos Testes

**⚠️ IMPORTANTE**: Os testes de qualidade:
- ✅ **NÃO acusam ERRO** por dependências transitivas
- ⚠️ **Emitem ALERTAS (warnings)** para revisão manual
- ⚠️ Alertam sobre dependências inesperadas (mas podem ser válidas se usadas diretamente)
- ⚠️ Alertam sobre dependências transitivas declaradas diretamente (redundantes, mas não erros)

### Exemplo de Saída

```
📦 Verificando: lllv-core
   ✅ Dependências corretas: json_atomic
   ⚠️  Dependências inesperadas (mas podem ser válidas se usadas diretamente): logline-core
      ℹ️  Se você usa tipos/funções diretamente (ex: `crate::Type`), declare diretamente.
      ℹ️  Caso contrário, a dependência transitiva é suficiente.
```

---

## 🎯 Regra de Ouro

**Se você usa `crate::Type` ou `crate::function()` no código, declare diretamente. Caso contrário, a transitiva é suficiente.**

### Checklist Rápido

Antes de adicionar uma dependência direta, pergunte:

1. [ ] Uso tipos/funções diretamente no código? (`crate::Type`, `crate::function()`)
   - ✅ **SIM** → Declare diretamente
   - ❌ **NÃO** → Continue para próxima pergunta

2. [ ] Preciso de versão específica diferente da transitiva?
   - ✅ **SIM** → Declare diretamente
   - ❌ **NÃO** → Continue para próxima pergunta

3. [ ] É dependência opcional (feature) que precisa estar disponível quando ativa?
   - ✅ **SIM** → Declare diretamente
   - ❌ **NÃO** → **NÃO declare**, use a transitiva

---

## 📚 Referências

- [Cargo Book: Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [Cargo Book: Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html)
- `ROTEIRO_PADRAO_QUALIDADE.md` - Fase 11: Dependências Crescentes
- `scripts/README.md` - Verificação de Dependências

---

## 🔄 Atualizações

Este documento deve ser atualizado quando:
- Novas crates são adicionadas ao ecossistema
- Padrões de dependências mudam
- Novas regras são estabelecidas

**Última atualização**: 2026-01-09
