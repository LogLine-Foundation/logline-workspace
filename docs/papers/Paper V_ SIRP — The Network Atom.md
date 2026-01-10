# Paper V: SIRP — The Network Atom

## 1. 0 Abstract 
**SIRP (Semantic Intent Routing Protocol)** defines the communication substrate for the verifiable agent economy. It is an application-layer overlay network designed to resolve the fundamental mismatch between the ephemeral nature of IP-based networking and the permanent, accountable requirements of autonomous economic agents.

Standard protocols like TCP/IP couple identity to topological location (`IP: Port`) and treat packet delivery as a "best-effort, " unaccountable event. SIRP inverts this model. It routes messages based on **Cryptographic Identity (DID)** and **Semantic Intent**, utilizing a content-addressed "Capsule" as the atomic unit of transport. Furthermore, it transforms routing from a sunk cost into a verifiable economic activity through the **Cryptographic Receipt Protocol**, enabling a decentralized, incentivized relay market.

This paper details the binary specification of the SIRP Capsule, the "Zero Silicon" Transport Abstraction Layer (TAL), the DHT-based discovery architecture, and the formal verification of packet delivery.

## 2. 0 The Problem: The Location Trap 
### 2. 1 The Fragility of Topological Identity 
The internet's current architecture binds an entity's identity to its network location. This creates three critical failure modes for autonomous agents: 
1. **Mobility Failure: ** An agent on a mobile device changes networks (Wi-Fi $\\to$ 5G), altering its IP address. This breaks all active TCP connections, invalidating sessions and disrupting long-running economic transactions. 
2. **Middlebox Opacity: ** The proliferation of Carrier-Grade NAT (CGNAT) and firewalls creates a "Hotel California" topology: Agents can make outbound requests but are unreachable for inbound commands without centralized, trusted intermediaries. 
3. **Accountability Void: ** IP packets are fungible and "best-effort. " There is no native protocol mechanism to prove that a specific packet was delivered, processed, or routed by a specific intermediary. In an economy of value, "I think I sent it" is insufficient.

### 2. 2 The Solution: Zero Location Networking 
SIRP establishes a "Zero Location" architecture where: 
* **Identity is Absolute: ** Addressing is done via Decentralized Identifiers (DIDs), e. g. , `did: logline: agent: alice`. This identifier remains constant regardless of network topology. 
* **Routing is Semantic: ** The network routes based on *what* the data is (Intent) and *who* it is for, dynamically resolving the path via a Distributed Hash Table (DHT). 
* **Transport is Verifiable: ** Every hop in the network generates a cryptographic proof of service (Receipt), linking the transport layer directly to the economic layer.

## 3. 0 Specification: The SIRP Capsule 
The **Capsule** is the atomic unit of the SIRP network. Unlike a TCP packet, which is a transient stream of bytes, a Capsule is a self-contained, immutable, and cryptographically verifiable object.

### 3. 1 The Wire Format Specification 
The Capsule utilizes a binary packing format optimized for zero-copy parsing and high-speed validation. All integers are Big-Endian.

| Field | Offset | Size (Bytes) | Type | Description | 
|: --- |: --- |: --- |: --- |: --- | 
| **`MAGIC`** | 0 | 2 | `u16` | **Protocol ID: ** `0x5199`. Fast rejection of non-SIRP traffic. | 
| **`VER`** | 2 | 1 | `u8` | **Version: ** Current `0x01`. Enables protocol evolution. | 
| **`FLAGS`** | 3 | 1 | `u8` | Bitmask: `0x01` (Encrypted), `0x02` (Receipt Req), `0x04` (Priority). | 
| **`TTL`** | 4 | 1 | `u8` | **Time-To-Live: ** Max hop count (default 64\) to prevent routing loops. | 
| **`CID`** | 5 | 32 | `[u8; 32]` | **Content ID: ** BLAKE3 hash of `PAYLOAD`. Used for deduplication and addressing. | 
| **`INTENT`** | 37 | 8 | `u64` | **Semantic Hash: ** First 64-bits of `BLAKE3("namespace. action")`. e. g. , `ubl. transact`. Allows routing logic to prioritize traffic types without decryption. | 
| **`TS`** | 45 | 8 | `u64` | **Timestamp: ** UTC Nanoseconds. Enforces strict replay protection windows. | 
| **`LEN`** | 53 | 4 | `u32` | **Length: ** Size of the Payload in bytes. | 
| **`SIG`** | 57 | 64 | `[u8; 64]` | **Signature: ** Ed25519 signature of `Header[0. .57] \+ PAYLOAD`. Proves authorship and integrity. | 
| **`PAYLOAD`** | 121 | Variable | `Bytes` | The encrypted data body (typically `CipherEnvelope`). |

### 3. 2 Security Invariants 
1. **Envelope Integrity: ** The `SIG` field covers the immutable header (including `CID`, `TS`, `INTENT`) and the payload. Any bit-flip by a middlebox invalidates the packet instantly. 
2. **Stateless Replay Protection: ** Nodes maintain a sliding window Bloom Filter of `(CID, TS)` pairs. Capsules with timestamps outside the validity window ($\\pm$30s) or with previously seen CIDs are dropped at the edge, mitigating DoS replay attacks without requiring persistent state. 
3. **Blind Prioritization: ** The `INTENT` field enables "Smart Routing. " A relay can prioritize `ubl. alert` (Critical) over `stream. video` (Bulk) based on policy, even though the payload remains fully encrypted.

