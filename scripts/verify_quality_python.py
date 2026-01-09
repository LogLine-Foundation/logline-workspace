#!/usr/bin/env python3
"""
Verificador de qualidade automatizado para crates Rust
Verifica presença e qualidade mínima de todos os itens do padrão
"""

import os
import sys
import subprocess
import re
from pathlib import Path
from typing import List, Tuple, Optional

class QualityChecker:
    def __init__(self, crate_dir: Path):
        self.crate_dir = Path(crate_dir).resolve()
        self.errors: List[Tuple[str, str]] = []
        self.warnings: List[Tuple[str, str]] = []
        
    def check_file(self, file: str, required: bool = True, desc: Optional[str] = None) -> bool:
        """Verifica se arquivo existe"""
        path = self.crate_dir / file
        desc = desc or file
        if path.exists():
            print(f"✅ {desc}")
            return True
        else:
            if required:
                print(f"❌ {desc} (OBRIGATÓRIO - faltando)")
                self.errors.append((desc, f"Arquivo obrigatório faltando: {file}"))
                return False
            else:
                print(f"⚠️  {desc} (recomendado - faltando)")
                self.warnings.append((desc, f"Arquivo recomendado faltando: {file}"))
                return False
    
    def check_content(self, file: str, pattern: str, desc: str, required: bool = True) -> bool:
        """Verifica se arquivo contém padrão"""
        path = self.crate_dir / file
        if not path.exists():
            if required:
                print(f"❌ {desc} (arquivo não existe)")
                self.errors.append((desc, f"Arquivo não existe: {file}"))
            return False
        
        try:
            content = path.read_text(encoding='utf-8')
            if pattern in content:
                print(f"✅ {desc}")
                return True
            else:
                if required:
                    print(f"❌ {desc} (padrão '{pattern}' não encontrado)")
                    self.errors.append((desc, f"Padrão não encontrado em {file}"))
                else:
                    print(f"⚠️  {desc} (padrão '{pattern}' não encontrado)")
                    self.warnings.append((desc, f"Padrão não encontrado em {file}"))
                return False
        except Exception as e:
            if required:
                print(f"❌ {desc} (erro ao ler: {e})")
                self.errors.append((desc, f"Erro ao ler {file}: {e}"))
            return False
    
    def check_rs_files(self, dir_path: str, min_count: int, desc: str, required: bool = True) -> bool:
        """Conta arquivos .rs em diretório"""
        path = self.crate_dir / dir_path
        if not path.exists():
            if required:
                print(f"❌ {desc} (diretório não existe)")
                self.errors.append((desc, f"Diretório não existe: {dir_path}"))
            else:
                print(f"⚠️  {desc} (diretório não existe)")
                self.warnings.append((desc, f"Diretório não existe: {dir_path}"))
            return False
        
        count = sum(1 for p in path.rglob("*.rs") if p.is_file())
        if count >= min_count:
            print(f"✅ {desc} ({count} arquivos .rs, mínimo: {min_count})")
            return True
        else:
            if required:
                print(f"❌ {desc} (apenas {count} arquivo(s), mínimo: {min_count})")
                self.errors.append((desc, f"Apenas {count} arquivo(s) .rs, mínimo: {min_count}"))
            else:
                print(f"⚠️  {desc} (apenas {count} arquivo(s), recomendado: {min_count})")
                self.warnings.append((desc, f"Apenas {count} arquivo(s) .rs, recomendado: {min_count}"))
            return False
    
    def check_cargo_command(self, args: List[str], desc: str, required: bool = True) -> bool:
        """Executa comando cargo e verifica sucesso"""
        try:
            result = subprocess.run(
                ["cargo"] + args,
                cwd=self.crate_dir,
                capture_output=True,
                text=True,
                timeout=300
            )
            if result.returncode == 0:
                print(f"✅ {desc}")
                return True
            else:
                if required:
                    print(f"❌ {desc} (comando falhou)")
                    self.errors.append((desc, f"Comando falhou: cargo {' '.join(args)}"))
                else:
                    print(f"⚠️  {desc} (comando falhou)")
                    self.warnings.append((desc, f"Comando falhou: cargo {' '.join(args)}"))
                return False
        except FileNotFoundError:
            if required:
                print(f"❌ {desc} (cargo não encontrado)")
                self.errors.append((desc, "cargo não encontrado"))
            else:
                print(f"⚠️  {desc} (cargo não encontrado)")
                self.warnings.append((desc, "cargo não encontrado"))
            return False
        except subprocess.TimeoutExpired:
            if required:
                print(f"❌ {desc} (timeout)")
                self.errors.append((desc, "Comando excedeu timeout"))
            return False
    
    def check_readme_quality(self) -> None:
        """Verifica qualidade do README.md"""
        readme_path = self.crate_dir / "README.md"
        if not readme_path.exists():
            return
        
        try:
            content = readme_path.read_text(encoding='utf-8')
            
            # Contar badges
            badge_count = content.count("img.shields.io") + content.count("docs.rs/badge")
            if badge_count >= 3:
                print(f"✅ README.md com {badge_count} badges (bom)")
            elif badge_count >= 1:
                print(f"⚠️  README.md com apenas {badge_count} badge(s) (recomendado: 3+)")
                self.warnings.append(("README badges", f"Apenas {badge_count} badge(s)"))
            else:
                print(f"⚠️  README.md sem badges (recomendado adicionar)")
                self.warnings.append(("README badges", "Sem badges"))
            
            # Verificar seções
            if "## Instalação" in content or "## Installation" in content:
                print("✅ Seção 'Instalação' no README")
            else:
                self.warnings.append(("README", "Seção 'Instalação' não encontrada"))
            
            if "## Quickstart" in content or "```rust" in content:
                print("✅ Seção Quickstart/Exemplo no README")
            else:
                self.warnings.append(("README", "Seção Quickstart/Exemplo não encontrada"))
        except Exception as e:
            self.warnings.append(("README", f"Erro ao verificar: {e}"))
    
    def run_all_checks(self) -> None:
        """Executa todas as verificações"""
        print(f"🔍 Verificando qualidade da crate")
        print(f"📁 Diretório: {self.crate_dir}\n")
        
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 1: ESTRUTURA BÁSICA")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_file("Cargo.toml", True)
        self.check_file("README.md", True)
        self.check_file("LICENSE", True)
        self.check_file(".gitignore", True)
        self.check_file("CHANGELOG.md", False)
        self.check_file("CITATION.cff", False)
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 2: CONFIGURAÇÃO CARGO.TOML")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_content("Cargo.toml", "name =", "Campo 'name'", True)
        self.check_content("Cargo.toml", "version =", "Campo 'version'", True)
        self.check_content("Cargo.toml", "edition =", "Campo 'edition'", True)
        self.check_content("Cargo.toml", "license =", "Campo 'license'", True)
        self.check_content("Cargo.toml", "description =", "Campo 'description'", True)
        self.check_content("Cargo.toml", "repository =", "Campo 'repository'", True)
        self.check_content("Cargo.toml", "readme =", "Campo 'readme'", True)
        self.check_content("Cargo.toml", "rust-version =", "Campo 'rust-version'", True)
        self.check_content("Cargo.toml", "documentation =", "Campo 'documentation'", True)
        self.check_content("Cargo.toml", "exclude", "Campo 'exclude'", False)
        self.check_content("Cargo.toml", "[package.metadata.docs.rs]", "Seção docs.rs", False)
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 3: ESTRUTURA DE CÓDIGO")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_rs_files("src", 1, "Diretório src/", True)
        self.check_rs_files("tests", 2, "Diretório tests/", True)
        self.check_rs_files("examples", 1, "Diretório examples/", True)
        self.check_rs_files("benches", 1, "Diretório benches/", False)
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 4: SEGURANÇA E QUALIDADE")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_file("SECURITY.md", False)
        self.check_file("CODE_OF_CONDUCT.md", False)
        self.check_file("deny.toml", False)
        self.check_content("src/lib.rs", "#![forbid(unsafe_code)]", "#![forbid(unsafe_code)]", False)
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 5: CI/CD E WORKFLOWS")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_file(".github/workflows/ci.yml", False, "Workflow CI")
        self.check_file(".github/workflows/audit.yml", False, "Workflow Audit")
        self.check_file(".github/workflows/deny.yml", False, "Workflow Deny")
        self.check_file(".github/workflows/sbom.yml", False, "Workflow SBOM")
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 6: TEMPLATES GITHUB")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_file(".github/ISSUE_TEMPLATE/bug_report.md", False, "Template Bug Report")
        self.check_file(".github/ISSUE_TEMPLATE/feature_request.md", False, "Template Feature Request")
        self.check_file(".github/ISSUE_TEMPLATE/config.yml", False, "Template Config")
        self.check_file(".github/pull_request_template.md", False, "Template Pull Request")
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 7: DOCUMENTAÇÃO")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_readme_quality()
        self.check_file("RELEASE_NOTES.md", False)
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 9: ANTI-PADRÕES (O QUE NÃO DEVE ESTAR)")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        
        # Verificar arquivos proibidos
        forbidden_files = [
            ("target", "Diretório target/ não deve estar no repositório", True),
            (".env", "Arquivo .env não deve estar no repositório", True),
            (".env.local", "Arquivo .env.local não deve estar no repositório", True),
            (".DS_Store", "Arquivo .DS_Store não deve estar no repositório", False),
            ("Thumbs.db", "Arquivo Thumbs.db não deve estar no repositório", False),
            (".idea", "Diretório .idea/ não deve estar no repositório", False),
            (".vscode", "Diretório .vscode/ não deve estar no repositório", False),
        ]
        
        for file, desc, is_error in forbidden_files:
            path = self.crate_dir / file
            if path.exists():
                if is_error:
                    self.fail(desc, f"Arquivo proibido encontrado: {file}")
                else:
                    self.warn(desc, f"Arquivo não recomendado: {file}")
            else:
                self.pass(f"{desc} (não encontrado)")
        
        # Verificar arquivos grandes
        try:
            large_files = []
            for path in self.crate_dir.rglob("*"):
                if path.is_file() and path.stat().st_size > 1024 * 1024:  # >1MB
                    if path.suffix not in [".md", ".pdf", ".png", ".jpg", ".jpeg"]:
                        if "target" not in str(path) and ".git" not in str(path):
                            large_files.append(path)
            
            if large_files:
                self.warn("Arquivos grandes encontrados", f"{len(large_files)} arquivo(s) >1MB")
                for f in large_files[:5]:
                    size_mb = f.stat().st_size / (1024 * 1024)
                    print(f"   ⚠️  {f.relative_to(self.crate_dir)} ({size_mb:.1f}MB)")
            else:
                self.pass("Nenhum arquivo grande desnecessário encontrado")
        except Exception as e:
            self.warn("Verificação de arquivos grandes", f"Erro: {e}")
        
        # Verificar secrets hardcoded
        try:
            secret_patterns = [
                (r'password\s*=\s*["\'][^"\']+["\']', "password"),
                (r'api_key\s*=\s*["\'][^"\']+["\']', "api_key"),
                (r'secret\s*=\s*["\'][^"\']+["\']', "secret"),
                (r'token\s*=\s*["\'][^"\']+["\']', "token"),
            ]
            
            found_secrets = False
            for pattern, name in secret_patterns:
                for path in (self.crate_dir / "src").rglob("*.rs"):
                    try:
                        content = path.read_text(encoding='utf-8')
                        if re.search(pattern, content, re.IGNORECASE):
                            # Ignorar se estiver em comentários de teste ou exemplo
                            if "test" not in content.lower() and "example" not in content.lower():
                                found_secrets = True
                                break
                    except:
                        pass
                if found_secrets:
                    break
            
            if found_secrets:
                self.fail("Possíveis secrets/credenciais hardcoded", "VERIFICAR código fonte!")
            else:
                self.pass("Nenhum secret/credencial hardcoded detectado")
        except Exception as e:
            self.warn("Verificação de secrets", f"Erro: {e}")
        
        # Verificar dependências não utilizadas
        try:
            result = subprocess.run(
                ["cargo", "udeps", "--all-targets", "--all-features"],
                cwd=self.crate_dir,
                capture_output=True,
                text=True,
                timeout=60
            )
            if "unused dependencies" in result.stdout.lower() or "unused dependencies" in result.stderr.lower():
                self.warn("Dependências não utilizadas", "Executar: cargo udeps")
            else:
                self.pass("Nenhuma dependência não utilizada (cargo udeps)")
        except FileNotFoundError:
            self.warn("cargo-udeps", "Não instalado (recomendado)")
        except subprocess.TimeoutExpired:
            self.warn("cargo-udeps", "Timeout na verificação")
        except:
            pass  # cargo-udeps pode não estar instalado
        
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📋 FASE 8: VALIDAÇÃO DE CÓDIGO")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        self.check_cargo_command(["fmt", "--all", "--", "--check"], "cargo fmt", True)
        
        # Clippy (não crítico se não estiver instalado)
        try:
            self.check_cargo_command(["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"], "cargo clippy", False)
        except:
            pass
        
        self.check_cargo_command(["test", "--all-features"], "cargo test", True)
    
    def print_summary(self) -> int:
        """Imprime resumo e retorna código de saída"""
        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("📊 RESUMO FINAL")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        
        if not self.errors and not self.warnings:
            print("✅ PERFEITO! Nenhum erro ou warning encontrado.")
            print("✅ Crate atende ao padrão completo de qualidade!")
            return 0
        elif not self.errors:
            print(f"⚠️  ATENÇÃO: {len(self.warnings)} warning(s) encontrado(s)")
            print("✅ Nenhum erro crítico. Crate atende ao padrão mínimo.")
            if self.warnings:
                print("\nWarnings:")
                for desc, msg in self.warnings:
                    print(f"  - {desc}: {msg}")
            return 0
        else:
            print(f"❌ ERRO: {len(self.errors)} erro(s) e {len(self.warnings)} warning(s) encontrado(s)")
            print("❌ Crate NÃO atende ao padrão mínimo de qualidade.")
            if self.errors:
                print("\nErros:")
                for desc, msg in self.errors:
                    print(f"  - {desc}: {msg}")
            if self.warnings:
                print("\nWarnings:")
                for desc, msg in self.warnings:
                    print(f"  - {desc}: {msg}")
            return 1

def main():
    crate_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    checker = QualityChecker(crate_dir)
    checker.run_all_checks()
    sys.exit(checker.print_summary())

if __name__ == "__main__":
    main()
