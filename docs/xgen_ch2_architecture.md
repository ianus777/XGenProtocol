# XGen Protocol — Chapter 2: Architecture
> Status: wip
> Version: 0.1
> Date: April 2026
> Last edited: April 2026
> Language: English
> Author: JozefN
> License: BSL 1.1 (converts to GPL upon project handover)

---

## Architecture Principles

Chapter 1 established the *why*. Chapter 2 establishes the *what* — the architecture that makes those philosophical commitments structurally real. Every design decision in this chapter is governed by a small set of principles. They are stated here once and assumed throughout.

**Spec-first.** The specification is the authority. No implementation — including the reference implementation — defines the protocol. Code proves the spec works. It does not define it.

**Thin core.** The protocol core defines interfaces and contracts, not implementations. The thinner the core, the longer it survives. This is how TCP/IP outlasted every technology built on top of it.

**Stable interfaces downward, swappable implementations upward.** Every layer defines a stable contract with the layer below it and accepts pluggable implementations above it. No layer may assume the internal details of the layer below.

**Primitives over features.** When a capability is needed by more than one part of the system, it becomes a protocol primitive — not an application feature. Space, Room, Thread, Event, Identity, Auth Module, Node are all primitives. They are defined in the spec, not improvised by clients.

**Isolation boundary is the Node, not the hardware.** Physical co-location of Nodes on the same machine has no protocol meaning. Nodes are unaware of each other unless they federate explicitly via the XGen protocol.

**meta-atts everywhere.** Every protocol primitive carries a `meta-atts` field — a namespaced key-value map of metadata attributes, opaque to the protocol core. This is the universal extension point. The protocol stores and propagates `meta-atts` but never interprets them.

**Open enums.** Enumerated types — capability lists, event types, node types — are intentionally open-ended. Old implementations ignore values they do not understand. New implementations add values without breaking existing ones. This is how the protocol grows without forking.

---

## Terminology & Primitive Lineage

XGen does not invent terminology unnecessarily. Where an existing concept is borrowed — and the concept is sound — the name is kept. This reduces cognitive load for developers familiar with Matrix or Discord, and is an honest acknowledgment of the work those projects did.

The tables below establish the canonical XGen vocabulary and its origins. Every term defined here is used consistently throughout Chapters 2–5.

### Table A — Primitive Lineage

What each XGen primitive is, where it comes from, and what changed.

| XGen Primitive | Origin | Origin Name | What XGen Changed |
|---|---|---|---|
| Event | Matrix | Event | Kept. Signed, immutable, the atom of the protocol. Signing now tied to server-independent keypair identity. |
| Room | Matrix | Room | Kept. Battle-tested organizing unit. Federation and identity model updated. |
| Space | Matrix | Space | Kept name. Redesigned as a true first-class protocol primitive — cryptographic identity, cascading permissions, portability between Nodes. Matrix Spaces were bolted on late; XGen Spaces are foundational. |
| Thread | Discord | Thread | Concept kept, implementation redesigned from first principles. Discord Threads were an afterthought with ambiguous lifecycle. XGen Thread model is designed deliberately — see Thread section. |
| Node | Matrix | Homeserver / Identity Server / Media Server | Unified into one Node type. Matrix required multiple specialized server types; XGen uses one codebase, one install, capabilities declared via open enum. |
| Identity | — | — | New. Server-independent cryptographic keypair. No predecessor protocol solved this correctly. |
| Auth Module | — | — | New. Pluggable authentication slot. The protocol owns the trust assertion, not the credential format. |
| Trust Assertion | — | — | New. Standardized trust level claim returned by any Auth Module to the protocol core. |
| meta-atts | — | — | New. Universal namespaced key-value extension field carried by every primitive. |

---

### Table B — Cross-Platform Analogue Concepts

For readers coming from Discord or Matrix — the concept mapping across platforms. XGen terminology is in the leftmost column.

*Note: Where XGen uses the same name as Matrix — Space, Room — the concept is the same. The implementation is XGen's own.*

| Concept | XGen | Matrix | Discord | Slack |
|---|---|---|---|---|
| Top-level container | Space | Space | Server | Workspace |
| Communication unit | Room | Room | Channel | Channel |
| Focused sub-conversation | Thread | Thread | Thread | Thread |
| Infrastructure unit | Node | Homeserver | — | — |
| User account | Identity (keypair) | @user:server | User ID | User ID |
| Permission grouping | Role | Power level | Role | Role |
| Room grouping | meta-atts `section` | — | Category | Section |
| Extension metadata | meta-atts | — | — | — |

**On the Matrix debt:** XGen borrows Space, Room, Event, and Thread from Matrix's conceptual vocabulary deliberately and with full acknowledgment. Matrix pioneered the federated communication primitive model. XGen stands on that foundation — not on Matrix's codebase — and builds the layer Matrix never completed. Where names are shared, it is a sign of respect for that pioneering work, not a lack of imagination.

**On the absence of Category:** Discord's Category is a visual sidebar grouping with no protocol meaning. XGen handles this at the client level via the `section` key in a Room's `meta-atts` field. No Category primitive exists in the XGen protocol. This keeps the hierarchy clean and the protocol thin.

---

## Primitive Hierarchy

XGen has one clear, dependency-ordered hierarchy of primitives. Lower primitives must be defined before higher ones can be understood. The hierarchy is not a containment model alone — it is a dependency model.

```
DISCORD                  MATRIX                   XGEN
───────                  ──────                   ────
Server                   Space                    Space
└── Category             └── Space (nested)       └── Room
    └── Channel              └── Room                 └── Thread
        └── Thread               └── (messages)           └── Event
            └── Message
```

XGen's hierarchy is the flattest and cleanest of the three. No Category level. No nested Space recursion. One clear path from the top-level container down to the atom.

**Dependency order — how the primitives are defined in this chapter:**

1. **Event** — the atom. Everything else is composed of Events.
2. **Thread** — a scoped, bounded sequence of Events within a Room.
3. **Room** — a persistent, federated container of Events and Threads with shared state.
4. **Space** — a governed, cryptographically identified collection of Rooms.
5. **Node** — the infrastructure unit that hosts Spaces and participates in federation.
6. **Identity** — the server-independent keypair that travels across all primitives.
7. **Auth Module** — the pluggable slot that produces Trust Assertions for Identity.

Cross-cutting across all primitives: **meta-atts**, **Role model**, **federation behavior**.

---

## Hardware, Node, and Space — The Infrastructure Stack

Before defining the protocol primitives, it is worth being precise about the relationship between physical infrastructure and the protocol entities that run on it. This is a common source of confusion.

```
Machine (hardware / VPS / Raspberry Pi)
├── Node A  (XGen software instance)
│   ├── Space A1
│   └── Space A2
└── Node B  (XGen software instance, isolated from Node A)
    └── Space B1
```

**Machine** — the physical or virtual hardware. Invisible to the protocol. The protocol has no concept of machines, only Nodes.

**Node** — the XGen software instance running on a machine. The Node is the protocol's unit of infrastructure. It has its own cryptographic identity, its own capability advertisement, its own federation relationships. One Node runs on exactly one machine. One machine can run multiple Nodes.

**Space** — a protocol-level entity hosted by a Node. One Space lives on exactly one Node at any given moment. Spaces are portable — they can migrate between Nodes without losing history or cryptographic identity — but at any point in time, one Space has one home Node.

**The isolation boundary is the Node, not the machine.** Two Nodes on the same machine are as isolated from each other as two Nodes on opposite sides of the world. They do not know each other exists unless they federate explicitly, using the same XGen federation protocol they would use across the network. Physical co-location is an infrastructure detail. The protocol does not model it.

This has an important consequence: a hosting provider can run dozens of fully independent Node instances on one powerful machine. Each Node is its own protocol participant. The machine is irrelevant to the network.

---

## Cryptographic Signatures & Algorithm Agility

Every signed artifact in XGen — Events, Identity credentials, Node announcements — carries a cryptographic signature. This section explains what a signature physically is, how it is structured, and why the protocol is designed to support any cryptographic system, including those that do not yet exist.

This section is written for protocol readers who are not cryptographers. Precise algorithm specifications belong to Chapter 3.

---

### What a Signature Physically Looks Like

A signature is a fixed-length string of bytes, produced by a mathematical operation that combines the content being signed with the signer's private key. It is typically encoded as base64 for transport — a compact, text-safe representation of binary data.

A UUID, for comparison:
```
550e8400-e29b-41d4-a716-446655440000
```

An XGen signature field:
```
ed25519:AABBCCDD:MCowBQYDK2VdAyEA4qGBZOaA1xPCBFYJFCTx...
────────── ──────── ─────────────────────────────
↑           ↑        ↑
algorithm   key ID   signature bytes (base64)
```

Three parts, separated by colons:

- **Algorithm** — declares which cryptographic algorithm produced this signature. Today typically `ed25519`. Tomorrow it may be `ml-dsa-65` (post-quantum) or something not yet standardized.
- **Key ID** — a short identifier for the specific keypair that signed this artifact. Allows the verifier to locate the correct public key when a user or node holds multiple keys.
- **Signature bytes** — the actual signature, base64-encoded. For Ed25519 this is 64 bytes — approximately 88 characters of base64. Compact, fast to verify, well-understood.

---

### How Signing Works — Plain Language

Think of a signature as a wax seal on a letter. The seal proves two things simultaneously: who sent it, and that nobody tampered with the contents after it was sealed.

In XGen:

1. A user holds a **keypair** — a private key that never leaves their device, and a public key that is shared with the network.
2. When a user produces an Event, their client takes the entire Event content and runs it through a signing algorithm using the **private key**.
3. The result is a **signature** — attached to the Event before transmission.
4. Any Node or client receiving the Event can verify it using the sender's **public key**. If the signature is valid, the content is authentic and untampered. If anything in the Event was changed after signing, verification fails.

The private key never travels. Only the signature travels. The public key is available to anyone on the network. This is the foundation of public-key cryptography — the same principle used in HTTPS, SSH, and Signal.

---

### Algorithm Agility — Designed for Cryptographic Systems That Do Not Yet Exist

XGen does not hardcode a cryptographic algorithm. The algorithm is **declared as part of the signature field itself** — a prefix that tells the verifier which algorithm to use.

This is called **algorithm agility**, and it is a deliberate, non-negotiable design principle.

The reason is simple: cryptographic standards change. SHA-1 was considered secure until it wasn't. RSA key sizes that were safe in 2010 are marginal today. Post-quantum computing will require an entirely new family of signature algorithms — NIST finalized its first post-quantum standards in 2024. Any protocol that hardcodes `ed25519` will need a breaking revision when the next transition arrives.

XGen will not need that revision. The algorithm is a declared field. Swapping from `ed25519` to `ml-dsa-65` — or to whatever comes after — is a configuration and module update, not a protocol change.

This connects directly to the **Temporal Resilience** pillar from Chapter 1:

> *"XGen is not optimized for today's best answer. It is optimized for the ability to adopt tomorrow's better answer without breaking what was built yesterday."*

Old Nodes that encounter an unknown algorithm prefix handle it gracefully — they flag the artifact as unverifiable by their current implementation but do not crash and do not reject the Event outright. The network does not fork. It grows.

**The same principle applies to Event IDs.** An Event ID in XGen is not a random UUID — it is a **hash of the Event content**, making it tamper-evident by construction. The hash algorithm is also declared as a prefix:

```
sha256:a1b2c3d4e5f6...
```

Today `sha256`. Tomorrow `sha3-256`, `blake3`, or whatever the community adopts. Same agility principle. Same graceful upgrade path.

---

### Summary

| Property | Value |
|---|---|
| Signature format | `algorithm:keyid:base64bytes` |
| Current default algorithm | Ed25519 (64-byte signature, ~88 base64 chars) |
| Algorithm declared in field | Yes — not hardcoded in protocol |
| Algorithm upgradeable | Yes — without protocol revision |
| Event ID format | `hashalgorithm:hexbytes` |
| Event ID is hash of content | Yes — tamper-evident by construction |
| Unknown algorithm handling | Graceful — flag as unverifiable, do not reject |
| Cryptographic detail | Chapter 3 — Specification |

