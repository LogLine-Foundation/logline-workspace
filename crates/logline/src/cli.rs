#![allow(clippy::too_many_arguments)]
use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use colored::Colorize;
use json_atomic as json_atomic;
use regex::Regex;
use std::{fs, path::PathBuf, time::Instant};
use tdln_compiler::{compile, CompileCtx};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "logline")]
#[command(about = "LogLine CLI: TDLN + Atomic send/tail/bench/dev-server")]
pub struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Compile natural language to TDLN AST
    Compile {
        text: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Show bundle info
    Info,
    /// Show version
    Version,
    /// Send capsule via HTTP
    Send {
        #[arg(long)]
        url: String,
        #[arg(long)]
        json: Option<PathBuf>,
        #[arg(long)]
        bytes: Option<PathBuf>,
        #[arg(long)]
        dim: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        hmac_key: Option<String>,
    },
    /// SIRP frame operations (encode/decode/verify)
    Sirp {
        #[command(subcommand)]
        sub: SirpCmd,
    },
    /// UBL ledger operations (append/info/verify)
    Ubl {
        #[command(subcommand)]
        sub: UblCmd,
    },
    Keygen {
        #[arg(long)]
        out_sk: Option<PathBuf>,
        #[arg(long)]
        out_pk: Option<PathBuf>,
    },
    Tail {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        pretty: bool,
    },
    Bench {
        #[arg(long)]
        url: String,
        #[arg(long)]
        dim: String,
        #[arg(long)]
        json: PathBuf,
        #[arg(long, default_value_t = 8)]
        concurrency: usize,
        #[arg(long, default_value_t = 100)]
        count: usize,
        #[arg(long)]
        hmac_key: Option<String>,
    },
    #[cfg(feature = "server")]
    DevServer {
        #[arg(long, default_value_t = 8080)]
        port: u16,
        #[arg(long)]
        sk_b64: Option<String>,
        #[arg(long)]
        sqlite: Option<PathBuf>,
        #[arg(long)]
        hmac_key: Option<String>,
    },
    Completions {
        #[arg(long)]
        shell: String,
    },
}

