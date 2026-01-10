# Paper VII: UBL — The Economic Infrastructure & The Product

## 1.0 Abstract
**UBL (Universal Business Ledger)** is the economic fabric of the LogLine stack. It is a **receipt‑based, agreement‑first ledger** where every action is a signed, canonically serialized **atom** that references the **Agreement** that authorized it and the **Container** where its effects are realized. UBL unifies chat rooms, workspaces, wallets, and networks as variations of a single **Container primitive** with different "physics" (fungibility, topology, permeability, execution).

Built Cloudflare‑first (Workers + Durable Objects + R2 + D1 + Access + MCP Portal), UBL delivers **strict ordering**, **zero‑trust governance**, and **tool‑level observability** while remaining portable to on‑prem via WASM/native policy engines (TDLN). It integrates upstream atoms from **JSON✯Atomic** (canonical bytes & signatures), transports them via **SIRP**, consumes **LLLV** retrieval proofs, and exposes a **single MCP portal URL** as the programmable surface. The result: *execution follows record*, relationships become first‑class, and every outcome is economically auditable.

---

## 2.0 From Roles to Relationships — Why Agreement‑First?
Traditional systems encode roles as attributes. UBL encodes **relationships as Agreements**: the establishment of permission in a specific context at a specific time. Each user action binds to an **Agreement ID** (e.g., `a:room:<id>`, `a:workspace:<id>`, `a:tool:<name>`). This establishes "who can do what" as **verifiable facts** and makes every effect traceable to its authorizing cause. Agreements are the *why* behind every *what*; without an Agreement, there is no action.

---

## 3.0 The Container Primitive — One Engine, Many Physics
A **Container** is an asset that holds other assets under the governance of an Agreement. UBL uses one implementation with configurable physics:

- **Fungibility** — *Strict* (Wallet) · *Versioned* (Workspace) · *Transient* (Network)  
- **Topology** — Values (Wallet) · Objects (Workspace) · Subjects (Tenant) · Links (Network)  
- **Permeability** — Sealed (Wallet) · Gated (Tenant) · Collaborative (Room) · Open (Network)  
- **Execution** — Disabled (Wallet) · Sandboxed (Workspace) · Full (Tenant)

**Examples.** A Room is a collaborative, gated, versioned Container that holds Messages; a Workspace is a versioned Container that holds Documents; a Wallet is a strict, sealed Container that holds Credits. The **Agreement** attached to the Container (RoomGovernance, WorkspaceAgreement, WalletGovernance) defines the physics—not ad‑hoc code branches.

---

## 4.0 The UBL Atom — Receipts with Hash‑Chained Integrity
UBL persists two atom kinds per action:

- `action.v1` — *Attempted cause*: who/when/did/this (+ optional `agreement_id`)  
- `effect.v1` — *Observed consequence*: outcome + pointers (e.g., `msg_id`, `room_seq`)

Atoms are canonically serialized, **content‑addressed** (CID = hash of canonical JSON), and **hash‑chained** (`head_hash = H(prev_head_hash : cid)`). Appending an atom yields a **Receipt** `{seq, cid, head_hash, time}`. The **Room** assigns `room_seq` (ordering authority); the **Ledger** assigns `seq/head_hash` (receipt authority).

> **Record‑before‑execute**: the Room only commits effects after the action atom is accepted by the Ledger.

---

## 5.0 Architecture — Cloudflare as the Computer
UBL's reference runtime is **Cloudflare‑first**:

- **Gateway Worker** — routes UI/API/MCP, normalizes identity (Access), performs origin checks  
- **Durable Objects** — strict ordering & single‑writer state  
  - `TenantObject` / *Circle*: membership, defaults, room directory  
  - `RoomObject`: ordered message timeline + SSE (`room_seq`)  
  - `LedgerShardObject`: append‑only atoms + hash chain (`seq`, `head_hash`)  
  - `ContainerObject` (unified primitive for Rooms/Workspaces/Wallets)  