---

## Event Model

The Event is the atom of XGen Protocol. Everything that happens in the protocol — a message sent, a user joining a Room, a permission changed, a file uploaded, a call started — is an Event. The protocol treats all Events the same way at the transport and federation level. It does not care whether an Event carries a chat message or a role assignment. It stores it, signs it, propagates it, and references it.

This uniformity is what makes federation consistent, the audit trail complete, and the protocol extensible without special cases.

---

### Core Properties of Every Event

An Event is defined by five immutable properties. These are not optional. Every Event in the protocol, regardless of type, satisfies all five.

**Immutable.** Once written, an Event is never changed. An edited message is a new Event that references the original. A deleted message is a new Event that signals deletion. The original Event always remains in the log. This is not a limitation — it is a guarantee. In a federated network where an Event has already propagated to dozens of Nodes before an edit arrives, pretending the original never existed creates inconsistent state. XGen does not pretend. The log is append-only. What clients *display* is a separate concern from what the protocol *stores*.

**Signed.** Every Event carries the cryptographic signature of the Identity that produced it. Authorship is always verifiable. Tampering is always detectable. See the Cryptographic Signatures & Algorithm Agility section for the full signature anatomy.

**Typed.** Every Event has a `type` field that declares what kind of Event it is. The type is an open enum — new types can be added without breaking old Nodes, which simply ignore types they do not understand. The type determines how clients render the Event and how the protocol interprets the `content` field.

**Referenceable.** Every Event has a unique, stable ID — a hash of its content, not a random UUID. Events can reference other Events: a reply references its parent, an edit references the original, a reaction references its target, a deletion references what it deletes. These references form the event graph that gives the Room its history and structure.

**Ordered.** Events carry both a local timestamp and references to previous Events (`prev_events`). In a federated system, clocks cannot be trusted — different Nodes may have slightly different system times. XGen uses the `prev_events` graph to establish a partial causal ordering that does not depend on clock accuracy. The full ordering algorithm is a Chapter 3 specification problem. The principle is established here: Event ordering is graph-based, not clock-based.

---

### Event Anatomy

Every Event has the same outer structure, regardless of type. The `content` field is the only part that varies — its schema is determined by the `type`.

```
event {
  id:           "sha256:a1b2c3d4e5f6..."         ← hash of all fields below
  type:         "message.text"                   ← open enum
  room:         "xgen://room/xyz..."              ← parent Room reference
  sender:       "xgen://identity/pubkey..."       ← sender Identity reference
  timestamp:    1714000000000                     ← unix ms — indicative, not authoritative
  prev_events:  ["sha256:aabb...", "sha256:ccdd..."] ← causal ordering references
  content:      { ... }                           ← payload, schema varies by type
  signature:    "ed25519:KEYID:BASE64..."         ← signs all fields above
  meta-atts:    { ... }                           ← opaque namespaced extension map
}
```

**Field notes:**

- `id` — derived from the content, not assigned. Two Events with identical content produce the same ID. This makes deduplication trivial and forgery impossible.
- `type` — dot-namespaced string. Core protocol types use the `xgen.` namespace. Application and module types use their own namespaces. Unknown types are stored and forwarded, never silently dropped.
- `room` — every Event belongs to exactly one Room. Events do not exist outside a Room context.
- `sender` — references an Identity, not a Node. The sender's identity is portable and Node-independent.
- `timestamp` — useful for display, unreliable for ordering. Never used as the sole ordering mechanism.
- `prev_events` — one or more IDs of Events this Event causally follows. Forms the DAG (directed acyclic graph) that gives the Room its history.
- `content` — the payload. Schema is defined per event type. Empty for some system events.
- `signature` — signs all preceding fields. Covers id through content. meta-atts signing policy is a Chapter 3 decision.
- `meta-atts` — extension map. Namespaced. Opaque to the protocol core. May or may not be federated depending on namespace convention.

---

### Event Type Taxonomy

Event types are organised into four families. The type field uses dot-namespaced strings — the family is the prefix.

#### Content Events
Things users produce. The visible activity of the protocol.

| Type | Description |
|---|---|
| `message.text` | Plain text message |
| `message.rich` | Formatted text (markdown, mentions, code blocks) |
| `message.edit` | Edit — references original Event, carries new content |
| `message.delete` | Deletion signal — references original Event |
| `message.reaction` | Emoji or reaction — references target Event |
| `message.reply` | Reply — references parent Event, carries new content |
| `file.upload` | File or media attachment |
| `call.start` | Voice or video call initiated |
| `call.join` | Participant joined a call |
| `call.leave` | Participant left a call |
| `call.end` | Call terminated |
| `stream.start` | Live stream initiated |
| `stream.end` | Live stream terminated |
| `thread.create` | New Thread created within a Room |
| `poll.create` | Poll created |
| `poll.vote` | Vote cast on a poll — references poll Event |

#### State Events
Things that change the persistent state of a Room or Space. State Events are special — they are not just appended to the log, they update the current state of the Room that all Nodes must agree on.

| Type | Description |
|---|---|
| `room.member.join` | User joined the Room |
| `room.member.leave` | User left the Room |
| `room.member.kick` | User removed from Room by moderator |
| `room.member.ban` | User banned from Room |
| `room.name.change` | Room name updated |
| `room.topic.change` | Room topic updated |
| `room.permission.change` | Room permission settings updated |
| `space.member.join` | User joined the Space |
| `space.member.leave` | User left the Space |
| `space.role.assign` | Role assigned to a user |
| `space.role.revoke` | Role revoked from a user |
| `space.settings.change` | Space-level settings updated |
| `room.pin.add` | Event pinned to the Room Board |
| `room.pin.remove` | Event unpinned from the Room Board |

#### System Events
Protocol-level housekeeping. Produced by Nodes, not users.

| Type | Description |
|---|---|
| `node.federation.join` | Node joined the federation for this Room |
| `node.federation.leave` | Node left the federation for this Room |
| `identity.key.rotate` | User's keypair updated — new public key published |
| `identity.auth.upgrade` | User's auth tier upgraded |
| `space.node.migrate` | Space migrated to a new home Node |
| `room.created` | Room created within a Space |
| `room.archived` | Room archived |

#### Bridge Events
Events originating from outside the XGen network, arriving via a bridge module. Every bridge Event is clearly marked as external — the trust boundary is explicit and visible.

| Type | Description |
|---|---|
| `bridge.message.in` | Message arriving from an external network (Discord, email, etc.) |
| `bridge.message.out` | Message forwarded out to an external network |
| `bridge.member.in` | External user represented in an XGen Room |

> *Bridge Events carry a declared origin and trust tier. Clients must visually distinguish bridged content from verified XGen content. The trust boundary is never hidden.*

---

### What Events Are Not

A few boundaries worth stating explicitly:

- **Events are not editable.** The log is append-only. Edits and deletions are new Events.
- **Events do not exist outside a Room.** There is no free-floating Event in the protocol.
- **Events are not messages.** A message is one type of Event. Most protocol activity is Events that are not messages.
- **Event timestamps are not authoritative for ordering.** The `prev_events` graph is.
- **Unknown Event types are not dropped.** They are stored, forwarded, and ignored by clients that do not understand them. The protocol is forward-compatible by design.

---

## Direct Messages

Direct Messages are private, person-to-person or small-group conversations. They are not a separate primitive in XGen. They are a minimal Space — a Space with no public presence, no governance overhead, and a single Room.

> *A DM is a Space stripped to its minimum viable form. The model is consistent. No special cases in the protocol.*

---

### Direct Message Model

A DM Space has the following constraints that distinguish it from a regular Space:

```
dm_space {
  type:          "dm"                          ← declared DM type
  visibility:    "invite-only"                 ← always invite-only, non-negotiable
  members:       [ identity_A, identity_B ]    ← exactly 2 for a DM, 3-N for group DM
  rooms:         [ one_room ]                  ← exactly one Room by default
  roles:         [ ]                           ← no roles — all members are equal
  invite_code:   null                          ← no shareable invite link
  discoverable:  false                         ← never listed in any directory
}
```

**Key properties:**

- **Always invite-only.** A DM Space is never public, never listed, never discoverable. It exists only between its members.
- **No roles.** All members are equal. There is no Owner, no Moderator. Either member can leave. When a member leaves a two-person DM, the Space is effectively ended — though history remains accessible to the remaining member.
- **One Room by default.** A DM Space contains one text Room. Additional Rooms can be created by any member — useful for group DMs that evolve into working groups.
- **Same Event model.** All messages in a DM are Events, signed, immutable, referenceable. The same protocol primitives apply. There is no special DM message type.
- **Same federation model.** If two users are on different Nodes, their DM Space federates between those Nodes exactly like any other Space. Private does not mean local.

---

### DM Initiation

A user initiates a DM by creating a minimal Space with the target user as the only other member. The protocol does not require the target to be online — the invitation is delivered when the target's Node next syncs.

The target may accept or decline:

- **Accept** — joins the DM Space. Conversation begins.
- **Decline** — the DM Space is effectively abandoned. The initiator sees a declined status. No messages were exchanged.
- **No response** — the invitation remains pending. The initiator can withdraw it.

Privacy controls at the Identity level determine who can initiate a DM:

| Privacy setting | Who can DM this user |
|---|---|
| `open` | Anyone with a valid Identity |
| `contacts_only` | Only users the recipient has previously interacted with |
| `spaces_only` | Only members of a shared Space |
| `closed` | Nobody — DMs disabled |

This setting lives in the Identity-scoped settings and replicates with the Identity record.

---

### Group DMs

A group DM is a DM Space with more than two members. The same model applies — invite-only, no roles, no public presence. Any member can add another member, subject to the new member's own DM privacy settings.

A group DM that grows in purpose — becomes a working group, a project team, a persistent community — can be promoted to a full Space by any member. Promotion is a State Event that lifts the DM constraints and enables full Space features: roles, governance, multiple Rooms, discoverability if desired.

> *DMs that become communities should become Spaces. The protocol makes this transition natural and non-destructive — history is preserved, members remain, the chat continues.*

---

## Thread Model

The Thread is the most misunderstood primitive in community communication. Discord has Threads. Matrix has Threads. Neither designed them deliberately. Both bolted them on in response to user demand, without first answering the fundamental question:

> *What is a Thread actually for?*

XGen answers that question before writing a single line of specification.

---

### What a Thread Is For

A Room is a continuous, shared flow of Events. It is excellent for live conversation, ongoing discussion, and community presence. But it has a structural weakness: parallel conversations collapse into each other. When multiple topics are active simultaneously in a busy Room, context is lost, threads of thought are interrupted, and the signal-to-noise ratio degrades.

A Thread is a **scoped, bounded conversation within a Room** — a focused space for a specific topic, question, or decision, with a clear beginning and a natural end. It does not replace the Room. It extends it.

The Thread answers three specific needs:

**Focus.** A Thread gives a specific topic its own space, separate from the main Room flow, without requiring a new Room to be created. The topic lives where it belongs — inside the Room it relates to — without polluting the main conversation.

**Persistence.** A Thread preserves a specific conversation intact. When the Room moves on, the Thread remains. It becomes a searchable, referenceable record of that specific discussion. This is the Kyberia principle applied at the Room level: community memory that does not evaporate.

**Resolution.** Unlike a Room, a Thread has a lifecycle. It can be open, resolved, or archived. It was started for a reason, and it can be closed when that reason is satisfied. This is what Discord Threads never got right — they existed but had no clear answer to: *when is a Thread done?*

---

### What a Thread Is Not

Before defining the Thread, it is worth being explicit about what it is not — because both Discord and Matrix confused these boundaries:

