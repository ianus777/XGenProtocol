# Appendix J — XGID: First-Class Identifiers in XGen Protocol
> **Status**: ACTIVE  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## J.1 Introduction

This appendix is the canonical expository home for **XGID** — the XGen Protocol's named type discipline for first-class identifiers. The architectural commitment is recorded in `DECISIONS.md` D-072; the terse normative statement appears in `xgen_ch3_specification.md` §3.X; this appendix is the place where the concept is explained at length, the taxonomy is enumerated, the wire-format invariance promise is unpacked, the immutability framing is grounded in construction, and worked examples of accepted scope and rejected proposals are recorded.

The discipline rests on a composition rule: **the field name carries the role, the type carries the contract** (formalised in `DECISIONS.md` D-073). XGID provides the type vocabulary; D-073 governs how that vocabulary is used at the field-name level. The two decisions land in the same Phase 1 canonical sources commit and are best read together.

This appendix complements but does not replace the load-bearing specs. Ch3's §3.X normative section is short by design; this appendix is long because the *why* of XGID matters as much as the *what*. A reader who only needs the rule reads §3.X. A reader who needs to understand or extend the model reads here.

---

## J.2 The seven flavours

XGID is not a single identifier type but a small closed family. The XGen Protocol distinguishes seven flavours of XGID at v1, organised into two families by how their string contents come into existence:

### Hash-anchored family (4 flavours)

These flavours' string contents are derived from the content of the object they identify, via cryptographic hash. The XGID is a content-binding name: given the object, the XGID is computable; given the XGID, the object's existence is referenced uniquely.

| Flavour | Identifies | Construction source |
|---|---|---|
| **EventXgid** | A single Event in the DAG | Content hash of the canonical-form Event payload |
| **SpaceXgid** | A Space (top-level container) | Content hash of the founding `state.space_create` event |
| **RoomXgid** | A Room (nested under a Space) | Content hash of the founding `state.room_create` event |
| **TrustAssertionXgid** | A Trust Assertion record | Content hash of the assertion payload |

### Principal family (3 flavours)

These flavours' string contents are derived from a cryptographic public key. The XGID is a key-binding name: it identifies a principal (an entity with signing capability) by its public key, not by any external naming authority.

| Flavour | Identifies | Construction source |
|---|---|---|
| **NodeXgid** | A Node (a server-side participant) | Ed25519 public key of the Node keypair |
| **IdentityXgid** | An Identity (a user-side principal) | Ed25519 public key of the Identity keypair |
| **AuthModuleXgid** | An Auth Module (a registered tier-verification service) | Ed25519 public key of the Auth Module keypair |

### Why two families

The split is not aesthetic; it reflects two genuinely different identity-binding strategies the protocol uses. Hash-anchored XGIDs name *immutable content*: an Event, a Space's founding event, a Room's founding event, an assertion record. Their meaning is bound to specific bytes; the XGID is a fingerprint over those bytes.

Principal XGIDs name *durable signing capability*: a keypair that authors events, signs assertions, holds protocol authority. Their meaning is bound to a private key (held by the principal) and verifiable via the corresponding public key (embedded in the XGID).

Both families share the wire-format invariances (J.5), the immutability per object (J.4), and the type representation discipline (J.6). They differ only in *where* their string contents come from. A reader writing protocol-level code does not usually need to distinguish them; a reader writing identifier-construction or identifier-verification code must.

### Why seven and not more

The seven flavours are exhaustive at v1 for the protocol objects XGen treats as first-class identifiable: Events, the two container structures (Space, Room), the trust-assertion record, and the three principal types (Node, Identity, Auth Module). Every other identifiable concept in the protocol is either a sub-axis of one of these seven (J.7), a transport-layer correlation handle that happens to carry an XGID-shaped string (J.8), or not a first-class protocol identifier at all (J.8).

The seventh flavour, **AuthModuleXgid**, was added via the promotion barrier described next: an Auth Module is a signing principal (it verifies and attests Identity tiers), so it is named by its key exactly as a Node or an Identity is — a third principal flavour, not a new family. It was promoted through DECISIONS.md **D-083** (auth-module-registry arc, AMR-D2), graduating the family from six to seven.

