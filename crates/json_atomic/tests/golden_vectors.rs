//! Golden vector tests for JSON✯Atomic canonicalization.
//!
//! These tests verify deterministic canonicalization against known fixtures.
//! Golden vectors ensure cross-implementation compatibility and detect
//! canonicalization regressions.
//!
//! ## Adding new vectors
//!
//! 1. Create a JSON file in `tests/vectors/` with:
//!    - `description`: Human-readable explanation
//!    - `input`: The JSON value to canonicalize
//!    - `expected_canon_hex`: Canonical bytes in hex (empty = auto-generate)
//!    - `expected_cid_hex`: BLAKE3 CID in hex (empty = auto-generate)
//!
//! 2. Run `cargo test -p json_atomic golden_vectors -- --nocapture` to verify
//!
//! ## Regenerating vectors
//!
//! Run `cargo test -p json_atomic regenerate_vectors -- --nocapture --ignored`

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A golden vector test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoldenVector {
    description: String,
    input: serde_json::Value,
    #[serde(default)]
    expected_canon_hex: String,
    #[serde(default)]
    expected_cid_hex: String,
    #[serde(default)]
    note: Option<String>,
}

/// Result of processing a golden vector.
#[derive(Debug)]
struct VectorResult {
    path: PathBuf,
    description: String,
    canon_hex: String,
    cid_hex: String,
    canon_match: Option<bool>,
    cid_match: Option<bool>,
}

impl VectorResult {
    fn passed(&self) -> bool {
        self.canon_match.unwrap_or(true) && self.cid_match.unwrap_or(true)
    }
}

fn vectors_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors")
}

fn load_vector(path: &Path) -> Result<GoldenVector, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {:?}: {}", path, e))
}

fn process_vector(path: &Path) -> Result<VectorResult, String> {
    let vector = load_vector(path)?;
    
    // Canonicalize
    let canon = json_atomic::canonize(&vector.input)
        .map_err(|e| format!("Canonize failed for {:?}: {:?}", path, e))?;
    let canon_hex = hex::encode(&canon);
    
    // Compute CID (BLAKE3)
    let cid = blake3::hash(&canon);
    let cid_hex = hex::encode(cid.as_bytes());
    
    // Compare if expected values exist
    let canon_match = if vector.expected_canon_hex.is_empty() {
        None
    } else {
        Some(vector.expected_canon_hex == canon_hex)
    };
    
    let cid_match = if vector.expected_cid_hex.is_empty() {
        None
    } else {
        Some(vector.expected_cid_hex == cid_hex)
    };
    
    Ok(VectorResult {
        path: path.to_path_buf(),
        description: vector.description,
        canon_hex,
        cid_hex,
        canon_match,
        cid_match,
    })
}

fn collect_vector_files() -> Vec<PathBuf> {
    let dir = vectors_dir();
    if !dir.exists() {
        return Vec::new();
    }
    
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("read vectors dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "json"))
        .collect();
    
    files.sort();
    files
}

#[test]
fn golden_vectors() {
    let files = collect_vector_files();
    if files.is_empty() {
        println!("⚠️  No vectors found in {:?}, skipping", vectors_dir());
        return;
    }
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              JSON✯Atomic Golden Vector Tests                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let mut passed = 0;
    let mut failed = 0;
    let mut missing = 0;
    let mut failures = Vec::new();
    
    for path in &files {
        match process_vector(path) {
            Ok(result) => {
                let name = path.file_name().unwrap().to_string_lossy();
                
                // Status indicators
                let canon_status = match result.canon_match {
                    Some(true) => "✅",
                    Some(false) => "❌",
                    None => "⚠️",
                };
                let cid_status = match result.cid_match {
                    Some(true) => "✅",
                    Some(false) => "❌",
                    None => "⚠️",
                };
                
                println!("┌─ {} ─", name);
                println!("│  {}", result.description);
                println!("│  Canon: {} | CID: {}", canon_status, cid_status);
                
                if result.canon_match == Some(false) || result.cid_match == Some(false) {
                    println!("│");
                    println!("│  Actual canon : {}", &result.canon_hex[..64.min(result.canon_hex.len())]);
                    println!("│  Actual CID   : {}", result.cid_hex);
                    failed += 1;
                    failures.push((name.to_string(), result));
                } else if result.canon_match.is_none() || result.cid_match.is_none() {
                    println!("│");
                    println!("│  Generated canon_hex: {}", result.canon_hex);
                    println!("│  Generated cid_hex  : {}", result.cid_hex);
                    missing += 1;
                } else {
                    passed += 1;
                }
                println!("└────────────────────────────────────────────────────────");
            }
            Err(e) => {
                println!("❌ ERROR: {}", e);
                failed += 1;
            }
        }
    }
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Summary: {} passed, {} failed, {} need values               ║", 
             passed, failed, missing);
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    if !failures.is_empty() {
        println!("\n🔴 FAILURES:\n");
        for (name, result) in &failures {
            println!("  {} ({}):", name, result.description);
            println!("    canon_hex: {}", result.canon_hex);
            println!("    cid_hex  : {}", result.cid_hex);
        }
        panic!("{} golden vector(s) failed", failures.len());
    }
    
    if missing > 0 {
        println!("⚠️  {} vector(s) missing expected values - run regenerate_vectors to fix", missing);
    }
}