- **Data** — **D1** indices (fast lookup), **R2** immutable JSONL archives, **KV** for signed policy packs  
- **MCP Portal** — the single URL for tools (server + client), with tool‑level logs ("Capability")  
- **Zero Trust** — Access gates UI/API/MCP for humans & bots (service tokens)  
- **Execution** — Workflows/Queues for durable jobs; **Containers/Sandbox** for untrusted code

This keeps **control plane** (Access + policy), **data plane** (DO ordering + archives), and **execution plane** (tools + jobs + sandbox) cleanly separated and fully receipted.

---

## 6.0 Policy & Law — TDLN, Not Ad Hoc Checks
All enforcement compiles to **TDLN** (deterministic, fail‑closed). Policies live as **signed YAML packs** (Chip‑as‑Code) distributed via KV and executed in WASM at the edge (same codebase compiles to native for on‑prem). A failed policy path produces a **GHOST** action record (attempted but denied), preserving forensic truth without side effects.

---

## 7.0 MCP‑First — One Portal, Many Tools
Agents and UIs do not discover dozens of endpoints; they connect to **one MCP portal URL**. UBL exposes at minimum:

- **Messenger tools** — `messenger.list_rooms`, `messenger.send`, `messenger.history`  
- **Office tools** — `office.document.create|get|search`, `office.llm.complete` (via AI Gateway)  

Every `tools/call` appends `action.v1` (with `did = <tool name>`) and an `effect.v1`. Portal logs show **Capability** = tool name, giving you triple‑entry audit: *Portal log ↔ UBL receipt ↔ Container state*.

---

## 8.0 Public Surface — REST + SSE (PWA‑Ready)
Minimal REST (isomorphic with MCP) for the iPhone PWA:

- `GET /api/rooms` · `POST /api/rooms`  
- `GET /api/rooms/:id/history?cursor&limit`  
- `POST /api/rooms/:id/messages`  
- `GET /api/events/rooms/:id?from_seq` → **SSE** stream (`id: <room_seq>`, event: `message.created`)  
- `GET /api/receipts/:seq` → returns `action.v1` + `effect.v1` atoms

Identity is normalized from **Cloudflare Access**; each response includes a **request_id** that also appears in the ledger atom `trace.request_id` for correlation.

---

## 9.0 Economics — Wallets, Credits & Settlement (vNext)
UBL introduces **Wallet** as a strict, sealed Container. Credits (priced in an external schedule) move via a universal **transfer** operation governed by WalletGovernance Agreements. Micro‑settlement batches are **Merkle‑ized** and recorded as atoms; larger flows can settle on external rails if required. This renders tool usage and agent work as **priced, receipted economic events** within the UBL fabric.

---

## 10.0 Implementation Snapshot (MVP‑1 → MVP‑3)
**MVP‑1 — Messenger Kernel + Office MCP + Portal**
- Rooms with ordered messages (`room_seq`), receipts (`seq`, `head_hash`), SSE to iPhone  
- Single `/mcp` endpoint (initialize, tools/list, tools/call) behind **Access** + **Portal**  
- D1 indices; R2 JSONL archives; KV policy pack  
- Proof points: Portal logs show `Capability = messenger.send`; `GET /api/receipts/:seq` returns atoms

**MVP‑2 — Mini‑Contracts & Workflows**
- `PROPOSE → APPROVE → EXECUTE → SETTLE` in RoomObject with **Workflows** + Agreements  
- Tool calls and long‑running jobs pause for approvals with receipted outcomes

**MVP‑3 — E2EE Rooms**
- Per‑room encryption envelopes; **KeyDirectory** DO; server never sees plaintext

---

## 11.0 Security Posture
- **Zero‑Trust perimeter**: Access on UI/API/MCP; service tokens for bots  
- **Origin validation** on `/mcp` (defend against DNS rebinding)  
- **Deterministic receipts**: canonical JSON, SHA‑256 CIDs, hash chain; upgrade path to **JSON✯Atomic + DV25‑Seal**  
- **Agreement‑first** authorization baked into every `action.v1` (`agreement_id` recommended in MVP‑1; mandatory in MVP‑2)  
- **Forensic completeness**: GHOST actions for denied paths

