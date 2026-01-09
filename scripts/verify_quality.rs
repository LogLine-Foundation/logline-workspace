//! Verificador de qualidade automatizado para crates Rust
//! Verifica presença e qualidade mínima de todos os itens do padrão

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
struct CheckResult {
    name: String,
    passed: bool,
    required: bool,
    message: String,
}

struct QualityChecker {
    crate_dir: PathBuf,
    errors: Vec<CheckResult>,
    warnings: Vec<CheckResult>,
}

impl QualityChecker {
    fn new(crate_dir: impl AsRef<Path>) -> Self {
        Self {
            crate_dir: crate_dir.as_ref().to_path_buf(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn check_file(&mut self, file: &str, required: bool, desc: &str) {
        let path = self.crate_dir.join(file);
        if path.exists() {
            self.pass(desc);
        } else if required {
            self.fail(desc, &format!("Arquivo obrigatório faltando: {}", file));
        } else {
            self.warn(desc, &format!("Arquivo recomendado faltando: {}", file));
        }
    }

    fn check_content(&mut self, file: &str, pattern: &str, desc: &str, required: bool) {
        let path = self.crate_dir.join(file);
        if !path.exists() {
            if required {
                self.fail(desc, &format!("Arquivo não existe: {}", file));
            }
            return;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                if required {
                    self.fail(desc, &format!("Não foi possível ler: {}", file));
                }
                return;
            }
        };

        if content.contains(pattern) {
            self.pass(desc);
        } else if required {
            self.fail(desc, &format!("Padrão '{}' não encontrado em {}", pattern, file));
        } else {
            self.warn(desc, &format!("Padrão '{}' não encontrado em {}", pattern, file));
        }
    }

    fn check_dir_count(&mut self, dir: &str, min: usize, desc: &str, required: bool) {
        let path = self.crate_dir.join(dir);
        if !path.exists() {
            if required {
                self.fail(desc, &format!("Diretório não existe: {}", dir));
            } else {
                self.warn(desc, &format!("Diretório não existe: {}", dir));
            }
            return;
        }

        let count = fs::read_dir(&path)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        if count >= min {
            self.pass(&format!("{} ({} arquivos, mínimo: {})", desc, count, min));
        } else if required {
            self.fail(desc, &format!("Apenas {} arquivo(s) encontrado(s), mínimo: {}", count, min));
        } else {
            self.warn(desc, &format!("Apenas {} arquivo(s) encontrado(s), recomendado: {}", count, min));
        }
    }

    fn check_rs_files(&mut self, dir: &str, min: usize, desc: &str, required: bool) {
        let path = self.crate_dir.join(dir);
        if !path.exists() {
            if required {
                self.fail(desc, &format!("Diretório não existe: {}", dir));
            }
            return;
        }

        let count = walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|s| s == "rs").unwrap_or(false))
            .count();

        if count >= min {
            self.pass(&format!("{} ({} arquivos .rs, mínimo: {})", desc, count, min));
        } else if required {
            self.fail(desc, &format!("Apenas {} arquivo(s) .rs encontrado(s), mínimo: {}", count, min));
        } else {
            self.warn(desc, &format!("Apenas {} arquivo(s) .rs encontrado(s), recomendado: {}", count, min));
        }
    }

    fn check_cargo_command(&mut self, args: &[&str], desc: &str, required: bool) {
        let output = Command::new("cargo")
            .args(args)
            .current_dir(&self.crate_dir)
            .output();

        match output {
            Ok(o) if o.status.success() => self.pass(desc),
            Ok(_) => {
                if required {
                    self.fail(desc, "Comando falhou");
                } else {
                    self.warn(desc, "Comando falhou");
                }
            }
            Err(_) => {
                if required {
                    self.fail(desc, "cargo não encontrado ou comando falhou");
                } else {
                    self.warn(desc, "cargo não encontrado");
                }
            }
        }
    }

    fn pass(&mut self, msg: &str) {
        println!("✅ {}", msg);
    }

    fn fail(&mut self, desc: &str, reason: &str) {
        println!("❌ {} - {}", desc, reason);
        self.errors.push(CheckResult {
            name: desc.to_string(),
            passed: false,
            required: true,
            message: reason.to_string(),
        });
    }

    fn warn(&mut self, desc: &str, reason: &str) {
        println!("⚠️  {} - {}", desc, reason);
        self.warnings.push(CheckResult {
            name: desc.to_string(),
            passed: false,
            required: false,
            message: reason.to_string(),
        });
    }

