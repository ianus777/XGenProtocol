# M12 — Attachments: Phase-0 Framing Brief (J-357 lock)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Framing brief that OPENS M12 — attachments. Authored by Chat at the M12-open doc-bridge
(J-379) after the concept, the Phase-0 scope, and forks **F1–F9** were Joe-LOCKED in
discussion. This brief carries the locked concept + the Phase-0 grounding scope + the nine
forks; it is **not** a design or a runbook.

D-071 arc discipline: Phase-0 audit (Clair, this brief is its agenda) → design → Joe-lock →
runbook → Clair implements → Chat doc-bridge → close. **No code before Joe locks the design.**

## Provenance (grounded J-379)

J-357 was a **planning lock** (existence + placement: full multi-file send/receive, pre-UI,
after M11), not a scope lock. Three philosophy-level calls were explicitly deferred to this
Phase-0: single-vs-multi-file shape, the deletion/GDPR posture, and eager-vs-lazy federation.

Grounded on `main` at open:
- **No blob store today.** The store is event-only — `EventStore` trait (`append(event)` /
  `get(EventXgid)`) over a single SQLite `events` table (`xgen-store-sqlite`). Every "blob"
  hit in prod is MLS opaque payload / Arc-H content-blindness, not file attachments. The blob
  store is **net-new**.
- **§3.1.1 size model is envelope-only + mostly unwired.** Two-layer model (Tier ceiling
  256/64/32/16/8 KB by tier + immutable Space `max_event_size` override) is spec-complete, but
  in code only the single 256 KB **frame ceiling** is enforced (`framing.rs:21`,
  `validation.rs:11`); the per-tier table is spec-but-unimplemented, and `max_event_size` is
  stored on the Space (`state.rs:191/289`) but not visibly enforced. §3.1.1 deliberately keeps
  **bytes out of protocol messages** ("signalling protocol, not file transfer").
- **`ModulePolicy` is the dormant extensible switch-bag.** `claims.extra["module_policy"]`
  (`trust_assertion.rs`) — forward-extensible, `erasability` its first member, "not its only
  one." **Zero production readers** (M10.3 populated + witnessed, enforcement D3-gated).
- **Event-kind breadcrumbs:** `file.upload` (ch2 "File or media attachment") +
  `message.attachment-meta` (Phase-9 survey naming) — possible existing descriptor kinds.

## Locked concept (J-357 + this discussion)

- **Metadata by value, content by reference.** The event carries a **descriptor** (content
  hash + filename + mime + size); the **bytes** live in a net-new **content-addressed blob
  store**, keyed by hash. The event references the hash, never a filesystem path.
- **Why bytes never ride the event** (three structural reasons, not stylistic): the §3.1.1
  size ceiling; the immutable signed DAG (`event_id` = hash of the event's bytes — embedding a
  file makes every sync/fanout/frontier/state-resolution op drag the payload forever and
  unsignable to reclaim); and **erasability** (you cannot tombstone bytes welded into an
  immutable signed object — the F2 mechanic exists *because* bytes are separable) + federation
  cost (forced eager carry).
- **Send = upload-and-persist**, not link-to-source and not Skype-style direct device→device.
  The client hashes the file and uploads the bytes into the **home node's blob store**; the
  source device is irrelevant thereafter (can be offline / wiped / HDD-cleaned — the node holds
  the durable copy). Download needs no second device online.
- **The `self` thread is M12's front door, not a downstream beneficiary.** Cross-device
  self-thread sharing is **intra-home, multi-device** (one identity, one home node, several
  clients) — it rides blob store + pipe upload + pipe fetch and **never touches federation**,
  so the M11/D-021 never-federated guarantee stays fully intact. It is the natural headline
  witness for M12.1.

## The nine forks (Joe-LOCKED, J-379; arc-local D-069 unless promoted)

- **F1 — descriptor shape:** a **multi-file list from day one** (`attachments: [Descriptor]`);
  build/test single-attachment first if useful, but the wire schema is plural (no later
  reshape).
- **F2 — GDPR/deletion posture:** **F2a (lean)** = tombstone/redaction only, inheriting the
  Arc-I erasure shape + D-088 + AI-D8 erasability; erasure offered uniformly, honest D-065
  boundary that cross-federation blob erasure is request-not-guarantee. **F2b** = an attachment
  redaction **reads the sender's `retention`** (T4/`Retained` refuses erasure) — the
  audit-grounded escape; activating it makes M12 the **first production reader** of the dormant
  AI-D8 enforcement. Lean: **F2a, with F2b named**.
