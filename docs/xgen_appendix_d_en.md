# XGen Protocol — Appendix D – Node Data, Privacy, and Storage
> **Status:** ACTIVE  
> Version: 0.1  
> Date: April 2026  
> **Last updated:** 2026-05-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

*A technical and policy reference for node operators, institutional evaluators, data protection officers, and contributors*

---

## Purpose of This Document

This appendix answers a single question that institutions, data protection officers, and privacy-conscious users consistently ask before adopting any federated communication system:

> *What exactly does this software store about users, where does it store it, how is it protected, and who controls it?*

The answer has two layers. The first is architectural — what XGen is designed to store and why. The second is operational — what a given Node operator's deployment actually stores, which depends on decisions the operator makes within the constraints the protocol provides.

This document covers both.

---

## Part 1 — Architectural Principles

### 1.1 The Data Minimisation Principle

XGen is designed around a hard principle: **a Node stores the minimum data required to perform its protocol function, and nothing else.**

This is not a privacy policy commitment — it is an architectural constraint enforced at the protocol level. A Node that stores more than this is storing data it has no protocol reason to hold. The XGen specification does not define fields, message types, or storage structures for surveillance-adjacent data: connection logs, IP address history, read receipts, typing indicators, or presence history.

The rationale is structural trust. A federated protocol where individual Node operators may be adversarial, underfunded, or subject to legal compulsion cannot rely on policy commitments alone. Minimising what Nodes are architected to store reduces the attack surface regardless of operator intent.

### 1.2 The Signed Event Log Is the Source of Truth

Every action in XGen — a message, a membership change, a permission update — is expressed as a signed, content-addressed Event stored in an append-only log. The Node does not interpret Events; it stores and propagates them. The sender's cryptographic signature is the source of authenticity, not the Node's assertion.

This means a Node cannot fabricate Events. It also means that what a Node "knows" about a user is limited to what that user's keypair has signed and sent.

### 1.3 Identity Is Self-Certifying

A user's Identity ID is derived from their Ed25519 public key — it is a cryptographic commitment to their keypair, not a record assigned by a Node. The Node does not issue, own, or control Identity IDs. A user can verify their Identity ID independently without asking any Node.

This means a Node's identity registry is a cache of self-certifying records, not an authority. The Node's deletion of a record does not destroy the Identity — it only removes the Node's local copy.

### 1.4 The Node Operator's Accountability

In a federated system, the Node operator is the data controller for the records held on their Node. XGen Protocol Foundation (when established) is the software publisher, not a data controller for any Node's data.

---

## Part 2 — What a Node Stores

### 2.1 Identity Records

When a user registers on a Node, the Node stores an identity record. The fields are:

| Field | Type | Why stored | Can be omitted? |
|---|---|---|---|
| `identity_id` | pubkey URI | Required — the unique identifier for this Identity | No |
| `display_name` | string | Optional — user-chosen display name | Yes — user may omit |
| `registered_at` | datetime | Required — timestamp of registration | No |
| `home_node` | node URI | Required — which Node this Identity registered on | No |
| `devices` | array | Required — list of device public keys (Phase 1: one device) | No |
| `trust_assertion` | object | Conditional — present if Tier 1+ Auth Module was used | Absent in Local Node mode |

**What is NOT stored in the identity record:**

- IP address of the registering client
- Email address or phone number *unless* the Trust Assertion explicitly includes it (see section 2.4)
- Password or passphrase (the keypair is the credential — passphrases never leave the client)
- Device fingerprint, browser agent, or hardware identifiers
- Login timestamps or session history

### 2.2 The Event DAG (Message Store)

The Node stores the full Event DAG for every Space and Room it participates in. This includes:

- All `message.text`, `message.image`, `message.file`, `message.reaction` Events
- All `state.*` Events (Space and Room creation, name changes, etc.)
- All `membership.*` Events (invites, joins, leaves, kicks, bans)
- All `system.*` Events

Each Event contains:

| Field | What it reveals |
|---|---|
| `event_id` | Content hash — verifiable, not linkable to sender beyond what `sender` already reveals |
| `sender` | The Identity ID (pubkey URI) of the sender — who sent this Event |
| `timestamp` | When the sender claims to have created this Event |
| `content` | The actual message or state change |
| `prev_events` | Which Events this Event causally follows — the conversation graph |