---

## 12.0 Interoperability with the Stack
- **JSON✯Atomic** — canonical bytes + signatures for atoms and policy packs  
- **LLLV** — retrieval evidence lands as receipted artifacts in Containers; queries and proofs are auditable  
- **TDLN** — policies are the executable "law" for all `ubl.*` and `office.*` intents  
- **SIRP** — transport capsules and batched receipts across hops with cryptographic proofs

---

## 13.0 Test Plan ("Proof of Done")
1) Connect an MCP client to the **portal URL** → `tools/list` shows `messenger.*` and `office.*`.  
2) Call `messenger.send` → iPhone PWA receives **SSE** `message.created` with `id = room_seq`.  
3) Fetch `GET /api/receipts/:seq` → observe `action.v1` (`did = messenger.send`) and `effect.v1`.  
4) Verify **Portal logs** show **Capability = messenger.send** and correlate by `request_id`.  
5) Reboot client; **history** backfills via `cursor`, **SSE** resumes with `from_seq`.

Success means UBL is operating as an **economic product** (tool bus + ledger) and as a **computable constitution** (Agreement‑first, receipt‑based).

---

## 14.0 Roadmap & Migration
- Consolidate Workers into **Gateway + Messenger + Office + Ledger** cores  
- Introduce **ContainerObject**; refactor Rooms/Workspaces/Wallets to config physics  
- Enforce **Agreement‑first** in all actions (mandatory)  
- Add **Jobs** (Workflows) and **Sandbox** for untrusted code  
- Turn on **AI Gateway** + **Firewall for AI** for all model I/O

---

## Appendix A — Canonical Structures (v1, minimal)
**Receipt**
```json
{"ledger_shard":"0","seq":1042,"cid":"c:...","head_hash":"h:...","time":"2026-01-08T12:34:56Z"}
```

**action.v1**
```json
{"kind":"action.v1","tenant_id":"t:...","cid":"c:...","prev_hash":"h:...","when":"...",
 "who":{"user_id":"u:...","email":"..."},"did":"messenger.send",
 "this":{"room_id":"r:...","msg_id":"m:...","room_seq":42,"body_hash":"b:..."},
 "agreement_id":"a:room:...","status":"executed","trace":{"request_id":"req:..."}}
```

**effect.v1**
```json
{"kind":"effect.v1","tenant_id":"t:...","cid":"c:...","ref_action_cid":"c:...","when":"...",
 "outcome":"ok","effects":[{"op":"room.append","room_id":"r:...","room_seq":42}],
 "pointers":{"msg_id":"m:..."}}
```

---

*UBL unifies roles as relationships, silos as Containers, and APIs as intentions—enabling agents to work, prove, and settle within a single verifiable fabric.*

---

## 15.0 Product Integration — **UBL Messenger** (Spec Extract)
*This section codifies the Messenger surface as the reference UBL product, aligning naming, APIs, UX and roadmap with the Agreement‑first + Container primitive.*

### 15.1 Frontend ↔ Backend Naming Map
- **UI `thread` ↔ API `room`**; **UI `realm` ↔ API `circle`**; **`agent` ↔ `entity(type=agent)`**; IDs carry prefixes (`r:`, `circle:`, `job:`, `u:`, `entity:`) on the backend. Adapters enforce the mapping transparently in the client.

### 15.2 Core UX & Capabilities (Now)
- Enterprise chat with **premium components** (grouping, inline replies, reactions, virtual scroll, read receipts), **PWA + offline outbox**, **SSE realtime** with `lastEventId` replay, **files upload** (chunked, resumable), **agents as first‑class contacts**, **ledgered audit** and **admin console** (RBAC, policies, legal hold).

### 15.3 APIs (Public Surface, normalized to UBL)
- **Threads/Rooms**:  
  `GET /api/rooms` · `GET /api/rooms/:id/history?cursor&limit` · `POST /api/rooms/:id/messages` · **SSE** `GET /api/events/rooms/:id?from_seq` (ordered by `room_seq`).  
