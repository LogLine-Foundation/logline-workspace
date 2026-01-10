# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-01-25

### Added
- `SessionType` enum: Work, Assist, Deliberate, Research
- `SessionMode` enum: Commitment, Deliberation
- `TokenBudget` struct for input/output/daily quota limits
- `DreamConfig` struct for maintenance intervals
- `OfficeHooks` trait for event callbacks
- Receipt types: `DecisionReceipt`, `ToolCallReceipt`, `DreamReceipt`, `HandoverReceipt`, `QuotaBreachReceipt`
- `NoopHooks` and `TracingHooks` implementations
- `ledger_path` in `OfficeConfig` for NDJSON ledger
- `needs_maintenance()` and maintenance counter in `Narrator`
- `can_execute_tools()` and `can_write()` helpers in `Office`
- New error variants: `PolicyViolation`, `ContextOverflow`, `QuotaExceeded`, `Ledger`, `Provider`

### Changed
- Enhanced Narrator with session-specific constraints
- Dream configuration now uses `DreamConfig` instead of simple counter
- Metrics now track input/output tokens, dreams, and decisions since dream

## [0.1.0] - 2025-01-24

### Added
- Initial release
- `Office<B>` runtime with Wake/Work/Dream lifecycle
- `OfficeState` enum: Opening, Active, Maintenance, Closing
- `MemorySystem` with remember/recall/consolidate
- `Narrator` for cognitive context building
- State observation via `watch::Receiver`
