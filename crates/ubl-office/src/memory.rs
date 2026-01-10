//! Memory system for the Office runtime.
//!
//! Manages short-term scratchpad and (future) long-term LLLV storage.

use std::collections::VecDeque;

/// Maximum items in short-term memory.
const MAX_SHORT_TERM: usize = 100;

/// Memory system for agent state.
#[derive(Debug, Default)]
pub struct MemorySystem {
    /// Short-term buffer (ephemeral notes).
    short_buffer: VecDeque<String>,
}

impl MemorySystem {
    /// Create a new memory system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            short_buffer: VecDeque::with_capacity(MAX_SHORT_TERM),
        }
    }

    /// Add a note to short-term memory.
    pub fn remember(&mut self, note: impl Into<String>) {
        if self.short_buffer.len() >= MAX_SHORT_TERM {
            self.short_buffer.pop_front();
        }
        self.short_buffer.push_back(note.into());
    }

    /// Get recent memories (for recall).
    #[must_use]
    pub fn recent(&self, n: usize) -> Vec<String> {
        self.short_buffer
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect()
    }

    /// Recall relevant memories for a signal.
    ///
    /// Currently returns recent memories; future: semantic search via LLLV.
    #[must_use]
    pub fn recall(&self, _signal: &str) -> Vec<String> {
        self.recent(10)
    }

    /// Clear short-term memory.
    pub fn clear_short_term(&mut self) {
        self.short_buffer.clear();
    }

    /// Consolidate recent events into long-term storage.
    ///
    /// Stub for now; future: embed → pack → commit proof to LLLV.
    pub fn consolidate(&mut self, _events: &[String]) {
        // TODO: Implement LLLV consolidation
        // For now, just trim the short buffer
        while self.short_buffer.len() > MAX_SHORT_TERM / 2 {
            self.short_buffer.pop_front();
        }
    }

    /// Get the number of items in short-term memory.
    #[must_use]
    pub fn short_term_len(&self) -> usize {
        self.short_buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_and_recall() {
        let mut mem = MemorySystem::new();
        mem.remember("event 1");
        mem.remember("event 2");
        mem.remember("event 3");

        let recent = mem.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], "event 3");
        assert_eq!(recent[1], "event 2");
    }

    #[test]
    fn memory_cap() {
        let mut mem = MemorySystem::new();
        for i in 0..150 {
            mem.remember(format!("event {i}"));
        }
        assert_eq!(mem.short_term_len(), MAX_SHORT_TERM);
    }

    #[test]
    fn consolidate_trims() {
        let mut mem = MemorySystem::new();
        for i in 0..80 {
            mem.remember(format!("event {i}"));
        }
        mem.consolidate(&[]);
        assert!(mem.short_term_len() <= MAX_SHORT_TERM / 2);
    }
}
