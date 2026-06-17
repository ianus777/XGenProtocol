# M12.4 — Erasure (redact + blob-delete + Retained-refusal): Design (Joe-LOCKED)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & status

The Joe-LOCKED M12.4 design, authored by Chat at the design-lock (J-388) after the fork
discussion on Clair's M12.4 D-071 Phase-0 audit. Sits on
`tasks/M12_4_ERASURE_PHASE0_AUDIT.md` (v1.0, GO; findings M12.4-A-01..09, forks FK-1..FK-8,
committed `31e17aa`, pushed) and the master M12 design `tasks/M12_ATTACHMENTS_DESIGN.md`
(M12-D6 universal-E2E/Retained, M12-D10 M12.4 scope) + the locked erasure architecture
**D-088** + **AH-D1**.

M12.4 is the **fourth and final** sub-arc of M12: **erasure** — `message.redact` made real
against attachments. M12.1 built the blob mechanism; M12.2 the client surface + F6 + F9;
M12.3 the cross-home fetch. M12.4 makes attachment content **erasable**: a redact deletes the
blob bytes (a real, complete erasure of every reachable copy — origin + any federated cache),
refuses on **Retained (T4)** content (the legal-hold floor), and tombstones the DAG residue for
display.

**The central enabling fact (audit A-01..A-03, Chat-reverified on `main`):** attachments split
content across **two stores of opposite mutability**. Blob ciphertext lives in the
content-addressed `BlobStore` — **separate from the DAG, mutable** → deleting bytes is a real,
complete erasure of that node's copy, **buildable now**. The `message.file` event residue
(existence + the plaintext per-blob key + any text) is **DAG-immutable** → crypto-shred,
**D3-gated** (per M12-D10 + D-088's cascade). So M12.4 builds the real GDPR win on the mutable
layer and marks-and-reserves the DAG residue — the M12.3 "build the mechanism, fence the
crypto-maturity" pattern. This is why M12.4 is GO-able and not a repeat of Arc I's design-only
close (J-253): there is a concrete mutable surface to actually erase.

D-071 arc discipline: this design → Clair authors the M12.4 runbook → implement spine-first →
Chat close-bridge → **M12.4 close = M12 close**. No code precedes the runbook; the runbook does
not precede this lock. Decisions are arc-local (D-069) **except** the M12-D6 promotion, which is
authored as **D-093** in the J-388 design-bridge (see §3 FK-9 / §7).

---

## §2 The grounded gap (audit A-01..A-09; Chat-reverified on `main @ 31e17aa`)

- **A-01 / FK-1** — `message.redact` is a **bare validated kind**: a permission arm
  (`exchange.rs:792` → `SendMessages`) + a no-op validation arm (`exchange.rs:846` → `Ok(())`),
  **no builder, no content schema, no applier**. F2a is all net-new.
- **A-02** — `message.*` (incl. redact) **never mutates `SpaceState` node-side** (falls to
  `_ => Ok(())`, `state.rs:655`); messages ride the DAG + fanout, materialized client-side. So a
  redact's node-side job is **the blob-bytes delete** (not a state apply); the display tombstone
  is client-side.
- **A-03 / FK-5** — `BlobStore` has **`new`/`put`/`get`/`contains` only — NO delete**
  (`blob_store.rs:100/120/135/151`). `BlobStore::delete` is the **load-bearing missing
  primitive** and lands cleanly (the store is a thin per-op handle).
- **A-04 / FK-2** — the sender's `Retention` is **reachable without new plumbing**:
  `IdentityRecord.trust_assertion: Option<serde_json::Value>` (`registry.rs:45`) → resolved via
  `IdentityRegistry::get` → `module_policy().erasability.retention`. **Zero production
  `Retention` readers today** (J-380/J-387 confirmed; F2b is M12's **first**).
- **A-06** — B **caches federated blobs**: `let _ = store.put(&bytes);` on the M12.3 fetch-miss
  path (`app.rs:1945`). A redact reaching B must delete B's cached copy — the federated dimension
  is concrete.
- **A-08 / FK-3** — a `Retention`-gated **admission** diverges (a node that can't resolve the
  author's tier would reject the redact, breaking the DAG) — the INV-EXP / M8.6 convergence
  family. The gate must be on the **erasure side-effect**, not admission.
- **A-09 / FK-8** — **shared-blob hazard:** content-addressed dedup (`blob_ref = hash(bytes)`)
  means two `message.file` events can reference one physical copy → a naive delete-on-redact
  erases bytes another live message still needs. **No `blob_ref → events` reverse index exists**
  on `main` (Chat-reverified; the M12.3-audit finding holds). This fork is resolved by **D-093
  clause 3**, not by an index (see §3 FK-8).

---

## §3 Locked decisions (M12.4-D1..D9)

### M12.4-D1 — redact content schema = `{ target_event_id }` only (FK-1: a)

The redact event carries **only the target event id**. The node resolves the blob_ref(s) from
the **target `message.file`'s descriptor** (the authoritative blob list, plaintext-readable per
R-1, single source of truth). Restating the blobs on the redact invites drift. **Lock: (a).**
Net-new: a `build_message_redact_event` builder + the `{ target_event_id }` content shape.

### M12.4-D2 — F2b reads the **original content author's** `Retention` (FK-2; load-bearing) — **Joe-lock**

The erasure side-effect is gated on the `Retention` of the **target `message.file`'s sender**
(the original content author), **NOT** the redactor's. Per **D-088 / D-093 clause 2**: retention
is a property of the **record** — a Retained (T4) author's content cannot be erased by anyone
(including a lower-tier redactor), and a redactor cannot elevate their own erasure power by being
lower-tier. The *permission* to issue a redact is a separate, already-existing gate
(`SendMessages` / moderation, A-04/G2) and is unchanged. **Lock: (a) original content author.**

### M12.4-D3 — F2b gates the **side-effect**, not admission (FK-3; convergence spine) — RED-on-revert

The redact event is **always admitted, stored, and fanned out** (it is a valid signed event);
**only the blob-erasure side-effect** is `Retention`-gated: Retained → **keep the bytes** (the
legal-hold floor, D-093 clause 2); Erasable/absent → **delete**. The display tombstone applies
regardless. This is the INV-EXP / M8.6 lesson (tier-conditioned admission diverges; side-effect
gates don't) and is the arc's **convergence spine** — witnessed RED-on-revert (admission-reject
on Retained → divergence repro). **Lock: (b).**

### M12.4-D4 — F2a scope = tombstone **and** delete-the-bytes (FK-4; exceeds the literal F2 lock) — Joe-confirmed

The brief's F2 "tombstone-only lean" was born of the **append-log** constraint (you cannot delete
a DAG event). Blobs are **not** in the DAG (§2), so blob-delete is possible and a real GDPR win
the append-log never blocked. M12.4 does **both**: **tombstone** the DAG event (the residue it
can't erase — that's D3) **and** **delete the blob bytes** (the content it can erase now).
Faithful to F2a's spirit: *tombstone what you can't erase, erase what you can.* **Lock: do both**
(explicitly confirmed as exceeding the literal "tombstone-only" lock).

### M12.4-D5 — node redact-hook placement (FK-5; mechanical)

The blob-delete side-effect lands in `process_inbound` after the redact is validated + stored,
sibling to M12.3's `BlobUploadEnd → store.put` (A-03/G10/G13). **Lock:** placement confirmed;
the runbook grounds the exact site. Light — no further design lock.

### M12.4-D6 — client tombstone = minimal, in M12.4 (FK-6)

M12.4 includes the **minimal** client tombstone: a redacted message must **not** render its
descriptor / fetch its blob (the user-visible witness that erasure happened). Richer redaction UI
(who/when/placeholder styling) is deferred to UI. **Lock: build the minimal tombstone; defer the
polish.**

### M12.4-D7 — reject/refusal wire codes (FK-7; domain 10) — re-grep at build

Domain 10 (attachments/blobs) is `10000–10999`; `10001`/`10002`/`10003` are live, **`10004`+
free**. Reserve **`10004 erasure_refused_retained`** for the F2b legal-hold refusal signal (a
typed signal to the redactor that the side-effect was suppressed; the event still converges), and
reuse the existing codes for target-not-found / malformed. As a **typed `BlobError` variant**
(the M12-D9 parallel error type, not `ExchangeError`), mirroring M12.3's `10003`. **RC-F-01
re-grep the register at build** before emitting (the 10002/10003 collision-check precedent).
**Lock: `10004` band; runbook firms the exact variants.**

### M12.4-D8 — shared-blob safety = **no shared physical copy across erasure-fate** (FK-8; resolved by D-093 clause 3) — **supersedes the audit's FK-8 rec**

The audit recommended FK-8 (b) reference-scan or (c) accept-the-hazard. The design **supersedes**
that with the **D-093 clause-3** invariant locked in the fork discussion (the audit predates the
T4-reasoning that produced it). **The reasoning:** retention/erasability is **per-record** (D-093
clause 2); a single shared physical copy would let one record's policy silently override
another's — a lower-tier erasure deleting bytes a T4 record holds (durability-floor breach), or a
T4 hold blocking a lower-tier record's valid erasure (right-to-erasure breach). A refcount /
reverse-index manages shared-copy bookkeeping but **cannot resolve the tier collision** (it can't
honor "A held, B erasable" on one physical blob) — heavy **and** insufficient.

**Lock:** an attachment blob's physical copy may only be shared among references that share the
same erasure-fate. **M12.4 v1 = no attachment dedup** — one physical copy per `message.file`
send, each with its **own deletable storage handle**; the **content-hash is retained as
descriptor metadata** (not as the storage key) so identical-file detection / policy-keyed
dedup-within-a-shared-fate-set remains a **future optimization** (not a correctness fix). A redact
deletes only that reference's own copy → the A-09 hazard cannot arise.

**Runbook-bound mechanism (grounded, not a Joe-lock):** how `blob_ref` is derived today (pure
`hash(bytes)` doubling as the storage key?) decides whether "one copy per send" is a clean handle
choice or a small additive scheme touch (e.g. a per-send salt folded into the **storage handle**
while the **content-hash stays pure** for the retained metadata). Clair grounds the derivation in
the runbook and picks the mechanism; the **invariant** above is the lock.

### M12.4-D9 — M12-D6 promoted to **D-093** at this design-lock (FK-9 / §6 standing flag) — Joe-LOCKED

M12-D6 (universal E2E at the protocol layer; "Retained (T4)" = ciphertext durability-floor +
erasure-refusal, not protocol escrow) is **promoted to `DECISIONS.md` D-093** — M12.4 is the arc
that first *exercises* it (D2/D3 = first enforcement read; D4/D8 = first mechanism), it is
principle-shaped and past the 3-recurrence bar (AH-D1 + D-088 lineage, carried J-381→J-387).
**D-093 carries three bound clauses:** (1) universal E2E / no protocol escrow; (2) Retained =
durability-floor + erasure-refusal, retain-and-produce reserved to operator/module; (3) **no
shared physical blob copy across erasure-fate** (the M12.4-D8 corollary). Authored in the J-388
design-bridge (atomic, D-074); M12-D6's design-doc flag flips to "promoted → D-093."

---

## §4 Witnesses (M12.4; RED-on-revert; runbook firms the set)

- **WE1 — blob-bytes erasure (headline).** A blob attached in the `self` thread (M12.1) is
  redacted → the ciphertext file is **gone** from `blobs_dir` → a subsequent fetch by a second
  same-identity client returns `10003 blob_unavailable`. RED-on-revert: skip the delete → blob
  present → RED.
- **WE2 — F2b Retained refusal (the floor).** An author whose stored Trust Assertion declares
  `Retention::Retained` has their content redacted → the blob is **NOT** deleted (legal-hold
  floor) + the redactor gets `10004 erasure_refused_retained`; an `Erasable`/absent author →
  deleted. First `Retention` reader exercised. RED-on-revert: drop the check → Retained content
  erased → RED.
- **WE3 — convergence (A-08 / D3).** The redact event is stored + fanned out **identically** on
  every node regardless of the side-effect outcome (gate is on the delete, not admission).
  RED-on-revert: admission-reject on Retained → divergence repro.
- **WE4 — federated erasure (A-06).** A redact propagates to home B that **cached** the blob
  (M12.3 fetch) → B's hook deletes B's cached copy. RED-on-revert: B ignores the redact → B's
  cache survives → RED.
- **WE5 — shared-fate safety (D-093 c3 / D8).** Two messages attach identical bytes → **two
  physical copies** (no dedup) → redacting one deletes only its copy; the other is untouched.
  RED-on-revert: collapse to one shared copy + delete → the live message's blob vanishes → RED.
- **WE6 — D3 boundary (a close-claim, not a test).** M12.4 erases the blob **content** (bytes) +
  refuses-on-Retained + tombstones for display; it does **NOT** crypto-shred the DAG-resident
  residue (the `message.file` event existence + the plaintext descriptor key + any text body) —
  that is **D3** (the descriptor `enc:`-wrap + the destroy-to-erase storage op). Stated honestly
  at the witness set and the close, exactly as M12.1 stated W2's boundary.

---

## §5 Out of scope / reserved (do NOT pull in)

- **Crypto-shred of the DAG residue** — the descriptor `enc:`-wrap + the destroy-to-erase storage
  op (A-05; the shared text-path D3 / M8.7 S+L arc). M12.4 erases bytes, not the DAG event.
- **The WORM / legal-hold production backend** — operator/module responsibility (A-07, D-093
  clause 2); M12.4 reserves the hook only.
- **Policy-keyed dedup / blob reverse-index** — reserved future optimization (D8); M12.4 v1 = no
  attachment dedup.
- **Any change to M12.1 blob crypto maturity** (R-1) — untouched.
- **Richer redaction UI** — deferred to UI (D6 = minimal tombstone only).

---

## §6 Sequence (Rule 0)

this design → Clair authors `tasks/M12_4_*_IMPL.md` (the M12.4 runbook, §3 scope, spine-first —
re-grounds the anchors, grounds the `blob_ref` derivation + picks the D8 mechanism, picks the
`10004` variant, defines the witness set) → Joe-lock the runbook values → implement spine-first,
per-commit, Joe pushes each → Chat close-bridge → **M12.4 close = M12 close** → Round-2 final
pre-UI gate → UI → Streams. No code until the runbook lands + Joe-locks.

## §7 Entry (Rule 0)

`CLAUDE.md` PLAY → `JOURNAL.md` J-388 → this design → `tasks/M12_4_ERASURE_PHASE0_AUDIT.md`
(findings + forks) → `tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D6/D10) →
`tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (R-1 + the built primitives) → `DECISIONS.md` **D-093**
(the promotion) / D-088 / AH-D1 → `docs/ROADMAP.md` (M12).