- **A Thread is not a Room.** A Room is permanent, always-open infrastructure. A Thread is bounded and purposeful. Creating a Thread does not create a new Room.
- **A Thread is not a reply chain.** A reply references a parent Event inline in the Room flow. A Thread is a separate scoped space. The distinction is: a reply is *in* the conversation. A Thread is *beside* it.
- **A Thread is not a sub-Room.** Threads do not nest. A Thread inside a Thread is not a design goal — it is a complexity trap. XGen Threads are flat: one level deep, always anchored to a Room.
- **A Thread is not permanent.** Rooms are permanent. Threads have a lifecycle. This is a deliberate and important distinction.

---

### Thread Anatomy

A Thread is anchored to a Room by a `thread.create` Event. That Event is the Thread's origin — it carries the Thread's topic, its creator, and its initial content. All subsequent Events in the Thread reference this origin.

```
thread {
  id:           "xgen://thread/sha256:..."    ← permanent, hash-derived
  room:         "xgen://room/sha256:..."      ← parent Room reference
  created_by:   "xgen://identity/..."         ← creator Identity reference
  created_at:   1714000000000                ← unix ms, immutable
  origin_event: "sha256:aabb..."             ← the thread.create Event ID
  title:        "Should we add OAuth?"        ← optional short label
  status:       "open"                        ← open | resolved | archived
  auth_tier_min: 1                            ← minimum tier to participate
  meta-atts:    { ... }                       ← opaque namespaced extension map
}
```

**Field notes:**

- `id` — permanent and hash-derived. Stable for the lifetime of the Thread.
- `room` — every Thread belongs to exactly one Room. A Thread cannot exist outside a Room.
- `origin_event` — the `thread.create` Event that started this Thread. The anchor point in the Room's Event log.
- `title` — optional but strongly encouraged. A Thread without a title is a Thread without a stated purpose.
- `status` — the Thread lifecycle field. The key field that Discord never had. `open` means active. `resolved` means the purpose was satisfied. `archived` means closed without resolution or after a period of inactivity.
- `auth_tier_min` — a Thread can require a higher auth tier than its parent Room. A Tier 1 public Room can contain a Tier 2 Thread for professional discussion. The Room is the outer boundary; the Thread can narrow it further but never widen it beyond the Room's own minimum.

---

### Thread Lifecycle

```
CREATED → OPEN → RESOLVED
               → ARCHIVED
```

**Created.** A `thread.create` Event is written to the Room. The Thread exists. It is immediately visible to all Room members who meet the Thread's auth tier minimum.

**Open.** The active state. Members can post Events into the Thread. The Thread appears in the Room's Thread list. Notifications are active.

**Resolved.** A member with sufficient permissions marks the Thread as resolved. A `thread.resolved` State Event is written. The Thread is closed to new Events. It remains fully readable and searchable — its content is community memory. Resolution is the clean end state: *the question was answered, the decision was made, the task is done.*

**Archived.** A Thread that becomes inactive without resolution, or is explicitly closed by a moderator, is archived. Same as Resolved in terms of access — fully readable, no new Events. Distinct in meaning: the Thread ended without a definitive conclusion.

> *A Thread is never deleted. Its history is part of the Room's history. Resolved and Archived Threads are the forum memory that makes `room.forum` valuable and that Kyberia demonstrated works at community scale.*

---

### Threads in Different Room Types

Threads behave differently depending on the Room type they live in. This is by design — the Thread is a primitive that adapts to its context.

| Room Type | Thread Behavior |
|---|---|
| `room.text` | Threads branch off specific messages. Optional, user-initiated. The Room remains the primary flow. |
| `room.forum` | Threads ARE the primary flow. Every top-level post is a Thread starter. The Room has no flat message flow — only Threads. This is the full Kyberia/forum model. |
| `room.announcements` | Threads allow replies to announcements without polluting a read-only main flow. Only designated roles can start Threads on an announcement. |
| `room.voice` | Threads not applicable. Voice Rooms are real-time only. |
| `room.video` | Threads not applicable. Same as voice. |
| `room.stage` | A companion text Thread may be auto-created when a Stage begins — for audience questions and reactions. Closes when the Stage ends. |

---

### The Forum Room and the Thread — The Full Model

The `room.forum` type deserves special attention here because it inverts the relationship between Room and Thread.

In a `room.text`, the Room is primary and Threads are optional branches. In a `room.forum`, **Threads are the Room**. There is no flat message flow. Every contribution is either a Thread starter (a new topic) or a reply inside an existing Thread.

This produces the classic forum experience:

```
room.forum: "Protocol Design"
├── Thread: "Should identity be keypair or DID?"     [open, 12 replies]
├── Thread: "State resolution algorithm options"      [open, 8 replies]
├── Thread: "Node capability trust levels"            [resolved, 23 replies]
└── Thread: "Encryption layer — MLS vs Megolm"       [archived, 5 replies]
```

This is what Discord's forum channels attempted and did not fully deliver. XGen's `room.forum` makes it a first-class protocol model — not a channel variant, but a distinct Room type with Threads as its native unit of communication.

---

### Notification Model

One of Discord's Thread failures was inconsistent notification behavior. XGen states the notification principle clearly as an architectural constraint, leaving the implementation to clients:

- **Room notifications** cover top-level Events in the Room flow and new Thread creation.
- **Thread notifications** are scoped to the Thread. Joining a Thread opts you into Thread-level notifications. Leaving does not delete your contributions.
- **`room.forum` notifications** default to notifying on new Thread creation only — not on every reply in every Thread. The member subscribes to specific Threads they care about.
- **Resolved and Archived Threads** generate no notifications. They are read-only record.

> *The notification model is a client concern. The protocol provides the status field and the event types. Clients implement notification logic. This keeps the protocol thin.*

---

## Room Model

The Room is the core communication unit of XGen Protocol. It is a persistent, federated container of Events with shared state. Users do not communicate with each other directly — they communicate inside Rooms. Every message, every call, every file transfer, every reaction happens inside a Room.

A Room belongs to a Space. It has a type, a name, a permission model, and a history that is never lost. It is a first-class protocol primitive — defined in the spec, not improvised by clients.

---

### What a Room Is

A Room is defined by four things:

**An append-only Event log.** Every thing that happens in a Room is an Event appended to this log. The log is the Room's history. It is never rewritten. Edits and deletions are new Events. The log is replicated across every Node participating in the Room's federation.

**A current state.** The Room has a set of properties that describe its present condition — its name, topic, member list, permissions, and settings. This state is derived from State Events in the log. When a State Event changes the Room name, the current state is updated. All Nodes must agree on the current state at any point in time. How Nodes reach agreement when they disagree is the state resolution problem — addressed in the Federation Model section.

**A permission model.** Every Room declares who can do what inside it — who can send messages, who can invite members, who can change settings, who can moderate. Permissions are inherited from the Space's Role model and can be overridden at the Room level. The minimum auth tier required to join the Room is also declared here.

**A cryptographic identity.** A Room has its own ID — a stable, unique reference that persists regardless of which Node currently hosts it. The Room ID is permanent. Node addresses are not.

---

### Room Types

A Room has a declared type that determines its primary communication mode. The type is set at creation and does not change. The type enum is open — new room types can be introduced without breaking existing Nodes.

| Type | Description |
|---|---|
| `room.text` | Persistent text chat. The default Room type. Messages, files, reactions, threads. |
| `room.voice` | Voice channel. Always-open voice space — join and leave freely, like Discord voice channels. Not a scheduled call. |
| `room.video` | Video channel. Same model as voice, with video. |
| `room.forum` | Forum-style Room. Posts are first-class objects and Thread starters, not a flat chat flow. Suited to Q&A, structured discussion, announcements with replies. Community memory that persists and accumulates. Inspired by the forum-as-community-memory tradition of early community platforms — see Chapter 1, Kyberia acknowledgment. |
| `room.announcements` | Read-only for most members. Only designated roles can post. |
| `room.stage` | One-to-many broadcast. Speakers are designated. Audience can react but not speak. |

> *Voice and video Rooms are always-open spaces, not scheduled calls. A user enters and leaves at will. This follows Discord's correct architectural decision — voice as infrastructure, not as an event.*

---

### Room Anatomy

A Room's current state — the snapshot that all Nodes agree on — contains the following fields:

```
room {
  id:           "xgen://room/sha256:..."     ← permanent, hash-derived
  space:        "xgen://space/sha256:..."    ← parent Space reference
  type:         "room.text"                  ← open enum
  name:         "general"                    ← display name
  topic:        "Welcome to XGen"            ← optional description
  created_at:   1714000000000               ← unix ms, immutable
  created_by:   "xgen://identity/..."        ← creator Identity reference
  auth_tier_min: 1                           ← minimum tier to join
  permissions:  { ... }                      ← role-based permission overrides
  members:      [ ... ]                      ← current member list with roles
  meta-atts:    {
    "xgen.section": "General"               ← client-side grouping label
    ...                                      ← any further namespaced entries
  }
}
```

**Field notes:**

- `id` — permanent and hash-derived. Stable across Node migrations. Never reassigned.
- `space` — every Room belongs to exactly one Space. A Room cannot exist outside a Space.
- `type` — set at creation, immutable. Determines the Room's communication mode and client rendering.
- `auth_tier_min` — the minimum Identity auth tier required to join this Room. Enforced at the protocol level, not the application level. A Tier 1 user attempting to join a Tier 3 Room receives a protocol-level rejection with a clear upgrade path.
- `permissions` — role-based overrides. The Space's Role model is the baseline. This field declares any Room-level deviations from that baseline.
- `meta-atts.xgen.section` — the client-side grouping label. This is how Rooms are visually grouped in the sidebar without a Category primitive. The protocol stores it, the client renders it. No Category level needed.

---

### Room Lifecycle

```
CREATED → ACTIVE → ARCHIVED
                → MIGRATED (to another Node, remains ACTIVE)
```

**Created.** A Room is created inside a Space by a member with sufficient permissions. A `room.created` System Event is written. The Room is immediately federated to all Nodes participating in the Space.

**Active.** The normal operating state. Events are appended, state is maintained, federation is live.

**Archived.** A Room can be archived — no new Events can be written, but the history remains fully accessible. Archival is a State Event. It is reversible — a Room can be unarchived by a member with sufficient permissions.

**Migrated.** When a Space moves to a new home Node, its Rooms move with it. The Room ID does not change. The history does not change. Federation relationships are re-established at the new Node. From the protocol's perspective, the Room is the same Room — only its infrastructure address changed.

> *A Room is never deleted. Its history is permanent. What clients display is a separate concern from what the protocol stores. This is consistent with the Event immutability principle.*

---

### The Board

Every Room has a **Board** — a curated, persistent display surface where selected Events are pinned and remain permanently visible to all Room members, regardless of how far the conversation has scrolled.

The Board is not a separate primitive. It is a field in the Room's current state — an ordered list of pinned Event references, agreed upon by all Nodes via the same state resolution mechanism as the rest of the Room state.

**Any Event type can be pinned.** The Board is not limited to messages:

- A `message.text` or `message.rich` — rules, announcements, onboarding information
- A `file.upload` — a key document everyone needs access to
- A `poll.create` — an ongoing vote that must stay visible
- A `thread.create` — a critical discussion that should not be buried
- A `stream.start` — a recording reference worth preserving prominently

**Board field in Room state:**

```
board: [
  {
    event_id:  "sha256:aabb..."          ← reference to the pinned Event
    pinned_by: "xgen://identity/..."     ← Identity that pinned it
    pinned_at: 1714000000000            ← unix ms
    label:     "Community Rules"         ← optional display label
  },
  ...
]
```

**Key design decisions:**

- `pinned` is **not a field on the Event itself.** Events are immutable — pinning is a Room state decision, not an Event property. The Board holds the pin. The Event does not know it is pinned. This is consistent with the Event immutability principle.
- Pinning is a **State Event** — `room.pin.add` adds an entry to the Board, `room.pin.remove` removes one. Both are federated and agreed upon by all Nodes.
- Pinning requires **`room.pin` permission** — a moderation-level action controlled by the Room's Role model. Reading the Board requires no special permission.
- The Board is **ordered** — the display order is explicit and controlled by moderators, not derived from Event timestamps.
- The `label` field is optional — when present, the client displays it as the pin's title instead of the Event's own content preview. Useful when pinning non-message Events that need context.
- **Client rendering is a client concern.** The protocol provides the Board list. Whether it renders as a sidebar panel, a top banner, a dedicated tab, or a pinned message strip is an application decision. The protocol stays thin.

