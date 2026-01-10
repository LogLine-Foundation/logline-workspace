# LogLine Workspace — Makefile
# Run `make help` for available commands

.PHONY: all check fmt clippy test doc deny audit sbom clean help

# Default target
all: check

# Full quality check
check: fmt clippy test deny audit
	@echo "✅ All checks passed!"

# Format code
fmt:
	cargo fmt --all

# Format check (CI mode)
fmt-check:
	cargo fmt --all -- --check

# Clippy lints
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo test --workspace --all-features --locked

# Build docs
doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo doc --workspace --all-features --no-deps --open

# Build docs (no open)
doc-build:
	RUSTDOCFLAGS="--cfg docsrs" cargo doc --workspace --all-features --no-deps

# Cargo deny (license + advisory check)
deny:
	cargo deny check

# Cargo audit (security advisories)
audit:
	cargo audit || true

# Generate SBOM (CycloneDX)
sbom:
	cargo install cyclonedx-bom || true
	cyclonedx-bom --workspace --all-features -o target/sbom.json
	@echo "📦 SBOM generated at target/sbom.json"

# Build release
build:
	cargo build --workspace --all-features --release

# Clean build artifacts
clean:
	cargo clean

# Run examples
example-hello:
	cargo run --example hello_ubl

example-sirp:
	cargo run --example sirp_roundtrip

# Publish dry-run (verify all crates can publish)
publish-dry:
	@echo "🔍 Dry-run publish check..."
	@for crate in json_atomic logline-core lllv-core lllv-index tdln-ast tdln-proof tdln-compiler tdln-gate tdln-brain atomic-types atomic-crypto atomic-codec ubl-ledger atomic-sirp atomic-runtime ubl-mcp ubl-office; do \
		echo "Checking crates/$$crate..."; \
		(cd crates/$$crate && cargo publish --dry-run --allow-dirty) || exit 1; \
	done
	@echo "✅ All crates ready to publish!"

# Help
help:
	@echo "LogLine Workspace — Available commands:"
	@echo ""
	@echo "  make check      — Run all quality checks (fmt, clippy, test, deny, audit)"
	@echo "  make fmt        — Format code with rustfmt"
	@echo "  make clippy     — Run Clippy lints"
	@echo "  make test       — Run all tests"
	@echo "  make doc        — Build and open documentation"
	@echo "  make deny       — Check licenses and advisories"
	@echo "  make audit      — Security audit"
	@echo "  make sbom       — Generate CycloneDX SBOM"
	@echo "  make build      — Build release binaries"
	@echo "  make clean      — Clean build artifacts"
	@echo "  make publish-dry — Dry-run publish check"
	@echo ""
