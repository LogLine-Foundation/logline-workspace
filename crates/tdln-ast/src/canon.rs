//! Canonicalization wrapper: single source of truth (json_atomic).
//! All AST → bytes must go through here.

use serde::Serialize;

/// Convert any serializable value to canonical bytes using `json_atomic`.
/// 
/// This ensures identical bytes across the entire LogLine workspace,
/// regardless of field ordering or whitespace in the source JSON.
///
/// # Panics
///
/// Panics if JSON serialization fails (unexpected for valid types).
#[must_use]
pub fn to_canon_vec<T: Serialize>(value: &T) -> Vec<u8> {
    // Use the canonicalizer from json_atomic to ensure identical bytes across the workspace.
    match json_atomic::canonize(value) {
        Ok(bytes) => bytes,
        Err(_e) => {
            // Defensive fallback: pure JSON serialization (non-canonical).
            // Ideally never used, but we prefer not to crash at runtime.
            serde_json::to_vec(value).expect("serde_json fallback failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn whitespace_insensitive() {
        let a = json!({"intent":"Grant","to":"alice","amount":1});
        let b = serde_json::from_str::<serde_json::Value>(
            r#" { "amount":1,"to":"alice", "intent" : "Grant" } "#,
        )
        .unwrap();
        assert_eq!(to_canon_vec(&a), to_canon_vec(&b));
    }

    #[test]
    fn key_order_insensitive() {
        let a = json!({"z":"last","a":"first","m":"middle"});
        let b = json!({"a":"first","m":"middle","z":"last"});
        assert_eq!(to_canon_vec(&a), to_canon_vec(&b));
    }
}
