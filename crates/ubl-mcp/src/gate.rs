//! Policy gate for tool call authorization.
//!
//! Every tool call passes through a `PolicyGate` before execution.
//! The gate returns `Permit`, `Deny`, or `Challenge`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Gate decision for a tool call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    /// Allow the tool call to proceed.
    Permit,
    /// Block the tool call.
    Deny {
        /// Reason for denial.
        reason: String,
    },
    /// Require additional approval (interactive consent).
    Challenge {
        /// Reason for challenge.
        reason: String,
    },
}

impl GateDecision {
    /// Check if this is a permit decision.
    #[must_use]
    pub fn is_permit(&self) -> bool {
        matches!(self, Self::Permit)
    }

    /// Check if this is a deny decision.
    #[must_use]
    pub fn is_deny(&self) -> bool {
        matches!(self, Self::Deny { .. })
    }

    /// Check if this is a challenge decision.
    #[must_use]
    pub fn is_challenge(&self) -> bool {
        matches!(self, Self::Challenge { .. })
    }
}

/// Trait for policy gates that decide on tool calls.
///
/// Implement this trait to create custom authorization logic.
///
/// # Example
///
/// ```rust
/// use ubl_mcp::gate::{PolicyGate, GateDecision};
/// use async_trait::async_trait;
/// use serde_json::Value;
///
/// struct DenyDestructive;
///
/// #[async_trait]
/// impl PolicyGate for DenyDestructive {
///     async fn decide(&self, tool: &str, _args: &Value) -> GateDecision {
///         if tool.starts_with("delete") || tool.starts_with("drop") {
///             GateDecision::Deny { reason: "destructive operations are blocked".into() }
///         } else {
///             GateDecision::Permit
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait PolicyGate: Send + Sync {
    /// Decide whether a tool call should proceed.
    ///
    /// # Arguments
    /// * `tool` - The tool name being called
    /// * `args` - The arguments being passed to the tool
    ///
    /// # Returns
    /// A `GateDecision` indicating whether to permit, deny, or challenge.
    async fn decide(&self, tool: &str, args: &Value) -> GateDecision;
}

/// A gate that allows all tool calls (for testing/development).
pub struct AllowAll;

#[async_trait]
impl PolicyGate for AllowAll {
    async fn decide(&self, _tool: &str, _args: &Value) -> GateDecision {
        GateDecision::Permit
    }
}

/// A gate that denies all tool calls (for testing).
pub struct DenyAll {
    /// The reason for denial.
    pub reason: String,
}

impl DenyAll {
    /// Create a new DenyAll gate.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl Default for DenyAll {
    fn default() -> Self {
        Self::new("all tool calls are denied")
    }
}

#[async_trait]
impl PolicyGate for DenyAll {
    async fn decide(&self, _tool: &str, _args: &Value) -> GateDecision {
        GateDecision::Deny {
            reason: self.reason.clone(),
        }
    }
}

/// A gate that challenges all tool calls (requires consent).
pub struct ChallengeAll {
    /// The reason for challenge.
    pub reason: String,
}

impl ChallengeAll {
    /// Create a new ChallengeAll gate.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl Default for ChallengeAll {
    fn default() -> Self {
        Self::new("all tool calls require consent")
    }
}

#[async_trait]
impl PolicyGate for ChallengeAll {
    async fn decide(&self, _tool: &str, _args: &Value) -> GateDecision {
        GateDecision::Challenge {
            reason: self.reason.clone(),
        }
    }
}

/// A gate based on a tool allowlist.
pub struct AllowlistGate {
    /// Set of allowed tool names.
    allowed: std::collections::HashSet<String>,
}