Adding an eighth flavour requires explicit promotion through a new DECISIONS.md entry, as the seventh was. The barrier is intentionally high: identifier vocabulary is one of the few things in a protocol that must be small and stable to remain usable across generations of implementations.

---

## J.3 Construction

The string contents of an XGID are produced deterministically from inputs the protocol already commits to. The construction rules are flavour-specific; the result is always a string that obeys the wire-format invariances (J.5).

### Hash-anchored construction

Given a protocol object O (an Event, a `state.space_create` event, a `state.room_create` event, or a trust assertion record), the corresponding XGID is computed as:

1. Serialise O to its canonical form (deterministic field ordering, deterministic encoding of primitives — the canonical form is specified per object type elsewhere in the protocol; this appendix does not re-state it).
2. Compute the SHA-256 hash of the canonical-form bytes.
3. Encode the hash as the XGID's string contents, with the flavour-appropriate URI prefix and structure.

The exact URI grammar (prefix, separator characters, length characteristics, character class) is fixed at v1 under invariance 4 (J.5). The grammar is documented at the points it is consumed (Ch3 for protocol use; per-flavour constructor documentation for code use); this appendix does not specify the grammar in normative form, because it lives in Ch3 §3.X.

**Determinism property.** Given the same input object, the hash-anchored XGID construction produces byte-identical output anywhere in the federation. This is invariance 3 (canonical form). It is what makes hash-anchored XGIDs work as content-binding names: two participants who hold the same object compute the same XGID for it, independently and without coordination.

### Principal construction

Given an Ed25519 public key K (held by a Node or by an Identity), the corresponding XGID is computed as:

1. Encode K in its canonical byte representation (32 bytes for Ed25519 public keys).
2. Encode those bytes as the XGID's string contents, with the flavour-appropriate URI prefix and structure.

The principal XGIDs do not use a hash — they use the public key directly. The construction is bijective with the public key: given the XGID, the public key can be recovered; given the public key, the XGID can be reconstructed. This is what allows signature verification to be expressed cleanly: `IdentityXgid::pubkey() -> VerifyingKey` recovers the verification key from the XGID without needing a separate lookup.

**Determinism property.** Same input key → same XGID, anywhere. This is the same invariance 3 property the hash-anchored family enjoys, just achieved by a different mechanism (encoding rather than hashing).

### What construction does not include

The construction rules deliberately do not include:

- **Random or per-instance nonces.** XGIDs are not unique-by-randomness; they are unique-by-content (hash-anchored) or unique-by-key (principal). Adding nonces would break invariance 3.
- **Wall-clock time or version-bump fields.** Time enters the protocol via event `created_at` fields and similar, but not via XGID construction. An object's XGID does not change as time passes.
- **External naming-authority lookups.** No DNS, no certificate authority, no centralised registry contributes to XGID construction. The protocol is sovereign with respect to its own identifier space.
- **Cross-flavour conversion.** A `NodeXgid` does not encode any information about an `IdentityXgid` (or vice versa). The two flavours are independent at construction; relationships between Nodes and Identities (e.g. operator-delegation chains) are encoded in event payloads, not in identifiers.

---

## J.4 Immutability

**An XGID is immutable. Once issued, the binding from XGID to object is permanent. Properties of the object MAY change via subsequent events; the XGID does not.**

This is the central immutability property. It is stated in normative form in Ch3 §3.X (short, declarative, load-bearing). This section explains *why* it holds — the property is not a policy choice but a consequence of how XGIDs are constructed.

### Hash-anchored immutability

For Events, Spaces, Rooms, and TrustAssertions, the XGID is the hash of the founding object's canonical form. The hash function (SHA-256) is deterministic and collision-resistant. Given the founding object, the hash has exactly one value. Changing the founding object — even by one byte — produces a different hash, which is a different XGID, which references a different protocol object.

There is no operation in the protocol that *modifies* a Space's founding `state.space_create` event. Subsequent events may change the Space's properties — rename it (a `state.space_metadata` update event), add or remove members (`membership.*` events), reconfigure federation (`state.federation_add` / `state.federation_remove` events), and so on — but these subsequent events have their own XGIDs (each is its own Event with its own content hash); they do not alter the founding event. The Space's XGID, derived from the founding event, is therefore permanent for as long as the Space exists in the protocol.

