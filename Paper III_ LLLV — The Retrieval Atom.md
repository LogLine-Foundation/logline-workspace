# LogLine Papers — **Paper III**  
**LLLV — The Retrieval Atom**  
*A Local-First, Verifiable Vector Engine with Temporal Narratives*

**Author**: Dan Voulez  
**Affiliation**: LogLine Foundation  
**Location**: Lisbon, Portugal  
**Date**: January 8, 2026  
**Status**: **Confidential Draft**

---

## Series Index — LogLine Papers
I — **The LogLine Protocol** — The Conceptual Atom of Verifiable Action  
II — **JSON✯Atomic** — The Cryptographic Atom  
III — **LLLV** — The Retrieval Atom *(this paper)*  
IV — **TDLN** — The Logical Atom & Compiler Specification  
V — **SIRP** — The Network Atom  
VI — **Chip as Code** — Redefining Computing as Protocol  
VII — **UBL** — The Economic Infrastructure & The Product

---

# Paper III — **LLLV: The Retrieval Atom**

## 1.0 Abstract  
**LLLV** defines the verifiable **retrieval substrate** of the LogLine stack. Each item of recall—document, chunk, or embedding—nasce como **Signed Fact**: um envelope JSON✯Atomic canônico com **endereçamento por conteúdo (BLAKE3)** e **assinatura Ed25519** sobre os bytes canônicos, fixando identidade, proveniência e política na ingestão. A busca opera sobre **Index Packs** portáveis (HNSW) cujos parâmetros, postings e estatísticas também são assinados e endereçados por conteúdo. Os resultados retornam com uma **Top-K Evidence Chain** (IDs, distâncias e prova) e uma **Temporal Narrative** opcional: um histórico compacto de como a evidência evoluiu em janelas de tempo. O LLLV expõe um **Intent Endpoint** (`lllv.ingest|query|verify|narrative`) que compõe com políticas TDLN, liquida recibos no UBL e trafega via SIRP — transformando “busca de melhor esforço” em contrato com prova.

---

## 2.0 The Problem: Unprovable Memory  
Pipelines modernos de retrieval são **não determinísticos e inauditáveis**: embeddings variam por toolchain/precisão; índices mudam sem linhagem; e “por que estes resultados?” vira relato, não criptografia. Em economias de agentes, isso é fatal: quando *execução precede registro*, a memória pode ser reescrita—ou inventada—no pós-fato. LLLV fecha esse **Intention Gap** para *recall* impondo **record-before-use**: ingestão canoniza e assina; a consulta produz evidência e recibos; a verificação reexecuta a cadeia localmente. *Recall* vira primitivo de primeira-classe, com prova.

---

## 3.0 Specification: The **Vector Capsule**  
A unidade atômica do LLLV é a **Vector Capsule**: um envelope assinado e endereçável por conteúdo que liga *who/what/when* aos bytes do vetor e à política associada. Ela espelha o “idioma de cápsulas” da série, permitindo **transporte zero-trust** e verificação offline.

### 3.1 Wire Format (overview)  
**Header (fixo):**  
`MAGIC u16 | VER u8 | FLAGS u8 (Encrypted, Receipt-Req) | TS u64 | CID [32]=blake3(payload) | DIM u16 | LEN u32 | SIG [64]=Ed25519(header‖payload)`

**Payload (binário):**  
`CipherEnvelope{nonce, aad, ciphertext}` com **AAD = vector_id ‖ CID**; os embeddings são **quantizados** (ex.: Q8/Q4 fix-point) para eficiência, mantendo o manifesto canônico limpo.

**Manifest (JSON✯Atomic, canônico):**  
`vector_id, source_uri, mime, content_hash, dim, quant, encoder{name,ver}, policy_ref, ts_ingest`

**Invariantes:**  
1) **Same Semantics ⇒ Same Bytes ⇒ Same Hash** (canoniza antes de assinar).  
2) **Assinatura cobre header+payload**; replay limitado por TS/política.  
3) **Content addressing** prende os bytes exatos servidos/recuperados.  
4) **Evidence Chain** é gerada para todo Top-K.

#### Figure 1 — Vector Capsule (schematic)
```
+---------------------+  Signed Header (Ed25519 over header‖payload)
| MAGIC | VER | FLAGS |
| TS (u64)            |
| CID [32] (BLAKE3)   |--> content address of payload
| DIM (u16) | LEN     |
| SIG [64]            |
+---------------------+
| CipherEnvelope      |  nonce | AAD=(vector_id‖CID) | ciphertext(quantized vectors)
+---------------------+
| Manifest (JSON✯Atomic, canonicalized & signed)
|   vector_id, source_uri, content_hash, dim, quant, encoder, policy_ref, ts_ingest
+--------------------------------------------------------------------------+
```