**The Node cannot selectively delete Events from the DAG** without breaking the cryptographic chain. This is a deliberate design choice: the append-only log is what makes the conversation history tamper-evident. The right-to-erasure tension this creates is addressed in section 3.3.

### 2.3 Federation Records

When a Node federates with a peer Node, it stores a federation relationship record:

| Field | Why stored |
|---|---|
| `peer_node_id` | The peer's Node ID (pubkey URI) |
| `session_id` | The negotiated session identifier |
| `shared_spaces` | Which Spaces are shared across this federation link |
| `established_at` | When federation was established |

Federation records do not contain IP addresses. The Node's transport layer handles connection management separately and does not persist connection metadata beyond what is required to maintain the active session.

### 2.4 Trust Assertions (Tier 1+)

If the Node serves Tier 1 or higher Spaces, it stores the Trust Assertion provided at registration. What the assertion contains depends on the Auth Module operator's policy (see spec 3.8.4). Plaintext contact details are not permitted in XGen Trust Assertions — two privacy-preserving options are available:

| Option | What is stored on the Node | What propagates to federated Nodes |
|---|---|---|
| Option A — Hashed | Salted hash of email/phone | Only the hash propagates |
| Option B — Flag only | Verification fact only | Nothing beyond the fact |

Plaintext email addresses and phone numbers are explicitly prohibited from appearing in Trust Assertions. The Auth Module holds the authoritative contact record. The protocol has no need to carry it, and once plaintext data enters a federated append-only log it cannot be reliably recalled under right-to-erasure requests.

### 2.5 Node Announcement Cache

Each Node maintains a cache of NodeAnnouncement records received from peer Nodes. These contain:

- Peer Node ID (pubkey URI)
- Peer Node endpoint URI (the address to connect to)
- Capabilities declared by the peer
- Announcement TTL (`valid_until`)

No user data is present in Node announcements.

### 2.6 What a Node Explicitly Does NOT Store

The following data is neither defined nor implied in any XGen storage structure:

- Client IP addresses (not stored at any layer)
- Connection timestamps beyond active session state
- Read receipts (no protocol primitive for this in Phase 1)
- Typing indicators (ephemeral, not persisted)
- Presence history (presence is a session-scoped signal, not a stored record)
- Geolocation data
- Device fingerprints
- Browser or client version strings (present in `meta_atts` at sender discretion, not extracted or stored separately by the Node)

---

## Part 3 — Security of Stored Data

### 3.1 Phase 1 Storage Security

In Phase 1, the Node's Event DAG is in-process memory only (no disk persistence across restarts). The files that do persist to disk are:

| File | Contents | Protection |
|---|---|---|
| Node keypair file | Ed25519 private key | Encrypted: ChaCha20-Poly1305 + Argon2id KDF |
| Identity registry | Identity records (JSON) | Plaintext — Phase 1 limitation, see 3.2 |
| Federation registry | Federation relationships (JSON) | Plaintext — Phase 1 limitation, see 3.2 |
| Node announcement | Signed announcement (JSON) | Signature provides integrity; no confidentiality needed (public record) |

### 3.2 Phase 2 Storage Security (Planned)

Phase 2 will add:

- Full disk-based Event DAG persistence with encryption at rest
- Encrypted identity registry and federation registry
- Configurable key derivation for the storage encryption layer (operator-supplied passphrase or HSM-backed key)
- Secure deletion support for right-to-erasure workflows (see 3.3)

### 3.3 The Right-to-Erasure Problem in Federated Systems

The GDPR right to erasure (Article 17) and equivalent provisions in other jurisdictions create a genuine tension with append-only federated Event logs. This tension is not unique to XGen — it is an open problem across all federated communication protocols, including Matrix and ActivityPub. XGen's position is stated here honestly.

**What can be handled cleanly:**
- The identity record on the home Node can be deleted on request
- The Trust Assertion on the home Node can be deleted on request
- The home Node's local cache of the user's Events can be cleared

**What cannot be undone without breaking cryptographic integrity:**
- Events already propagated to federated peer Nodes — XGen has no mechanism to compel peer Nodes to delete specific Events
- The `sender` field in Events already distributed — these are signed and hash-chained into the DAG

