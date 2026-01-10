//! Small utilities for token budgets and guards.

/// Clamp a token budget to a hard maximum.
///
/// Returns `(input_budget, output_budget)` where:
/// - `output_budget` is the clamped requested value
/// - `input_budget` is the remaining budget for input
///
/// # Example
///
/// ```rust
/// use tdln_brain::util::clamp_budget;
///
/// let (input, output) = clamp_budget(Some(2000), 4096);
/// assert_eq!(output, 2000);
/// assert_eq!(input, 2096);
/// ```
#[must_use]
pub fn clamp_budget(requested: Option<u32>, hard_max: u32) -> (u32, u32) {
    let output = requested.unwrap_or(hard_max).min(hard_max);
    let input = hard_max.saturating_sub(output);
    (input, output)
}

/// Estimate token count from text (rough: ~4 chars per token).
#[must_use]
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() as u32 + 3) / 4
}

/// Check if a context would exceed a token limit.
#[must_use]
pub fn would_overflow(text: &str, limit: u32) -> bool {
    estimate_tokens(text) > limit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_budget_respects_max() {
        let (input, output) = clamp_budget(Some(5000), 4096);
        assert_eq!(output, 4096);
        assert_eq!(input, 0);
    }

    #[test]
    fn clamp_budget_uses_requested() {
        let (input, output) = clamp_budget(Some(1000), 4096);
        assert_eq!(output, 1000);
        assert_eq!(input, 3096);
    }

    #[test]
    fn clamp_budget_default_to_max() {
        let (input, output) = clamp_budget(None, 4096);
        assert_eq!(output, 4096);
        assert_eq!(input, 0);
    }

    #[test]
    fn estimate_tokens_rough() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("test"), 1);
        assert_eq!(estimate_tokens("hello world"), 3);
    }

    #[test]
    fn would_overflow_check() {
        assert!(!would_overflow("short", 100));
        assert!(would_overflow(&"x".repeat(500), 100));
    }
}
