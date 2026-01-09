#!/usr/bin/env bash
set -euo pipefail

# Camadas do ecossistema (base -> topo)
L1="logline-core"
L2="json_atomic"
L3="lllv-core"
L4="lllv-index"
T1="tdln-ast"
T2="tdln-proof"
T3="tdln-compiler"
T4="tdln-gate"
C1="chip-core"
C2="chip-serde"
C3="chip-exec"
C4="chip-ledger"

ORDER=($L1 $L2 $L3 $L4 $T1 $T2 $T3 $T4 $C1 $C2 $C3 $C4)
declare -A POS
i=0; for c in "${ORDER[@]}"; do POS["$c"]=$i; i=$((i+1)); done

MD=$(cargo metadata --format-version=1)
pkgs=$(jq -r '.packages[] | @base64' <<<"$MD")

fail=0
while IFS= read -r p; do
  pkg=$(echo "$p" | base64 -d)
  name=$(jq -r '.name' <<<"$pkg")
  deps=$(jq -r '.dependencies[] | select(.kind == null or .kind == "normal") | .name' <<<"$pkg")

  for d in $deps; do
    [[ -n "${POS[$name]:-}" && -n "${POS[$d]:-}" ]] || continue
    if [[ ${POS[$d]} -gt ${POS[$name]} ]]; then
      echo "⛔ Camada inválida: '$name' (nível ${POS[$name]}) depende de '$d' (nível ${POS[$d]})"
      fail=1
    fi
  done
done <<< "$pkgs"

# Diretórios para scans adicionais (somente os que existem)
SEARCH_ROOTS=()
for d in logline-core json_atomic lllv-core lllv-index crates external; do
  [[ -d "$d" ]] && SEARCH_ROOTS+=("$d")
done

if ((${#SEARCH_ROOTS[@]})); then
  if grep -R --include "Cargo.toml" -n 'version = "\*"' "${SEARCH_ROOTS[@]}" >/dev/null 2>&1; then
    echo "⛔ Dependência com wildcard (*) detectada"
    fail=1
  fi

  if grep -R --include "Cargo.toml" -n 'git\s*=' "${SEARCH_ROOTS[@]}" >/dev/null 2>&1; then
    echo "⚠️  Dependência git encontrada (permitido em dev; proibir ao publicar)"
  fi
fi

exit $fail