**XGen's planned approach (Phase 2):**
A `message.redact` Event type is defined in the EventType registry. Redaction replaces the `content` of a prior Event with a tombstone marker while preserving the Event's position in the DAG. The Event ID and `sender` field remain (to maintain DAG integrity); the content is cleared and replaced with a redaction notice. This satisfies the practical intent of erasure requests for message content while preserving cryptographic chain integrity.

For identity erasure requests, the planned approach is: remove the identity record from the home Node, propagate a signed deletion notice to federated Nodes, and rely on federation TTLs to expire cached records. This does not remove the `sender` field from historical Events — that is a structural limitation of append-only federated logs, disclosed here for institutional evaluators.

Operators subject to strict right-to-erasure obligations should discuss this limitation with their data protection officer before deployment.

---

## Part 4 — Operator Responsibilities

### 4.1 The Node Operator as Data Controller

In GDPR terms: the Node operator is the **data controller** for Identity records, Trust Assertions, and Event DAGs held on their Node. XGen Protocol Foundation (when established) is the software publisher, not a data controller for any Node's data. The Foundation does not have access to any Node's stored data.

### 4.2 What Operators Control

Node operators configure:

- Which Auth Module(s) to trust (and therefore which Trust Assertion format they accept)
- Whether Local Node mode is active (bypasses Trust Assertions entirely — development use only)
- Which Spaces the Node participates in (and therefore which Event DAGs it holds)
- File paths for all persistent data — keypair, registries, DAG store (operator-configurable, not hardcoded)
- Retention policy (no built-in automatic deletion in Phase 1 — operator's responsibility)

### 4.3 What Operators Cannot Control

Node operators cannot:

- Modify Events in the DAG (cryptographic signatures prevent fabrication or alteration)
- Create Events attributed to Identities they do not hold the keypair for
- Impersonate another Node (Node IDs are self-certifying keypairs)
- Prevent federated peer Nodes from retaining their own copies of Events already received

### 4.4 Recommended Operator Practices

For operators handling personal data under data protection law:

1. Use Trust Assertion Option A (hashed) or Option B (flag only) — plaintext contact details are prohibited by the protocol
2. Document your data retention policy and communicate it to users at registration
3. Implement a process for handling right-to-erasure requests (manual coordination in Phase 1; tooling planned for Phase 2)
4. Keep the Node keypair file on encrypted storage (full-disk encryption at the OS level, at minimum)
5. If operating in a high-risk jurisdiction or federating with Nodes in other jurisdictions, consult legal counsel on cross-border data transfer obligations

---

## Part 5 — Summary Table for Evaluators

| Question | Answer |
|---|---|
| Does the Node store IP addresses? | No — not in any protocol-defined storage structure |
| Does the Node store passwords or passphrases? | No — keypairs are the credential; passphrases never leave the client device |
| Does the Node store email or phone numbers? | No. Plaintext contact details are prohibited in Trust Assertions. Only hashed values (Option A) or verification flags (Option B) are permitted. |
| Does the Node log message read times? | No — no read receipt primitive exists in the protocol |
| Does the Node store typing or presence history? | No — both are ephemeral and not persisted |
| Can message content be deleted? | Redaction (Phase 2) removes content while preserving DAG position. Phase 1: manual only. |
| Can identity records be deleted from the home Node? | Yes |
| Can identity records be deleted from federated Nodes? | Via signed deletion notice + TTL expiry — not guaranteed to propagate instantly |
| Who controls the data on a Node? | The Node operator. XGen Foundation has no access to any Node's data. |
| Is stored data encrypted at rest? | Keypair: yes (ChaCha20-Poly1305 + Argon2id). Registry files and DAG: Phase 2. |
| Is the software open source? | Core library: GPL. Node/client shells: BSL 1.1 converting to GPL on community handover. |
| Who is the data controller under GDPR? | The Node operator for data on their Node. The Foundation is the software publisher only. |
| Does the Node produce an audit log? | Yes — a permanent, append-only protocol audit log records all membership and state Events. Cannot be disabled. See Part 6. |
| Who can read the audit log? | The Node operator and any party the operator grants filesystem access to. The Foundation has no access. |
| How long are audit logs retained? | Operator decision at Tier 1/2. Regulatory minimum at Tier 3 (7 years) and Tier 4 (10 years healthcare). |

---

## Part 6 — Audit Logging

XGen nodes produce two independent log types. This section covers the audit log. The debug log (a technical diagnostic tool for operators) is documented in Ch4 section 4.17.1 and is not relevant to this document's audience.

### 6.1 The Protocol Audit Log

Every XGen Node maintains a permanent, append-only protocol audit log. This log records all membership and state-change Events that occur in Spaces hosted by or federated to this Node. It cannot be disabled by configuration.

The audit log is distinct from the debug log. It serves auditors, compliance officers, and regulators — not developers.

**What is recorded:** every occurrence of the following Event types:

- `membership.join` — who joined which Space, when
- `membership.leave` — who left, when
- `membership.invite` — who invited whom
- `membership.kick` — who was removed, by whom, reason if stated
- `membership.ban` — who was banned, by whom, reason if stated
- `state.space_create` — Space created, by whom, at what Auth Tier
- `state.room_create` — Room created, by whom, in which Space
- `state.federation_add` — federation established between two Nodes for a Space
- `state.federation_remove` — federation ended
- `identity.register` — Identity registered on this Node
- `system.key_rotation` — Identity keypair rotated

**What is not recorded:** message content is never written to the audit log. If E2E encryption is active (Phase 2), the Node cannot access message content at all. Even without E2E encryption, message content is not an audit log concern — only protocol-level facts about membership and structure are recorded.

**Format:** JSON Lines — one JSON object per line. Each line carries a UTC timestamp, the EventType, the event ID (which links back to the full Event in the DAG), the Node ID, and EventType-specific fields (identity IDs, Space IDs, etc.). The format is machine-readable and directly importable into standard log aggregation systems.

**Location:** `audit/` subfolder in the Node's working directory. One file per calendar month: `audit/protocol_audit_YYYY-MM.jsonl`.

**Retention:** audit files must not be auto-deleted by the Node. Deletion is the operator's decision, subject to applicable regulatory requirements.

### 6.2 Retention Requirements by Tier

| Tier | Regulatory context | Minimum retention |
|---|---|---|
| Tier 1 | General — GDPR baseline | No protocol minimum. Operator's decision. |
| Tier 2 | ISO 27001 Professional | No protocol minimum. Operator's decision. |
| Tier 3 | Corporate — SOX, Basel II/III | 7 years (SOX §802). Banking: Basel II/III requirements apply additionally. |
| Tier 4 | Government / Healthcare | 10 years minimum for healthcare (HDS, SGB V). Government: jurisdiction-defined. |

### 6.3 Auth Module Audit Log

Separate from the Node's protocol audit log, Tier 3 and Tier 4 Auth Module operators are required to maintain their own audit log of identity verification decisions. This log lives inside the Auth Module, not the Node. The Node has no access to it.

The Auth Module audit log records: who was verified, what evidence was presented, what verification state was assigned, when the Trust Assertion was issued and when it expires. At Tier 4, it additionally records eIDAS LoA evidence, government credential binding details, clearance verification, and data access events per GDPR Art. 30.

Full requirements are specified in 3.11.8.

### 6.4 What the Audit Log Does Not Replace

The protocol audit log establishes *that* something happened at the protocol level. It does not establish *why* it happened, or whether it was authorised under the operator's internal policies. A complete compliance picture for a Tier 3 or Tier 4 deployment requires both the Node's protocol audit log and the Auth Module's verification audit log.

---

## Session Log

### Session 1 — April 2026 (JozefN)
**Covered:** Appendix D written in full. Triggered by institutional evaluator and colleague questions about user data handling. Covers: architectural data minimisation principle, signed event log as source of truth, self-certifying identity model, identity record fields (and explicit non-fields), event DAG contents, federation records, trust assertion storage options and their privacy tradeoffs, what a Node explicitly does not store, Phase 1 vs Phase 2 storage security, the right-to-erasure problem in federated append-only logs, operator responsibilities and limits, recommended operator practices, summary table for non-technical evaluators and DPOs.

### Session 2 — April 2026 (JozefN)
**Covered:** Part 6 Audit Logging added. Two log types distinguished for DPO/evaluator audience. Protocol audit log documented: permanent, append-only, cannot be disabled, JSON Lines monthly rotation, 11 EventTypes, message content never recorded. Retention by Tier table added: Tier 1/2 operator decision, Tier 3 seven years SOX, Tier 4 ten years healthcare. Auth Module audit log summarised (Tier 3/4 only, full requirements in 3.11.8). Summary table extended with three new rows.