A reader from a Web2 background may instinctively reach for "but couldn't we just rename a Space and keep the XGID?" — and the answer is no, not because the protocol forbids it as a policy, but because the XGID is not *of* the Space's current name; it is of the Space's founding event's bytes. Renaming produces a new event with a new XGID; the Space's XGID stays the same because the founding event's bytes stay the same. The XGID is, by construction, divorced from mutable properties.

### Principal immutability

For Nodes and Identities, the XGID is derived from the public key. The public key is the protocol-level identity of the principal. Changing the public key means a different principal (a new keypair, owned by whomever holds the new private key). The XGID of a Node or Identity is therefore tied to one specific signing capability, for life.

This is not a policy decision either. A protocol that allowed Identities to "change their key" while keeping the same XGID would have to define what "same Identity" means independent of the key — which would require an external naming authority (a server that says "this XGID now means this new key"), which is the centralised identity model XGen exists to avoid. Principal XGID immutability is structural: it is the absence of any mechanism for the centralised re-binding.

Key rotation, when it becomes a feature of the protocol, will be expressed as the *retirement* of one Identity (or Node) and the *introduction* of another, with cryptographic linkage between them at the protocol layer. The two Identities have two XGIDs. Neither XGID is mutated; the relationship between them is event-recorded.

### Why immutability matters

A protocol whose identifiers can drift between releases or implementations breaks federation at every release boundary. Cached references, durable bookmarks, content-addressed storage, audit logs, and cross-implementation interoperability all depend on XGIDs meaning the same thing forever. XGen's immutability property is what makes those use cases viable.

The discipline also pays off in design conversations: "what does XGID X reference?" is a tractable question with one answer. There is no "as of when" qualifier; there is no "in which version of the protocol" qualifier; there is no "according to which authority" qualifier. The XGID names exactly one protocol object, by construction, forever.

---

## J.5 Wire-format invariance

XGen Protocol guarantees five **wire-format invariances** for XGIDs across every boundary where they cross between processes. The invariances apply to **both wire crossings** the protocol exposes:

- The **federation wire** — Node-to-Node WebSocket messages carrying Events, state operations, and federation control.
- The **AI control / batch JSONL wire** — the protocol-shaped surface between an AI driver (or batch script) and a reference implementation, documented in `docs/xgen_aicontrol_implementation.md`, Appendix F's batch reply schemas, and Ch6 §6.15.

Any boundary where XGID strings cross a process is bound by these rules. The protocol does not get to be sloppy at the implementation-protocol seam.

### The five invariances

**Invariance 1 — Field names.** The JSON field name carrying an XGID does not change between v1 and any future retrofit pass. If a v1 message carries `"event_id": "..."`, every future message at the same protocol position carries the same field name `event_id`. Renames are not retrofits; renames require explicit protocol-version negotiation.

**Invariance 2 — Field types.** The on-wire JSON type for any XGID is `string`. This holds regardless of which Rust newtype wraps it on the reference implementation side. A `NodeXgid` and a bare `String` containing the same XGID value serialise identically; they are wire-indistinguishable. This is what makes the layered-newtype Rust implementation (J.6) compatible with non-Rust XGen clients: any language that can produce and consume JSON strings can produce and consume XGIDs.

**Invariance 3 — Canonical form.** The string contents of any XGID are byte-identical when produced from the same inputs anywhere in the federation. No normalisation, no case-folding, no whitespace tolerance. Two participants who hold the same object (hash-anchored case) or the same public key (principal case) compute the same XGID string — character by character, byte by byte.

**Invariance 4 — URI grammar.** The structural shape of XGID strings is fixed at v1. This includes: the prefix (which flavour the XGID belongs to), the separator characters, the length characteristics, and the character class (which characters can appear). Any retrofit pass that changes Rust-level type representation does not change the URI grammar. Two implementations of XGen can validate XGID strings against the same grammar and reach the same accept/reject decision.

**Invariance 5 — String-equality semantics.** Two XGIDs are equal iff their string contents are equal. There is no flavour-aware comparison (a `NodeXgid` and an `IdentityXgid` with somehow-identical string contents would be equal at the string level, though by construction this cannot happen — see J.6). There are no normalisation hooks; there is no "equivalent up to case." Equality is bytes-equal-bytes, full stop.

### Why these five and not others