    fn run_all_checks(&mut self) {
        println!("🔍 Verificando qualidade da crate");
        println!("📁 Diretório: {}\n", self.crate_dir.display());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 1: ESTRUTURA BÁSICA");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.check_file("Cargo.toml", true, "Cargo.toml");
        self.check_file("README.md", true, "README.md");
        self.check_file("LICENSE", true, "LICENSE");
        self.check_file(".gitignore", true, ".gitignore");
        self.check_file("CHANGELOG.md", false, "CHANGELOG.md");
        self.check_file("CITATION.cff", false, "CITATION.cff");

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 2: CONFIGURAÇÃO CARGO.TOML");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.check_content("Cargo.toml", "name =", "Campo 'name'", true);
        self.check_content("Cargo.toml", "version =", "Campo 'version'", true);
        self.check_content("Cargo.toml", "edition =", "Campo 'edition'", true);
        self.check_content("Cargo.toml", "license =", "Campo 'license'", true);
        self.check_content("Cargo.toml", "description =", "Campo 'description'", true);
        self.check_content("Cargo.toml", "repository =", "Campo 'repository'", true);
        self.check_content("Cargo.toml", "readme =", "Campo 'readme'", true);
        self.check_content("Cargo.toml", "rust-version =", "Campo 'rust-version'", true);
        self.check_content("Cargo.toml", "documentation =", "Campo 'documentation'", true);
        self.check_content("Cargo.toml", "exclude =", "Campo 'exclude'", false);
        self.check_content("Cargo.toml", "[package.metadata.docs.rs]", "Seção docs.rs", false);

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 3: ESTRUTURA DE CÓDIGO");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.check_rs_files("src", 1, "Diretório src/", true);
        self.check_rs_files("tests", 2, "Diretório tests/", true);
        self.check_rs_files("examples", 1, "Diretório examples/", true);
        self.check_rs_files("benches", 1, "Diretório benches/", false);

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 4: SEGURANÇA E QUALIDADE");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.check_file("SECURITY.md", false, "SECURITY.md");
        self.check_file("CODE_OF_CONDUCT.md", false, "CODE_OF_CONDUCT.md");
        self.check_file("deny.toml", false, "deny.toml");
        self.check_content("src/lib.rs", "#![forbid(unsafe_code)]", "#![forbid(unsafe_code)]", false);

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 5: CI/CD E WORKFLOWS");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.check_file(".github/workflows/ci.yml", false, "Workflow CI");
        self.check_file(".github/workflows/audit.yml", false, "Workflow Audit");
        self.check_file(".github/workflows/deny.yml", false, "Workflow Deny");
        self.check_file(".github/workflows/sbom.yml", false, "Workflow SBOM");

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 6: TEMPLATES GITHUB");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.check_file(".github/ISSUE_TEMPLATE/bug_report.md", false, "Template Bug Report");
        self.check_file(".github/ISSUE_TEMPLATE/feature_request.md", false, "Template Feature Request");
        self.check_file(".github/ISSUE_TEMPLATE/config.yml", false, "Template Config");
        self.check_file(".github/pull_request_template.md", false, "Template Pull Request");

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 7: DOCUMENTAÇÃO");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if let Ok(content) = fs::read_to_string(self.crate_dir.join("README.md")) {
            let badge_count = content.matches("img.shields.io").count() + content.matches("docs.rs").count();
            if badge_count >= 3 {
                self.pass(&format!("README.md com {} badges", badge_count));
            } else {
                self.warn("README.md badges", &format!("Apenas {} badge(s), recomendado: 3+", badge_count));
            }

            if content.contains("## Instalação") || content.contains("## Installation") {
                self.pass("Seção 'Instalação' no README");
            } else {
                self.warn("README.md", "Seção 'Instalação' não encontrada");
            }

            if content.contains("## Quickstart") || content.contains("```rust") {
                self.pass("Seção Quickstart/Exemplo no README");
            } else {
                self.warn("README.md", "Seção Quickstart/Exemplo não encontrada");
            }
        }

        self.check_file("RELEASE_NOTES.md", false, "RELEASE_NOTES.md");

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 FASE 8: VALIDAÇÃO DE CÓDIGO");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.check_cargo_command(&["fmt", "--all", "--", "--check"], "cargo fmt", true);
        // Clippy pode falhar mas não é crítico se não estiver instalado
        let _ = Command::new("cargo")
            .args(&["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"])
            .current_dir(&self.crate_dir)
            .output()
            .map(|o| {
                if o.status.success() {
                    self.pass("cargo clippy");
                } else {
                    self.warn("cargo clippy", "Warnings encontrados");
                }
            });
        self.check_cargo_command(&["test", "--all-features"], "cargo test", true);
    }

    fn print_summary(&self) -> i32 {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📊 RESUMO FINAL");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("✅ PERFEITO! Nenhum erro ou warning encontrado.");
            println!("✅ Crate atende ao padrão completo de qualidade!");
            return 0;
        } else if self.errors.is_empty() {
            println!("⚠️  ATENÇÃO: {} warning(s) encontrado(s)", self.warnings.len());
            println!("✅ Nenhum erro crítico. Crate atende ao padrão mínimo.");
            return 0;
        } else {
            println!("❌ ERRO: {} erro(s) e {} warning(s) encontrado(s)", self.errors.len(), self.warnings.len());
            println!("❌ Crate NÃO atende ao padrão mínimo de qualidade.");
            return 1;
        }
    }
}

fn main() {
    let crate_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut checker = QualityChecker::new(crate_dir);
    checker.run_all_checks();
    std::process::exit(checker.print_summary());
}
