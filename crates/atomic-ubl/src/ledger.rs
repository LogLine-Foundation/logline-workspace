//! Minimal NDJSON ledger with canonical bytes, CID verification, and optional signing.
//!
//! This module provides a simplified, deterministic ledger API using:
//! - `json_atomic::canonize` for canonical bytes
//! - `BLAKE3(canonical_bytes)` for CID
//! - Optional Ed25519 signing with domain `UBL:LEDGER:v1`
//!
//! ## Production features
//!
//! - **Rotation**: by size or hourly
//! - **Fsync**: configurable durability policy
//! - **Lock**: single writer per directory
//! - **Metrics**: optional via `metrics` feature

use crate::UBL_DOMAIN_SIGN;
use ubl_types::{Cid32, PublicKeyBytes, SignatureBytes};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

// ══════════════════════════════════════════════════════════════════════════════
// Errors
// ══════════════════════════════════════════════════════════════════════════════

/// Errors from ledger operations.
#[derive(Debug, Error)]
pub enum LedgerError {
    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization error.
    #[error("serde json: {0}")]
    Serde(#[from] serde_json::Error),
    /// Canonicalization error.
    #[error("canon error: {0}")]
    Canon(String),
    /// CID mismatch (content integrity).
    #[error("cid mismatch")]
    CidMismatch,
    /// Signature missing (partial signature state).
    #[error("signature missing")]
    SigMissing,
    /// Signature verification failed.
    #[error("signature invalid")]
    SigInvalid,
    /// Writer lock already held.
    #[error("writer lock held by another process")]
    LockHeld,
}

// ══════════════════════════════════════════════════════════════════════════════
// Types
// ══════════════════════════════════════════════════════════════════════════════

/// Result of append for idempotency/offset tracking.
#[derive(Debug, Clone)]
pub struct AppendResult {
    /// Path of the file written to.
    pub path: String,
    /// Line number (1-based).
    pub line_no: u64,
    /// CID of the appended entry.
    pub cid: Cid32,
}

/// Rotation policies for NDJSON files.
#[derive(Debug, Clone, Default)]
pub enum RotatePolicy {
    /// Rotate when file exceeds N bytes.
    BySizeBytes(u64),
    /// Rotate every hour (YYYY-MM-DD/HH.ndjson).
    #[default]
    Hourly,
    /// No rotation.
    None,
}

/// Fsync (durability) policies.
#[derive(Debug, Clone)]
pub enum FsyncPolicy {
    /// Fsync every N lines.
    EveryNLines(u32),
    /// Fsync at least every N milliseconds.
    IntervalMs(u64),
    /// Manual fsync only.
    Manual,
}

impl Default for FsyncPolicy {
    fn default() -> Self {
        FsyncPolicy::EveryNLines(100)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// LedgerEntry
// ══════════════════════════════════════════════════════════════════════════════

/// Minimal ledger entry (NDJSON line).
///
/// Contains canonical bytes of the intent, their CID, and optional signature.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerEntry {
    /// ISO-8601 timestamp (UTC).
    pub ts: String,
    /// CID = BLAKE3(canonical intent bytes).
    pub cid: Cid32,
    /// Canonical intent bytes (from `json_atomic::canonize`).
    #[serde(with = "serde_bytes")]
    pub intent: Vec<u8>,
    /// Logical actor (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    /// Extra blob (forward-compatible).
    #[serde(with = "serde_bytes", default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<u8>,
    /// Public key (if signed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<PublicKeyBytes>,
    /// Signature (if signed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureBytes>,
}

impl LedgerEntry {
    /// Creates an unsigned entry. CID = BLAKE3(canonical bytes).
    ///
    /// # Errors
    ///
    /// Returns error if canonicalization fails.
    pub fn unsigned(
        intent_value: &serde_json::Value,
        actor: Option<String>,
        extra: &[u8],
    ) -> Result<Self, LedgerError> {
        let canon =
            json_atomic::canonize(intent_value).map_err(|e| LedgerError::Canon(format!("{e:?}")))?;
        let cid = ubl_crypto::blake3_cid(&canon);
        Ok(Self {
            ts: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into()),
            cid,
            intent: canon,
            actor,
            extra: extra.to_vec(),
            pubkey: None,
            signature: None,
        })
    }

    /// Signs the entry with domain `UBL:LEDGER:v1` + CID.
    #[cfg(feature = "signing")]
    #[must_use]
    pub fn sign(mut self, sk: &ubl_crypto::SecretKey) -> Self {
        let pk = ubl_crypto::derive_public_bytes(&sk.0);
        let msg = sign_message(&self.cid);
        let sig = ubl_crypto::sign_bytes(&msg, &sk.0);
        self.pubkey = Some(pk);
        self.signature = Some(sig);
        self
    }