The five invariances are the minimum that closes the practical drift surface between v1 and any future change to the reference implementation. Other properties that *could* have been listed as invariances — XGID length, XGID character set, XGID URL-safety — are subsumed by invariance 4 (URI grammar) and therefore not listed separately. The five-item list is the smallest set that names the practical concerns; smaller lists would leave gaps, longer lists would dilute the principle.

### What the invariances do not promise

The invariances apply at v1 and through all retrofit work that lands under D-072's "XGID Adoption v1" milestone and its five subsequent Retrofit Passes. They do not foreclose future protocol versions making different choices — but those would be explicit version bumps with explicit migration paths, not silent changes. The invariances are a strong default, not an eternal lock.

### Wire crossings and the second-wire promise

The federation wire is the obvious place to specify wire invariances; it is the protocol's primary surface. Naming the AI control / batch JSONL wire as a second crossing — explicitly, equally bound by all five invariances — closes a gap that would otherwise let the AI-control surface drift while the federation wire stayed stable.

The reasoning: an AI driver that produces or consumes XGIDs needs the same guarantees a federating Node needs. A driver that submits an event over `--batch`, observes the resulting `EventAccepted` reply, and later references the same Event by XGID in a downstream command must see the same XGID byte-for-byte at every step. A driver-side implementation in any language must be able to interoperate with the reference implementation's batch surface without re-implementing XGID normalisation, equality, or grammar.

The promise is named here, in the canonical XGID appendix, rather than only in the AI-control documentation, because invariance is one promise across both surfaces — not two independent promises that happen to coincide.

---

## J.6 Type representation (Rust reference implementation)

XGen's Rust reference implementation realises XGID through a **layered newtype** discipline. This section documents the representation; it is the reference implementation's choice, not the protocol's mandate (per J.5 invariance 2, future implementations in other languages MAY choose differently).

### The layered newtype

The base type is a single newtype wrapping a `String`:

```rust
pub struct Xgid(String);
```

Above this base, seven flavour wrappers exist, one per flavour from J.2:

```rust
pub struct EventXgid(Xgid);
pub struct SpaceXgid(Xgid);
pub struct RoomXgid(Xgid);
pub struct TrustAssertionXgid(Xgid);
pub struct NodeXgid(Xgid);
pub struct IdentityXgid(Xgid);
pub struct AuthModuleXgid(Xgid);
```

Each flavour wrapper implements `Deref<Target = Xgid>`, allowing read-only access to the underlying string through standard Rust ergonomics. Each implements `Display`, `Debug`, `Eq`, `Hash`, and `Clone`. Each is serde-transparent: it serialises to a JSON string (its underlying bytes) and deserialises from a JSON string, indistinguishable on the wire from a bare `String`.

### Why layered rather than flat

Two alternatives were considered:

- **Flat — a single `Xgid` type with no flavour wrappers.** Rejected: would lose the type-system enforcement that a function parameter expecting an `IdentityXgid` cannot accidentally receive a `NodeXgid`. The whole point of typed identifiers is that miscalls become compile errors; a flat type loses that.
- **Disjoint — seven wrappers with no common base.** Rejected: would force common operations (Display, Debug, Eq, Hash, Clone) to be implemented seven times. Would also force operations that genuinely don't care about flavour (e.g. logging a generic XGID for tracing) to enumerate all seven flavours at every use site.

The layered approach gets both: type-level distinction at flavour boundaries, code-level uniformity at the shared-operation level. The `Deref<Target = Xgid>` chain means flavour wrappers behave as Xgids when treated read-only, and behave as their specific flavour when typed explicitly.

### The XgidLike trait

A trait `XgidLike` is defined alongside the types. It exposes the operations any XGID supports (read the string contents, compute equality, hash, format for display). Generic code that genuinely needs to operate over "any XGID" can take `T: XgidLike` instead of enumerating flavours.

**Sparingly used.** Most code is explicit about flavour because most code is explicit about which protocol object it operates on. `XgidLike` is reserved for code that is genuinely generic — for example, a trace event that logs "an XGID got rejected" without caring which flavour. Overuse of `XgidLike` would defeat the purpose of typed flavours by silently re-flattening them.

### Flavour-specific constructors

Each flavour wrapper provides flavour-appropriate constructors that hide the double-wrap. Examples (illustrative, not exhaustive):