#[derive(Subcommand)]
pub enum SirpCmd {
    /// Encode JSON to a SIRP TLV frame
    Frame {
        /// Input JSON file
        #[arg(long)]
        input: PathBuf,
        /// Output .bin file
        #[arg(long)]
        out: PathBuf,
        /// Sign with secret key (base64url)
        #[arg(long)]
        sk: Option<String>,
    },
    /// Decode a SIRP TLV frame to JSON
    Decode {
        /// Input .bin file
        #[arg(long)]
        input: PathBuf,
        /// Output JSON file (or stdout)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Verify a signed SIRP frame
    Verify {
        /// Input .bin file
        #[arg(long)]
        input: PathBuf,
        /// Expected public key (base64url, optional)
        #[arg(long)]
        pk: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum UblCmd {
    /// Append a canonical intent to the ledger (with optional signing)
    Append {
        /// Input JSON file with intent
        #[arg(long)]
        input: PathBuf,
        /// Ledger file path (NDJSON)
        #[arg(long)]
        ledger: PathBuf,
        /// Optional actor label
        #[arg(long)]
        actor: Option<String>,
        /// Optional secret key (base64url) for signing
        #[arg(long)]
        sk: Option<String>,
    },
    /// Show ledger info (line count, last CID, etc.)
    Info {
        /// Ledger file path (NDJSON)
        #[arg(long)]
        path: PathBuf,
    },
    /// Verify all entries in a ledger file
    Verify {
        #[arg(long)]
        path: PathBuf,
    },
}

/// Executa a CLI do `logline`.
///
/// # Errors
///
/// Propaga falhas de parsing da CLI, IO e transportes HTTP.
pub async fn run(bin_name: &str) -> Result<()> {
    let mut cmd = Cli::command();
    // Leak the provided binary name to satisfy clap's `'static` name requirement; this runs once per process.
    let bin: &'static str = Box::leak(bin_name.to_string().into_boxed_str());
    cmd = cmd.name(bin);
    let matches = cmd.clone().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;
    match cli.cmd {
        Command::Compile { text, out } => cmd_compile(&text, out),
        Command::Info => {
            cmd_info();
            Ok(())
        }
        Command::Version => {
            cmd_version();
            Ok(())
        }
        Command::Send {
            url,
            json,
            bytes,
            dim,
            out,
            hmac_key,
        } => cmd_send(&url, json, bytes, dim, out, hmac_key).await,
        Command::Sirp { sub } => cmd_sirp(sub),
        Command::Ubl { sub } => cmd_ubl(sub),
        Command::Keygen { out_sk, out_pk } => cmd_keygen(out_sk, out_pk),
        Command::Tail { path, kind, pretty } => cmd_tail(path, kind, pretty),
        Command::Bench {
            url,
            dim,
            json,
            concurrency,
            count,
            hmac_key,
        } => cmd_bench(url, dim, json, concurrency, count, hmac_key).await,
        #[cfg(feature = "server")]
        Command::DevServer {
            port,
            sk_b64,
            sqlite,
            hmac_key,
        } => cmd_dev_server(port, sk_b64, sqlite, hmac_key).await,
        Command::Completions { shell } => {
            cmd_completions(&shell, bin);
            Ok(())
        }
    }
}

fn cmd_info() {
    println!("LogLine bundle: TDLN + JSON✯Atomic + LogLine core + LLLV + Atomic CLI");
}

fn cmd_version() {
    println!("logline {}", env!("CARGO_PKG_VERSION"));
}

fn cmd_compile(text: &str, out: Option<PathBuf>) -> Result<()> {
    let ctx = CompileCtx {
        rule_set: "v1".into(),
    };
    let compiled = compile(text, &ctx)?;
    let canon_str = String::from_utf8(compiled.canon_json.clone())?;
    
    if let Some(path) = out {
        let output = serde_json::json!({
            "kind": compiled.ast.kind,
            "cid": hex::encode(compiled.cid),
            "ast_cid": hex::encode(compiled.proof.ast_cid),
            "canon_cid": hex::encode(compiled.proof.canon_cid),
            "canonical": canon_str,
        });
        fs::write(&path, serde_json::to_string_pretty(&output)?)?;
        println!("compiled → {}", path.display());
    } else {
        println!("kind       : {}", compiled.ast.kind);
        println!("cid        : {}", hex::encode(compiled.cid));
        println!("ast_cid    : {}", hex::encode(compiled.proof.ast_cid));
        println!("canon_cid  : {}", hex::encode(compiled.proof.canon_cid));
        println!("canonical  : {canon_str}");
    }
    Ok(())
}

fn cmd_sirp(sub: SirpCmd) -> Result<()> {
    match sub {
        SirpCmd::Frame { input, out, sk } => {
            let data = fs::read_to_string(&input)
                .with_context(|| format!("read {}", input.display()))?;
            let v: serde_json::Value = serde_json::from_str(&data)?;
            let canon = json_atomic::canonize(&v)
                .map_err(|e| anyhow::anyhow!("canonize: {:?}", e))?;
            let cid = atomic_crypto::blake3_cid(&canon);
            
            // Build frame: TLV with CID + canonical bytes
            let mut frame = Vec::new();
            // Tag 0x01 = CID, Length = 32
            frame.push(0x01);
            frame.push(32);
            frame.extend_from_slice(&cid.0);
            // Tag 0x02 = Payload, Length = varint
            frame.push(0x02);
            write_varint(&mut frame, canon.len() as u64);
            frame.extend_from_slice(&canon);
            
            // Sign if sk provided
            #[cfg(feature = "signing")]
            if let Some(sk_b64) = sk {
                use atomic_crypto::{b64_decode, SecretKey, derive_public_bytes, sign_bytes};
                let sk_bytes = b64_decode(&sk_b64)?;
                let sk = SecretKey(sk_bytes.try_into().map_err(|_| anyhow::anyhow!("bad sk len"))?);
                let pk = derive_public_bytes(&sk.0);
                let domain = b"SIRP:FRAME:v1";
                let mut msg = Vec::with_capacity(domain.len() + 32);
                msg.extend_from_slice(domain);
                msg.extend_from_slice(&cid.0);
                let sig = sign_bytes(&msg, &sk.0);
                
                // Tag 0x03 = PubKey
                frame.push(0x03);
                frame.push(32);
                frame.extend_from_slice(&pk.0);
                // Tag 0x04 = Signature
                frame.push(0x04);
                frame.push(64);
                frame.extend_from_slice(&sig.0);
            }
            #[cfg(not(feature = "signing"))]
            if sk.is_some() {
                anyhow::bail!("signing feature not enabled");
            }
            
            fs::write(&out, &frame)?;
            println!("frame: {} bytes → {}", frame.len(), out.display());
            println!("cid  : {}", hex::encode(&cid.0));
            Ok(())
        }
        SirpCmd::Decode { input, out } => {
            let frame = fs::read(&input)
                .with_context(|| format!("read {}", input.display()))?;
            
            // Parse TLV (simple)
            let mut cid_hex = String::new();
            let mut payload = Vec::new();
            let mut pk_hex = String::new();
            let mut sig_hex = String::new();
            let mut i = 0;
            
            while i < frame.len() {
                let tag = frame[i];
                i += 1;
                let (len, consumed) = read_varint(&frame[i..])?;
                i += consumed;
                let data = &frame[i..i + len as usize];
                i += len as usize;
                
                match tag {
                    0x01 => cid_hex = hex::encode(data),
                    0x02 => payload = data.to_vec(),
                    0x03 => pk_hex = hex::encode(data),
                    0x04 => sig_hex = hex::encode(data),
                    _ => {}
                }
            }
            
            let payload_str = String::from_utf8_lossy(&payload);
            let output = serde_json::json!({
                "cid": cid_hex,
                "payload": serde_json::from_str::<serde_json::Value>(&payload_str).unwrap_or(serde_json::Value::Null),
                "pubkey": if pk_hex.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(pk_hex) },
                "signature": if sig_hex.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(sig_hex) },
            });
            
            if let Some(path) = out {
                fs::write(&path, serde_json::to_string_pretty(&output)?)?;
                println!("decoded → {}", path.display());
            } else {
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            Ok(())
        }
        SirpCmd::Verify { input, pk: _pk } => {
            let frame = fs::read(&input)
                .with_context(|| format!("read {}", input.display()))?;
            
            // Parse TLV
            let mut cid: Option<[u8; 32]> = None;
            let mut payload = Vec::new();
            let mut pk_bytes: Option<[u8; 32]> = None;
            let mut sig_bytes: Option<[u8; 64]> = None;
            let mut i = 0;
            
            while i < frame.len() {
                let tag = frame[i];
                i += 1;
                let (len, consumed) = read_varint(&frame[i..])?;
                i += consumed;
                let data = &frame[i..i + len as usize];
                i += len as usize;
                
                match tag {
                    0x01 if data.len() == 32 => {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(data);
                        cid = Some(arr);
                    }
                    0x02 => payload = data.to_vec(),
                    0x03 if data.len() == 32 => {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(data);
                        pk_bytes = Some(arr);
                    }
                    0x04 if data.len() == 64 => {
                        let mut arr = [0u8; 64];
                        arr.copy_from_slice(data);
                        sig_bytes = Some(arr);
                    }
                    _ => {}
                }
            }
            
            // Verify CID
            let cid = cid.ok_or_else(|| anyhow::anyhow!("no CID in frame"))?;
            let computed = atomic_crypto::blake3_cid(&payload);
            if computed.0 != cid {
                anyhow::bail!("CID mismatch: expected {}, got {}", hex::encode(cid), hex::encode(computed.0));
            }
            println!("✅ CID verified: {}", hex::encode(cid));
            
            // Verify signature if present
            #[cfg(feature = "signing")]
            if let (Some(pk), Some(sig)) = (pk_bytes, sig_bytes) {
                let domain = b"SIRP:FRAME:v1";
                let mut msg = Vec::with_capacity(domain.len() + 32);
                msg.extend_from_slice(domain);
                msg.extend_from_slice(&cid);
                
                let pk_typed = atomic_types::PublicKeyBytes(pk);
                let sig_typed = atomic_types::SignatureBytes(sig);
                if atomic_crypto::verify_bytes(&msg, &pk_typed, &sig_typed) {
                    println!("✅ Signature verified: pk={}", hex::encode(pk));
                } else {
                    anyhow::bail!("❌ Signature invalid");
                }
            }
            
            Ok(())
        }
    }
}

fn write_varint(buf: &mut Vec<u8>, mut n: u64) {
    while n >= 0x80 {
        buf.push((n as u8) | 0x80);
        n >>= 7;
    }
    buf.push(n as u8);
}

fn read_varint(data: &[u8]) -> Result<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in data.iter().enumerate() {
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((result, i + 1));
        }
        shift += 7;
        if shift >= 64 {
            anyhow::bail!("varint overflow");
        }
    }
    anyhow::bail!("incomplete varint");
}