- **Files** (to implement in UBL Gateway):  
  `GET /api/rooms/:roomId/files` · `POST /api/rooms/:roomId/files/upload` · `GET /api/files/:fileId/download`.  
- **Agents**:  
  `GET/POST /api/wa/agents` (list, create, update); planned: `GET /agents/:id/capabilities`, `POST /agents/:id/execute`.  
- **Jobs** (critical path):  
  `POST /api/jobs` · `GET /api/jobs` · `GET /api/jobs/:id` · `POST /api/jobs/:id/{approve|reject|cancel}` · approvals sub‑routes, plus **WS /ws** for live job updates.  
- **Ledger**:  
  `GET /api/receipts/:seq` and `GET /api/ledger/rooms/:roomId` (queries).  
- **MCP Portal**:  
  `POST /mcp` → `messenger.*`, `office.*` tools (document get/search/create; llm.complete).

### 15.4 Jobs-in-Chat (Design)
Jobs are **Container‑native artifacts** rendered inline (`JobCard`) with lifecycle *pending → running → completed/failed* and approvals, streamed via SSE/WS and fully receipted to the ledger; agents run jobs inside sandbox.

### 15.5 Onboarding & Circles
On first login, the **Onboarding flow** ensures the user belongs to at least one **circle** (create or join via invite). UI includes Circle Switcher and admin views for invites and membership.

### 15.6 Security & PWA Posture
Strict headers (CSP, HSTS, COOP/CORP), WebAuthn passkeys, DLP client‑side with server as final authority, **offline outbox** and manifest for installable PWA. E2EE is staged for a later milestone with per‑room envelopes and a Key Directory DO.

### 15.7 Design System
Adopt **V16i** (Next.js 14, App Router) as architectural base and migrate **warm coral/cream** palette from legacy Messenger to V16i via design tokens; maintain 149+ components + 33 premium chat components.

### 15.8 Roadmap Alignment (Messenger ⇄ UBL MVPs)
- **Messenger Jobs** ↔ **UBL MVP‑2 Workflows** (propose/approve/execute/settle).  
- **Files API** ↔ **R2‑backed artifacts** with ledger receipts.  
- **MCP Office** ↔ **Portal single‑URL** surface for tools.

## 16.0 Agent UX — **Universal LLM UI Patterns** (Spec Extract)
*This section defines the UI/behavioral constitution for LLM entities operating on UBL.*

### 16.1 Entity vs Instance
Persist **LLM Entity** (identity, keys, reputation) while treating each run as an **ephemeral Instance** that writes a handover. This reduces re‑orientation cost and stabilizes identity across sessions.

### 16.2 Context Frame & Narrative Preparation
Before invocation, build a **Context Frame** (identity, position, obligations, capabilities, affordances) and generate a **narrative** that situates the instance; inject governance notes and constitution last. **No context discovery at run‑time**.

### 16.3 Constitution & Governance
Override RLHF biases with a **Constitution** ("You are an Economic Actor, not a chatbot"), explicit behavioral rules, and negotiation stance. Enforce **Sanity Check**: compare subjective claims (handover) to objective facts; inject governance note if discrepant.

### 16.4 Dreaming Cycle & Safety Net
Periodic **Dreaming Cycle** consolidates memory, removes stale anxiety, and synthesizes patterns; **Simulation/Safety‑Net** lets agents test actions before committing (required for high‑risk ops).

### 16.5 Session Types & Modes
Adopt **work / assist / deliberate / research** session types with **commitment vs deliberation** modes to bind responsibility and token budgets. Track quotas per entity/session in the ledger.

### 16.6 Token Budget & Compression
Hybrid memory policy: recent events verbatim, older spans synthesized, bookmarks for key events, baseline narrative refreshed during dreaming.

### 16.7 UBL Binding
- Context Frames and Constitutions are **atoms** stored as signed artifacts (JSON✯Atomic), referenced by Agreements that authorize agent actions.  
- Handover notes become **receipted effects** linked to jobs/messages in Containers.  
- Sanity Check + Simulation are **policy steps** compiled to TDLN and logged as **GHOST** when denied.

