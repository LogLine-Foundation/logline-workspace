//! Narrative rendering for cognitive context.
//!
//! Converts a [`CognitiveContext`] into a vector of messages suitable
//! for LLM input, including system protocol, constraints, and memory.

use crate::{CognitiveContext, Message};

/// Render the cognitive context into LLM-ready messages.
///
/// The system message includes:
/// - Your directive (identity + role)
/// - System protocol (JSON-only output instructions)
/// - Active kernel constraints
/// - Relevant memory (recall)
///
/// Then appends conversation history.
///
/// # Errors
///
/// Returns an error if the context is malformed (currently infallible).
pub fn render(ctx: &CognitiveContext) -> Result<Vec<Message>, String> {
    let mut out = Vec::new();

    let mut sys = String::new();
    sys.push_str(&ctx.system_directive);

    // System protocol
    sys.push_str("\n\n### SYSTEM PROTOCOL ###\n");
    sys.push_str("You operate under LogLine OS. Output MUST be a valid JSON object representing a TDLN Semantic Unit.\n");
    sys.push_str("Do NOT include natural language outside JSON. If you must think, put it BEFORE the JSON and then a single JSON block.\n");
    sys.push_str("Format example: {\"kind\":\"verb\",\"slots\":{\"k\":\"v\"}}\n");

    // Constraints
    if !ctx.constraints.is_empty() {
        sys.push_str("\n### ACTIVE KERNEL CONSTRAINTS ###\n");
        sys.push_str("Violating these causes gate rejection:\n");
        for c in &ctx.constraints {
            sys.push_str(&format!("- {c}\n"));
        }
    }

    // Recall (memory)
    if !ctx.recall.is_empty() {
        sys.push_str("\n### RELEVANT MEMORY (RECALL) ###\n");
        for m in &ctx.recall {
            sys.push_str(&format!("- {m}\n"));
        }
    }

    out.push(Message::system(sys));
    out.extend(ctx.history.clone());

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_includes_protocol() {
        let ctx = CognitiveContext {
            system_directive: "You are a planner.".into(),
            ..Default::default()
        };
        let msgs = render(&ctx).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].content.contains("SYSTEM PROTOCOL"));
        assert!(msgs[0].content.contains("JSON"));
    }

    #[test]
    fn render_includes_constraints() {
        let ctx = CognitiveContext {
            system_directive: "Brain".into(),
            constraints: vec!["No transfers over 500".into()],
            ..Default::default()
        };
        let msgs = render(&ctx).unwrap();
        assert!(msgs[0].content.contains("No transfers over 500"));
        assert!(msgs[0].content.contains("KERNEL CONSTRAINTS"));
    }

    #[test]
    fn render_includes_recall() {
        let ctx = CognitiveContext {
            system_directive: "Brain".into(),
            recall: vec!["User balance: 420".into()],
            ..Default::default()
        };
        let msgs = render(&ctx).unwrap();
        assert!(msgs[0].content.contains("User balance: 420"));
        assert!(msgs[0].content.contains("RECALL"));
    }

    #[test]
    fn render_appends_history() {
        let ctx = CognitiveContext {
            system_directive: "Brain".into(),
            history: vec![
                Message::user("hello"),
                Message::assistant("hi"),
            ],
            ..Default::default()
        };
        let msgs = render(&ctx).unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[1].role, "user");
        assert_eq!(msgs[2].role, "assistant");
    }

    #[test]
    fn stable_render() {
        let ctx = CognitiveContext {
            system_directive: "Test".into(),
            recall: vec!["mem".into()],
            history: vec![Message::user("hello")],
            constraints: vec!["rule".into()],
        };
        let a = render(&ctx).unwrap();
        let b = render(&ctx).unwrap();
        assert_eq!(a, b);
    }
}