```rust
impl EventXgid {
    pub fn from_event(event: &Event) -> Self { ... }
    // hash the event's canonical form, wrap result
}

impl NodeXgid {
    pub fn from_pubkey(pk: &VerifyingKey) -> Self { ... }
    // encode the pubkey, wrap result
    pub fn pubkey(&self) -> Result<VerifyingKey, XgidDecodeError> { ... }
    // decode the pubkey from the string
}
```

The constructor surface is flavour-specific because the *meaning* of construction is flavour-specific. Hash-anchored flavours' constructors take the object to be identified; principal flavours' constructors take the public key. Construction is never "give me a Foo XGID from this arbitrary string" — that would let invalid XGIDs into the type system.

### Flavour-specific methods

Principal flavours carry methods that exploit their construction source. `NodeXgid::pubkey()`, `IdentityXgid::pubkey()`, and `AuthModuleXgid::pubkey()` recover the public key from the XGID, returning the type the verification API expects. Hash-anchored flavours carry methods that exploit theirs — for example, helpers that verify a candidate XGID matches a given canonical-form payload.

These methods exist because the construction-source data is structurally recoverable from the XGID. A `NodeXgid` is not just any string; it is a string whose bytes encode a specific public key. The type system makes that structural fact accessible via methods, rather than requiring every caller to re-implement the encoding rules.

### Cross-flavour conversion (not provided)

The flavour wrappers are deliberately not interconvertible at the type level. There is no `From<NodeXgid> for IdentityXgid`. Code that genuinely needs to construct one flavour's XGID from another's string content must do so explicitly: extract the base `Xgid` via `Deref`, examine it, construct the target flavour from its construction source. The friction is intentional. Silent flavour drift at use sites is what the newtype discipline exists to prevent.

---

## J.7 Sub-axes and refinements

The seven flavours from J.2 are exhaustive at v1 as *top-level* identifier types. The protocol does, however, recognise *sub-axes* within some flavours — narrower categorisations that are useful in specific contexts but do not warrant first-class type status.

### Ephemeral Event XGIDs (session_id)