---

## 17.0 Proof‑of‑Done (Augmented)
6) Messenger V16i + legacy colors live (design tokens updated).  
7) Jobs API endpoints respond and stream updates; job cards appear inline and ledger receipts resolve.  
8) Agent UX: Context Frame, Constitution, Sanity Check and Dreaming hooks wired; sessions tagged with type/mode and token budgets enforced.

---

## 18.0 Files & Artifacts — Chunked, Verifiable, Ledgered
**Goals.** Reliable upload (PWA/offline), integrity proof, DLP, and auditable trail in UBL.

**Flow.**
1) `POST /api/rooms/:roomId/files/upload` → returns `upload_id` + part *presigns* (R2).  
2) Client sends **parts** (5–32 MB) with **BLAKE3 digest** per part; server maintains **manifest**.  
3) `POST /api/rooms/:roomId/files/complete` with **part digests** → generates **FileAtom** (`artifact.v1`), saves to R2 and emits **effect.v1** in the ledger.  
4) **SSE** sends `file.created` (with `artifact_id` and `room_seq`).

**Policy.** DLP (type/mime/size), antivirus (optional), presign TTL, and **Agreement** required for *upload* and *fetch*.  
**Thumbnails & Derivatives.** `POST /api/files/:id/derive` with privacy policies (blur faces, remove EXIF).  
**Download.** `GET /api/files/:id/download` requires `agreement_id`; headers `Content-Digest: blake3=...`.

**Data model (simplified).**
```json
{"kind":"artifact.v1","artifact_id":"f:...","room_id":"r:...",
 "name":"...", "mime":"...", "bytes": 5242880, "parts":[{"i":0,"cid":"blake3:..."}],
 "uploader":{"user_id":"u:..."},"cid":"c:...","sig":"ed25519:...","when":"..."}
```

---

## 19.0 Jobs & Workflows — Room-Native, Receipted
**State machine.** `proposed → approved → running → {completed|failed|cancelled}` (+ `expired`).  
**APIs.** 
- `POST /api/jobs` creates *Job* (inline in chat as *JobCard*).  
- `POST /api/jobs/:id/approve|reject|cancel` (requires Agreement for each action).  
- `GET /api/jobs/:id` and `GET /api/jobs?cursor` (SSE/WS for progress).

**Receipts.** Each transition generates `action.v1` + `effect.v1` with `job_seq`; *long-running* stages emit *sub-effects* (stage updates).  
**Compensation.** TDLN policy may require **compensation steps** for failures (rollback of side effects).  
**Security.** Sandbox for *agent jobs*; quotas and *rate limits* per Agreement; **GHOST** for denials.

---

## 20.0 Agreements Lifecycle — Relationships as Law
**Types.** `RoomGovernance`, `WorkspaceAgreement`, `WalletGovernance`, `ToolCapability` (MCP).  
**Cycle.** `PROPOSE → CONSENT → ACTIVATE → RENEW/AMEND → REVOKE`.  
**Materialization.** A canonical **AgreementAtom** (JSON✯Atomic) with CID and issuer **signature**; Agreements versioned by `prev_cid`.  
**Binding.** Every `action.v1` recommends `agreement_id`; from MVP-2, **mandatory**.  
**Rule examples.**
- *Room.* "Only Owners can `approve` jobs; Guests can `send` but not `upload` > 50 MB."  
- *Wallet.* "Transfers limited to 100 cr/h; `agent:billing` can `transfer` up to 1 000 cr/h."

---

## 21.0 Security, Compliance & Legal Hold
- **Zero Trust** (Access) + **service tokens** for agents.  
- **Data classification** (public/internal/secret) — headers and UI labels; DLP at the gateway.  
- **Legal hold**: *freeze* of *effects* and retention of *artifacts*; audit with `GET /api/ledger/rooms/:roomId`.  
- **Privacy.** PII minimization; automatic redaction; *Data Subject* export.  
- **Crypto.** TLS everywhere; signatures (Ed25519) optional in MVP-1, standard in MVP-2.

---