### 3.2 Security & Policy Invariants  
- **Canonical First**: canonização → BLAKE3 → Ed25519 (DV25-Seal).  
- **Policy Before Use**: toda `lllv.*` compila em **TDLN**; cada chamada vira LogLine com `if_*`.  
- **Transport Without Trust**: cápsulas roteáveis em **SIRP** (DID-anchored) com **receipts**.  
- **Ledger as Truth**: consultas/ingests geram **receipts** liquidados no **UBL**.

---

## 4.0 The **Index Pack** — A Portable, Verifiable ANN Artifact

**Definition.** O *Index Pack* é um artefato portável (`.lllv.idx`) contendo: (i) parâmetros ANN (HNSW), (ii) vetores quantizados e *postings*, (iii) *doc-table* canônica, (iv) *summary stats* e (v) um **Manifest** JSON✯Atomic assinado e endereçado por conteúdo.

**Security/Policy Model.** O Pack é **conteúdo-endereçado** e o Manifest é assinado (Ed25519), seguindo o ciclo **Canonização → Hash (BLAKE3) → Signature (DV25)**. O índice ANN vira **átomo verificável**: transportável em SIRP, com **receipts** e *settlement* no UBL.

### 4.1 Binary Layout (overview)
```
PACK HEADER: MAGIC|VER|FLAGS|TS|PACK_CID|MANIFEST_SIG
TOC: n_blocks | [BlockDesc { kind, off, len }]

BLOCKS:
  HNSW_PARAMS     (M, efConstruction, efSearch, space)
  VECTOR_STORAGE  (quantized bytes, fixed-point)
  POSTINGS        (ids, neighbor lists, levels)
  DOC_TABLE       (canonical JSON✯Atomic rows)
  STATS           (centroids, norms, histograms)
  MERKLE_INDEX    (roots + per-block trees)
  MANIFEST        (JSON✯Atomic canonicalized & signed)
```

**Merkleization.** Cada bloco é hasheado; um **Merkle root** cobre o conjunto, permitindo **partial verifies** (inclusão de sub-árvore) ao checar Top-K sem reprocessar o pack inteiro.

### 4.2 Pack Manifest (JSON✯Atomic, mínimo viável)
```json
{
  "type": "LLLV_INDEX_PACK", "ver": 1,
  "created_ts": "2026-01-08T00:00:00Z",
  "encoder": {"name": "glassbox-encoder", "ver": "1.0"},
  "ann": {"algo": "hnsw", "space": "cosine", "M": 32, "ef_construction": 200, "ef_search": 64},
  "dim": 1536, "quant": "q8", "vector_count": 1048576,
  "blocks": [{"kind":"HNSW_PARAMS","cid":"blake3:..."}, {"kind":"VECTOR_STORAGE","cid":"blake3:..."}],
  "root": "blake3:...", "policy_ref": "tdln://policy/lllv.pack@v1",
  "provenance": {"built_by":"lllv index-build","host":"lab512","ts":1736294400}
}
```
**Invariantes:** Manifest **canônico** e **assinado**; seu CID referenciado no cabeçalho do Pack. Anti-malleability, endereço único e reprodutibilidade bit-a-bit.

### 4.3 Top-K Proofs (Evidence Chain)
A resposta inclui: **(a)** Top-K (IDs, distâncias), **(b)** *proof material* (sub-árvores Merkle/CIDs de blocos) e **(c)** um **Evidence Chain** em JSON✯Atomic — verificado localmente, sem confiar no servidor.
```json
{
  "type":"LLLV_TOPK_EVIDENCE_V1",
  "query_cid":"blake3:...",
  "index_pack_cid":"blake3:...",
  "results":[
    {"id":"doc:123","dist":0.1831,"proof":{"block":"POSTINGS","merkle_path":["..."]}},
    {"id":"doc:818","dist":0.1879,"proof":{"block":"POSTINGS","merkle_path":["..."]}}
  ],
  "stats":{"ef_search":64,"visited":812},
  "sig":"ed25519:..."
}
```

### 4.4 Build / Append / Compact
- **Build**: `index-build` produz o Pack e um **receipt** (LogLine).  
- **Append**: novos vetores → *delta-pack*; *compactor* mescla e atualiza `root`.  
- **Compact**: reordena blocos, preservando CIDs históricos como *historical packs*.

### 4.5 Verification Procedure (client-side)
1) Verificar **assinatura** do Manifest e **CID** (BLAKE3).  
2) Recomputar **Merkle root** dos blocos referenciados.  
3) Validar cada `merkle_path` dos resultados.  
4) Validar **Evidence Chain** (JSON✯Atomic assinado).

---

## 5.0 The **Temporal Narrative** — Time as Evidence

**Motivação.** Retrievals ignoram **temporalidade**; a economia de agentes, não. LogLine já exige **record-before-execute**. O LLLV aplica a mesma lei ao **recall**: cada mutação relevante vira **Signed Fact**, permitindo explicar *por que* algo foi recuperado **naquele** tempo — não só *o que*.