---

### Room State and Federation

This is the hardest problem in the Room model — and it is worth stating clearly before the Federation Model section addresses it in full.

A Room's current state — member list, permissions, name, settings — must be consistent across every Node participating in the Room's federation. But in a distributed network, two Nodes can receive State Events in different orders. When they do, they may temporarily disagree about what the current state is.

The **state resolution algorithm** determines the canonical current state when disagreement exists. Matrix's state resolution algorithm is computationally expensive and does not scale to large rooms — this is one of the documented failures that XGen explicitly improves upon.

XGen's state resolution approach is governed by one design constraint established here:

> *State resolution must be deterministic, convergent, and scale-aware. Given the same set of Events, any Node must arrive at the same current state. The algorithm must remain tractable as Room membership and federation breadth grow.*

The specific algorithm is a Chapter 3 specification problem. The architectural commitment is made here.

---

### What a Room Is Not

- **A Room is not a Node.** The Room exists at the protocol level. The Node is infrastructure. A Room can migrate between Nodes.
- **A Room is not a direct message channel.** DMs are a Space-level concept — a minimal Space with two members and one Room. Not a separate primitive.
- **A Room is not deleted when its Space migrates.** Migration preserves all Rooms, all history, all identities.
- **A Room's history is not owned by the Node that hosts it.** The history belongs to the Room. Any Node with federation access holds a replica.

---

## Space Model

The Space is the top-level container in XGen Protocol. It is the entity users join, identify with, and belong to. A Space contains Rooms. It has its own cryptographic identity, its own governance, its own permission model, and its own portable history. It is the Discord Server done correctly — a first-class protocol primitive from day one, not a feature added late.

Everything below the Space — Rooms, Threads, Events — exists in service of the Space. The Space is what users think of as "the community."

---

### What a Space Is

A Space is defined by five things:

**A cryptographic identity.** A Space has its own keypair — not borrowed from a Node, not derived from a user. The Space exists as an independent entity on the network. Its ID is permanent. Node addresses are not. When a Space migrates to a new Node, it takes its identity with it.

**An ordered collection of Rooms.** A Space contains one or more Rooms. Rooms are created inside the Space, carry the Space's permission baseline, and migrate with the Space. The Space is the governance layer above the Rooms.

**A cascading permission model.** The Space defines Roles. Roles cascade down to Rooms. A Room inherits the Space's Role definitions and may override specific permissions at the Room level — but cannot introduce Roles that do not exist in the Space. The Space is the permission root.

**An ownership model.** A Space is owned by its members — specifically by members holding the Owner role. No Node owns a Space. No company owns a Space. The Space owns itself, cryptographically. Ownership is transferable. The Space is never hostage to infrastructure.

**Portability.** A Space can migrate between Nodes without losing its identity, history, member list, or permission model. Migration is a protocol-level operation, not a manual export/import. The Space arrives at the new Node intact.

---

### Space Anatomy

```
space {
  id:             "xgen://space/sha256:..."      ← permanent, hash-derived
  name:           "Retro Gamers"                 ← display name
  description:    "For those who remember..."    ← optional
  created_at:     1714000000000                  ← unix ms, immutable
  created_by:     "xgen://identity/..."          ← founder Identity reference
  home_node:      "xgen://node/..."              ← current home Node
  auth_tier_min:  1                              ← minimum tier to join
  visibility:     "public"                       ← public | private | invite-only
  roles:          [ ... ]                        ← Role definitions
  members:        [ ... ]                        ← member list with assigned roles
  rooms:          [ ... ]                        ← Room references
  board:          [ ... ]                        ← Space-level pinned Events
  invite_code:    "xgen.gg/s/retrogamers"       ← optional shareable join link
  meta-atts:      { ... }                        ← opaque namespaced extension map
}
```

**Field notes:**

- `id` — permanent and hash-derived. The Space's identity on the network. Unchanged by migration, rename, or ownership transfer.
- `home_node` — the Node currently hosting this Space. Changes on migration. The Space ID does not change.
- `auth_tier_min` — the minimum Identity auth tier required to join this Space at all. Individual Rooms within the Space may require higher tiers. No Room may require a lower tier than the Space itself.
- `visibility` — controls discoverability. `public` Spaces are listed in node directories. `private` Spaces are not listed but joinable via invite. `invite-only` Spaces require explicit member approval.
- `roles` — the Role definitions for this Space. The permission root. All Rooms inherit from here.
- `board` — the Space-level Board. Same mechanism as Room Board — an ordered list of pinned Event references. Visible to all Space members. Typically used for Space-wide announcements, rules, and key documents.
- `invite_code` — a short, shareable URL that joins a user to the Space instantly. The invite link as a viral growth mechanism, borrowed from Discord's correct design. Optional — not all Spaces want public discovery.

---

### The Role Model

Roles are the permission primitive of XGen. Every permission decision in the protocol — who can post, who can moderate, who can invite, who can change settings — is expressed through Roles.

Roles are defined at the Space level and cascade down to Rooms. A Room may override specific permissions for specific Roles, but it cannot create new Roles or grant permissions the Space has not defined.

**A Role has three components:**

```
role {
  id:           "xgen://role/sha256:..."       ← unique ID within the Space
  name:         "Moderator"                    ← display name
  color:        "#E74C3C"                      ← optional display color
  permissions:  [ ... ]                        ← list of granted permissions
  position:     3                              ← hierarchy position (higher = more authority)
  meta-atts:    { ... }
}
```

**Permission hierarchy — how cascading works:**

```
Space Role: Moderator
  permissions: [send_messages, delete_messages, kick_members]
    └── Room override: #announcements
          permissions: [send_messages: false]   ← Moderators cannot post in announcements
            └── Room override: #mod-only
                  permissions: [send_messages: true]  ← Moderators can post here
```

Room-level overrides are additive or restrictive — they narrow or extend what a Role can do in a specific Room, but the Role itself is always defined at the Space level.

**Built-in Roles — every Space has these by default:**

| Role | Description |
|---|---|
| `Owner` | Full control. Can transfer ownership. Cannot be removed by other roles. |
| `Admin` | Full control except ownership transfer. |
| `Moderator` | Content moderation, member management. Cannot change Space settings. |
| `Member` | Standard participation. Default role on joining. |
| `Guest` | Read-only or limited participation. Optional — used for trial access. |

Custom Roles sit between these built-ins in the hierarchy. A Space can define as many custom Roles as needed.

**Auth tier and Role are independent.** A user's auth tier is a protocol-level trust claim produced by their Auth Module. A Role is a Space-level governance assignment. A Tier 2 user may hold a Member role. A Tier 1 user cannot hold a role in a Tier 2 Space at all — they cannot join. But within the Space, role assignment is the Space's own governance, not the protocol's.

---

### Space Lifecycle

```
CREATED → ACTIVE → ARCHIVED
                 → MIGRATED (to another Node, remains ACTIVE)
```

**Created.** A Space is created by a user with a valid Identity at or above the Space's own `auth_tier_min`. A `space.created` System Event is written. The Space is registered with its home Node. The founder is automatically assigned the Owner role.

**Active.** The normal operating state. Members join, Rooms are created, Events flow, federation is live.

**Archived.** A Space can be archived by the Owner. No new members can join, no new Events can be written in any Room, but all history remains fully accessible. Archival is reversible.

**Migrated.** A Space can be moved to a new home Node by a member with sufficient permissions. The protocol executes migration as a sequence of signed Events — the Space's cryptographic identity, its full Event history, its member list, its Rooms and their histories, all transfer intact. Federation relationships are re-established at the new Node. The Space ID does not change. From the network's perspective, the same Space now lives at a different Node.

> *A Space is never deleted. Its history is permanent. Even a migrated or archived Space retains its full Event log. This is the ownership model working as designed — the Space belongs to its members, and its history cannot be destroyed by any Node operator.*

---

### Space Federation

A Space federates with the network through its home Node. When a member on a remote Node joins a Space, that remote Node establishes a federation relationship with the Space's home Node. Events flow between them. The Space's current state is replicated to all federated Nodes.

Key federation properties:

- **Members can be on any Node.** A Space hosted on Node A can have members whose Identity is anchored to Nodes B, C, and D. Federation is what makes this work transparently.
- **The Space's home Node does not own the Space.** It hosts it. The distinction is architectural and legal. The Node operator cannot delete the Space, cannot revoke memberships, cannot alter the Event log.
- **Room state is replicated to all participating Nodes.** Every Node with a federated member holds a replica of the Rooms that member participates in. No single Node is the single point of failure for a Space's history.

---

### Space Discoverability

How users find Spaces is a protocol-level concern, not purely a client concern, because it involves Node-level directory services.

| Visibility | Discovery mechanism |
|---|---|
| `public` | Listed in Node directory. Searchable. Joinable via invite link or direct search. |
| `private` | Not listed. Joinable only via invite link. Existence is not advertised. |
| `invite-only` | Not listed. Invite link exists but joining requires explicit approval by Owner or Admin. |

The `invite_code` is a short URL that resolves to the Space's full ID and home Node address. A client receiving an invite code can locate the Space, verify its identity cryptographically, and initiate the join sequence. The code can be revoked and regenerated by an Owner or Admin at any time — a `space.invite.regenerate` State Event is written.

---

### What a Space Is Not

- **A Space is not a Node.** The Space is a protocol entity. The Node is infrastructure. One Node can host many Spaces. One Space can migrate between Nodes.
- **A Space is not owned by its Node operator.** The Node hosts the Space. The members own the Space. These are different relationships.
- **A Space is not a Room.** Rooms are communication units inside a Space. The Space is the governance and identity layer above them.
- **A Space cannot be taken hostage.** Its cryptographic identity, its history, and its member relationships exist at the protocol level. No platform, no company, no Node operator can revoke them. This is the structural answer to Discord's fundamental failure.

---

## Node Model

### Node and Space — The Essential Distinction

A **Space** is a community. It has members, Rooms, history, governance, and a cryptographic identity. It is what users belong to. It exists at the protocol level and is owned by its members.

A **Node** is a server. It is the software and infrastructure that hosts Spaces and connects them to the network. It is what administrators operate. It exists at the infrastructure level and is owned by whoever runs it.

**The concrete / abstract boundary:**

| Layer | What it is | Tied to hardware? | Owned by |
|---|---|---|---|
| Machine | Physical or virtual hardware | Yes — it *is* the hardware | Operator |
| Node | XGen software running on a machine | Yes — runs on exactly one machine | Operator |
| Space | Community entity hosted by a Node | No — portable, migrates freely | Its members |
| Room | Communication unit inside a Space | No — migrates with the Space | Its Space |
| Event | Immutable record inside a Room | No — replicated across Nodes | Permanent record |

Machine and Node are concrete — they are physical reality and software running on it. Everything above the Node is abstract — it exists at the protocol level and is independent of any specific hardware or operator.

The relationship is one of hosting, not ownership. A Node hosts a Space the way a landlord rents out a flat — except in XGen, the tenant owns the furniture, holds the key, and can move out at any time without losing anything. The Node provides the address. The Space provides the identity.

> *If the Node disappears, the Space migrates. If the Space migrates, the Node remains. They are independent entities that happen to be co-located at a point in time.*

---

### Three Deliverables — Protocol, Node, Client

XGen is not one piece of software. It is three distinct deliverables that implement the same protocol.

**The Protocol** is the specification — the contract that defines how everything communicates. It is a document, not a program. Developers read it, implementations follow it. The protocol is the permanent thing. Everything else is an expression of it.

**The Node** is the server software. It hosts Spaces, stores Event logs, participates in federation, and manages Identity for its members. Node operators install and run it on machines they control.

