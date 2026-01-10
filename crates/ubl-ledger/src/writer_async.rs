//! Asynchronous UBL writer backed by a Tokio task and WAL files.
use crate::{
    event::UblEvent,
    paths::{daily_dir, ts_rfc3339},
};
use anyhow::Result;
use ubl_codec as codec;
use ubl_crypto::{b64_encode, blake3_hex, key_id_v1, sign_cid_hex, SecretKey};
use ubl_types::{ActorId, AppId, NodeId, TenantId, TraceId};
use ed25519_dalek::SigningKey;
use serde::Serialize;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tokio::{fs, io::AsyncWriteExt, sync::mpsc};

/// Config de async writer
#[derive(Clone)]
pub struct AsyncConfig {
    /// Capacidade do canal interno.
    pub channel_cap: usize,
}
impl Default for AsyncConfig {
    fn default() -> Self {
        Self { channel_cap: 4096 }
    }
}

/// Writer assíncrono com mpsc.
pub struct UblWriterAsync {
    root: PathBuf,
    app: AppId,
    tenant: TenantId,
    node: NodeId,
    actor: ActorId,
    sk: SigningKey,
    key_id: String,
    tx: mpsc::Sender<(String, String)>,
}
impl UblWriterAsync {
    /// Cria writer assíncrono (spawn de task interna).
    #[allow(clippy::unused_async)]
    ///
    /// # Errors
    ///
    /// - Propaga erros de I/O ao inicializar diretórios ou canal interno
    pub async fn new(
        root: impl AsRef<Path>,
        app: AppId,
        tenant: TenantId,
        node: NodeId,
        actor: ActorId,
        sk: SigningKey,
        cfg: AsyncConfig,
    ) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let key_id = key_id_v1(&sk.verifying_key());
        let (tx, mut rx) = mpsc::channel::<(String, String)>(cfg.channel_cap);

        // task de writer
        tokio::spawn(async move {
            while let Some((dir_str, line)) = rx.recv().await {
                let dir = PathBuf::from(dir_str);
                let _ = fs::create_dir_all(&dir).await;
                let path = dir.join("ubl-async.ndjson");
                let wal = dir.join("ubl-async.ndjson.wal");
                // WAL
                if let Ok(mut f) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&wal)
                    .await
                {
                    let _ = f.write_all(line.as_bytes()).await;
                    let _ = f
                        .write_all(
                            b"
",
                        )
                        .await;
                    let _ = f.flush().await;
                }
                // append
                if let Ok(mut f) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .await
                {
                    let _ = f.write_all(line.as_bytes()).await;
                    let _ = f
                        .write_all(
                            b"
",
                        )
                        .await;
                    let _ = f.flush().await;
                }
                let _ = fs::remove_file(&wal).await;
            }
        });

        Ok(Self {
            root,
            app,
            tenant,
            node,
            actor,
            sk,
            key_id,
            tx,
        })
    }

    /// Enfileira evento e retorna estrutura preenchida.
    ///
    /// # Errors
    ///
    /// - Erros de serialização ou ao enviar para o canal interno
    pub async fn append<T: Serialize + Sync>(
        &self,
        kind: &str,
        payload: &T,
        trace_id: Option<TraceId>,
        refs: Option<Vec<String>>,
    ) -> Result<UblEvent> {
        let now = OffsetDateTime::now_utc();
        let dir = daily_dir(&self.root, &self.app, &self.tenant, now);

        let canon = codec::to_canon_vec(payload)?;
        let cid_hex = blake3_hex(&canon);
        let sig = sign_cid_hex(&SecretKey(self.sk.to_bytes()), &cid_hex);
        let sig_b64 = b64_encode(&sig);
        let pk_b64 = b64_encode(self.sk.verifying_key().as_bytes());
        let canon_b64 = b64_encode(&canon);

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
            prev_cid_hex: None,
        };
        let line = serde_json::to_string(&ev)?;
        self.tx
            .send((dir.to_string_lossy().to_string(), line))
            .await
            .ok();
        Ok(ev)
    }
}
