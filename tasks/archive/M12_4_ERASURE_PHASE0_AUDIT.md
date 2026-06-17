# M12.4 — Erasure: D-071 Phase-0 audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & verdict

The Clair-authored D-071 Phase-0 grounding for **M12.4 — erasure**, the final M12 sub-arc
(`tasks/M12_ATTACHMENTS_DESIGN.md` M12-D10). M12.4 is the **first implementation slice of D-088's
content-erasure axis**, scoped to attachments: the `message.redact` content applier (F2a, net-new),
the F2b sender-`Retention` read (M12's first production `Retention` reader), crypto-shred
destroy-to-erase (D3-gated), and the reserved WORM/legal-hold operator hook (F7, mark not build).

This is an **audit-only** deliverable (Clair's seat) — grounding the five M12-D10 scope items to
file:line on `main`, surfacing findings + forks with a recommendation each, and a GO/NO-GO. No
canonical-record flips (CLAUDE/JOURNAL/ROADMAP); those are Chat's design-bridge + close-bridge.

**VERDICT: GO.** M12.4 has a genuinely **buildable + witnessable** slice (the `message.redact`
event + content schema + a node-side blob-bytes erasure side-effect + the F2b `Retention` refusal +
free federation propagation), distinct from Arc I (J-253), which was design-only architecture.
The slice realises D-088's "implementation-within-frame" erasure on the **mutable** content layer
(the blob store), while the **DAG-resident** crypto-shred residue (the `message.file` event's
existence + its plaintext descriptor `key` + any `message.text` body) stays **D3-gated** exactly as
M12-D10 + D-088's cascade say. The honest boundary is crisp and defensible. This is the M12.3-style
"build the mechanism, fence the crypto-maturity" pattern, **not** the Arc-I design-only pattern.

**Grounded against `main` @ `76b93e7`** (tree clean, in-suite 1448/0). Every claim below was read
in production code this session (D-078); the line numbers held.

---

## §2 The central finding — the buildable vs D3-gated split

D-088 (the locked erasure architecture, J-253) resolves content-erasure by **crypto-shredding over
the encryption boundary**: the immutable DAG retains ciphertext, key-destruction makes content
unrecoverable without mutating the log. **The decisive M12.4 fact is that attachments split the
content across two stores of opposite mutability:**

| Content | Lives in | Mutable? | Erasure mechanism | M12.4 maturity |
|---|---|---|---|---|
| Blob **ciphertext bytes** | `blobs_dir` content-addressed store (`BlobStore`), **separate from the DAG** | **Yes** (a file per blob) | **delete the bytes** — real, complete erasure of that node's copy | **Buildable now** (add `BlobStore::delete`) |
| `message.file` event: existence + descriptor (`filename`/`mime`/`size`) + the **plaintext per-blob `key`** (R-1) | the immutable signed DAG event | **No** (append-only, signed) | crypto-shred (zero the key at rest) — needs the descriptor inside `enc:` first | **D3-gated** |
| `message.text` body (if present) | the immutable signed DAG event (plaintext today, R-1) | **No** | crypto-shred over the `enc:` envelope | **D3-gated** |

**The asymmetry is load-bearing and favourable.** For *text*, erasure is impossible without
crypto-shred (the content is DAG-immutable → only key-destruction works → D3). For *blobs*, the
ciphertext bytes are NOT in the DAG — they sit in a mutable content-addressed store — so **deleting
the bytes is a real, complete erasure of that node's copy of the content**, achievable now, no
crypto needed. Once the bytes are gone, the plaintext descriptor `key` (still in the immutable DAG)
**decrypts nothing** — so for blobs the residue that survives is only **metadata / existence**
(the event itself, the filename/mime/size, the now-useless key). That residue is the D3 crypto-shred
target; the *content* is genuinely erased by the byte-delete.

→ **M12.4 builds the F2a blob-bytes erasure + F2b refusal + redact propagation (the real GDPR win on
the mutable layer); it marks-and-reserves the DAG-resident crypto-shred (text body + descriptor
existence) to D3.** This is GO-able, honest, and is exactly what M12-D10 scoped.

---

## §3 Grounding ledger (seams confirmed to file:line, `main` @ `76b93e7`)

| # | Seam | Location | M12.4 relevance |
|---|---|---|---|
| G1 | `EventType::MessageRedact` → `"message.redact"`; validation-wired (round-trip, from_str) | `xgen-common/src/wire.rs:36,173,267,739` | the kind exists; everything else is net-new |
| G2 | redact's **only** code today: permission gate `MessageRedact => SendMessages` + `event_room_permission`/`check_permission` arms | `xgen-core/src/message/exchange.rs:792,846` | **no content schema, no builder, no apply arm** (F2a all net-new) — A-01 |
| G3 | `message.*` (incl. redact) **does not apply to `SpaceState`** — falls to `_ => Ok(())` | `xgen-core/src/space/state.rs:655` | no node-side message store to tombstone; the tombstone is a client display concern — A-02 |
| G4 | `BlobStore`: `new`/`put`/`get`/`contains` — **no delete/remove/erase** | `xgen-core/src/blob_store.rs:97-156` | the load-bearing missing primitive (`BlobStore::delete`) — A-03 |
| G5 | `IdentityRecord.trust_assertion: Option<serde_json::Value>` (full assertion stored) | `xgen-core/src/identity/registry.rs:45` | the sender's assertion → `Retention` is **reachable at apply** via `IdentityRegistry::get` — A-04 |
| G6 | `Retention { Erasable, Retained }`, read via `TrustClaims.module_policy().erasability.retention`; doc: *"expression only … the enforcement … is the deferred D3-gated consumer"* | `xgen-common/src/trust_assertion.rs:155-194,222-229` | the F2b read target; **zero production readers** today — A-04 |
| G7 | **zero** production readers of `module_policy()`/`erasability`/`Retention::` outside the def file (+ tests) | (grep, empty) | F2b = M12's first `Retention` reader (J-380/J-387 hold) — A-04 |
| G8 | per-blob `key` rides as **plaintext** in the `message.file` descriptor content (R-1: descriptor plaintext at text-maturity; `enc:`-wrap = shared text D3 cutover) | `tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` §3 R-1; descriptor V3 | crypto-shred of the DAG residue is **D3-gated** — A-05 |
| G9 | `envelope_with_destroyed_key` demonstrates the crypto-shred **effect** at per-message granularity; doc: *"the destroy-to-erase storage operation … is the deferred erasure-impl arc per the D-088 cascade"* | `xgen-core/src/encryption/client_mls.rs:203-224,339-344` | confirms the destroy-to-erase *storage op* (and the descriptor `enc:`-wrap) = D3 — A-05 |
| G10 | node processes every event through `process_inbound` → `dispatch_event`; `message.*` is validated + stored + fanned out (no apply arm) | `xgen-node/src/app.rs:3009`; `xgen-core/src/node/runtime.rs:1001` | a redact's node side-effect (blob-delete) is a net-new hook here, sibling to M12.3's `BlobUploadEnd` `store.put` — A-06 |
| G11 | a `message.redact` rides `apply_federation_push` like any DAG event (F-1 push; F-3/F-5 guards unchanged) | `xgen-node/src/app.rs:2028,2445,2612`; `federation_session.rs` | federation propagation is **free** — A-06 |
| G12 | **B caches federated blobs**: on a client↔node fetch miss B fetches from the Space home and `store.put(&bytes)` | `xgen-node/src/app.rs:1945` (also `:1884`) | a redact reaching B must delete **B's cached copy** — the concrete federated fork — A-06 |
| G13 | `BlobStore::new(&blobs_dir)` is a per-op thin handle (instantiated at use sites, not a long-lived owned object) | `xgen-node/src/app.rs:1883,1917,2639` | `BlobStore::new(&blobs_dir).delete(blob_ref)` lands cleanly at the redact hook |
| G14 | content-addressed store **dedups** identical blobs (idempotent `put`, one file per hash) | `xgen-core/src/blob_store.rs:120-128` | a blob may be referenced by >1 message → naive delete-on-redact is a **shared-blob hazard** — A-09 |
| G15 | D-088 (locked, J-253): content = crypto-shred over PG-05 boundary; *"erasure enrichments (display-layer scrubbing) are implementation-within-frame on the rebuildable materialization layer"*; T4 = "destruction of no record … Art.17(3)"; T2/T3 = module-declared `Retention` on the Trust Assertion | `DECISIONS.md` D-088 | M12.4 = the first impl slice of this; F2a tombstone = the "implementation-within-frame" display layer |

---

## §4 Findings (M12.4-A-01 … A-09)

- **A-01 — `message.redact` is a bare validated kind (F2a all net-new).** The event type exists +
  is validation-wired (G1), but it has **no content schema, no builder (`build_message_redact_event`
  absent), and no apply/side-effect arm** (G2). M12.4 builds: the redact content schema (what it
  targets), the builder, and the node-side erasure hook. Matches J-380's catalogue.

- **A-02 — messages don't materialize node-side; the redact's two jobs split.** `message.*` never
  enters `SpaceState` (`_ => Ok(())`, G3) — messages ride the DAG + fanout and are materialized
  **client-side** (history replay). So a redact has two distinct effects: **(a) the tombstone** =
  a client display concern (the client suppresses the target on seeing the redact — D-088's
  "implementation-within-frame on the rebuildable materialization layer"); **(b) the blob-bytes
  erasure** = a node-side side-effect on the mutable blob store (the real content erasure). There
  is **no node-side message store to tombstone** — the node's *erasure* job is the blob delete.

- **A-03 — `BlobStore` has no delete primitive.** `new`/`put`/`get`/`contains` only (G4). The
  load-bearing net-new piece is `BlobStore::delete(blob_ref)`. Because blobs are **not** DAG-immutable
  (G3 vs the content-addressed store), deleting the ciphertext bytes is a **real, complete erasure**
  of that node's copy of the content — the favourable asymmetry of §2.

- **A-04 — sender `Retention` is reachable; zero readers (F2b is the first).** The full Trust
  Assertion is stored on `IdentityRecord.trust_assertion` (G5), reachable at apply via
  `IdentityRegistry::get(sender)`; `Retention {Erasable, Retained}` reads through
  `TrustClaims.module_policy().erasability.retention` (G6). **Zero production readers exist** (G7) —
  the types' own doc-comments mark them "expression only … the deferred D3-gated consumer." So F2b
  needs **no new storage plumbing** — just a (lenient) parse + read. M12.4 is that first consumer.

- **A-05 — crypto-shred destroy-to-erase is genuinely D3-gated.** The per-blob `key` rides as
  **plaintext** in the immutable signed `message.file` descriptor (R-1, G8); you cannot destroy a
  key sitting in plaintext in an append-only signed event. `envelope_with_destroyed_key` (G9)
  demonstrates the crypto-shred *effect* but its own doc states the **destroy-to-erase storage
  operation** (zeroing the wrapped key at rest) **+** the descriptor `enc:`-wrap are "the deferred
  erasure-impl arc per the D-088 cascade." → M12.4 **cannot** crypto-shred the DAG residue; it
  marks-and-reserves it to D3 (the shared text-path `enc:` cutover, M8.7 S+L). **For blobs this is
  moot for the content** (byte-delete erases it regardless; the plaintext key then decrypts nothing)
  — only the DAG **metadata/existence** residue waits on D3.

- **A-06 — federation propagation is free; B caches.** A `message.redact` rides `apply_federation_push`
  like any DAG event (G11); B caches federated blobs via `store.put(&bytes)` on a fetch miss (G12).
  So the federated erasure = **the redact propagates eagerly; each Space home runs the same
  blob-delete hook on apply** → eventual cross-home byte erasure. **Honest boundary:** an offline /
  unreachable home lags until it reconnects + applies the redact; and the DAG residue (existence +
  plaintext key + any text body) persists everywhere until D3 crypto-shred.

- **A-07 — F7 WORM/legal-hold stays a reserved hook.** D-088 + M12-D6 reserve T4 retain-and-produce
  (the producibility vault) to the **operator/module layer**. M12.4 builds only the **protocol-layer
  refusal** (Retained → don't erase = the "ciphertext durability floor + erasure refusal" half of
  M12-D6); it does **not** build the vault. Confirmed: reserve, don't build.

- **A-08 — convergence hazard on a `Retention`-gated *admission* (the INV-EXP / M8.6 lesson).** A
  redact is a valid signed event; if the node **rejected** it on a tier-conditioned local read of
  the redactor/author's `Retention`, two nodes could disagree (A accepts, B — with a different /
  absent stored assertion — rejects) → DAG divergence, exactly the INV-EXP origin-gating /
  forced-expiry family. → the redact event MUST converge everywhere; only the **erasure side-effect
  (the blob delete)** may be `Retention`-gated. Shapes FK-3.

- **A-09 — shared-blob refcount hazard (content-addressed dedup).** The store dedups identical blobs
  (G14): two messages attaching identical bytes share one blob file. A naive delete-on-redact would
  erase a blob **still referenced** by another live (non-redacted) message. M12.4 must address this
  (refcount / reference-scan / accept-and-flag). The M12.1 self-thread single-reference case is the
  common path; cross-message dedup is the edge. A genuine Phase-0 catch.

---

## §5 Forks (FK-1 … FK-8) with recommendations

**FK-1 — redact content schema (what it references).**
(a) `{ target_event_id }` only — the node resolves the blob_refs from the target `message.file`'s
descriptor (plaintext-readable per R-1; single source of truth). (b) `{ target_event_id, blob_refs:
[...] }` — the redact restates the blobs.
**Rec (a)** — the descriptor is the authoritative blob list; making the client restate it invites
drift. The node reads the target event from its EventStore → descriptor → blob_refs. Surface for the
design lock (Joe-lock the exact field set).

**FK-2 — whose `Retention` does F2b read? (load-bearing semantic).**
(a) the **original content author** — the *target* `message.file`'s sender (whose verification
strength governs whether *their* content is erasable). (b) the **redactor** — the redact event's
sender.
**Rec (a)** — D-088 ties retention to the **record**: "T4 = destruction of no record at all … under
Art.17(3)"; the content author's tier governs *their* content's erasability (the GDPR-faithful
reading; "read-sender-retention" = the original sender). (b) answers a different question (may this
principal perform erasure) which is the permission gate (already `SendMessages`/moderation, G2). This
is the most load-bearing ambiguity in the arc — **flag for explicit Joe-lock at design.**