**The Client** is the user-facing application. It connects to a Node, renders Spaces and Rooms, sends and receives Events. Regular users download and run it. They never interact with a Node directly.

These three deliverables are built separately, maintained separately, and distributed separately. Any developer can write a compatible client or Node implementation by following the protocol specification. This is the open protocol model — the same way anyone can write an email client or email server by following SMTP.

---

### Vanilla Node — As Simple as the Client

The vanilla Node is a Node running entirely on defaults. No database configuration. No certificate management. No capability tuning. No network expertise required. The setup experience must be as fast and frictionless as setting up the client.

**The setup comparison:**

| Step | Client setup | Vanilla Node setup |
|---|---|---|
| Download | XGen Client app | XGen Node package |
| Question 1 | What is your name? | What is your Node called? |
| Question 2 | Verify Identity (Tier 1 — email + phone) | Where should data be stored? |
| Question 3 | Join a Space or create one | — (defaults handle everything else) |
| Time | ~2 minutes | ~2 minutes |
| Result | Connected user on the network | Live Node on the network |

The Node asks one fewer personal question and one more infrastructure question. That is the entire visible difference for a vanilla deployment.

**What defaults cover automatically on a vanilla Node:**

| Setting | Default value |
|---|---|
| Capabilities | `messaging`, `identity`, `federation`, `gateway` |
| Auth tier | Tier 1 only |
| Capacity | Auto-detected from available RAM and disk |
| Network | Auto-detects public IP — uses relay if behind NAT |
| Encryption | Keypair generated on first run |
| Federation | Connects to XGen bootstrap network automatically |
| Storage | Local disk, sensible default path |
| Spaces hosted | Zero at start — operator creates or imports |
| Updates | Opt-in automatic updates |

Nothing requires manual configuration. Everything can be changed later by an operator who knows what they are doing — but nothing needs to be changed to get started.

**The one honest difference between Node and Client:**

A client is stateless. If you lose your device, you restore your Identity from backup and reconnect. Nothing is lost that you did not already have.

A Node is stateful. It holds Event logs on behalf of its members. If you lose the Node without a backup, that data is gone. This is not a complexity problem — it is a responsibility difference. The Node software makes backup visible, easy, and encouraged from day one. But the responsibility itself cannot be automated away, and the software will not pretend otherwise.

> *Running a vanilla Node is as simple as running the client. The difference is not complexity — it is stewardship. A Node operator is a custodian of other people's history.*

---

### What a Node Is

A Node is defined by four things:

**A cryptographic identity.** A Node has its own keypair — generated on first run, permanent for the life of the Node. The Node signs its own announcements and federation messages. Other Nodes and clients verify these signatures to confirm they are talking to who they think they are.

**A capability set.** A Node declares what it can do via an open-ended capability enum. The capability set determines how other Nodes and clients interact with it. A Node that does not declare `media_relay` will not be asked to relay voice calls. A Node that declares `auth_tier_3` is trusted to handle corporate PKI verification. Capabilities are honest self-declaration — not assigned by any central authority.

**A federation stance.** A Node participates in the network by establishing federation relationships with other Nodes. It routes Events, replicates Room state, and propagates Space membership changes. Federation is the mechanism that makes XGen a network rather than a collection of isolated servers.

**A jurisdiction declaration.** A Node declares the legal jurisdiction it operates under. This is not enforced by the protocol — it is an honest declaration that allows Space operators and members to make informed decisions about where their data lives. Relevant for GDPR compliance, data residency requirements, and institutional deployments.

---

### Node Anatomy

```
node {
  id:           "xgen://node/sha256:..."         ← permanent, hash-derived
  name:         "retrogamers.net"                ← human-readable name
  capabilities: [messaging, identity,
                 federation, gateway]             ← open capability enum
  capacity:     "medium"                         ← low | medium | high
  auth_tiers:   [1]                              ← supported auth tiers
  jurisdiction: "EU"                             ← declared legal jurisdiction
  version:      "xgen/0.1"                       ← protocol version
  signature:    "ed25519:KEYID:BASE64..."         ← Node's own signature
  meta-atts:    { ... }                          ← opaque namespaced extension map
}
```

**Field notes:**

- `id` — permanent and hash-derived. Never changes. Even if the Node moves to new hardware, the operator may choose to preserve the identity by migrating the keypair.
- `capabilities` — the open enum. See Capability Enum section below.
- `capacity` — an honest self-assessment. `low` for a Raspberry Pi or small VPS. `medium` for a dedicated small server. `high` for enterprise infrastructure. Used by clients and other Nodes to make routing decisions.
- `auth_tiers` — which authentication tiers this Node can process and verify. A vanilla Node supports Tier 1 only. A corporate Node may support Tiers 1–3. A government Node may support all four.
- `jurisdiction` — the declared legal context. Not a technical enforcement — an honest label.
- `version` — the protocol version this Node implements. Allows the network to handle version differences gracefully.

---

### Capability Enum

Capabilities are what a Node does. The enum is open-ended — new capabilities can be added without breaking existing Nodes. Old Nodes ignore capability values they do not understand. New Nodes that speak a new capability find each other and interact. The network grows without forking.

**Core capabilities:**

| Capability | Function |
|---|---|
| `messaging` | Stores and relays text messages and Events |
| `identity` | Manages user Identity and cryptographic keypairs |
| `federation` | Routes Events and state between Nodes |
| `gateway` | Client entry point and connection management |
| `media_relay` | Voice/video TURN relay for real-time calls |
| `file_storage` | Large file hosting and transfer |
| `bridge` | Connects XGen to external networks (Discord, email, etc.) |

**Auth tier capabilities:**

| Capability | Function |
|---|---|
| `auth_tier_1` | Handles Tier 1 community verification (email + phone) |
| `auth_tier_2` | Handles Tier 2 professional verification (ID + business) |
| `auth_tier_3` | Handles Tier 3 corporate PKI verification |
| `auth_tier_4` | Handles Tier 4 government eID verification |

**Vanilla Node default capabilities:** `messaging`, `identity`, `federation`, `gateway`, `auth_tier_1`

**The open enum principle:** The capability list is intentionally incomplete. Future capabilities — `ai_agent`, `legal_notarization`, `reputation_scoring`, `healthcare_relay` — become new enum values when the community needs them. Old Nodes ignore values they do not understand. The protocol does not fork. The network grows.

---

### Capability Combinations by Node Size

```
Vanilla Node (Raspberry Pi / small VPS)
→ capabilities: [messaging, identity, federation, gateway, auth_tier_1]
→ capacity: low
→ serves: personal use, small family, small community
→ setup time: ~2 minutes

Community Node (mid-range VPS)
→ capabilities: [messaging, identity, federation, gateway,
                 file_storage, auth_tier_1, auth_tier_2]
→ capacity: medium
→ serves: small to mid-size community, professional Spaces

Full Node (dedicated server)
→ capabilities: [all including media_relay, all auth_tiers]
→ capacity: high
→ serves: large communities, enterprises, institutions

Corporate Node (managed internal infrastructure)
→ capabilities: [messaging, identity, federation, gateway,
                 file_storage, auth_tier_1, auth_tier_2, auth_tier_3]
→ capacity: high
→ serves: internal company communication — no public federation required

Government Node (agency-managed, certified)
→ capabilities: [messaging, identity, federation, gateway,
                 file_storage, auth_tier_1, auth_tier_2,
                 auth_tier_3, auth_tier_4]
→ capacity: high
→ serves: regulated institutional communication
```

No hierarchy. No privileged node types. No chokepoints to capture or monetize. The same software runs everywhere. Capacity and capabilities determine behavior.

---

### High-Responsibility Capabilities

Some capabilities carry higher trust requirements than others. A Node declaring `identity` is trusted by other Nodes and clients to manage cryptographic keypairs responsibly. A Node declaring `auth_tier_3` or `auth_tier_4` is trusted to verify corporate or government credentials correctly. If these Nodes are compromised or dishonest, the damage is significant.

The protocol must therefore define what verification is required before a Node may advertise certain capabilities. This is not about central approval — it is about declared accountability.

| Capability | Trust requirement |
|---|---|
| `messaging`, `gateway` | None beyond valid Node identity |
| `federation` | Valid Node identity + reachable endpoint |
| `identity` | Operator accountability declaration + audit log capability |
| `file_storage` | Operator accountability declaration |
| `auth_tier_1`, `auth_tier_2` | Compliance with XGen Tier verification standard |
| `auth_tier_3` | Certified by relevant institutional authority |
| `auth_tier_4` | Certified by national eID authority |
| `bridge` | Operator declaration of bridge target and trust boundary |

The specific certification and verification mechanisms for high-responsibility capabilities are Chapter 3 specification problems. The architectural principle is established here: **capability advertisement is a trust claim, not a technical fact. The network must have mechanisms to verify it.**

---

### What a Node Is Not

- **A Node is not a Space.** A Node hosts Spaces. It does not own them.
- **A Node is not a gatekeeper.** Any client that speaks XGen can connect to any Node that accepts it. No central authority approves Node deployments.
- **A Node is not required to be always-on.** A low-capacity personal Node may go offline. Federation handles this gracefully — other Nodes that replicate the Room state continue serving members.
- **A Node is not the identity of its members.** Member Identities are portable keypairs. They are not tied to the Node. A member whose Node goes offline retains their Identity and can register with another Node.
- **A Node operator is not a platform.** The operator provides infrastructure. The protocol, not the operator, defines the rules. An operator cannot change the protocol. They can only choose which capabilities to offer.

---

## Compliance & Data Retention by Auth Tier

One of the unresolved tensions identified in Chapter 1 was the collision between federated identity, no anonymity, and the GDPR right to be forgotten. The provisional answer given there was directionally sound but not fully specified. This section closes that gap at the architectural level.

The core insight is simple:

> *GDPR compliance requirements are not uniform. They scale with the sensitivity of the data and the institutional context of the Space. XGen maps this directly onto the auth tier model. The deletion obligation scope and enforcement mechanism is determined by the auth tier of the Space — not by the protocol globally.*

This is not a workaround. It is the correct architectural answer. Different tiers handle fundamentally different data, operate under different legal frameworks, and carry different levels of institutional accountability. The protocol provides the mechanism. The Auth Module carries the compliance obligation.

---

### The Regulatory Landscape — Mapped to XGen Tiers

Data retention requirements across sectors follow a clear pattern of increasing obligation as institutional sensitivity increases. The mapping to XGen auth tiers is direct and intentional.

| Auth Tier | Context | Governing Framework | Retention Obligation | Deletion Enforcement |
|---|---|---|---|---|
| **Tier 1** Community | Gaming, hobby, general public | GDPR Art. 5 baseline | No fixed period — delete when purpose ends | Best-effort. Protocol provides the mechanism. No certified propagation required. |
| **Tier 2** Professional | Freelancers, SMEs, businesses | GDPR + ISO 27001 | Minimum 3 years (ISO 27001 data log standard) | Documented retention policy required. Auth Module carries obligation. |
| **Tier 3** Corporate | Enterprises, finance, legal | GDPR + ISO 27001 + SOX / PCI DSS / Basel II | 3–7 years depending on sector (SOX: 7 years, Basel II: 3–7 years) | Strict deletion propagation mandatory. Certified module required. |
| **Tier 4** Government / Healthcare | Agencies, hospitals, legal | GDPR Art. 9 + ISO 27001 + national sector law | 10–20+ years depending on jurisdiction and data type (Germany: 10 years healthcare, France: 20 years) | Highest standard. Node must be certified. National authority involvement mandatory. |

---

### How This Resolves the GDPR Tension

**The right-to-erasure problem in federated systems** — an Event has propagated to dozens of Nodes before a deletion request arrives. There is no central delete button. This is a structural property of federation, not a bug.

XGen's answer is architectural, not procedural:

**Tier 1 — best-effort deletion.** A `message.delete` Event is propagated. Nodes that receive it remove the content from display. No certified propagation guarantee is required. Legal exposure for Tier 1 operators is low — general GDPR baseline applies, and the data involved is community communication, not sensitive personal records.