/// Regenerates all vector files with correct expected values.
/// Run with: `cargo test -p json_atomic regenerate_vectors -- --nocapture --ignored`
#[test]
#[ignore]
fn regenerate_vectors() {
    let files = collect_vector_files();
    if files.is_empty() {
        println!("No vectors found");
        return;
    }
    
    println!("\n🔄 Regenerating golden vectors...\n");
    
    for path in &files {
        let mut vector = match load_vector(path) {
            Ok(v) => v,
            Err(e) => {
                println!("❌ {}: {}", path.display(), e);
                continue;
            }
        };
        
        let canon = json_atomic::canonize(&vector.input).expect("canonize");
        let cid = blake3::hash(&canon);
        
        vector.expected_canon_hex = hex::encode(&canon);
        vector.expected_cid_hex = hex::encode(cid.as_bytes());
        vector.note = None;
        
        let json = serde_json::to_string_pretty(&vector).expect("serialize");
        fs::write(path, json).expect("write");
        
        println!("✅ {}", path.file_name().unwrap().to_string_lossy());
        println!("   canon: {}...", &vector.expected_canon_hex[..40.min(vector.expected_canon_hex.len())]);
        println!("   cid  : {}", vector.expected_cid_hex);
    }
    
    println!("\n✅ Done regenerating {} vectors", files.len());
}

/// Validates invariants that must hold for ANY input.
#[test]
fn canonicalization_invariants() {
    use serde_json::json;
    
    // Note: JSON✯Atomic does NOT allow floats - they are non-deterministic
    let test_cases = [
        json!({}),
        json!(null),
        json!(true),
        json!(false),
        json!(0),
        json!(-1),
        json!(""),
        json!("hello"),
        json!([]),
        json!([1, 2, 3]),
        json!({"a": 1}),
        json!({"z": 1, "a": 2}),
        json!({"nested": {"b": 2, "a": 1}}),
    ];
    
    println!("\n🔬 Canonicalization Invariants\n");
    
    for (i, input) in test_cases.iter().enumerate() {
        let canon1 = json_atomic::canonize(input).expect("first canonize");
        let canon2 = json_atomic::canonize(input).expect("second canonize");
        
        // Invariant 1: Determinism - same input → same output
        assert_eq!(canon1, canon2, "Invariant violated: non-deterministic for case {}", i);
        
        // Invariant 2: Valid UTF-8
        let s = String::from_utf8(canon1.clone())
            .expect(&format!("Invariant violated: non-UTF8 for case {}", i));
        
        // Invariant 3: Parses back to equivalent JSON
        let reparsed: serde_json::Value = serde_json::from_str(&s)
            .expect(&format!("Invariant violated: invalid JSON for case {}", i));
        assert_eq!(input, &reparsed, "Invariant violated: round-trip failed for case {}", i);
        
        // Invariant 4: CID is always 32 bytes
        let cid = blake3::hash(&canon1);
        assert_eq!(cid.as_bytes().len(), 32, "Invariant violated: CID not 32 bytes");
        
        println!("  ✅ Case {}: {} bytes → CID {}", i, canon1.len(), &hex::encode(cid.as_bytes())[..16]);
    }
    
    println!("\n✅ All invariants hold for {} test cases", test_cases.len());
}

/// Tests key ordering (RFC 8785 / JCS style).
#[test]
fn key_ordering() {
    use serde_json::json;
    
    let input = json!({
        "z": 1,
        "a": 2,
        "m": 3,
        "B": 4,
        "A": 5,
        "_": 6,
        "1": 7,
        "10": 8,
        "2": 9
    });
    
    let canon = json_atomic::canonize(&input).expect("canonize");
    let s = String::from_utf8(canon).expect("utf8");
    
    // Keys should appear in lexicographic order
    let positions: Vec<_> = ["\"1\"", "\"10\"", "\"2\"", "\"A\"", "\"B\"", "\"_\"", "\"a\"", "\"m\"", "\"z\""]
        .iter()
        .map(|k| s.find(k).expect(&format!("key {} not found", k)))
        .collect();
    
    // Verify strictly increasing positions
    for i in 1..positions.len() {
        assert!(
            positions[i] > positions[i-1],
            "Key ordering violation: keys not in lexicographic order\nCanon: {}",
            s
        );
    }
    
    println!("✅ Key ordering verified: {}", s);
}