async fn cmd_send(
    url: &str,
    json: Option<PathBuf>,
    bytes: Option<PathBuf>,
    dim: Option<String>,
    out: Option<PathBuf>,
    hmac_key: Option<String>,
) -> Result<()> {
    validate_payload_args(json.as_ref(), bytes.as_ref())?;
    let mut capsule: Vec<u8> = if let Some(j) = json {
        let data = fs::read_to_string(&j).with_context(|| format!("read json {}", j.display()))?;
        let v: serde_json::Value = serde_json::from_str(&data)?;
        atomic_codec::to_canon_vec(&v)?
    } else if let Some(b) = bytes {
        fs::read(&b).with_context(|| format!("read bytes {}", b.display()))?
    } else {
        anyhow::bail!("--json or --bytes")
    };
    if let Some(d) = dim {
        let dim_val = parse_dim(&d)?;
        let mut with_hdr = Vec::with_capacity(2 + capsule.len());
        with_hdr.push(((dim_val >> 8) & 0xFF) as u8);
        with_hdr.push((dim_val & 0xFF) as u8);
        with_hdr.extend_from_slice(&capsule);
        capsule = with_hdr;
    }
    let key = hmac_key.as_deref().map(str::as_bytes);
    let resp = atomic_sirp::transport_http::post_capsule_hmac(url, &capsule, key).await?;
    if let Some(p) = out {
        fs::write(&p, &resp)?;
        println!("written {len} bytes to {}", p.display(), len = resp.len());
    } else {
        println!("{bytes} bytes", bytes = resp.len());
    }
    Ok(())
}

