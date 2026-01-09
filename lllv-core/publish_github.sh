#!/bin/bash
# Script para publicar lllv-core v0.1.0 no GitHub

set -e

cd "$(dirname "$0")"

echo "🚀 Publicando lllv-core v0.1.0 no GitHub..."

# Verificar se git user está configurado
if ! git config user.name > /dev/null 2>&1; then
    echo "⚠️  Git user não configurado. Configure com:"
    echo "   git config user.name 'Seu Nome'"
    echo "   git config user.email 'seu@email.com'"
    exit 1
fi

# Adicionar remote se não existir
if ! git remote get-url origin > /dev/null 2>&1; then
    git remote add origin https://github.com/LogLine-Foundation/lllv-core.git
fi

# Fazer commit inicial se necessário
if [ -z "$(git log --oneline -1 2>/dev/null)" ]; then
    echo "📝 Fazendo commit inicial..."
    git add -A
    git commit -m "lllv-core v0.1.0: Verifiable Capsules with hardening"
fi

# Criar tag se não existir
if ! git rev-parse v0.1.0 > /dev/null 2>&1; then
    echo "🏷️  Criando tag v0.1.0..."
    git tag -a v0.1.0 -m "lllv-core v0.1.0"
fi

# Push para GitHub
echo "📤 Fazendo push para GitHub..."
git push -u origin main 2>&1 || git push -u origin master 2>&1 || git push -u origin HEAD 2>&1
git push origin v0.1.0

# Criar release no GitHub
echo "🎉 Criando release no GitHub..."
gh release create v0.1.0 \
    --title "lllv-core v0.1.0 — Verifiable Capsules" \
    --notes-file RELEASE_NOTES.md \
    --repo LogLine-Foundation/lllv-core

echo "✅ Publicação no GitHub concluída!"
echo "   📦 crates.io: https://crates.io/crates/lllv-core"
echo "   🏷️  GitHub: https://github.com/LogLine-Foundation/lllv-core/releases/tag/v0.1.0"
