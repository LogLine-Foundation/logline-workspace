//! `ubl-office` — The Agent Runtime (Wake · Work · Dream)
//!
//! Run agents like reliable services, not fragile notebooks.
//!
//! `ubl-office` is the execution environment for LogLine agents. It coordinates
//! thinking (TDLN), acting (MCP tools), memory, and policy (Gate) under one tight loop.
//! No root access, no mystery state, no shrug emojis.
//!
//! # What it does (in one screen)
//!
//! - **Boot**: load identity/constitution, attach transports, warm caches
//! - **Orient**: build a typed `CognitiveContext` (system directive, recall, constraints)
//! - **Decide**: call `tdln-brain` to produce a strict `SemanticUnit` (TDLN AST)
//! - **Gate**: run `tdln-gate` → Permit | Deny | Challenge
//! - **Act**: execute via `ubl-mcp` (MCP tools)
//! - **Dream**: consolidate short-term into durable memory; compact context; keep it fresh
//! - **Repeat**, with backpressure, watchdog timers, exponential backoff on failure
//!
//! # Mental Model
//!
//! ```text
//! ┌────────────┐  prepares   ┌──────────────┐  thinks   ┌──────────────┐   acts    ┌──────────────┐
//! │ Narrator   ├────────────▶│ ContextFrame ├──────────▶│ tdln-brain   ├──────────▶│ ubl-mcp/tools│
//! └────┬───────┘             └──────┬───────┘           └──────┬───────┘           └──────┬───────┘
//!      │                            │                          │                          │
//!      │ recall                     │ handover                 │ decision                 │ receipts
//!      ▼                            ▼                          ▼                          ▼
//! ┌────────────┐              ┌──────────────┐           ┌──────────────┐          ┌──────────────┐
//! │ Memory     │◀─────────────│ UBL Ledger   │◀─────────▶│ TDLN Gate    │          │ UBL Ledger   │
//! └────────────┘              └──────────────┘           └──────────────┘          └──────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use ubl_office::{Office, OfficeConfig, OfficeState, SessionType, SessionMode};
//!
//! let config = OfficeConfig {
//!     session_type: SessionType::Work,
//!     session_mode: SessionMode::Commitment,
//!     ..Default::default()
//! };
//! // Office::new(config, brain) to start
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod runtime;
mod memory;
mod narrator;
mod errors;
mod hooks;

pub use runtime::*;
pub use memory::*;
pub use narrator::*;
pub use errors::*;
pub use hooks::*;