impl AllowlistGate {
    /// Create a new allowlist gate.
    #[must_use]
    pub fn new(allowed: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            allowed: allowed.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl PolicyGate for AllowlistGate {
    async fn decide(&self, tool: &str, _args: &Value) -> GateDecision {
        if self.allowed.contains(tool) {
            GateDecision::Permit
        } else {
            GateDecision::Deny {
                reason: format!("tool '{tool}' is not in the allowlist"),
            }
        }
    }
}

/// A gate based on a tool denylist.
pub struct DenylistGate {
    /// Set of denied tool names.
    denied: std::collections::HashSet<String>,
}

impl DenylistGate {
    /// Create a new denylist gate.
    #[must_use]
    pub fn new(denied: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            denied: denied.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl PolicyGate for DenylistGate {
    async fn decide(&self, tool: &str, _args: &Value) -> GateDecision {
        if self.denied.contains(tool) {
            GateDecision::Deny {
                reason: format!("tool '{tool}' is in the denylist"),
            }
        } else {
            GateDecision::Permit
        }
    }
}

/// TDLN Gate integration (requires `gate-tdln` feature).
#[cfg(feature = "gate-tdln")]
pub mod tdln_impl {
    use super::*;
    use tdln_gate::PolicyCtx;

    /// Gate that uses TDLN policy evaluation.
    pub struct TdlnGate {
        /// Policy context for TDLN evaluation.
        pub ctx: PolicyCtx,
    }

    impl TdlnGate {
        /// Create a new TDLN gate with the given policy context.
        #[must_use]
        pub fn new(ctx: PolicyCtx) -> Self {
            Self { ctx }
        }

        /// Create with default policy context.
        #[must_use]
        pub fn with_defaults() -> Self {
            Self::new(PolicyCtx {
                allow_freeform: true,
            })
        }
    }

    #[async_trait]
    impl PolicyGate for TdlnGate {
        async fn decide(&self, tool: &str, args: &Value) -> GateDecision {
            // Create a virtual intent for the tool call
            let intent_text = format!("call tool {} with {}", tool, args);

            // Compile to TDLN
            let compile_ctx = tdln_compiler::CompileCtx {
                rule_set: "v1".into(),
            };
            let compiled = match tdln_compiler::compile(&intent_text, &compile_ctx) {
                Ok(c) => c,
                Err(e) => {
                    return GateDecision::Deny {
                        reason: format!("failed to compile intent: {e}"),
                    }
                }
            };

            // Run gate preflight
            let preflight = match tdln_gate::preflight(&compiled, &self.ctx) {
                Ok(p) => p,
                Err(e) => {
                    return GateDecision::Deny {
                        reason: format!("gate preflight error: {e}"),
                    }
                }
            };

            // Map TDLN decision to our GateDecision
            match preflight.decision {
                tdln_gate::Decision::Allow => GateDecision::Permit,
                tdln_gate::Decision::Deny => GateDecision::Deny {
                    reason: "policy denied the tool call".into(),
                },
                tdln_gate::Decision::NeedsConsent => GateDecision::Challenge {
                    reason: "tool call requires consent".into(),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allow_all_permits() {
        let gate = AllowAll;
        let decision = gate.decide("any_tool", &serde_json::json!({})).await;
        assert!(decision.is_permit());
    }

    #[tokio::test]
    async fn deny_all_denies() {
        let gate = DenyAll::default();
        let decision = gate.decide("any_tool", &serde_json::json!({})).await;
        assert!(decision.is_deny());
    }

    #[tokio::test]
    async fn challenge_all_challenges() {
        let gate = ChallengeAll::default();
        let decision = gate.decide("any_tool", &serde_json::json!({})).await;
        assert!(decision.is_challenge());
    }

    #[tokio::test]
    async fn allowlist_permits_listed() {
        let gate = AllowlistGate::new(["echo", "read"]);
        assert!(gate.decide("echo", &serde_json::json!({})).await.is_permit());
        assert!(gate.decide("read", &serde_json::json!({})).await.is_permit());
        assert!(gate
            .decide("delete", &serde_json::json!({}))
            .await
            .is_deny());
    }

    #[tokio::test]
    async fn denylist_denies_listed() {
        let gate = DenylistGate::new(["delete", "drop"]);
        assert!(gate
            .decide("delete", &serde_json::json!({}))
            .await
            .is_deny());
        assert!(gate.decide("drop", &serde_json::json!({})).await.is_deny());
        assert!(gate.decide("echo", &serde_json::json!({})).await.is_permit());
    }
}