fn parse_dim(s: &str) -> Result<u16> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x") {
        Ok(u16::from_str_radix(h, 16)?)
    } else {
        Ok(s.parse::<u16>()?)
    }
}

fn validate_payload_args(json: Option<&PathBuf>, bytes: Option<&PathBuf>) -> Result<()> {
    match (json, bytes) {
        (Some(_), Some(_)) => anyhow::bail!("--json and --bytes are mutually exclusive"),
        (None, None) => anyhow::bail!("--json or --bytes"),
        _ => Ok(()),
    }
}

fn cmd_ubl(sub: UblCmd) -> Result<()> {
    match sub {
        UblCmd::Append {
            input,
            ledger,
            actor,
            sk,
        } => {
            let data = fs::read_to_string(&input)
                .with_context(|| format!("read intent {}", input.display()))?;
            let intent: serde_json::Value = serde_json::from_str(&data)?;
            let mut entry = atomic_ubl::LedgerEntry::unsigned(&intent, actor, b"")
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            #[cfg(feature = "signing")]
            if let Some(sk_b64) = sk {
                use atomic_crypto::{b64_decode, SecretKey};
                let bytes = b64_decode(&sk_b64)?;
                let sk = SecretKey(bytes.try_into().map_err(|_| anyhow::anyhow!("bad sk len"))?);
                entry = entry.sign(&sk);
            }
            #[cfg(not(feature = "signing"))]
            if sk.is_some() {
                anyhow::bail!("signing feature not enabled");
            }

            let mut w = atomic_ubl::SimpleLedgerWriter::open_append(&ledger)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            w.append(&entry).map_err(|e| anyhow::anyhow!("{e}"))?;
            w.sync().map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("appended cid={} to {}", hex::encode(entry.cid.0), ledger.display());
            Ok(())
        }
        UblCmd::Info { path } => {
            let reader = atomic_ubl::SimpleLedgerReader::from_path(&path)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut count = 0u64;
            let mut last_cid: Option<String> = None;
            for entry in reader.iter() {
                let entry = entry.map_err(|e| anyhow::anyhow!("{e}"))?;
                count += 1;
                last_cid = Some(hex::encode(entry.cid.0));
            }
            println!("file     : {}", path.display());
            println!("entries  : {count}");
            if let Some(cid) = last_cid {
                println!("last_cid : {cid}");
            }
            Ok(())
        }
        UblCmd::Verify { path } => {
            let n = atomic_ubl::verify::verify_file(&path)?;
            println!("verified {n} events");
            Ok(())
        }
    }
}