    /// Verifies: (1) CID == BLAKE3(intent); (2) signature if present.
    ///
    /// # Errors
    ///
    /// Returns error if CID mismatch or signature invalid.
    pub fn verify(&self) -> Result<(), LedgerError> {
        // 1) CID check
        let cid_check = ubl_crypto::blake3_cid(&self.intent);
        if cid_check != self.cid {
            return Err(LedgerError::CidMismatch);
        }

        // 2) Signature check (optional)
        match (&self.pubkey, &self.signature) {
            (Some(pk), Some(sig)) => {
                #[cfg(feature = "signing")]
                {
                    let msg = sign_message(&self.cid);
                    if !ubl_crypto::verify_bytes(&msg, pk, sig) {
                        return Err(LedgerError::SigInvalid);
                    }
                }
                #[cfg(not(feature = "signing"))]
                {
                    let _ = (pk, sig);
                    return Err(LedgerError::SigInvalid);
                }
            }
            (None, None) => { /* ok: unsigned entry */ }
            _ => return Err(LedgerError::SigMissing),
        }
        Ok(())
    }
}

/// Builds the signing message: domain + CID.
#[cfg(feature = "signing")]
fn sign_message(cid: &Cid32) -> Vec<u8> {
    let mut m = Vec::with_capacity(UBL_DOMAIN_SIGN.len() + 32);
    m.extend_from_slice(UBL_DOMAIN_SIGN);
    m.extend_from_slice(&cid.0);
    m
}

// ══════════════════════════════════════════════════════════════════════════════
// Writer Lock (single writer per directory)
// ══════════════════════════════════════════════════════════════════════════════

/// Guard for exclusive writer access.
#[derive(Debug)]
struct WriterLock {
    lock_path: PathBuf,
}

impl WriterLock {
    /// Acquires exclusive lock via lock file.
    fn acquire(dir: &Path) -> Result<Self, LedgerError> {
        let lock_path = dir.join(".ubl-writer.lock");

        // Try to create exclusively
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&lock_path)
            {
                Ok(_) => Ok(Self { lock_path }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    Err(LedgerError::LockHeld)
                }
                Err(e) => Err(LedgerError::Io(e)),
            }
        }

        #[cfg(not(unix))]
        {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(_) => Ok(Self { lock_path }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    Err(LedgerError::LockHeld)
                }
                Err(e) => Err(LedgerError::Io(e)),
            }
        }
    }
}

impl Drop for WriterLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// LedgerWriter (production)
// ══════════════════════════════════════════════════════════════════════════════

/// Production NDJSON writer with rotation, fsync, and locking.
pub struct LedgerWriter {
    file: BufWriter<File>,
    current_path: PathBuf,
    base_dir: PathBuf,
    line_no: u64,
    last_sync_line: u64,
    opened_at: OffsetDateTime,
    rotate: RotatePolicy,
    fsync: FsyncPolicy,
    _lock: WriterLock,
}

/// Global timestamp for interval-based fsync.
static LAST_FSYNC: Lazy<Mutex<SystemTime>> = Lazy::new(|| Mutex::new(SystemTime::now()));

impl LedgerWriter {
    /// Opens with custom rotation and fsync policies.
    ///
    /// # Errors
    ///
    /// Returns error if lock cannot be acquired or file cannot be opened.
    pub fn open_with<P: AsRef<Path>>(
        path: P,
        rotate: RotatePolicy,
        fsync: FsyncPolicy,
    ) -> Result<Self, LedgerError> {
        let path = path.as_ref().to_path_buf();
        let base_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        std::fs::create_dir_all(&base_dir)?;
        let _lock = WriterLock::acquire(&base_dir)?;

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let line_no = 0u64; // Could count lines on open if needed

        Ok(Self {
            file: BufWriter::new(file),
            current_path: path,
            base_dir,
            line_no,
            last_sync_line: 0,
            opened_at: OffsetDateTime::now_utc(),
            rotate,
            fsync,
            _lock,
        })
    }