**FK-3 — where does F2b bite? (convergence-critical, per A-08).**
(a) admission-reject the redact event when Retained [**DIVERGENCE HAZARD**]. (b) accept + store +
fanout the redact (it's a valid signed event), and gate **only the blob-erasure side-effect** on
`Retention` (Retained → keep the bytes = the legal-hold floor; Erasable/absent → delete).
**Rec (b), strongly** — the INV-EXP / M8.6 lesson is direct: tier-conditioned *admission* gates
diverge; *side-effect* gates don't. The redact converges everywhere; the tombstone (display) still
applies; the legal-hold floor holds the **bytes**. A real D-065 catch — surface it as the spine.

**FK-4 — F2a scope: tombstone-only-lean vs delete-the-blob-bytes (exceeds the literal F2 lock).**
The brief F2 locked "tombstone-only lean" — born of the append-log constraint (you cannot delete a
DAG event → you tombstone it). But blobs are **not** in the DAG (§2), so blob-delete is possible and
a real GDPR win the append-log constraint never blocked.
**Rec: do both** — tombstone the DAG event (can't erase it; that's D3) **and** delete the blob bytes
(can erase them now). Faithful to F2a's *spirit* ("tombstone what you can't erase, erase what you
can"). Surface explicitly because it **exceeds the literal "tombstone-only" lock** — Joe-confirm.

**FK-5 — node redact-hook location (mechanical).** The blob-delete side-effect lands in
`process_inbound` after the redact is validated + stored, sibling to M12.3's `BlobUploadEnd` →
`store.put` (G10/G13). **Rec:** confirm-at-build; light, no design lock needed beyond placement.

**FK-6 — client-side tombstone display: M12.4 scope or display-polish-deferred?** The redact event +
node blob-erasure + F2b refusal are the protocol mechanics. The client display-suppression (suppress
the target message on seeing the redact) is a thin read.
**Rec:** build the mechanics in M12.4; **include** the minimal client tombstone (a redacted message
must not render its descriptor/blob) as the user-visible witness; richer UI deferred. Joe's call on
how thin.

**FK-7 — reject/refusal wire code.** A redact-refused-on-Retained (FK-3b: the side-effect is
suppressed) — does the redactor get a typed signal, or is it silent (the event converges, the bytes
just stay)? And a hash/target-not-found needs a code.
**Rec:** domain 10 (attachments/blobs) has 10004+ free — e.g. `10004 erasure_refused_retained` for
the F2b floor + reuse `10001 blob_hash_mismatch`. **Re-grep the register at build** (RC-F-01 / M10.1
collision discipline) before emitting. Surface the band choice.

**FK-8 — shared-blob refcount (per A-09).** (a) refcount blobs (put increments, delete decrements,
erase at zero). (b) on redact, **scan** the Space's live (non-redacted) `message.file` events for any
other reference before deleting. (c) accept-the-hazard, flag it (M12.1 self-thread is single-ref;
dedup-across-messages is the edge).
**Rec (b) or (c)** — (a) adds persistent refcount state to a deliberately-stateless content store;
(b) is O(events) but correct and stateless; (c) is honest for the M12.4 self/DM-first surface with a
named follow-on. Surface — Joe picks the rigour level.

---

## §6 M12-D6 — DECISIONS.md promotion candidate (standing flag)

M12-D6 ("E2E universal at the protocol layer; T4 retain-and-produce reserved to operator/module;
**Retained = ciphertext durability floor + erasure refusal**") is the protocol-wide invariant that
**M12.4 actually exercises** — F2b is its first enforcement, the blob-delete + refusal is its first
mechanism. It is principle-shaped and past the recurrence bar (it reinforces **AH-D1** (the
`erasing_wrapped_key_defeats_epoch_holder` crypto-shred invariant) + **D-088** (the erasure
architecture) — a three-instance lineage). M12.4 is the natural arc at which to author the promotion:
the principle stops being a design-doc lock and becomes live enforced code.

**Recommendation: surface M12-D6 as the DECISIONS.md promotion to author during M12.4** (at the
design-lock or close, per the established "no DECISIONS change unless Joe promotes" pattern). **Not
auto-promoted — Joe's explicit call.** Flagged here so the design phase carries it.

---

## §7 Witness sketch (what M12.4 would witness; RED-on-revert)

- **WE1 — blob-bytes erasure (the headline).** A blob attached in the `self` thread (M12.1) is
  redacted; the ciphertext file is **gone** from `blobs_dir`; a subsequent fetch by a second
  same-identity client returns `10003 blob_unavailable`. RED-on-revert: skip the delete → blob still
  present → RED.
- **WE2 — F2b Retained refusal (the floor).** A redactor/author whose stored Trust Assertion declares
  `Retention::Retained` issues a redact; the blob is **NOT** deleted (legal-hold floor holds the
  bytes); an `Erasable`/absent posture deletes. First `Retention` reader exercised. RED-on-revert:
  drop the Retention check → Retained content erased → RED.
- **WE3 — convergence (A-08/FK-3).** The redact event is stored + fanned out identically on every
  node regardless of the Retention side-effect outcome (the gate is on the delete, not the
  admission). RED-on-revert: admission-reject on Retained → divergence repro.
- **WE4 — federated erasure (A-06).** A redact propagates to a second home (B) that **cached** the
  blob (M12.3 fetch); B's redact hook deletes B's cached copy. RED-on-revert: B ignores the redact →
  B's cache survives → RED.
- **WE5 — shared-blob safety (A-09/FK-8).** Two messages attach identical bytes (one deduped blob);
  redacting one does not erase the blob while the other still references it (per the chosen FK-8
  rigour). RED-on-revert: naive delete → live message's blob vanishes → RED.
- **WE6 — D3 boundary stated honestly (not a test, a close-claim).** M12.4 erases the blob *content*
  (bytes) + refuses-on-Retained + tombstones for display; it does **NOT** crypto-shred the
  DAG-resident residue (the `message.file` event's existence + plaintext descriptor key + any text
  body) — that is D3 (the descriptor `enc:`-wrap + the destroy-to-erase storage op, per A-05). State
  this at the witness set and the close, exactly as M12.1 stated W2's honest boundary.

---

## §8 GO/NO-GO + scope fence

**GO.** M12.4 builds the F2a/F2b erasure slice on the **mutable** content layer (real, witnessable,
federated), and marks-and-reserves the **DAG-resident** crypto-shred to D3 + the WORM vault to the
operator/module layer. The buildable mechanism is concrete (redact schema + builder +
`BlobStore::delete` + node hook + F2b read + federation-free-ride); the gated parts are
honestly-named and consistent with M12-D10 + D-088's cascade.

**In scope (M12.4):** `message.redact` content schema + builder (A-01); `BlobStore::delete` (A-03);
the node-side redact erasure hook + federation propagation (A-02/A-06); F2b `Retention` refusal of the
erasure side-effect (A-04, FK-2/FK-3); the convergence-safe gate placement (A-08); shared-blob safety
(A-09, FK-8); the minimal client tombstone (FK-6); the reject/refusal wire code in domain 10 (FK-7);
WORM/legal-hold **reserved hook** only (A-07); the M12-D6 promotion (Joe's call, §6).

**Out (D3-gated / reserved / later):** crypto-shred of the DAG residue — the descriptor `enc:`-wrap +
the destroy-to-erase storage op (A-05; the shared text-path D3 / M8.7 S+L arc); the WORM/archival
production backend (operator/module, A-07); any change to the M12.1 blob crypto maturity (R-1).

---

## §9 Sequence + entry (Rule 0)

this audit (Clair) → Joe pushes → **Chat design discussion on FK-1..FK-8 + the M12-D6 promotion → Joe
lock by-recomms → Chat authors `tasks/M12_4_ERASURE_DESIGN.md` + design-bridge** → Clair authors the
M12.4 runbook → implement → Chat close-bridge → **M12.4 close = M12 close** → Round-2 final pre-UI
gate → UI → Streams. No code until the design is Joe-locked.

**Entry (Rule 0):** `CLAUDE.md` PLAY → `JOURNAL.md` J-387 → this audit →
`tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D6/D10) → `tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (§3 R-1 +
the built blob primitives) → `DECISIONS.md` D-088 / AH-D1 → `docs/ROADMAP.md` (M12).