fn cmd_keygen(out_secret: Option<PathBuf>, out_public: Option<PathBuf>) -> Result<()> {
    use atomic_crypto::{b64_encode, Keypair};
    let kp = Keypair::generate();
    let sk_b64 = b64_encode(&kp.sk.0);
    let pk_b64 = b64_encode(kp.vk.as_bytes());
    if let (Some(skp), Some(pkp)) = (out_secret, out_public) {
        fs::write(&skp, sk_b64)?;
        fs::write(&pkp, pk_b64)?;
        println!("wrote SK -> {} , PK -> {}", skp.display(), pkp.display());
    } else {
        println!("SK(b64url)={sk_b64}");
        println!("PK(b64url)={pk_b64}");
    }
    Ok(())
}

fn cmd_tail(path: PathBuf, kind: Option<String>, pretty: bool) -> Result<()> {
    let filter = kind.map(|k| Regex::new(&k).unwrap());
    if path.is_file() {
        atomic_ubl::tail_file(path, |ev| print_ev(&ev, pretty, filter.as_ref()))?;
    } else {
        for e in WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
        {
            if e.file_type().is_file() {
                for res in atomic_ubl::UblReader::iter_file(e.path())? {
                    let ev = res?;
                    print_ev(&ev, pretty, filter.as_ref());
                }
            }
        }
    }
    Ok(())
}

fn print_ev(ev: &atomic_ubl::event::UblEvent, pretty: bool, re: Option<&Regex>) {
    if let Some(r) = re {
        if !r.is_match(&ev.kind) {
            return;
        }
    }
    if !pretty {
        println!("{}", serde_json::to_string(ev).unwrap());
        return;
    }
    let k = ev.kind.as_str();
    println!(
        "{} {} {}",
        "[UBL]".bold(),
        k.green(),
        ev.cid_hex[..8].to_string().blue()
    );
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::multiple_crate_versions
)]
async fn cmd_bench(
    url: String,
    dim: String,
    json: PathBuf,
    concurrency: usize,
    count: usize,
    hmac_key: Option<String>,
) -> Result<()> {
    let payload = fs::read_to_string(&json)?;
    let v: serde_json::Value = serde_json::from_str(&payload)?;
    let canon = atomic_codec::to_canon_vec(&v)?;
    let dim_val = parse_dim(&dim)?;
    let mut capsule = Vec::with_capacity(2 + canon.len());
    capsule.push(((dim_val >> 8) & 0xFF) as u8);
    capsule.push((dim_val & 0xFF) as u8);
    capsule.extend_from_slice(&canon);
    let key = hmac_key.as_deref().map(str::as_bytes);
    let start = Instant::now();
    let mut tasks = Vec::new();
    for _ in 0..concurrency {
        let url = url.clone();
        let capsule = capsule.clone();
        let rounds = count / concurrency + usize::from(count % concurrency != 0);
        let key = key.map(<[u8]>::to_vec);
        tasks.push(tokio::spawn(async move {
            let mut lat = Vec::new();
            let mut ok = 0usize;
            for _ in 0..rounds {
                let t0 = Instant::now();
                if atomic_sirp::transport_http::post_capsule_hmac(&url, &capsule, key.as_deref())
                    .await
                    .is_ok()
                {
                    ok += 1;
                    lat.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
            }
            (ok, lat)
        }));
    }
    let mut sent_ok = 0usize;
    let mut all_lat = Vec::new();
    for t in tasks {
        let (ok, lat) = t.await?;
        sent_ok += ok;
        all_lat.extend(lat);
    }
    let dur = start.elapsed().as_secs_f64();
    all_lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let qps = sent_ok as f64 / dur;
    println!("sent_ok={sent_ok} in {dur:.3}s (target {count})");
    println!(
        "QPS={:.2} P50={:.2}ms P95={:.2}ms P99={:.2}ms",
        qps,
        percentile(&all_lat, 50.0),
        percentile(&all_lat, 95.0),
        percentile(&all_lat, 99.0)
    );
    Ok(())
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return 0.0;
    }
    let clamped = p.clamp(0.0, 100.0);
    let span = sorted_ms.len().saturating_sub(1);
    let idx = ((clamped / 100.0) * (span as f64)).round();
    let idx = usize::min(span, idx as usize);
    sorted_ms[idx]
}