### 5.1 Data Model (append-only)
Eventos por `vector_id`/`source_uri`:  
`INITIAL` • `MINOR_EDIT` • `MAJOR_EDIT` • `RETIRED`  
Cada evento é JSON✯Atomic assinado, encadeado por `prev_cid → new_cid` e *diff-hash* opcional.
```json
{
  "type":"LLLV_NARRATIVE_V1",
  "vector_id":"vec:9f3...","source_uri":"s3://bucket/docA",
  "delta":"MAJOR_EDIT","prev_cid":"blake3:old...","new_cid":"blake3:new...",
  "author_did":"did:logline:agent:alice","policy_ref":"tdln://policy/lllv.narrative@v1",
  "ts":"2026-01-08T12:34:56Z","sig":"ed25519:..."
}
```

### 5.2 Half-Life Weighting
\[
w(t)=e^{-\frac{t_{\text{now}}-t_i}{\tau}},\quad \text{score}=\text{sim}(q,d)\cdot w(t)\cdot \Pi(\text{policy})
\]
`τ` por *workflow* (notícias vs. manuais). `Π(policy)` aplica boosts/penalties determinísticos (TDLN).

### 5.3 Evidence Chain with Time
Top-K inclui **Temporal Evidence**: janelas (7d/30d/∞), `best_state` e sequência de deltas relevante ao *rationale*.
```json
{
  "type":"LLLV_TEMPORAL_EVIDENCE_V1",
  "window":"30d",
  "best_state":{"cid":"blake3:new...","ts":"2026-01-08T12:34:56Z"},
  "deltas":[
    {"delta":"INITIAL","cid":"blake3:..."},
    {"delta":"MINOR_EDIT","cid":"blake3:..."},
    {"delta":"MAJOR_EDIT","cid":"blake3:new..."}
  ],
  "weights":{"tau":"7d","w_best":0.91}
}
```

### 5.4 Transport & Settlement
Narratives e *Top-K Evidence* viajam como **Capsules** em **SIRP** (com **cryptographic receipts** por hop). *Batches* Merkle liquidam no **UBL** como micro-transações — narrativa **economicamente auditável** de ponta a ponta.

### 5.5 Operations
- **Ingest Hooks:** `lllv.ingest` emite `INITIAL`; edits disparam `MINOR/MAJOR_EDIT` conforme política TDLN.  
- **Retire:** `RETIRED` mantém prova e precedente.  
- **Export:** NDJSON append-only, *roll-ups* por janela.  
- **Explain:** cada resposta inclui *why* + *when* na Evidence Chain.

### 5.6 System Invariants
1) **Record-Before-Use**: nada afeta score sem fato assinado prévio.  
2) **Same Semantics ⇒ Same Bytes ⇒ Same Hash**: estados/deltas canônicos, hashes estáveis.  
3) **Zero-Trust Transport**: SIRP entrega com recibos.  
4) **Economic Memory**: UBL assenta recibos em ledger ordenado.

---

## 6.0 API — Intent Endpoint (TDLN-first, Proof-Carrying)

**Intents canônicos:**
- `lllv.ingest` — canoniza → BLAKE3 → assina → persiste (**ghost** se barrado).  
- `lllv.query` — ANN contra Index Packs; retorna **Top-K Evidence** + provas.  
- `lllv.verify` — valida cápsulas/packs: canonização, CIDs, Ed25519, Merkle.  
- `lllv.narrative` — retorna **Temporal Narrative** (deltas + half-life) como **Signed Facts**.

**Gating por política.** Toda chamada compila via **TDLN** (DSL determinística, fail-closed, AST com `source_hash`).  
**Transporte.** INTENT em **SIRP Capsule**, com **receipts** por hop; liquidação no **UBL**.

### 6.1 `lllv.ingest` — Create a Vector Capsule
**Request (payload canônico):**
```json
{
  "intent":"lllv.ingest",
  "this":{
    "vector_id":"vec:9f3...","source_uri":"s3://bucket/docA","mime":"text/plain",
    "dim":1536,"quant":"q8","encoder":{"name":"glassbox-encoder","ver":"1.0"},
    "policy_ref":"tdln://policy/lllv.ingest@v1",
    "bytes":"base64:...","aad_bind":"vector_id|cid"
  }
}
```
**Response (Receipt + Capsule pointer):**
```json
{
  "type":"LLLV_INGEST_RECEIPT_V1",
  "capsule_cid":"blake3:...","manifest_sig":"ed25519:...",
  "logline":{"did":"lllv.ingest","status":"COMMITTED","if_ok":"append_vector","if_not":"rollback_and_alert"}
}
```