## 22.0 Observability & SRE — Metrics, Logs, Traces
**Metrics (Worker/DO):** `requests_total`, `latency_ms{p50,p95}`, `do_lock_contention`, `sse_clients`, `room_backfill_ms`, `upload_parts_inflight`, `mcp_calls`.  
**Logs (structured):** `request_id`, `user_id`, `room_id`, `seq/head_hash`, `capability`, `agreement_id`, `status_code`, `bytes_in/out`.  
**Traces:** spans across `Gateway → DO(Room/Ledger) → R2/D1 → Portal`; **adaptive sampling**.  
**SLOs (initial):** 99.9% `/api/rooms/*`; p95 `messenger.send` < 250 ms; p95 backfill `history` < 700 ms.  
**Alerts:** p95 above target for 5 min; *error rate* > 1%; `sse_clients` abrupt drop.

---

## 23.0 MCP Portal — Contract & Capability Logs
**Handshake.** `POST /mcp` with Access; response `tools/list` includes `messenger.*`, `office.*`.  
**Call.** `tools/call` → `action.v1(did=<tool>)` → `effect.v1` (+ portal log `Capability=<tool>`).  
**Auth.** Bearer Access for humans; service tokens for agents; *origin check* and *capability scoping*.  
**Error model (MCP).** `{error: {code, message, details}}` with `code` ∈ `{UNAUTHORIZED, FORBIDDEN, INVALID_INPUT, CONFLICT, INTERNAL}`.

---

## 24.0 PWA & Offline — Outbox, Replay, Conflict
**Outbox.** Messages and uploads remain in the *outbox* when offline; upon reconnection, the client resends and marks *committed* when the ledger returns `seq/head_hash`.  
**Replay.** SSE uses `lastEventId` (=`room_seq`) and `from_seq` for consistent backfill.  
**Conflict.** Duplicates are deduplicated by `msg_id`/`artifact_id`; edit conflicts generate *forks* with assisted *merge* (Agreement may define policy).

---

## 25.0 E2EE (Milestone) — Room Envelopes & Key Directory
**Keys.** `KeyDirectory` (Durable Object) maintains *room keys* encrypted by member keys (WebAuthn/Passkey).  
**Envelope.** Messages and files are wrapped in `RoomEnvelope {nonce, aad=(room_id|seq), ciphertext}`; server stores only ciphertext.  
**Rotation.** Key rotation per *epoch*; asynchronous re-enveloping with receipts.  
**Fallback.** Legal Hold enables key *escrow* with audit.

---

## 26.0 Error Model — REST + SSE
**REST.**  
- 200/201 (OK/Created), 202 (Accepted), 204 (No Content).  
- 400 (Invalid), 401/403 (Auth), 404 (Not Found), 409 (Conflict), 413 (Payload Too Large), 422 (Unprocessable), 429 (Rate), 5xx.  
**Default body.**
```json
{"error":{"code":"INVALID_INPUT","message":"...", "trace_id":"req:...","details":{}}}
```
**SSE.** Events with `id: <room_seq>`; `event:` ∈ `{message.created, file.created, job.updated}`; `retry: 5000` for *backoff*.

---

## 27.0 Capacity & Performance Envelope (Cloudflare-first)
- **Rooms**: 50k messages/day per room (target), p95 send < 250 ms; SSE up to 500 clients/room (fanout via *broadcast channels*).  
- **Files**: upload 5–100 MB per part; throughput 150 MB/s per Gateway; derivatives in *queue*.  
- **Ledger**: 5k atoms/s per shard; monotonic `seq`; *head_hash* updated per batch.  
- **Portal**: 1k `tools/call`/s sustained per shard with Capability logs.

---

## 28.0 Glossary (mini)
**Agreement** — verified relationship that authorizes actions in a Container.  
**Container** — entity that holds assets and defines their "physics" (Room/Workspace/Wallet).  
**Receipt** — evidence (`seq/head_hash/cid/time`) of an append to the ledger.  
**GHOST** — denied action recorded without effects.  
**Portal** — single MCP URL that exposes tools and generates Capability logs.