**Tier 2 — documented deletion policy.** The Space's Auth Module must carry a documented retention and deletion policy. ISO 27001 compliance requires this. Deletion propagation must be logged. The module is accountable for ensuring the policy is followed across all federated Nodes participating in the Space.

**Tier 3 — certified deletion propagation.** Deletion Events are cryptographically signed and propagated with delivery confirmation. The certified Auth Module is legally accountable for propagation across all Nodes. A Tier 3 Space operator cannot claim ignorance of where their data lives — the module tracks it.

**Tier 4 — national authority certified.** Deletion obligations follow national sector law. Healthcare records in France must be retained for 20 years — a deletion request before that period may be legally refused. The certified module implements the jurisdiction-specific rules. The Node itself must be certified by the relevant national authority.

> *The protocol provides a uniform deletion Event mechanism. What differs by tier is the obligation to propagate, confirm, and audit that deletion. The Auth Module is the compliance layer. The protocol is the mechanism layer. They are separate by design.*

---

### Practical vs Theoretical — The Implementation Split

This distinction is foundational and must be stated explicitly:

**Theoretically**, all four Auth Modules are fully defined in this specification. Chapters 2 and 3 define the interface, the compliance obligations, and the trust requirements for every tier. This is necessary for the protocol to be complete — any institution must be able to read the spec and build a compliant module.

**Practically**, only the Tier 1 Auth Module ships with XGen as a reference implementation built by the core team. Higher tiers require institutional collaboration that cannot happen unilaterally.

| Tier | Specification | Reference Implementation | Development Path |
|---|---|---|---|
| **Tier 1** | Full — Ch2 + Ch3 | ✓ Built by XGen core team. Ships with the protocol. | Core team |
| **Tier 2** | Full — Ch2 + Ch3 | ✗ Not shipped by default | Collaboration with professional verification providers and business registries |
| **Tier 3** | Full — Ch2 + Ch3 | ✗ Not shipped by default | Collaboration with corporate PKI authorities and enterprise IT departments |
| **Tier 4** | Full — Ch2 + Ch3 | ✗ Not shipped by default | Collaboration with national eID authorities — eIDAS bodies, government agencies, healthcare regulators |

**Why Tiers 2–4 cannot be built unilaterally:**

- **Legal** — a Tier 4 module claiming to verify government identity must be certified by the relevant national authority. Self-certification is not legally valid.
- **Technical** — higher tiers require access to institutional infrastructure: government eID systems, corporate PKI registries, professional license databases. XGen cannot build against systems it has no access to.
- **Strategic** — this is the correct business model. The Foundation specifies the slot. Institutions build the plug. The Foundation certifies the plug meets the spec. This is the certified module fee income stream defined in Chapter 1 — the most significant and stable revenue source.

> *The Tier 1 Auth Module is the reference implementation that proves the pluggable model works and sets the quality bar. Every higher-tier module is developed in collaboration with the institution that has the legal authority and technical infrastructure to certify it.*

---

## Auth Module & Trust Assertion

The Auth Module is the pluggable authentication slot in XGen Protocol. It is the mechanism by which a user's Identity is verified to a declared trust level and a Trust Assertion is produced for the protocol to consume.

The protocol does not care how verification happened. It cares only about the result: a standardised Trust Assertion that declares a verified tier level. The Auth Module is everything between the user and that assertion. The protocol owns the slot interface. The module owns everything inside it.

---

### The Slot and the Plug

The Auth Module interface is a contract between the protocol and any compliant implementation:

```
Auth Module Contract

  INPUT:  User Identity reference + verification request
  PROCESS: Internal to the module — any method the tier requires
  OUTPUT: Trust Assertion (standardised format)
           OR rejection with reason code
```

The protocol never sees inside the module. It sends a verification request and receives either a Trust Assertion or a rejection. What happens inside — whether it is an email confirmation, a government eID check, a PKI certificate chain, or a biometric scan — is the module's business.

This is the principle from Chapter 1 stated architecturally:

> *The protocol owns the trust assertion, not the credential format. A module may use any internal structure it requires. What it returns to the protocol is always the same: a standardised trust level claim.*

---

### Trust Assertion Anatomy

The Trust Assertion is the output of every Auth Module. It is always the same structure regardless of which module produced it or what verification method was used internally.

```
trust_assertion {
  identity:      "xgen://identity/pubkey:..."    ← the verified Identity
  tier:          2                               ← verified tier level (1-4)
  issued_at:     1714000000000                  ← unix ms
  expires_at:    1745536000000                  ← unix ms — assertions expire
  issuer:        "xgen://module/tier2-eu-v1"    ← the Auth Module that issued this
  jurisdiction:  "EU"                           ← legal jurisdiction of verification
  signature:     "ed25519:KEYID:BASE64..."       ← module's own signature
  meta-atts:     { ... }                        ← opaque module-specific claims
}
```

**Field notes:**

- `tier` — the verified trust level. This is what the protocol uses for access control decisions. A Space declaring `auth_tier_min: 2` checks this field.
- `expires_at` — Trust Assertions are not permanent. They expire and must be renewed. Expiry period varies by tier — Tier 1 may be valid for months, Tier 4 may expire within days for high-security contexts.
- `issuer` — identifies which Auth Module produced this assertion. Allows Nodes and Spaces to enforce module-specific requirements — a government Space may require assertions from a specific certified national module, not just any Tier 4 module.
- `jurisdiction` — the legal context of the verification. A German eID verification and a French eID verification are both Tier 4 but may produce assertions with different jurisdiction values. Spaces can filter on this.
- `signature` — the module signs its own assertion. The recipient can verify the assertion was produced by the declared issuer and has not been tampered with.
- `meta-atts` — opaque module-specific claims. A Tier 3 module may include the company registration number. A Tier 4 module may include a reference to the national ID verification record. The protocol ignores these. Applications may use them.

---

### The Four Auth Modules — Theoretical Specification

All four modules are fully specified here. Only Tier 1 ships as a reference implementation.

#### Tier 1 — Community Auth Module
*Reference implementation built by XGen core team. Ships with the protocol.*

**Verification method:** Email address confirmation + phone number confirmation.
**What it proves:** A real person controls this email and phone number. Not necessarily their legal name.
**Compliance:** GDPR Art. 5 baseline. Data minimisation. No unnecessary retention.
**Assertion validity:** Up to 12 months. Renewable.
**Right to erasure:** Best-effort deletion propagation. No certified delivery required.
**Implementation:** Built by XGen core team. Open source. Ships as default with vanilla Node and client.

#### Tier 2 — Professional Auth Module
*Specification complete. Implementation requires collaboration with verification providers.*

**Verification method:** Government-issued ID document verification + business registration number (where applicable).
**What it proves:** A real, named individual. May include professional or business identity.
**Compliance:** GDPR + ISO 27001. Minimum 3-year data log retention. Documented deletion policy.
**Assertion validity:** Up to 6 months. Renewal requires re-verification.
**Right to erasure:** Documented propagation. Module must log deletion confirmations.
**Implementation:** Developed in collaboration with KYC (Know Your Customer) providers and business registry APIs. Certified by XGen Foundation against Tier 2 standard.

#### Tier 3 — Corporate Auth Module
*Specification complete. Implementation requires collaboration with PKI authorities.*

**Verification method:** PKI certificate chain issued by a recognised corporate Certificate Authority. IT-managed enrollment.
**What it proves:** An employee of a specific organisation, verified through that organisation's PKI infrastructure.
**Compliance:** GDPR + ISO 27001 + sector law (SOX, PCI DSS, Basel II as applicable). 3–7 year retention. Certified deletion propagation.
**Assertion validity:** Tied to PKI certificate validity. Typically 1 year maximum.
**Right to erasure:** Certified propagation with delivery confirmation. Module is legally accountable.
**Implementation:** Developed in collaboration with enterprise PKI providers and corporate IT departments. Each corporate deployment may customise the module within the Tier 3 specification.

#### Tier 4 — Government / Institutional Auth Module
*Specification complete. Implementation requires national authority collaboration and certification.*

**Verification method:** National eID (eIDAS-compatible), FIDO2 hardware keys, biometric where required by jurisdiction.
**What it proves:** A legally verified individual identity, recognised by a national authority.
**Compliance:** GDPR Art. 9 + national sector law. Retention periods are jurisdiction-specific (10–20+ years for healthcare). Node must be independently certified.
**Assertion validity:** Short — days to weeks in high-security contexts. Hardware-bound in some jurisdictions.
**Right to erasure:** Jurisdiction-specific. Some data may be legally required to be retained beyond erasure requests. The module implements the correct jurisdictional rules.
**Implementation:** Developed in collaboration with national eID authorities, healthcare regulators, and government IT bodies. Each jurisdiction certifies their own module variant. All variants plug into the same protocol slot.

---

### Module Lifecycle

An Auth Module is not static. It has its own lifecycle within the protocol:

```
SPECIFIED → DEVELOPED → CERTIFIED → ACTIVE → DEPRECATED
                                              → REVOKED
```

**Specified.** The module interface and compliance requirements are defined in the protocol spec. No implementation exists yet.

**Developed.** An implementation is built — either by the XGen core team (Tier 1) or in collaboration with an institutional partner (Tiers 2–4).

**Certified.** The XGen Foundation reviews the implementation against the tier specification. Certification confirms the module meets the protocol interface, the compliance obligations, and the trust requirements of its tier. The Foundation issues a module identifier that appears in Trust Assertions.

**Active.** The module is live. Trust Assertions it produces are recognised by the network.

**Deprecated.** A newer version of the module exists. The old version continues to produce valid assertions until its expiry, but new verifications should use the updated module.

**Revoked.** A module has been found non-compliant, compromised, or its issuing authority has withdrawn certification. All Trust Assertions from this module are immediately invalid. Nodes must re-verify affected users.

---

### What the Auth Module Is Not

- **The Auth Module is not the Identity.** Identity is the user's keypair. The Auth Module verifies claims about the person behind that keypair. They are independent.
- **The Auth Module is not the protocol.** The protocol defines the slot. The module is a plug. The protocol continues to function when modules are upgraded or replaced.
- **A higher tier module does not replace lower tier assertions.** Tiers are cumulative. A Tier 3 user holds a Tier 3 assertion that implicitly satisfies Tier 1 and Tier 2 requirements. They do not hold separate assertions for each tier.
- **The Auth Module does not own the user's data.** The module verifies and issues assertions. Data minimisation applies — the module retains only what the tier's compliance framework requires, not more.

---

## Identity Model

Identity is the most important primitive in XGen Protocol. It is the thing no predecessor protocol got right. Matrix ties identity to servers. Discord ties identity to a company. Signal ties identity to a phone number. In every case, the platform or infrastructure owns the identity. When the platform dies, bans you, or changes its terms, your identity goes with it.

XGen inverts this entirely.

> *Your Identity is a cryptographic keypair. You own it. No server can take it. No company can revoke it. No Node operator can delete it. Your Identity travels with you across the entire XGen network, independent of any infrastructure.*

This is not a feature. It is a foundational architectural decision that shapes everything else in the protocol.

---

### What Identity Is

An Identity in XGen is defined by three things:

**A keypair.** A private key that never leaves the user's device, and a public key that is shared with the network. The public key is the user's address on the network. The private key is the proof of ownership. Together they are the Identity.

**A Trust Assertion.** A signed statement from an Auth Module that the person behind this keypair has been verified to a declared tier level. The Trust Assertion is not the Identity — it is a claim about the Identity. The Identity exists independently of any assertion.

**A history.** Every Event the Identity has ever signed is permanently attributable to it. The Identity is not just a key — it is a record of participation. This is the Kyberia principle at the protocol level: identity carries consequence.

---

### Identity Anatomy