A session_id identifies a single client-Node connection session. It is constructed as an Event-flavour XGID (its bytes are hash-anchored over the session's establishing event payload), but its lifecycle is ephemeral: it is meaningful only while the session is active, and it is not referenced after the session ends.

Phase 1 walkthrough placed session_id as a sub-axis of Event XGIDs, not a new flavour. The reasoning: a session_id is structurally an Event XGID (same construction, same wire properties, same immutability per session); its ephemeral nature is a *lifecycle* property, not a *type* property. Marking session_id as `EventXgid` in the type system, and tagging it as session-shaped via the field name (`session_id`, by D-073) and the surrounding code context, is enough.

### Hash-anchored XGIDs without composite structure (trust_assertion_id)

A trust_assertion_id was briefly considered as a "composite" XGID — one that encodes both an asserter Identity and a subject in its structure. Phase 1 walkthrough rejected the composite framing. `trust_assertion_id` is a plain hash-anchored XGID over the assertion payload's canonical form. The (asserter, subject) pair lives in the payload, not in the identifier. Composing structure into XGIDs would break invariance 4 (URI grammar) by introducing flavour-specific internal structure.

### Why sub-axes don't get new flavours

Promoting a sub-axis to a new flavour would:

- Expand the seven-flavour family unnecessarily (J.2's exhaustiveness argument).
- Introduce type-system friction where the existing flavour already handles the cases correctly.
- Risk breaking invariance 4 by suggesting that different sub-axes have different URI grammars (they don't; they share the parent flavour's grammar).

Sub-axes are recorded here for completeness and for design reference. They do not change the type system at v1.

---

## J.8 What XGID is and is not

The boundary cases below are explicitly documented because each was considered during the Phase 1 walkthrough and decided either to be in or out of XGID scope. Recording them here prevents re-litigation in future design conversations.

### Not XGIDs — explicit exclusions

**Wire-envelope correlation handles.** M6 (new) Phase 2's `event_id: Option<String>` field on `TransportMessage` is a transport-layer correlation handle that lets an originator correlate `EventAccepted` or `Error` signals back to events it sent. By construction, its string value is byte-equal to the corresponding Event XGID (when the field is populated). But it is *not* itself an Event XGID — it is a transport-layer field with a different lifecycle (per-message, per-session) and a different purpose (signal correlation, not protocol-object identification). The type-level separation prevents miscalls between protocol-layer code (which operates over `EventXgid`) and transport-layer code (which operates over `Option<String>` on the envelope).

**Error codes.** Numeric error codes (`4002`, `4006`, `4007`, etc.) and string-tagged error codes are not XGIDs. They are a separate identifier space (the protocol's error taxonomy), with different invariance rules, different lifetimes, and different semantics.

**Config field names and in-memory handle types.** A field like `[sync].batch_size` is a configuration key, not an XGID. A map key in `FederationPeerSenders` is structurally a `NodeXgid` (the keys' string values *are* Node XGIDs), but the map itself, the map's keys-as-handles in code, the lookup operations against the map — these are in-memory data structures, not XGIDs.

**File paths, log line tokens, debug formatters.** XGID types appear in these surfaces via `Display` or `Debug` — but the paths, tokens, and formatters themselves are not XGIDs. A log line containing an XGID string is a log line; it is not an XGID structurally.

**Bootstrap discovery URIs.** A bootstrap node's address (e.g. `wss://bootstrap.example.org/`) is an operational network address, not a protocol-object identifier. Bootstrap addresses route to Nodes; the Nodes themselves have `NodeXgid` identifiers. The two surfaces are distinct and should not be conflated.

### Boundary cases that ARE XGIDs

For completeness, the cases below were considered and confirmed as XGIDs:

- **Event references in `prev_events` arrays.** These are EventXgids, by direct construction.
- **Identity references in event `sender` fields.** These are IdentityXgids.
- **Space and Room references in event headers** (`space_id`, `room_id`). These are SpaceXgids and RoomXgids.
- **Node references in federation events** (`peer_node_id`, `introducer_node_id`, etc.). These are NodeXgids.
- **Trust assertion references** in any context where one assertion references another (rare at v1, structurally supported). These are TrustAssertionXgids.

The pattern: if a field's value references a protocol-object-identifier, it is an XGID, typed with the appropriate flavour, named with the role per D-073.

---

## J.9 Worked examples of rejected proposals

This section records design proposals that were considered and rejected, with the reasoning preserved so future design conversations do not have to re-derive the rejection. Two proposals are documented here; a third slot is intentionally left open in case future walkthrough surfaces another wire-invariance-targeting proposal worth capturing.

### J.9.1 Rejected — "Use the in-memory handle type as the wire type"

**The proposal.** Have the Rust newtype serialisation expose flavour information on the wire — for example, by serialising `EventXgid` as `{"flavour": "event", "value": "xgen:event:..."}` rather than the bare string `"xgen:event:..."`. Rationale offered: "the wire would carry richer type information; receivers could enforce flavour at the deserialisation boundary; the protocol would gain end-to-end type safety."

**The rejection.** This proposal breaks invariance 2 (field types must be `string`) and invariance 4 (URI grammar — wrapping in an object changes the structural shape).

The argument that "the wire would carry richer type information" misframes the situation. The wire already carries flavour information — it carries it *in the field name* (per D-073). A field named `event_id` carrying a string `"xgen:event:..."` already tells the receiver everything: the role (event_id) is encoded in the field name, the flavour (event) is encoded in the URI prefix, the value is the string. Adding object-wrapping wire shape adds redundancy, not safety.

The end-to-end type safety the proposal seeks is real, but lives at a different layer. It lives in the *implementation-side* type system (J.6's layered newtypes), not in the wire. A receiver that deserialises a `string` into an `EventXgid` newtype at the boundary gets the type-safety benefit; the wire stays minimal. This is the standard "thick boundary, thin wire" pattern in well-designed protocols, and XGen follows it.

The further reason: changing the wire shape to carry flavour information would force every non-Rust implementation to either parse the object wrapper (extra cost for no benefit) or be incompatible with the reference implementation (federation-breaking). Invariance 2's commitment to `string` is precisely what keeps XGen accessible to clients in any language.

**The decision.** Wire stays string; flavour lives in the reference implementation's type system; the field name carries the role. Layered newtypes (J.6) are the implementation strategy, not the wire format.

### J.9.2 Rejected — "Shorten the URI grammar for compactness"

**The proposal.** Compress the XGID URI grammar to save bytes on the wire — for example, by dropping the flavour prefix (assuming the field name disambiguates), or by shortening the hash representation to fewer characters (assuming a shorter hash is collision-resistant enough for practical purposes), or by removing separator characters.

**The rejection.** This proposal breaks invariance 3 (canonical form) and invariance 4 (URI grammar).

The compactness argument is genuine — a federation that exchanges millions of events does pay a real bandwidth cost for verbose identifiers. The cost is bounded, though, and the proposed savings are dwarfed by what the change would cost:

- **Invariance 3 (canonical form) breakage.** If implementations are free to "compress" XGIDs locally and "expand" them on the wire, two implementations holding the same object would produce different XGIDs depending on local compression rules. Federation would break the moment one implementation expanded and another did not. The invariance exists precisely to prevent this.
- **Invariance 4 (URI grammar) breakage.** Shortening hash representation reduces collision resistance. SHA-256 in full has well-understood properties; truncated SHA-256 has different (weaker) properties. Even a small truncation changes the security analysis. A federation that ran on truncated XGIDs would have a smaller hash collision domain, and would have to formally re-analyse cryptographic assumptions.
- **Future-compatibility cost.** A "compressed v1.1" URI grammar that differed from "verbose v1" would force every implementation to support both grammars, complicating parsing, serialisation, equality, and storage. The compatibility surface would grow without bound as more "optimisations" landed.

The bandwidth saved is also small in practice. Hash-anchored XGIDs are 64+ characters; total event payloads are typically far larger (multi-kilobyte for non-trivial content). XGID bytes are a single-digit percentage of event size in realistic traffic; saving half of them saves single-digit-percent total bandwidth, while paying compatibility and security costs that compound forever.

**The decision.** URI grammar fixed at v1; bandwidth savings are not a sufficient reason to break invariance. Future protocol versions may revisit the grammar with explicit version bumps; v1 holds.

### J.9.3 — Reserved slot

A third worked-example slot is intentionally left open in this appendix. Future walkthroughs that surface non-obvious wire-invariance-targeting proposals can document the rejection here. The reservation prevents the temptation to force a third example into v1 prematurely; two well-chosen examples teach the principle better than three forced ones.

### See also — boundary clarifications

Three further design-decision boundary cases that *could* have appeared as rejected proposals appear earlier in this appendix as scope clarifications rather than rejections, because they are about defining XGID's scope rather than rejecting a wire-affecting change:

- **session_id as a new flavour** — clarified at J.7 (sub-axis of Event XGID, not a new flavour).
- **trust_assertion_id as a composite XGID** — clarified at J.7 and J.2 (plain hash-anchored, not composite).
- **M6 (new) `event_id` envelope field as an Event XGID** — clarified at J.8 (transport-layer correlation handle, by construction byte-equal but type-level distinct).

---

## J.10 Worked examples of immutability

This section records example proposals that conflict with the immutability property (J.4), preserved so future design conversations do not have to re-derive why they fail. One example is documented here; the section is open to additions if further worked examples surface.

### J.10.1 Rejected — "Rename a Space, keep its XGID"

**The proposal.** Allow a Space's human-readable name to change while preserving its XGID. Rationale offered: "users rename things; their bookmarks, references, and shared links should not break when a Space is renamed."

**The rejection.** This proposal misunderstands what a Space's XGID is.

A Space's XGID is not the Space's name. It is the hash of the Space's founding `state.space_create` event. Renaming a Space is not a modification of the founding event; it is a separate event (a `state.space_metadata` update event, or similar) that records "the name property changed at this point." The founding event is unchanged; its hash is unchanged; the Space's XGID is unchanged.

In other words: the user's instinct — bookmarks should not break when a Space is renamed — is exactly what XGID's immutability already delivers. Bookmarks reference the Space by XGID, not by name. After a rename, the XGID still references the same Space, with the same founding event, with the same history. The display name has changed; nothing identity-bearing has.

The proposal's confusion is over which thing the XGID *is*. Once that is clarified, the proposal dissolves: the protocol already supports renaming Spaces without breaking references, by construction. The XGID and the display name are simply different things, and only the display name is mutable.

**The decision.** Renaming is a property update; XGIDs are not properties; the two are not in conflict and never need to be reconciled. Reference: J.4 immutability framing.

### J.10.2 — Open slot

The section is open to further worked examples if future walkthroughs surface them. The pattern is the same as J.9: document the proposal, explain why immutability makes it impossible (not just inadvisable), preserve the reasoning.

---

## J.11 Adoption discipline and retrofit

This section summarises the adoption discipline for XGID v1. The authoritative source is `DECISIONS.md` D-072; this section is a brief overview for readers who land in Appendix J first.

### The principle

> *"XGID Adoption v1 ships the types and adopts them in new code. Retrofitting existing XGID-string fields is staged into subsystem-scoped retrofit milestones. The codebase MAY carry mixed discipline transitionally; every new field, new signature, and new trace event field MUST use XGID types from this milestone forward."*

### Shape γ + ASAP

The v1 milestone ships the type vocabulary, the `xgen-common` implementations, the wire-invariance promise, and Phase 7.5's `introducer_node_id: NodeXgid` (the inaugural production use). Existing String-typed XGID fields across the codebase are NOT retyped in v1; they convert pass-by-pass over the five subsequent Retrofit Passes:

- **Pass 1** — `xgen-common` core types
- **Pass 2** — `xgen-core` validation/dispatch/pending-buffer
- **Pass 3** — `xgen-node` federation/fanout/app surfaces
- **Pass 4** — `xgen-client` ops/AI-behaviour/batch, closing the AI control / batch JSONL documentation surface
- **Pass 5** — test fixtures, helpers, trace events, remaining surfaces

The five passes land in ROADMAP.md Near future immediately after v1 ships — ASAP discipline, not Far future. After Pass 5 closes, the "mixed discipline transitionally" clause no longer applies; the codebase has uniform XGID type discipline end-to-end.

### Why staged retrofit and not a single mega-milestone

A single milestone that retyped every XGID-carrying field in the codebase would either delay v1 by months or ship a v1 with cut corners. Staged retrofit is honest about the cost: v1 ships the discipline and the types; the codebase reaches them through subsystem-scoped passes whose scope can be reviewed and verified pass-by-pass. This is the same principle as D-065 (honest behaviour over polite behaviour) applied to a project-management surface: the project does not pretend the retrofit is free or instant; it states the cost as a real and named project phase.

---

## J.12 Cross-references

| Reference | Relationship |
|---|---|
| `DECISIONS.md` D-072 | Architectural commitment for XGID Adoption v1. The authority for what this appendix expounds. |
| `DECISIONS.md` D-073 | The field-name-vs-type composition rule that governs how XGID types are used at every field. Echoed once in this appendix's introduction (J.1) with a pointer here. |
| `DECISIONS.md` D-065 | Sibling principle (honest behaviour over polite behaviour). Cited at J.5 (wire-format honesty) and J.11 (retrofit-discipline honesty). |
| `DECISIONS.md` D-069 | Canonical-document rule. This appendix is the canonical home; Ch3 §3.X carries the terse normative form; D-072 carries the architectural commitment. Three surfaces, no duplication. |
| `DECISIONS.md` D-070 | The envelope-level `event_id: Option<String>` correlation handle. Discussed at J.8 as a boundary case (byte-equal to an Event XGID at the string level, but type-level distinct as a transport-layer handle). |
| `DECISIONS.md` D-071 | Subsystem-audits-precede-dependent-milestones discipline. Phase 7.5's audit-flavoured design pattern produced the worked precedent (`introducer_node_id`) that motivated XGID Adoption. |
| `docs/xgen_ch3_specification.md` §3.X | The terse normative section. The load-bearing single sentence on wire-format invariance, the load-bearing single sentence on immutability, the forward-reference to this appendix for the long-form. |
| `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 | Originating precedent. The `introducer_node_id` field naming was Joe-locked at §5.6 with the principle wording "the field name carries the role, the type carries the contract." D-073 promotes that one-off forward-aware decision into a project-wide discipline; D-072 promotes the implied XGID typing into a v1 milestone. |
| `docs/xgen_aicontrol_implementation.md` | AI control / batch JSONL wire surface. Carries a v1 pointer noting XGID discipline applies; full annotation pending Retrofit Pass 4. |
| `docs/xgen_appendix_f_en.md` | Batch reply schemas + Node-side identifier references. Carries a v1 pointer noting XGID discipline applies; full retype pending the appropriate Retrofit Pass. |

---

*End of Appendix J.*  