### 6.2 `lllv.query` — Top-K with Evidence
**Request:**
```json
{"intent":"lllv.query","this":{"index_pack_cid":"blake3:...","query":{"text":"deterministic proof of retrieval"},"topk":10,"ef_search":64,"temporal":{"window":"30d","tau":"7d"}}}
```
**Response:** *(Top-K + provas Merkle + Evidence assinada, com bloco temporal)*

### 6.3 `lllv.verify` — Local, Zero-Trust Verification
**Request:**
```json
{"intent":"lllv.verify","this":{"capsule_cid":"blake3:...","index_pack_cid":"blake3:...","evidence":{"...":"..."}}}
```
**Response:**
```json
{"ok":true,"checks":["manifest_sig","cid","merkle_paths","hnsw_params","evidence_sig"]}
```

### 6.4 `lllv.narrative` — Time as Evidence
**Request:**
```json
{"intent":"lllv.narrative","this":{"vector_id":"vec:9f3...","window":"∞"}}
```
**Response:** *(Signed Facts com deltas + weights)*

### 6.5 Receipts
**Network Receipt (por hop, SIRP)** e **Ledger Receipt (UBL)** com *batch Merkle* para micro-settlement.

### 6.6 LogLine Record (envelope de ação)  
Cada `lllv.*` emite LogLine com campos do átomo de ação (verbo, payload tipado, `when`, `if_*`, `status`), inclusive **GHOST** quando barrada.

### 6.7 Segurança & Invariantes  
Canonical-First • Policy-Before-Use • Zero-Trust Transport • Economic Memory.

---

## 7.0 Performance & Bench — Proving the Retrieval Atom

**Goal.** Provar que LLLV entrega **retrieval com prova** com latências classe-DB.

### 7.1 Métricas  
Ingest p95 • Pack Build • Query p50/p95 • Verify p95 • Recall@K/MRR • Footprint.

### 7.2 Cenários  
**S-100K / S-1M** (dim=1536, q8, M=32, efC=200, efS=64) • **Temporal 30d/∞** (τ=7d) • **Zero-trust verify**.

### 7.3 Metodologia  
Fixtures reprodutíveis; **PoD** via NDJSON com recibos `ingest|build|query`; verificadores de Manifest/Merkle/Evidence; liquidação SIRP→UBL.

### 7.4 Tabelas (com targets)

**Tabela A — Latências (ms)**
```
Scenario    Ingest p95   Pack Build          Query p50   Query p95   Verify p95
S-100K      2.1          60,000              3.5         11.8        4.0
S-1M        2.3          600,000             8.2         22.6        6.8
```

**Tabela B — Qualidade & Footprint**
```
Scenario    Recall@10    MRR@10    Bytes/vec(q8)    Pack Size       Pack:Raw
S-100K      0.79         0.61      1,536            220 MB          0.35×
S-1M        0.76         0.58      1,536            2.2  GB         0.38×
```
(*Baseline LAB 512; ver notas de operação.*)

### 7.5 Invariantes de Medição  
Canonical-First • Policy-Before-Use • Zero-Trust Transport.

---

## 8.0 Implementation Notes — Wire, Crypto, Temporal

### 8.1 Canonicalização & Selo (DV25)  
Pipeline de 3 fases: **Canonização C(A) → BLAKE3 (CID) → Ed25519 (DV25-Seal)**.  
“No Floats” no manifesto; números determinísticos (q8) para reprodutibilidade.

### 8.2 Vector Capsule (normativo)  
Header assinado; payload cifrado com **AAD = vector_id‖CID**; manifesto canônico; invariantes (bytes==hash, replay-bounded, content-addressed, Evidence em toda Top-K).

### 8.3 Index Pack  
Blocos mínimos; Manifest assinado + Merkle root; **partial verifies**.

### 8.4 Temporal Narrative  
Eventos `INITIAL/MINOR_EDIT/MAJOR_EDIT/RETIRED`; half-life τ por workflow; boosts/penalties TDLN.

### 8.5 Política & Compilação (TDLN)  
Determinista, Turing-incompleta, fail-closed; AST com `source_hash`; **record-before-use** com LogLines (inclui **GHOST**).

### 8.6 Transporte & Economia (SIRP → UBL)  
SIRP Capsules + **receipts**; batches Merkle liquidam no **UBL** (ledger ordenado, classe-DB).

### 8.7 Hardening & Operação  
Fail-closed; **Ghost Records** como defesa forense; artefatos portáveis/verificáveis/versionados.

---

## Proof-of-Done (Paper III)  
1) Publicar **fixtures** + **NDJSON** com recibos `ingest|build|query`, CIDs/assinaturas e Evidence.  
2) Disponibilizar **Index Packs** e **verificador** CLI (Manifest, Merkle, Evidence).  
3) Exportar **SIRP Receipts** batched + **UBL TX** de liquidação.