```
identity {
  id:              "xgen://identity/pubkey:ed25519:BASE64..." ← the public key IS the ID
  display_name:    "JozefN"                                  ← user-chosen, not unique
  created_at:      1714000000000                             ← unix ms, immutable
  home_node:       "xgen://node/sha256:..."                  ← current home Node
  current_key:     "ed25519:BASE64..."                       ← active public key
  previous_keys:   ["ed25519:BASE64...", ...]                ← rotated keys — history preserved
  trust_assertion: { ... }                                   ← current Trust Assertion
  devices:         [ ... ]                                   ← registered devices
  meta-atts:       { ... }                                   ← opaque namespaced extension map
}
```

**Field notes:**

- `id` — the public key itself is the Identity ID. There is no separate identifier. If you have the public key, you have the address. This is how server-independent identity works — no server needs to assign or store the ID.
- `display_name` — user-chosen, human-readable, not unique across the network. Two people can have the same display name. The public key is the unique identifier. Clients display names for convenience — the protocol uses keys.
- `home_node` — the Node currently storing and serving this Identity's data. Changes when the user migrates. The Identity ID does not change.
- `current_key` — the active public key. Used to verify all new Events signed by this Identity.
- `previous_keys` — rotated keys. Events signed by a previous key remain valid and attributable. The history is unbroken.
- `trust_assertion` — the current Trust Assertion issued by an Auth Module. Expires and is renewed. The Identity exists without it — the assertion is a claim, not a requirement for existence.
- `devices` — the list of devices that hold the private key for this Identity. Multi-device support is a protocol concern, not just a client concern. See Device Model below.

---

### Server-Independent Identity — How It Works

In Matrix, your identity is `@joe:matrix.org`. The server is baked into your name. If matrix.org goes away, so do you.

In XGen, your identity is `xgen://identity/pubkey:ed25519:MCowBQYD...`. The public key is your name. No server is in the address. The Node is your current home — it stores and serves your data — but it is not part of who you are.

**The practical consequence:**

```
User registers on Node A
└── Identity created: xgen://identity/pubkey:ed25519:XXXX
    └── Node A stores the Identity record

Node A goes offline
└── User registers the same Identity on Node B
    └── Identity ID unchanged: xgen://identity/pubkey:ed25519:XXXX
    └── All previous Events still attributable to this Identity
    └── All Space memberships intact
    └── All contacts recognise the same key
```

The Node is infrastructure. The Identity is permanent. This is what Matrix's original sin prevented — and what XGen makes structurally impossible to break.

---

### The Device Model

A user rarely has one device. They have a phone, a laptop, a work machine, a tablet. Each device needs to be able to sign Events as the same Identity. This creates a real security problem: if the private key is on all devices, compromising one device compromises the Identity entirely.

XGen uses a **device key model** — each device has its own keypair, and device keys are authorised by the Identity's primary key.

```
Identity (primary keypair)
├── Device: Phone      (device keypair, authorised by primary key)
├── Device: Laptop     (device keypair, authorised by primary key)
└── Device: Work PC   (device keypair, authorised by primary key)
```

**How it works:**

- Each device generates its own keypair on setup
- The user authorises the new device from an existing trusted device using the primary key
- Events are signed by the device key, with a reference to the authorising Identity
- Verifiers check: (1) is the device key valid? (2) was it authorised by the Identity's primary key?
- If a device is lost or compromised, it is revoked using any other authorised device

**Device authorisation is a signed Event** — `identity.device.add` and `identity.device.revoke` are System Events in the Event taxonomy. The authorisation chain is part of the permanent record.

**The first device problem:** When a user registers for the first time, there is no existing trusted device to authorise from. The Auth Module handles this — Tier 1 verification and initial device authorisation happen together during setup. This is the moment the Identity is born.

---

### Key Rotation

Cryptographic keys should be rotated periodically. Old keys may be compromised. New algorithms may be required. XGen makes key rotation a first-class protocol operation.

**What happens during key rotation:**

1. User generates a new keypair on their device
2. The new public key is signed by the old private key — proving continuity of ownership
3. An `identity.key.rotate` System Event is written, containing both the old key reference and the new public key, signed by the old key
4. All Nodes and contacts that hold the old key receive the rotation Event
5. Future Events are signed by the new key
6. Old Events remain valid — they were correctly signed at the time of signing

**The chain of trust is unbroken.** Each key rotation is cryptographically linked to the previous key. An attacker cannot rotate your key without access to your current private key. An observer can verify the entire chain from the original key to the current one.

```
Key history:
ed25519:AAAA (created 2026-04)  →  signed rotation to ed25519:BBBB
ed25519:BBBB (rotated 2027-01)   →  signed rotation to ed25519:CCCC
ed25519:CCCC (current)
```

The `previous_keys` field in the Identity record preserves this chain. All Events signed by AAAA or BBBB remain attributable to the same Identity.

---

### Identity Replication — Resilience Without a Primary

The Identity record contains only public information — public key, Trust Assertion, device list, Space memberships, Identity-scoped settings. None of this is sensitive. All of it is cryptographically verifiable regardless of which Node serves it.

XGen therefore replicates Identity records across multiple Nodes automatically. No single Node is the authoritative source. All replicas are equal peers.

```
Identity: xgen://identity/pubkey:ed25519:XXXX

  Node A  ──  replica  (user's home Node)
  Node B  ──  replica  (random federation peer)
  Node C  ──  replica  (random federation peer)
  Node D  ──  replica  (random federation peer)
```

**How it works:**

- When an Identity is created, the record is automatically propagated to N random Nodes in the federation
- When the Identity is updated — key rotation, new Trust Assertion, new device — an update Event propagates to all replica Nodes
- When any Node or client needs to resolve an Identity, it queries the network — any replica Node can answer
- The response is verified cryptographically by the requester — the Node's word is not trusted, the signature is
- If the home Node disappears, the user re-registers on any Node — that Node resolves the full Identity record from any surviving replica and the user continues without loss

**The `home_node` field is a routing hint, not an authority.** It is the Node the user registered on and currently uses as their preferred entry point — the first in the lookup row. If it does not respond, any other replica answers. The result is identical because the cryptographic verification does not depend on the source.

**What is replicated:**

| Data | Replicated | Notes |
|---|---|---|
| Public key | ✓ | Core of the Identity record |
| Trust Assertion | ✓ | Public, verifiable |
| Device list (public keys) | ✓ | Required for Event verification |
| Space memberships | ✓ | Via Space federation |
| Identity-scoped settings | ✓ | Display name, avatar, privacy prefs |
| Private key | ✗ | Never leaves the user's device |
| Recovery key | ✗ | User's responsibility — see Key Recovery |
| Client-scoped settings | ✗ | Device-only — not a protocol concern |

> *The private key is never replicated. Only public records travel the network. Cryptographic verification means no Node needs to be trusted — only verified.*

---

### Where Data Lives — The Complete Picture

This is a common source of confusion and worth stating cleanly in one place.

| Data | Lives in | Replicated how | Backed up how |
|---|---|---|---|
| Messages / Events | Space → Room Event log | Federation across all participating Nodes | Automatic via federation |
| Space membership & roles | Space state | Federation across Space's Nodes | Automatic via federation |
| Space-scoped settings | Space membership record | With the Space | Automatic via federation |
| Identity record (public) | Identity replicas across N Nodes | Identity replication | Automatic via replication |
| Identity-scoped settings | Identity record | Identity replication | Automatic via replication |
| Private key | User's device only | Never replicated | User responsibility |
| Recovery key | User's choice | Never replicated | User responsibility |
| Client-scoped settings | Device only | Never replicated | Not a protocol concern |

**Space-scoped settings** are things that define how a user exists within a specific Space — notification preferences per Room, nickname within the Space, muted Rooms, read position markers. These belong to the Space membership record and travel with the Space.

**Identity-scoped settings** follow the user across all Spaces and devices — global display name, avatar, privacy preferences (who can DM me, who can see my online status), blocked users list. These belong to the Identity record and replicate with it.

**Client-scoped settings** are purely about how the application behaves on a specific device — UI theme, font size, keyboard shortcuts, local cache preferences. These never leave the device. The protocol has nothing to say about them.

---

### Key Recovery

Key rotation assumes the user still has access to their private key. Key recovery is the harder problem: what happens when the private key is lost entirely?

This is where the honest tradeoff must be stated clearly:

> *There is no recovery without prior preparation. A lost private key with no backup and no registered recovery mechanism means the Identity cannot be recovered. The Events it signed remain permanently in the log, but new Events cannot be signed as that Identity. This is the price of true ownership — no company can restore what you lost, because no company held it.*

XGen provides three recovery mechanisms, all requiring prior setup:

**Recovery 1 — Device-based recovery.** If the user has multiple authorised devices, any surviving device retains the private key. This is the primary recovery path and the reason multi-device support is important. A user with a phone and a laptop who loses their phone is not locked out — the laptop still holds the key.

**Recovery 2 — Recovery key (offline).** During setup, the user may generate a recovery keypair — a special keypair stored offline (printed, written down, kept on a USB drive). If all devices are lost, the recovery key can be used to authorise a new device and rotate the primary key.

```
Recovery key setup:
└── User generates recovery keypair during Identity setup
    └── Recovery public key is registered with the Identity
    └── Recovery private key is stored OFFLINE by the user
    └── If all devices lost: recovery private key signs a new device authorisation
    └── New device generates new keypair, Identity continues
```

**Recovery 3 — Encrypted cloud backup.** The recovery keypair may be encrypted and stored in any cloud service the user chooses — Google Drive, iCloud, OneDrive, a password manager, or any other storage. The encrypted blob is useless without the decryption passphrase, which never leaves the user's memory.

```
Encrypted cloud backup:
└── User generates recovery keypair
    └── Recovery keypair is encrypted with a strong user-chosen passphrase
    └── Encrypted blob is uploaded to user's cloud storage of choice
    └── Passphrase is NEVER uploaded — memorised or stored separately
    └── If all devices lost:
        └── User downloads encrypted blob from cloud
        └── User decrypts with passphrase
        └── Recovery key signs new device authorisation
        └── Identity continues
```

This is the same model used by modern password managers — an encrypted vault stored in the cloud, with the passphrase never transmitted. If the cloud service is breached, the attacker holds an encrypted blob they cannot use without the passphrase. The security guarantee depends entirely on passphrase strength and secrecy.

> *USB drives fail silently. Cloud services are available everywhere. Recovery 3 is the recommended default for most users — but only with a strong passphrase stored separately from the encrypted blob. The client presents all three options during onboarding and explains the tradeoffs clearly.*

**The client must make recovery setup visible, easy, and strongly encouraged during onboarding.** Not mandatory — a Tier 1 community user may choose to skip it and accept the risk. But the risk must be clearly communicated. The software does not hide it.

**What cannot be recovered without a recovery mechanism:**
- The ability to sign new Events as the lost Identity
- Space memberships that require active Identity verification
- Trust Assertions — these must be re-issued by the Auth Module for the new key

**What is never lost regardless:**
- All Events previously signed by the Identity — they remain in the log permanently
- The Identity's public reputation and history — it exists in the network record

---

### Identity Portability — Migrating Between Nodes

A user can move their Identity from one Node to another at any time. This is not an edge case — it is a core feature. Node operators can change their terms. Nodes can go offline. A user may simply prefer a different Node.

**Migration sequence:**

1. User registers their existing Identity on the new Node
2. The new Node verifies the Identity's keypair
3. An `identity.node.migrate` System Event is written, signed by the Identity
4. The old Node forwards the user's data to the new Node (or the user retrieves it directly)
5. Space memberships are updated with the new home Node address
6. The Identity ID is unchanged throughout

**Federation handles continuity.** Other Nodes and clients that hold the user's public key continue to recognise the same Identity at its new home. The move is transparent to other participants in any Space the user belongs to.

---

### Identity Lifecycle

```
CREATED → ACTIVE → SUSPENDED (by user)
                 → MIGRATED (to another Node, remains ACTIVE)
                 → ORPHANED (home Node gone, awaiting re-registration)
```

**Created.** The user generates a keypair, completes Tier 1 verification, and receives their first Trust Assertion. The Identity is written to the home Node. The first device is authorised.

