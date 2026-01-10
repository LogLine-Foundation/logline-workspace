#!/usr/bin/env python3
"""
Script para publicar lllv-core no GitHub usando GitHub App authentication
"""
import os
import sys
import subprocess
import json
from pathlib import Path

# Procurar chave PEM
base_dir = Path(__file__).parent.parent
pem_files = list(base_dir.glob("*.pem"))
if not pem_files:
    print("❌ Nenhum arquivo .pem encontrado na raiz")
    sys.exit(1)

pem_file = pem_files[0]
print(f"📄 Usando chave PEM: {pem_file}")

# Procurar credenciais do GitHub App
app_id = os.getenv("GITHUB_APP_ID") or os.getenv("APP_ID")
app_installation_id = os.getenv("GITHUB_APP_INSTALLATION_ID") or os.getenv("INSTALLATION_ID")
org = os.getenv("GITHUB_ORG") or "LogLine-Foundation"
repo = os.getenv("GITHUB_REPO") or "lllv-core"

if not app_id:
    print("⚠️  GITHUB_APP_ID não encontrado. Tentando usar gh CLI...")
    # Tentar usar gh CLI se disponível
    try:
        subprocess.run(["gh", "auth", "status"], check=True, capture_output=True)
        print("✅ GitHub CLI autenticado")
        use_gh = True
    except:
        print("❌ GitHub App ID necessário. Configure GITHUB_APP_ID no ambiente.")
        sys.exit(1)
else:
    print(f"✅ GitHub App ID encontrado: {app_id}")
    use_gh = False

# Configurar git
os.chdir(Path(__file__).parent)
subprocess.run(["git", "config", "user.name", "LogLine Foundation"], check=False)
subprocess.run(["git", "config", "user.email", "ops@logline.foundation"], check=False)

# Fazer commit se necessário
try:
    subprocess.run(["git", "add", "-A"], check=True)
    subprocess.run(["git", "commit", "-m", "lllv-core v0.1.0: Verifiable Capsules with hardening"], check=False)
except:
    pass

# Criar tag se não existir
try:
    subprocess.run(["git", "rev-parse", "v0.1.0"], check=True, capture_output=True)
    print("✅ Tag v0.1.0 já existe")
except:
    subprocess.run(["git", "tag", "-a", "v0.1.0", "-m", "lllv-core v0.1.0"], check=True)
    print("✅ Tag v0.1.0 criada")

# Configurar remote
subprocess.run(["git", "remote", "remove", "origin"], check=False)
subprocess.run(["git", "remote", "add", "origin", f"https://github.com/{org}/{repo}.git"], check=False)

# Push usando GitHub App ou gh CLI
if use_gh:
    print("📤 Fazendo push usando GitHub CLI...")
    subprocess.run(["git", "push", "-u", "origin", "HEAD"], check=True)
    subprocess.run(["git", "push", "origin", "v0.1.0"], check=True)
    
    # Criar release
    print("🎉 Criando release no GitHub...")
    subprocess.run([
        "gh", "release", "create", "v0.1.0",
        "--title", "lllv-core v0.1.0 — Verifiable Capsules",
        "--notes-file", "RELEASE_NOTES.md",
        "--repo", f"{org}/{repo}"
    ], check=True)
else:
    print("📤 Fazendo push usando GitHub App...")
    # Para GitHub App, precisaríamos gerar um token JWT primeiro
    # Por enquanto, usar gh CLI que já está autenticado
    subprocess.run(["git", "push", "-u", "origin", "HEAD"], check=True)
    subprocess.run(["git", "push", "origin", "v0.1.0"], check=True)
    
    subprocess.run([
        "gh", "release", "create", "v0.1.0",
        "--title", "lllv-core v0.1.0 — Verifiable Capsules",
        "--notes-file", "RELEASE_NOTES.md",
        "--repo", f"{org}/{repo}"
    ], check=True)

print("✅ Publicação no GitHub concluída!")
print(f"   📦 crates.io: https://crates.io/crates/lllv-core")
print(f"   🏷️  GitHub: https://github.com/{org}/{repo}/releases/tag/v0.1.0")