- **F3 — eager-vs-lazy federation:** **lazy lean, audit-grounded-not-locked** (decided at
  design after the pipe/federation grounding). **Overridden toward eager/replicated for
  Retained (T4) content** (F7 coupling — can't legal-hold a single droppable copy). Lazy-miss
  availability surfaces the held-pending/unavailable client signal (carry-over UX gap).
- **F4 — sub-arc split:** open **monolithic**; Phase-0 reveals the seams. Anticipated split:
  **M12.1** local blob store + descriptor + single-node multi-device pipe round-trip (**`self`
  thread = headline witness**) → **M12.2** pipe transfer + `--attach` surface (4 D-092 arms) →
  **M12.3** federation (eager/lazy) → **M12.4** erasure/tombstone.
- **F5 — namespace** (deliverable, not a lock): reserve the attachment event-kind, steer clear
  of the reserved `stream.*`/`media.*` band; ground the existing `file.upload` /
  `message.attachment-meta` kinds first.
- **F6 — blob size limit:** **Pattern A** — a tier-derived spec ceiling (default, MB-scale,
  descending by tier in §3.1.1 spirit) + a **tighter-only immutable Space override**, mirroring
  §3.1.1's two layers. New enforcement point at **transfer/ingest** (the blob doesn't ride the
  signed envelope, so it's a parallel gate, not the reject-before-signature one). Values Phase-0.
- **F7 — storage model + retention-aware lifecycle:** upload-to-content-addressed-store; the
  blob store's reclaim/GC is **tier-retention-aware** — T1 **reclaimable + erasable**, T4
  **pinned / legal-held / undeletable** (WORM-shaped — undeletable even by the operator).
  Retention sets a **durability floor** that overrides F3's lazy lean for retained tiers. A
  **tiering/offload dimension**: a Retained blob may move to a cheaper archived/backed-up store
  to reclaim primary space **while staying immutable+retained** — provided as a **reserved
  operator/module hook**; M12 **marks + reserves, does not build the WORM vault** (a compliant
  archival backend is an operator/module responsibility).
- **F8 — blob lifetime / retention duration:** **Pattern A** tier-set TTL, **two modes**:
  lower tiers = a **reclaim deadline** (kept *at most*; GC ages out — default + tighter-only
  manual Space override); T4 = a **retention minimum / legal-hold** (kept *at least*,
  undeletable, **overrides erasure**, possibly open-ended until a hold is explicitly cleared).
  Values Phase-0.
- **F9 — blob storage location config:** the blob store is **rooted as a sibling of the event
  log under one durable node data root** (`<data_root>/events.db` + `<data_root>/blobs/`), so
  events + their referenced blobs **back up / snapshot / tier as a single coherent unit** — the
  T4 requirement (a retained descriptor pointing at a dropped blob is a broken record). The data
  root **defaults outside the install/system folder** (lifecycle independence, capacity/mount
  reality, backup boundaries, privilege separation), **operator-overridable to any absolute
  path/volume**, **startup-validated** (durable, writable, **not** tmp/volatile). **Node config,
  never tier-module, never assertion-set** (placement is the operator's; an assertion dictating
  a local path is a category error + security hole). Plus the reserved **archive/offload path or
  module hook** (pairs with F7). Per-object size/lifetime stays **per-blob tier-driven** — a
  shared root ≠ shared retention.

**Pattern-A discipline (the spine across F6/F8/F9):** size, lifetime, and placement are
**node/spec concerns keyed by tier**, NOT module-declared switches. A module attests the
*tier*; the spec maps tier → hard ceilings/durations; the operator owns physical placement.
The dormant `ModulePolicy` switch-bag (Pattern B) is **deliberately not used** for these — a
module must not be able to loosen a hard ceiling, shorten a legal-hold, or set a local path.

## Real-world grounding (WORM — J-379 web check)

Joe's T4 model = **WORM + legal hold + tiered archival**, validated against government records
practice: federal retention obligations (NARA / Federal Records Act — unscheduled records may
not be destroyed; electronic records kept usable/searchable/authentic for the period); WORM
immutability (undeletable even by admins); two modes (time-based interval vs legal-hold-until-
cleared, the hold overriding an expired interval); and immutability **independent of storage
tier** — so a Retained blob tiers down to cheap archive storage to reclaim space while staying
immutable. The reference-implementation posture is **mark + reserve the hook**, not build the
vault.

## Phase-0 audit agenda (Clair grounds, in order)

1. Message event build + apply seam — where the descriptor attaches; the existing message kind.
2. Blob store — net-new; rooted as the event-log sibling under the node data root; hash scheme;
   client staging. **F9 grounding:** is there a configurable node-data-dir today to extend, or
   is the event-store path implicit/hardcoded (M12 establishes the shared data-root convention)?
3. **D-056 pipe byte-transfer** — binary / base64 / chunked? *The load-bearing unknown; gates
   even the federation-free self-thread slice.*
4. Federation seam — eager vs lazy; any large-payload/fetch path today, or both net-new (F3);
   the retention durability-floor interaction (F7).
5. Erasure inheritance — Arc-I + D-088 + AI-D8 erasability; tombstone reuse-vs-new kind (F2);
   the dormant retention-reader status / F2b enforcement readiness.
6. Size + lifetime enforcement state — the §3.1.1 envelope tier-table + `max_event_size` are
   spec-but-(apparently)-unwired; only the 256 KB frame ceiling is enforced. Ground the actual
   state (F6/F8); decide blob enforcement independent-of vs pulls-in the envelope enforcement.
7. Retention / GC + tiering hook — node store lifecycle today (none); the reserved WORM/archive
   offload hook surface (F7); single-home durability vs replication for Retained (F7/F3).
8. Client `Send` surface + `--attach` landing + the 4 D-092 arms (F4 / M12.2).
9. Wire-code + event-kind namespace + the streams fence (F5).

## Routed / flagged topics (survive beyond M12)

- **Pattern-B "module-as-policy-bearer":** reconsider §3.1.1 message-size (and other limits) as
  a **tier-auth-module-defined limitation** carried on the `ModulePolicy` switch-bag — sibling
  to the **erasure-via-general-setting** idea Joe parked. Named here + on the ROADMAP horizon,
  **not invented inside M12**. Future return.
- The actual **WORM/archival backend** = operator/module responsibility; M12 reserves the hook.
- Carry-over (pre-existing, non-blocking): client UX for federation-derived held-pending /
  unavailable signals (a lazy-fetch miss surfaces here — F3); federation-under-load stress
  measurement (no scheduled home).

## Design input — blob encryption = same Arc-H envelope as text (added J-380, post-audit)

Captured in discussion after the audit (a design input, **not** an audit finding). **Blob bytes
should be E2E-encrypted the same way message text is.** The text path: the client encrypts
content client-side, the content becomes an opaque `enc:`-prefixed string in the free-form
content field, and the node stores/forwards it **content-blind** ("plaintext never in transit");
`client_mls.rs` already carries the **per-message random content-key (CK) envelope**
(`encrypt_message_envelope` — a fresh random CK encrypts the plaintext, CK wrapped for the group).

The blob maps onto that one level out (the standard E2E-attachment shape): the client generates a
**fresh per-blob key**, encrypts the bytes, uploads the **ciphertext** to the node's blob store
(node content-blind; content-addresses by the **ciphertext** hash); the per-blob key + the
plaintext hash travel **in the descriptor**, which itself rides inside the `enc:` E2E message
content. The same MLS group (or, for the `self` thread, the user's own devices) that can read the
text can read the blob — no new key distribution.

Two payoffs: (a) **no privacy regression** — keeps the node content-blind for attachments too
(otherwise the blob store is the soft underbelly, the most sensitive payloads least protected);
(b) it puts the bytes on the **crypto-shred** substrate (D-088) — erasure = **destroy the per-blob
key** → every replica, including unreachable federated homes, becomes permanently unreadable.
That turns F2's federated right-to-be-forgotten from request-not-guarantee into a real guarantee
for the content (ciphertext lingers as noise, but is unreadable).

**Two flags ride with this input:**

1. **Inherits the text path's crypto maturity.** Arc-H's per-message-CK envelope + crypto-shred
   are **demonstration-grade** today; real RFC 9420 HPKE wrapping + the destroy-to-erase storage
   op are **D3-fenced** (`client_mls.rs` is explicit), production openmls is the M8.7 S+L arc.
   M12 builds blob-encryption in the *same Arc-H shape*, inheriting that maturity — interface
   now, production crypto when the text path's D3 work lands. Not stronger than the text path it
   mirrors.
2. **Crypto-shred vs T4 legal-hold/WORM = a philosophy fork (Joe-lock, above the design-seat
   bar).** Crypto-shred (erasability) and T4 retention / WORM / legal-hold (F7/F8) are
   **opposites** — a content-blind node holding only ciphertext + a destroyable key **cannot
   legally produce a retained record**, and crypto-shred makes it permanently unproducible.
   Resolving it is the no-anonymity / institutional-independence axis in storage form: is E2E
   **universal**, or **tier-conditioned** (e.g. the accountable T4 authority holds an **escrow
   key** to retain + produce, while T1–T3 stay crypto-shreddable)? Must reconcile with Arc-H's
   text-E2E posture (universal vs tier-gated). Route to a Joe-lock at design.

*Disposition:* adopt **same-as-text** as the default; the design carries the two flags — flag 1
as an inherited-maturity note, flag 2 as a Joe-lock the design must surface, not decide.

## Sequence

M12 OPEN (J-379) → Clair D-071 Phase-0 audit (this brief = its agenda) → design → Joe-lock →
runbook → implement (likely sub-arc'd M12.1–M12.4) → Chat doc-bridge per arc → close →
Round-2 final pre-UI gate → UI → Streams.

## Entry (Rule 0)

this brief → `CLAUDE.md` PLAY → `JOURNAL.md` J-379 → `docs/ROADMAP.md` (M12) → `ch3 §3.1.1`
(size model) → `DECISIONS.md` D-021 / D-056 / D-088 (context).