**Active.** The normal state. The user signs Events, participates in Spaces, holds a valid Trust Assertion.

**Suspended.** A user can voluntarily suspend their Identity — no new Events will be accepted. Useful for accounts that are temporarily inactive but should not be deleted.

**Migrated.** The Identity has moved to a new home Node. The ID is unchanged. History is intact.

**Orphaned.** The home Node has gone offline or been decommissioned without migration. The Identity still exists in the network — other Nodes hold replicas of its Event history and Space memberships. The user re-registers on a new Node using their keypair. The Identity is recovered from Orphaned to Active.

> *An Identity is never deleted by the protocol. It may become orphaned if its home Node disappears, but its history remains in the network and it can be re-activated on a new Node by the keyholder.*

---

### Identity and Trust Assertion — The Relationship

These two things are frequently confused. They are not the same.

| | Identity | Trust Assertion |
|---|---|---|
| What it is | A keypair | A signed claim about the keypair's owner |
| Who creates it | The user (keypair generation) | An Auth Module (after verification) |
| How long it lasts | Forever | Until it expires (tier-dependent) |
| What it proves | You control this key | Someone has verified who you are to a tier level |
| Can it be revoked? | No — the key exists | Yes — the module can revoke the assertion |
| Protocol requirement | Always required | Required for tier-gated access |

A user can hold a valid Identity with no Trust Assertion — they simply cannot access Spaces or Rooms that require verified tier levels. The base Identity — a keypair with no assertion — can still sign Events in open Spaces.

---

### What Identity Is Not

- **Identity is not a username.** Display names are human-readable conveniences. The public key is the unique identifier. Two users can share a display name. No two users can share a public key.
- **Identity is not owned by a Node.** The Node hosts the Identity record. It does not own the keypair. The private key never leaves the user's device.
- **Identity is not an account.** An account implies a relationship with a platform. An Identity is a cryptographic fact. It exists regardless of whether any platform recognises it.
- **Identity is not the same as Trust Assertion.** The keypair is the Identity. The assertion is a claim about the person behind it. They are independent.
- **A lost Identity is not recoverable by anyone else.** No support ticket. No account recovery email. No admin override. This is the honest price of true ownership. The software makes recovery preparation easy. The responsibility is the user's.

---

## Session Log

### Session 1 — April 2026 (JozefN)
**Covered:** Architecture principles defined. Terminology introduced — Table A (primitive lineage) and Table B (cross-platform analogues). Primitive hierarchy established and compared to Matrix and Discord. Hardware / Node / Space infrastructure stack clarified — isolation boundary is the Node, not the machine. Node cardinality rules locked (one Node per machine, one machine can run many Nodes, one Space per Node at any moment, Spaces are portable). Category primitive explicitly rejected in favour of Room meta-atts `section` field.

### Session 2 — April 2026 (JozefN)
**Covered:** Cryptographic Signatures & Algorithm Agility section added. Signature anatomy explained (algorithm:keyid:base64 format). Algorithm agility established as a non-negotiable design principle — no hardcoded cryptographic algorithm anywhere in the protocol. Event ID established as a content hash with declared hash algorithm prefix, not a random UUID. Graceful handling of unknown algorithms defined. Connected to Temporal Resilience pillar from Chapter 1.

### Session 3 — April 2026 (JozefN)
**Covered:** Event Model written. Five core properties defined (Immutable, Signed, Typed, Referenceable, Ordered). Event anatomy documented with full field-by-field notes. Event type taxonomy defined across four families: Content Events, State Events, System Events, Bridge Events. Ordering principle established — graph-based via prev_events DAG, not clock-based. Boundaries clarified in "What Events Are Not" section.

### Session 4 — April 2026 (JozefN)
**Covered:** Room Model written. Room defined by four properties (append-only Event log, current state, permission model, cryptographic identity). Six Room types defined (text, voice, video, forum, announcements, stage) — voice and video as always-open infrastructure, not scheduled events. Room anatomy documented with full field-by-field notes including meta-atts.xgen.section as the Category replacement. Room lifecycle defined (Created, Active, Archived, Migrated). State resolution architectural commitment made — deterministic, convergent, scale-aware. DMs defined as a minimal two-member Space, not a separate primitive.

### Session 5 — April 2026 (JozefN)
**Covered:** Thread Model written from first principles. Core question answered: what is a Thread for? Three purposes defined — Focus, Persistence, Resolution. Explicit boundaries: not a Room, not a reply chain, not a sub-Room, not permanent. Thread anatomy documented with full field-by-field notes including the status lifecycle field Discord never had. Thread lifecycle defined (Created, Open, Resolved, Archived). Thread behavior per Room type documented. Forum Room model clarified — in room.forum, Threads ARE the primary flow, not branches. Notification model stated as architectural constraint, implementation left to clients. Kyberia forum-as-community-memory principle applied at Thread level. Thread Model moved before Room Model to reflect correct bottom-up dependency order. Dependency order list in Primitive Hierarchy updated accordingly.

### Session 6 — April 2026 (JozefN)
**Covered:** Board added to Room Model — a curated, ordered list of pinned Event references in Room state. Any Event type can be pinned. Pinning is a State Event (room.pin.add / room.pin.remove), not a field on the Event itself — consistent with Event immutability. Board is ordered, moderator-controlled, fully federated. Optional label field per pin entry. room.pin.add and room.pin.remove added to State Events taxonomy in Event Model. Client rendering left to application layer.

### Session 7 — April 2026 (JozefN)
**Covered:** Space Model written. Space defined by five properties (cryptographic identity, ordered Room collection, cascading permission model, ownership model, portability). Space anatomy documented with full field-by-field notes including visibility modes and Space-level Board. Role Model defined — Space-level permission root cascading to Rooms, five built-in roles, custom roles supported, auth tier and Role explicitly independent. Space lifecycle defined (Created, Active, Archived, Migrated). Space federation properties stated — home Node hosts but does not own. Space discoverability model defined (public / private / invite-only) with invite code mechanism. Boundaries clarified in "What a Space Is Not".

### Session 8 — April 2026 (JozefN)
**Covered:** Node Model written. Node/Space essential distinction established with concrete/abstract boundary table. Three deliverables defined — Protocol, Node, Client. Vanilla Node principle established — same ~2 minute setup as client, all defaults, Tier 1 auth out of the box. Node anatomy, capability enum, capability combinations by size, high-responsibility capabilities, and boundaries documented.

### Session 9 — April 2026 (JozefN)
**Covered:** Compliance & Data Retention by Auth Tier section written — GDPR tension from Chapter 1 resolved architecturally. Regulatory landscape mapped to XGen tiers. Deletion enforcement model defined per tier. Practical vs Theoretical implementation split stated explicitly — Tier 1 ships with XGen, Tiers 2-4 developed in institutional collaboration. Auth Module & Trust Assertion section written — slot/plug contract defined, Trust Assertion anatomy documented, all four modules fully specified theoretically, Module lifecycle defined.

### Session 10 — April 2026 (JozefN)
**Covered:** Identity Model written. Identity defined by three things: keypair, Trust Assertion, history. Identity anatomy documented — public key IS the ID, no server-assigned identifier. Server-independent identity explained with before/after comparison to Matrix. Device Model defined — device keypairs authorised by primary key, device add/revoke as System Events, first device problem solved at Auth Module level. Key rotation defined as a first-class protocol operation with unbroken chain of trust. Key recovery defined honestly — two mechanisms (device-based, recovery key), both require prior setup, no recovery without preparation. Identity portability and migration sequence documented. Identity lifecycle defined (Created, Active, Suspended, Migrated, Orphaned). Identity vs Trust Assertion comparison table. Boundaries clarified in "What Identity Is Not".

### Session 11 — April 2026 (JozefN)
**Covered:** Identity Replication section added — equal peers model, no primary, home_node is routing hint not authority, replication table. Where Data Lives section added — complete picture of what lives where across Space, Identity, device, three settings categories defined (Space-scoped, Identity-scoped, client-scoped). Key Recovery updated to three mechanisms — device-based, offline recovery key, encrypted cloud backup. Cloud backup model explained — encrypted blob, passphrase never uploaded, recommended default. Direct Messages section added — DM as minimal Space, dm_space anatomy, DM initiation with accept/decline/no-response, DM privacy settings on Identity, group DMs, promotion to full Space. References section added — 21 references across Regulatory & Legal, Standards, Prior Art & Intellectual Lineage.

**Next session to begin with:**
> **Federation Model.** How Nodes discover each other, how Events propagate, how Room state is replicated and resolved across federated Nodes. The state resolution architectural commitment from the Room Model section made concrete.

---

## References

The following standards, regulations, and specifications are cited in this document. References are numbered in order of first appearance.

### Regulatory & Legal

| Ref | Document | Relevance |
|---|---|---|
| REF-01 | GDPR — Regulation (EU) 2016/679, Art. 5 | Data minimisation and purpose limitation principles. Tier 1 compliance baseline. |
| REF-02 | GDPR — Regulation (EU) 2016/679, Art. 9 | Special category data — health, biometric, government. Tier 4 compliance baseline. |
| REF-03 | GDPR — Regulation (EU) 2016/679, Art. 17 | Right to erasure (right to be forgotten). Foundation of the deletion enforcement model. |
| REF-04 | eIDAS — Regulation (EU) No 910/2014 | Electronic identification and trust services. Basis for Tier 4 government identity verification. |
| REF-05 | SOX — Sarbanes-Oxley Act 2002, Section 802 | 7-year document retention requirement for US public companies. Tier 3 corporate compliance. |
| REF-06 | Basel II — International Convergence of Capital Measurement, BCBS 2004 | 3–7 year data retention for banking institutions. Tier 3 financial compliance. |
| REF-07 | PCI DSS v4.0 — Payment Card Industry Data Security Standard | Data security requirements for payment processing. Tier 3 financial compliance. |
| REF-08 | HDS — Hébergeur de Données de Santé, France | Mandatory certification for hosting personal health data in France. Tier 4 healthcare compliance. |
| REF-09 | SGB V § 630f — Patientendaten, Germany | 10-year minimum retention for medical records in Germany. Tier 4 healthcare. |
| REF-10 | Code de la santé publique, Art. R1112-7, France | 20-year minimum retention for adult medical records in France. Tier 4 healthcare. |

### Standards

| Ref | Document | Relevance |
|---|---|---|
| REF-11 | ISO/IEC 27001:2022 | Information security management system standard. Tier 2 and Tier 3 compliance baseline. |
| REF-12 | ISO/IEC 27002:2022 | Controls for information security. Supplements ISO 27001 implementation. |
| REF-13 | NIST Post-Quantum Cryptography Standards, 2024 | ML-DSA-65 and related post-quantum signature algorithms. Algorithm agility design basis. |
| REF-14 | RFC 8032 — Edwards-Curve Digital Signature Algorithm (EdDSA) | Ed25519 specification. Default signature algorithm for XGen Protocol. |
| REF-15 | RFC 8037 — CFRG Elliptic Curves for JOSE | Ed25519 key representation in JSON. |
| REF-16 | FIDO2 / WebAuthn — W3C Recommendation 2021 | Hardware-bound authentication for Tier 4 high-security contexts. |

### Prior Art & Intellectual Lineage

| Ref | Source | Relevance |
|---|---|---|
| REF-17 | Matrix Specification — matrix.org/docs/spec | Event model, Room model, Space model, federation architecture. Primary technical predecessor. |
| REF-18 | Matrix State Resolution v2 — spec.matrix.org | State resolution algorithm. Referenced as known scalability limitation XGen improves upon. |
| REF-19 | Discord — Engineering Blog | Server/Community primitive, Thread design, voice channel model. Product design predecessor. |
| REF-20 | Kyberia — kyberia.sk (est. 2001) | Community governance model, forum-as-memory, identity-as-earned-capital. Cultural and philosophical predecessor. |
| REF-21 | Signal Protocol — signal.org/docs | End-to-end encryption model, double ratchet algorithm. Referenced for encryption layer (Chapter 3). |