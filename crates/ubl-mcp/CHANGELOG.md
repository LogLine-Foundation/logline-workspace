# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-25

### Added
- `gate` module with `PolicyGate` trait and implementations:
  - `AllowAll`, `DenyAll`, `ChallengeAll` for testing
  - `AllowlistGate`, `DenylistGate` for simple policies
  - `TdlnGate` for TDLN policy evaluation (requires `gate-tdln` feature)
- `audit` module with `AuditSink` trait and implementations:
  - `NoAudit`, `TracingAudit`, `MemoryAudit`
  - `UblAudit` for UBL Ledger (requires `audit` feature)
- `ToolCallRecord` struct for audit entries
- `SecureToolCall` pattern with fluent `.tool().execute()` API
- `RpcEndpoint` trait for transport abstraction
- `MockEndpoint` for testing
- New features: `transport-stdio`, `transport-http`, `gate-tdln`, `audit`
- Comprehensive integration tests for gate and audit

### Changed
- `McpClient` now takes generic `Gate`, `Audit`, and `Endpoint` parameters
- Client API changed from `call_tool_secure()` to `tool().execute()`
- TDLN dependencies now optional (only with `gate-tdln` feature)
- Lighter default build (no TDLN deps unless needed)

### Removed
- `GateContext` (replaced by `PolicyGate` trait)
- `McpTransport` (replaced by `RpcEndpoint`)
- `MockTransport` (replaced by `MockEndpoint`)

## [0.1.0] - 2025-01-24

### Added
- Initial release
- JSON-RPC 2.0 protocol types
- MCP tool definitions and results
- `McpClient` with TDLN Gate enforcement
- `ServerBuilder` with schema-first tool registration
- stdio transport
