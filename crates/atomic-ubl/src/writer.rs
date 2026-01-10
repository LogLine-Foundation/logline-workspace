//! Synchronous UBL writer with rotation, WAL, and Ed25519 signing.
use crate::{
    event::UblEvent,
    paths::{base_file_name, daily_dir, ts_rfc3339},
};
use anyhow::{Context, Result};
use ubl_codec as codec;
use ubl_crypto::{b64_encode, blake3_hex, key_id_v1, sign_cid_hex, SecretKey};
use ubl_types::{ActorId, AppId, NodeId, TenantId, TraceId};
use ed25519_dalek::{SigningKey, VerifyingKey};
use fs2::FileExt;
use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;

/// Estratégia de rotação
#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    /// Gira 1 arquivo por dia.
    Daily,
    /// Gira a cada hora.
    Hourly,
    /// Gira por tamanho (MB).
    SizeMb(u64),
}

/// Writer síncrono com bloqueio por arquivo.
pub struct UblWriter {
    root: PathBuf,
    app: AppId,
    tenant: TenantId,
    node: NodeId,
    actor: ActorId,
    sk: SigningKey,
    vk: VerifyingKey,
    key_id: String,
    rotation: Rotation,
}

impl UblWriter {
    /// Cria writer
    ///
    /// # Errors
    ///
    /// - Retorna erros de I/O ao preparar chaves ou caminhos
    pub fn new(
        root: impl AsRef<Path>,
        app: AppId,
        tenant: TenantId,
        node: NodeId,
        actor: ActorId,
        sk: SigningKey,
    ) -> Result<Self> {
        let vk = sk.verifying_key();
        let key_id = key_id_v1(&vk);
        Ok(Self {
            root: root.as_ref().to_path_buf(),
            app,
            tenant,
            node,
            actor,
            sk,
            vk,
            key_id,
            rotation: Rotation::Daily,
        })
    }
    /// Ajusta rotação
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_rotation(mut self, r: Rotation) -> Self {
        self.rotation = r;
        self
    }
    fn ensure_parent(dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)?;
        Ok(())
    }
    fn open_rotated(&self, now: OffsetDateTime) -> Result<(fs::File, PathBuf)> {
        let dir = daily_dir(&self.root, &self.app, &self.tenant, now);
        Self::ensure_parent(&dir)?;
        let mut part = 0u32;
        loop {
            let name = base_file_name(
                &self.node,
                if matches!(self.rotation, Rotation::SizeMb(_)) {
                    Some(part)
                } else {
                    None
                },
            );
            let path = dir.join(name);
            if let Rotation::SizeMb(max) = self.rotation {
                if path.exists() {
                    let md = fs::metadata(&path)?;
                    let mb = md.len() / (1024 * 1024);
                    if mb >= max {
                        part += 1;
                        continue;
                    }
                }
            }
            let file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            return Ok((file, path));
        }
    }
    fn last_cid(path: &Path) -> Result<Option<String>> {
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(path)?;
        for line in data.lines().rev() {
            if line.trim().is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line)?;
            if let Some(cid) = v.get("cid_hex").and_then(|x| x.as_str()) {
                return Ok(Some(cid.to_string()));
            }
        }
        Ok(None)
    }
    /// Escreve um evento (atomically com WAL .wal)
    ///
    /// # Errors
    ///
    /// - Erros de I/O ao criar/rotacionar arquivos ou ao escrever dados
    /// - Erros de serialização ao gerar o payload canônico
    pub fn append<T: Serialize>(
        &self,
        kind: &str,
        payload: &T,
        trace_id: Option<TraceId>,
        refs: Option<Vec<String>>,
    ) -> Result<UblEvent> {
        let now = OffsetDateTime::now_utc();
        let (file, path) = self.open_rotated(now)?;
        file.lock_exclusive()?;

        let canon = codec::to_canon_vec(payload).context("canon encode")?;
        let cid_hex = blake3_hex(&canon);
        let sig = sign_cid_hex(&SecretKey(self.sk.to_bytes()), &cid_hex);
        let sig_b64 = b64_encode(&sig);
        let pk_b64 = b64_encode(self.vk.as_bytes());
        let canon_b64 = b64_encode(&canon);
        let prev = Self::last_cid(&path)?;

        let ev = UblEvent {
            ts: ts_rfc3339(now),
            app: self.app.clone(),
            tenant: self.tenant.clone(),
            node: self.node.clone(),
            actor: self.actor.clone(),
            kind: kind.to_string(),
            trace_id,
            refs: refs.unwrap_or_default(),
            payload: serde_json::to_value(payload)?,
            canon_b64,
            cid_hex,
            sig_b64,
            pk_b64,
            key_id: self.key_id.clone(),
            prev_cid_hex: prev,
        };

        // WAL
        let wal_path = path.with_extension("ndjson.wal");
        let mut wal = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)?;
        writeln!(&mut wal, "{}", serde_json::to_string(&ev)?)?;
        wal.sync_all()?;

        // append definitivo
        let mut f = file;
        writeln!(&mut f, "{}", serde_json::to_string(&ev)?)?;
        f.sync_all()?;
        // limpa WAL (recria zero)
        drop(wal);
        let _ = fs::remove_file(&wal_path);

        Ok(ev)
    }
}