## 4. 0 The Network Architecture 
### 4. 1 Discovery: The Modified Kademlia DHT 
SIRP utilizes a modified implementation of the **Kademlia Distributed Hash Table** for peer discovery and routing resolution. 
* **Node ID: ** `SHA256(DID_PublicKey)`. This cryptographically binds the network address to the agent's identity. 
* **Keyspace: ** 256-bit flat address space. XOR metric for distance calculation. 
* **Values: ** Stored records are signed "Peer Descriptors": 
 ```json 
 { 
 "did": "did: logline: agent: alice", 
 "endpoints": ["udp: //203. 0. 113. 5: 9000", "wss: //relay. node. io/alice"], 
 "relay_did": "did: logline: relay: gamma", 
 "ts": 1704456000, 
 "sig": "ed25519: .. ." 
 } 
 ``` 
When Agent A wants to message Agent B, it queries the DHT for `Hash(B. did)`. The network returns B's most recent signed Peer Descriptor, allowing A to connect directly or via B's designated relay.

### 4. 2 Transport Layer Abstraction (TAL) 
The **Transport Layer Abstraction (TAL)** allows SIRP to function as a "Protocol Chameleon. " It it rides on top of whatever is available.

**Driver Selection Logic: ** 
1. **UDP (Datagram): ** *Preferred. * Lowest latency, minimal overhead. Used for direct P2P and high-throughput streams. 
2. **QUIC: ** *Strategic. * Used when multiplexing is required or when UDP is throttled but not blocked. Solves Head-of-Line blocking. 
3. **WebSocket (WSS): ** *Fallback. * Used for browser-based agents or traversing restrictive corporate firewalls (Port 443). 
4. **TCP: ** *Legacy. * Last resort fallback.

**"Zero Silicon" Principle: ** By implementing the TAL in software (Rust/WASM), SIRP requires no specialized router hardware or OS kernel modifications. It runs in user-space on any device.

## 5. 0 The Economic Layer: Cryptographic Receipts 
This section details the mechanism that transforms routing from a cost center into a verifiable market.

### 5. 1 The Receipt Protocol Specification 
Upon successfully validating and processing a Capsule, a receiving node *must* emit a **Signed Receipt**. This is a financial instrument.

**Receipt Schema: ** 
```json 
{ 
 "type": "SIRP_RECEIPT_V1", 
 "capsule_cid": "blake3: 8f1e3c. .. ", // Links to specific packet 
 "sender_did": "did: logline: agent: alice", 
 "receiver_did": "did: logline: node: relay_01", 
 "ts_received": "2026-01-05T12: 00: 00. 123Z", 
 "metrics": { 
 "latency_ingress_ms": 12, // Time from TS to TS_Received 
 "verification_cost_us": 450 // Compute used for Ed25519 verify 
 }, 
 "outcome": "FORWARDED", // or "DELIVERED", "DROPPED_TTL" 
 "sig": "ed25519: .. ." // Receiver's signature 
} 
```

### 5. 2 The Economic Feedback Loop 
These receipts are ingested by the UBL Ledger to drive the network economy: 
1. **Micro-Settlement: ** The sender creates a UBL transaction aggregating receipts. 
 $$Payment \= \\sum (BaseFee \+ \\alpha \\cdot Latency^{-1} \+ \\beta \\cdot Compute)$$ 
 *Note: * Faster delivery (lower latency) earns a higher fee. 
2. **Reputation Score: ** The DHT uses receipt volume and quality to score nodes. 
 * High uptime \+ Low latency \= High Reputation \= More Traffic \= More Fees. 
 * This incentivizes the organic growth of high-quality Relay Nodes. 
3. **Provable Audit: ** In the event of data loss, the sender holds a chain of receipts up to the point of failure. The last node to issue a receipt but fail to forward is mathematically identifiable as the point of failure.

## 6. 0 Comparative Analysis: SIRP vs. Overlay Networks 
To contextualize SIRP, we compare it against established overlay networks: **Tor**, **I2P**, and **libp2p**.

| Feature | SIRP | libp2p | Tor | I2P | 
|: --- |: --- |: --- |: --- |: --- | 
| **Primary Design Goal** | **Accountability & Verifiability** | Modularity & P2P Tooling | Anonymity & Privacy | Anonymity & Resilience | 
| **Addressing Model** | **Semantic Intent (DID+CID)** | Key-Based (PeerID) | Onion Layers | Garlic Routing | 
| **Atomic Unit** | **Signed Capsule** | Frame / Stream | Cell (512 bytes) | Message | 
| **Routing Economy** | **Receipt-Based Market** | None (Barter/Credit) | Volunteer | Volunteer | 
| **Content Addressing** | **Native (CID in Header)** | Native (IPFS) | No | No | 
| **Latency Overhead** | **Low (\~5ms)** | Low | High (\~300ms) | High (\~500ms) | 
| **NAT Traversal** | **Aggressive (TAL \+ Relays)** | Aggressive (Hole Punching) | TCP Tunnels | UDP/TCP Tunnels |

**Strategic Differentiator: ** 
* **Tor/I2P** are designed to *hide* the participants. This creates high latency and makes economic attribution impossible. 
* **SIRP** is designed for *economic agents*. They generally *want* to be identified (to receive funds/services) but require location independence and secure routing. SIRP provides the **Accountability** that Tor explicitly removes.

## 7. 0 Conclusion: The Postal Service of Intent 
SIRP represents the evolution of the network layer from a "dumb pipe" to a "smart, accountable substrate. "

In the agent economy, the mere delivery of bits is insufficient. We require **Proof of Delivery**. We need a network that understands the difference between a video stream (High Bandwidth, Low Value) and a financial transaction (Low Bandwidth, Critical Value) and routes them accordingly.

SIRP achieves this by wrapping data in the **Capsule**, routing it via the **DHT**, and verifying it with **Receipts**. It ensures that the network is not a black hole of best-effort packets, but a transparent, audited supply chain for digital intent. It acts as the "Postal Service" for the LogLine economy: guaranteed, tracked, and insured. 