    /// Opens with default policies (Hourly rotation, fsync every 100 lines).
    ///
    /// # Errors
    ///
    /// Returns error if lock cannot be acquired or file cannot be opened.
    pub fn open_append<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        Self::open_with(
            path,
            RotatePolicy::Hourly,
            FsyncPolicy::EveryNLines(100),
        )
    }

    /// Appends an entry with automatic rotation and fsync.
    ///
    /// # Errors
    ///
    /// Returns error if write fails.
    pub fn append(&mut self, entry: &LedgerEntry) -> Result<AppendResult, LedgerError> {
        // 1) Check rotation
        self.maybe_rotate()?;

        // 2) Canonical serialization
        let v = serde_json::to_value(entry)?;
        let canon = json_atomic::canonize(&v).map_err(|e| LedgerError::Canon(format!("{e:?}")))?;
        self.file.write_all(&canon)?;
        self.file.write_all(b"\n")?;
        self.line_no += 1;

        // 3) Fsync per policy
        self.maybe_fsync()?;

        #[cfg(feature = "metrics")]
        metrics::counter!("ubl_entries_appended_total").increment(1);

        Ok(AppendResult {
            path: self.current_path.to_string_lossy().into_owned(),
            line_no: self.line_no,
            cid: entry.cid,
        })
    }

    /// Forces fsync to disk.
    ///
    /// # Errors
    ///
    /// Returns error if sync fails.
    pub fn fsync(&mut self) -> Result<(), LedgerError> {
        self.file.flush()?;
        self.file.get_ref().sync_data()?;
        self.last_sync_line = self.line_no;
        Ok(())
    }

    /// Returns current file path.
    #[must_use]
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// Returns current line number.
    #[must_use]
    pub fn line_no(&self) -> u64 {
        self.line_no
    }

    fn maybe_fsync(&mut self) -> Result<(), LedgerError> {
        match &self.fsync {
            FsyncPolicy::EveryNLines(n) => {
                if (self.line_no - self.last_sync_line) >= u64::from(*n) {
                    self.fsync()?;
                }
            }
            FsyncPolicy::IntervalMs(ms) => {
                let mut guard = LAST_FSYNC.lock();
                if guard.elapsed().unwrap_or(Duration::ZERO) >= Duration::from_millis(*ms) {
                    self.fsync()?;
                    *guard = SystemTime::now();
                }
            }
            FsyncPolicy::Manual => {}
        }
        Ok(())
    }

    fn maybe_rotate(&mut self) -> Result<(), LedgerError> {
        let need = match &self.rotate {
            RotatePolicy::None => false,
            RotatePolicy::BySizeBytes(max) => {
                let size = self.file.get_ref().metadata()?.len();
                size >= *max
            }
            RotatePolicy::Hourly => {
                let now = OffsetDateTime::now_utc();
                now.hour() != self.opened_at.hour() || now.date() != self.opened_at.date()
            }
        };

        if !need {
            return Ok(());
        }

        // Flush and sync current file
        self.file.flush()?;
        self.file.get_ref().sync_data()?;

        // Open new file with path YYYY-MM-DD/HH.ndjson
        let now = OffsetDateTime::now_utc();
        let dir = self.base_dir.join(format!("{}", now.date()));
        std::fs::create_dir_all(&dir)?;
        let next = dir.join(format!("{:02}.ndjson", now.hour()));

        let f = OpenOptions::new().create(true).append(true).open(&next)?;
        self.file = BufWriter::new(f);
        self.current_path = next;
        self.line_no = 0;
        self.last_sync_line = 0;
        self.opened_at = now;

        #[cfg(feature = "metrics")]
        metrics::counter!("ubl_writer_rotate_total").increment(1);

        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// SimpleLedgerWriter (basic, no rotation/lock)
// ══════════════════════════════════════════════════════════════════════════════

/// Basic append-only NDJSON writer (no rotation, no lock).
pub struct SimpleLedgerWriter {
    file: File,
}

impl SimpleLedgerWriter {
    /// Opens (or creates) file for append.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be opened.
    pub fn open_append<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file })
    }

    /// Appends an entry as a canonical NDJSON line.
    ///
    /// # Errors
    ///
    /// Returns error if write fails.
    pub fn append(&mut self, entry: &LedgerEntry) -> Result<(), LedgerError> {
        let v = serde_json::to_value(entry)?;
        let canon = json_atomic::canonize(&v).map_err(|e| LedgerError::Canon(format!("{e:?}")))?;
        self.file.write_all(&canon)?;
        self.file.write_all(b"\n")?;
        Ok(())
    }

    /// Flushes and syncs to disk.
    ///
    /// # Errors
    ///
    /// Returns error if sync fails.
    pub fn sync(&mut self) -> Result<(), LedgerError> {
        self.file.sync_all()?;
        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Reader
// ══════════════════════════════════════════════════════════════════════════════

/// NDJSON reader with verification.
pub struct SimpleLedgerReader<R: BufRead> {
    inner: R,
}

impl<R: BufRead> SimpleLedgerReader<R> {
    /// Creates a new reader.
    #[must_use]
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl SimpleLedgerReader<BufReader<File>> {
    /// Opens a file for reading.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be opened.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        let f = File::open(path)?;
        Ok(Self {
            inner: BufReader::new(f),
        })
    }
}

impl<R: BufRead> SimpleLedgerReader<R> {
    /// Returns an iterator over verified entries.
    pub fn iter(self) -> SimpleLedgerIter<R> {
        SimpleLedgerIter {
            inner: self.inner,
            buf: String::new(),
        }
    }
}

/// Iterator over ledger entries with verification.
pub struct SimpleLedgerIter<R: BufRead> {
    inner: R,
    buf: String,
}

impl<R: BufRead> Iterator for SimpleLedgerIter<R> {
    type Item = Result<LedgerEntry, LedgerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.clear();
        match self.inner.read_line(&mut self.buf) {
            Ok(0) => None, // EOF
            Ok(_) => {
                // Parse → verify → yield
                let entry: LedgerEntry = match serde_json::from_str(&self.buf) {
                    Ok(e) => e,
                    Err(e) => return Some(Err(LedgerError::Serde(e))),
                };
                Some(entry.verify().map(|()| entry))
            }
            Err(e) => Some(Err(LedgerError::Io(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn unsigned_roundtrip() -> Result<(), LedgerError> {
        let intent = json!({"intent":"Grant","to":"alice","amount":3});
        let e = LedgerEntry::unsigned(&intent, Some("tester".into()), b"")?;
        e.verify()?;

        let tmp = NamedTempFile::new().unwrap();
        {
            let mut w = SimpleLedgerWriter::open_append(tmp.path())?;
            w.append(&e)?;
            w.sync()?;
        }
        let r = SimpleLedgerReader::from_path(tmp.path())?
            .iter()
            .next()
            .unwrap()?;
        assert_eq!(e.cid, r.cid);
        assert_eq!(e.intent, r.intent);
        Ok(())
    }

    #[test]
    fn cid_mismatch_detected() -> Result<(), LedgerError> {
        let intent = json!({"test":"value"});
        let mut e = LedgerEntry::unsigned(&intent, None, b"")?;
        e.cid.0[0] ^= 0xFF;
        assert!(matches!(e.verify(), Err(LedgerError::CidMismatch)));
        Ok(())
    }

    #[cfg(feature = "signing")]
    #[test]
    fn signed_roundtrip() -> Result<(), LedgerError> {
        use ubl_crypto::Keypair;

        let kp = Keypair::generate();
        let intent = json!({"intent":"Freeze","id":"X"});
        let e = LedgerEntry::unsigned(&intent, None, b"")?.sign(&kp.sk);
        e.verify()?;

        let tmp = NamedTempFile::new().unwrap();
        {
            let mut w = SimpleLedgerWriter::open_append(tmp.path())?;
            w.append(&e)?;
            w.sync()?;
        }
        let r = SimpleLedgerReader::from_path(tmp.path())?
            .iter()
            .next()
            .unwrap()?;
        assert_eq!(e.cid, r.cid);
        assert!(r.pubkey.is_some());
        assert!(r.signature.is_some());
        Ok(())
    }

    #[test]
    fn multiple_entries() -> Result<(), LedgerError> {
        let tmp = NamedTempFile::new().unwrap();
        {
            let mut w = SimpleLedgerWriter::open_append(tmp.path())?;
            for i in 0..5 {
                let intent = json!({"seq": i});
                let e = LedgerEntry::unsigned(&intent, None, b"")?;
                w.append(&e)?;
            }
            w.sync()?;
        }
        let count = SimpleLedgerReader::from_path(tmp.path())?.iter().count();
        assert_eq!(count, 5);
        Ok(())
    }

    #[test]
    fn production_writer_append_result() -> Result<(), LedgerError> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ledger.ndjson");
        let mut w = LedgerWriter::open_with(path, RotatePolicy::None, FsyncPolicy::Manual)?;

        let intent = json!({"action":"test"});
        let e = LedgerEntry::unsigned(&intent, None, b"")?;
        let res = w.append(&e)?;

        assert_eq!(res.line_no, 1);
        assert_eq!(res.cid, e.cid);
        w.fsync()?;
        Ok(())
    }

    #[test]
    fn writer_lock_prevents_double_open() -> Result<(), LedgerError> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ledger.ndjson");

        let _w1 = LedgerWriter::open_with(&path, RotatePolicy::None, FsyncPolicy::Manual)?;
        let w2 = LedgerWriter::open_with(&path, RotatePolicy::None, FsyncPolicy::Manual);

        assert!(matches!(w2, Err(LedgerError::LockHeld)));
        Ok(())
    }
}