#[cfg(feature = "server")]
async fn cmd_dev_server(
    port: u16,
    sk_b64: Option<String>,
    sqlite: Option<PathBuf>,
    hmac_key: Option<String>,
) -> Result<()> {
    use atomic_crypto::{b64_decode, SecretKey};
    use atomic_sirp::server as sirp_srv;
    use atomic_sirp::server::FnProcessor;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    let sk = if let Some(s) = sk_b64 {
        SecretKey(
            b64_decode(&s)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("bad sk len"))?,
        )
    } else {
        atomic_crypto::Keypair::generate().sk
    };
    let idem = if let Some(p) = sqlite {
        Some(atomic_sirp::idempotency::SqliteIdem::open(
            p.to_string_lossy().as_ref(),
        )?)
    } else {
        None
    };
    let hkey = hmac_key.map(String::into_bytes);
    let proc = FnProcessor(|capsule: &[u8]| -> Result<Vec<u8>> { Ok(capsule.to_vec()) });
    let app = sirp_srv::router(proc, sk, 24 * 3600, idem, hkey);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    println!("dev-server listening on http://{addr}");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn cmd_completions(shell: &str, bin_name: &'static str) {
    use clap_complete::{generate, shells};
    use std::io;
    let mut cmd = Cli::command();
    cmd = cmd.name(bin_name);
    match shell {
        "bash" => generate(shells::Bash, &mut cmd, bin_name, &mut io::stdout()),
        "zsh" => generate(shells::Zsh, &mut cmd, bin_name, &mut io::stdout()),
        "fish" => generate(shells::Fish, &mut cmd, bin_name, &mut io::stdout()),
        "powershell" => generate(shells::PowerShell, &mut cmd, bin_name, &mut io::stdout()),
        "elvish" => generate(shells::Elvish, &mut cmd, bin_name, &mut io::stdout()),
        _ => eprintln!("unknown shell: {shell}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dim_accepts_hex_and_dec() {
        assert_eq!(parse_dim("0x0001").unwrap(), 1);
        assert_eq!(parse_dim("17").unwrap(), 17);
        assert_eq!(parse_dim(" 0x00A1 ").unwrap(), 0x00A1);
    }

    #[test]
    fn parse_dim_rejects_bad_inputs() {
        assert!(parse_dim("0xGG").is_err());
        assert!(parse_dim("").is_err());
    }

    #[test]
    fn percentile_handles_edges_and_middle() {
        let empty: Vec<f64> = Vec::new();
        assert!((percentile(&empty, 50.0) - 0.0).abs() < f64::EPSILON);
        let vals = [10.0, 20.0, 30.0, 40.0];
        assert!((percentile(&vals, 0.0) - 10.0).abs() < f64::EPSILON);
        assert!((percentile(&vals, 50.0) - 30.0).abs() < f64::EPSILON);
        assert!((percentile(&vals, 100.0) - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn payload_args_are_mutually_exclusive() {
        let jp = Some(PathBuf::from("a.json"));
        let bp = Some(PathBuf::from("b.bin"));
        assert!(validate_payload_args(jp.as_ref(), bp.as_ref()).is_err());
        assert!(validate_payload_args(None, None).is_err());
        assert!(validate_payload_args(jp.as_ref(), None).is_ok());
        assert!(validate_payload_args(None, bp.as_ref()).is_ok());
    }
}
