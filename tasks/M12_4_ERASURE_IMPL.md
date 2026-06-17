# M12.4 — Erasure (redact + blob-delete + Retained-refusal): Implementation runbook
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

The Clair-authored M12.4 runbook — the **final** M12 sub-arc; **M12.4 close = M12 close**. Executes
the Joe-LOCKED design (`tasks/M12_4_ERASURE_DESIGN.md` v1.0, M12.4-D1..D9) on the GO Phase-0 audit
(`tasks/M12_4_ERASURE_PHASE0_AUDIT.md`). M12.4 makes attachment content **erasable**: a
`message.redact` deletes the blob bytes (a real, complete erasure of every reachable copy — origin +
any federated cache), **refuses** on **Retained (T4)** content (the legal-hold floor), and tombstones
the DAG residue for display. The DAG-resident crypto-shred residue (the `message.file` event +
plaintext descriptor key + any text) stays **D3-gated** (WE6 close-claim).

**Grounded against `main` @ `24bc4f2`** (tree clean, in-suite **1448/0**). Every anchor below was
re-read in production code this session (D-065/D-078 — the design's line numbers drift; these are
re-confirmed). Decisions are arc-local (D-069); **D-093** (the M12-D6 promotion, 3 clauses) is now a
real DECISIONS entry — the build honors all three, esp. **clause 3** (the D8 invariant).

**Locked, do not re-litigate (M12.4-D1..D9):** D1 redact schema `{ target_event_id }` · D2 F2b reads
the **original content author's** `Retention` · D3 gate the **side-effect** not admission · D4
tombstone **and** delete-the-bytes · D5 `process_inbound` hook · D6 minimal client tombstone · D7
`10004 erasure_refused_retained` (typed `BlobError`) · D8 no shared physical copy across erasure-fate
→ v1 = no attachment dedup · D9 M12-D6 → D-093.

---

## §2 Grounding ledger (anchors re-confirmed to file:line, `main` @ `24bc4f2`)

| # | Seam | Location | M12.4 action |
|---|---|---|---|
| G1 | `EventType::MessageRedact` → `"message.redact"`; permission arm `=> SendMessages`; no-op validation arm `=> Ok(())` — **no builder, no content schema, no applier** | `xgen-common/src/wire.rs:36,173,267`; `xgen-core/src/message/exchange.rs:792,846` | net-new builder + content schema (D1); the node side-effect (D5) |
| G2 | `Descriptor { blob_ref, plaintext_hash, key, filename, mime, size }`; `message.file` content = `json!({ "attachments": attachments })` (`Vec<Descriptor>`) | `xgen-core/src/message/exchange.rs:935-941,943-972` | the node reads `target.content["attachments"]` → `blob_ref`s |
| G3 | `build_message_file_event(key, space_id, room_id, prev_events, &[Descriptor])` (twin builder) | `xgen-core/src/message/exchange.rs:949-972` | `build_message_redact_event` twin (D1) |
| G4 | `encrypt_blob` mints a **fresh key + fresh nonce per call**; test `fresh_key_and_ciphertext_each_call` asserts `c1 != c2` for identical plaintext | `xgen-core/src/encryption/blob.rs:59-74,132-138` | **the D8 mechanism** — `blob_ref` is per-send-unique by construction (§3) |
| G5 | `BlobStore`: `new`/`put`/`get`/`contains` — **NO delete**; `BlobError`+`to_wire_code` (10001/10002/10003); `get` verifies content-address (W3) | `xgen-core/src/blob_store.rs:100,120,135,151,43-90` | add `BlobStore::delete` (C1) + `BlobError::ErasureRefusedRetained` → 10004 (C1) |
| G6 | `NodeRuntime.identity_registry: IdentityRegistry` (pub field); `NodeRuntime.stores: HashMap<SpaceXgid, Box<dyn EventStore>>` (pub field) | `xgen-core/src/node/runtime.rs:266,280` | the redact hook re-locks `runtime` → reads target event + author Retention |
| G7 | `EventStore::get(&self, id: &EventXgid) -> Result<Option<Event>, StoreError>` (owned) | `xgen-core/src/dag/store.rs:82` | `rt.stores.get(&space).get(&target_id)` → target `message.file` event (D1) |
| G8 | `IdentityRecord.trust_assertion: Option<serde_json::Value>` holds the **full** stored assertion (`accept_registration` sets `trust_assertion.cloned()`) | `xgen-core/src/identity/registry.rs:45`; `registration.rs:533` | F2b read: `record.trust_assertion → ["claims"] → TrustClaims → module_policy().erasability.retention` (D2) |
| G9 | `Retention { Erasable, Retained }`; `TrustClaims::module_policy() -> Option<ModulePolicy>` → `.erasability -> Option<Erasability>` → `.retention -> Option<Retention>`; doc: *"expression only … the deferred D3-gated consumer"* — **zero readers today** | `xgen-common/src/trust_assertion.rs:189-194,225-229,161-181` | F2b = M12's **first** `Retention` reader (D2) |
| G10 | `process_inbound(conn, msg, identity_id, home_node_id, local_mode, runtime, identities_path, spaces_dir, origin, policy_store)` — handles **both** `LocallySubmitted` + `ReceivedViaFederation`; `Inbound::Event` arm → `dispatch_event` → `DispatchOutcome::Accepted { … }` (event validated+persisted, fanout not yet begun) | `xgen-node/src/app.rs:3009-3024,3050,3158-3168,3170` | the redact hook lands in the **`Accepted` arm** (D5); fires on both origins → B's federated redact deletes B's cache (WE4) |
| G11 | `blobs_dir` resolved at startup (`config.paths.blobs_dir … unwrap_or data_dir/blobs`), threaded into `handle_connection` → in scope at the `process_inbound` call site (the WS upload handler uses it at `:1883`) | `xgen-node/src/app.rs:765,1071,1505,1578,1883` | thread `blobs_dir: &Path` into `process_inbound` (additive param) for the hook |
| G12 | B caches federated blobs: `let _ = store.put(&bytes);` on the M12.3 fetch-miss path | `xgen-node/src/app.rs:1945` | the cached copy a federated redact deletes (WE4) |
| G13 | `blob_err(code, name)` Error helper + `TransportMessage::Error { event_id, error_code, … }` wire shape (M6 §3.2; blob 10001/10002/10003 ride it) | `xgen-node/src/app.rs:1878,1893`; `connection.rs:858` | the `10004` refusal signal to a `LocallySubmitted` redactor (D3/D7) |
| G14 | RC-F-01: domain 10 uses **only** 10001/10002/10003 (grep); **10004 free** | `xgen-core/src/blob_store.rs:84-86` (+ no other 1000x) | reserve `10004` (D7); **re-grep at build** before emitting |
| G15 | no `redact` verb anywhere in `ops`/CLI/batch (grep empty); `--attach`/`fetch` are the M12.1/M12.2 verb precedents (4-arm D-092 + Appendix F) | `xgen-client/src/{ops,app,batch}.rs` | net-new `redact` verb (C3): 4-arm D-092 + Appendix F |

---

## §3 The D8 mechanism — **grounded pick, flagged for Joe** (D-093 clause 3)

**The invariant is locked (D8 / D-093 c3): no shared physical blob copy across erasure-fate.** The
design left me the *mechanism* ("Clair grounds `blob_ref` derivation and picks; flag it explicitly").

**Grounding (G4):** `blob_ref = hash_uri(ciphertext)` and `encrypt_blob` mints a **fresh per-blob key
+ fresh nonce on every call** — proven by the existing test `fresh_key_and_ciphertext_each_call`
(blob.rs:132, asserts `c1 != c2` for identical plaintext). Therefore **two independent `message.file`
sends of the same file produce different ciphertext → different `blob_ref` → different storage file.**
There is **no cross-send dedup** for independent sends. The store dedups only *identical ciphertext*,
which only a *re-used descriptor* (a forward/re-attach feature) could produce — and that does **not
exist** on the M12.1–M12.3 send path.

**PICK — Mechanism "the existing per-send-unique ciphertext-hash IS the per-send handle" (the design's
anticipated "clean handle choice" branch; NO salt, NO dedup change, NO storage-primitive reshape):**

- `blob_ref` (= `hash_uri(ciphertext)`) **is already** the per-send-unique, deletable storage handle —
  by construction of the fresh-per-blob key. The D-093-c3 invariant holds **today**, additive-zero to
  the shipped `BlobStore`.
- `plaintext_hash` (= `hash_uri(plaintext)`) **is already** the **content-hash retained as descriptor
  metadata** (separate from the storage key) — exactly D8's "content-hash … not the storage key." The
  hash that would enable identical-*file* detection is `plaintext_hash`, and it is metadata, never the
  store key. So policy-keyed dedup-within-a-shared-fate-set stays a **reserved future optimization**
  on `plaintext_hash`, never a correctness dependency.
- `delete(blob_ref)` erases exactly the redacted reference's own copy **+ its same-erasure-fate
  federated caches** (a cache shares `blob_ref` ⇒ it is the *same logical blob* under the *same*
  descriptor ⇒ same redact deletes all — permitted by D8). No other independent send shares `blob_ref`.

**Why no salt:** the design's alternative (a per-send salt folded into the storage handle) would
*split* `blob_ref` into `storage_ref` + `content_hash`, touching the descriptor schema, the
`BlobStore` key API, the M12.3 fetch wire (`BlobFetchRequest{blob_ref}`), and the W3 integrity-check
site — a non-trivial reshape of **three shipped primitives** to achieve a property the fresh-per-blob
key **already guarantees**. Mechanism-first: do not reshape a shipped primitive to re-derive an
existing invariant.

**Forward-constraint (flagged, the one residual):** the invariant relies on every `message.file` send
re-encrypting (fresh `blob_ref`). A **future** forward / re-attach feature MUST re-encrypt the bytes
(fresh key → fresh `blob_ref`), **never** copy an existing descriptor's `blob_ref`, or it would put
two events on one physical copy across erasure-fate (the A-09 hazard). Recorded as a code-comment at
`encrypt_blob` + the redact hook, and as an explicit M12.4-close note. **No code in M12.4 enables a
forward**, so the hazard cannot arise on the shipped surface.

> **JOE-LOCK ASK (D8 mechanism):** confirm the "existing per-send-unique `blob_ref` is the per-send
> handle; no salt, no dedup change; `plaintext_hash` is the metadata content-hash; forward-constraint
> flagged" pick. If Joe instead wants the explicit `storage_ref`/`content_hash` split **now** (future-
> proofing against a not-yet-built forward), C1 grows the descriptor + the fetch wire — flag at lock.

---

## §4 Runbook values — pick at lock (the design left these to me)

| Value | Pick | Grounding |
|---|---|---|
| **V1 `BlobStore::delete` signature/semantics** | `pub fn delete(&self, blob_ref: &str) -> Result<bool, BlobError>` — **idempotent**: `Ok(true)` if a file was removed, `Ok(false)` if already absent (erasure is idempotent — a re-applied redact / an already-gone blob is a no-op success, not an error); `Err(MalformedRef)` on a bad ref; `Err(Io)` on an fs failure (mirrors `put`/`get`'s error shape) | G5; the content-store idempotent philosophy (put is idempotent) |
| **V2 `10004` variant + wire** | new `BlobError::ErasureRefusedRetained` sibling to `Unavailable`/`TooLarge`; `to_wire_code` → `Some((10004, "erasure_refused_retained"))`; **RC-F-01 re-grep** at C1 before emitting | G14; M12.3's `10003` precedent |
| **V3 redact builder + content** | `build_message_redact_event(key, space_id, room_id, prev_events, target_event_id: &str) -> Event`, content `json!({ "target_event_id": target_event_id })`, `EventType::MessageRedact`; twin of `build_message_file_event` (G3); field name **`target_event_id`** (D1) | G3; D1 |
| **V4 node target→blob_ref resolution** | redact `content["target_event_id"]` → `rt.stores.get(&redact.space_id).get(&EventXgid(target))` → target `Event` → `content["attachments"]` deserialized `Vec<Descriptor>` → each `.blob_ref`. **Target absent on this node ⇒ no-op** (the blob isn't here either) — honest boundary, the event still converges (D3) | G6/G7/G2 |
| **V5 F2b read + default** | target `Event.sender` (the **author**, D2) → `rt.identity_registry.get(author).trust_assertion` (Option<Value>) → `value["claims"]` → `serde_json::from_value::<TrustClaims>` → `module_policy()?.erasability?.retention == Some(Retention::Retained)` ⇒ **refuse**. **Absent / unparseable / `Erasable` ⇒ erase** (right-to-erasure is the default; only an explicit module-declared `Retained` blocks — D-088: T1/no-module = max-erasable) | G8/G9; D2/D-088 |
| **V6 refusal signal** | on `Retained`: the redact event is **still admitted + fanned out** (D3); the blob is **not** deleted; for a `LocallySubmitted` redactor, send `blob_err(10004, "erasure_refused_retained")` carrying the redact `event_id` (reuses the `TransportMessage::Error` shape, G13). A federation-origin redactor is on another Node → no signal | D3/D7; G13 |
| **V7 redact CLI verb (C3)** | net-new `redact` client verb (`ops::redact` + clap `RedactArgs { space, room, target }`); **4-arm D-092** (CLI / run-path / batch / aicontrol) + **Appendix F** entry (J-323 obligation, the `--attach`/`fetch` precedent). The C2 spine drives the redact **in-process** (builder + dispatch directly); C3 adds the verb for the box-gated e2e | G15; D6 |
| **V8 minimal client tombstone (D6)** | client history-read (`ops::history` / the self-thread read) suppresses a `message.file` whose `event_id` is the `target_event_id` of any seen `message.redact` (don't render its descriptor, don't fetch its blob). Minimal — richer UI deferred | D6 |

---

## §5 Build sequence (spine-first; per-commit; Joe pushes each)

Three code commits, then Chat's close-bridge. Per-commit DoD §7. **No code until Joe locks §3 + §4.**

- **C1 — `BlobStore::delete` + the `10004` variant (the store spine; small, typed).**
  `xgen-core/src/blob_store.rs`: add `delete` (V1) + `BlobError::ErasureRefusedRetained` (V2) +
  `to_wire_code` arm. **RC-F-01 re-grep** the register first (confirm 10004 still free). Unit tests:
  `delete` present → `Ok(true)` + `contains` false after; `delete` absent → `Ok(false)` (idempotent);
  `delete` malformed → `Err(MalformedRef)`; `10004` wire tuple. No protocol surface — pure primitive +
  reject type. **Spines nothing RED-on-revert** (the convergence spine is C2).

- **C2 — the redact spine: builder + node hook + F2b + side-effect gate + federation (RED-on-revert).**
  - `xgen-core/src/message/exchange.rs`: `build_message_redact_event` (V3) + the `{ target_event_id }`
    content shape. (Permission arm `:792` + validation arm `:846` already admit `MessageRedact` — D1
    unchanged; the redact rides the DAG + fanout like any message event.)
  - `xgen-node/src/app.rs`: thread `blobs_dir: &Path` into `process_inbound` (G11, additive) + into its
    call site. In the **`DispatchOutcome::Accepted` arm** (G10, after persist, before returning the
    `FanoutRequest`): when `event.event_type == MessageRedact`, run the hook — re-lock `runtime`,
    resolve the target (V4), read the author's `Retention` (V5); **Retained ⇒** keep bytes + send
    `10004` to a LocallySubmitted redactor (V6); **else ⇒** `BlobStore::new(&blobs_dir).delete(blob_ref)`
    for each descriptor blob_ref (V1, drop the runtime lock before the fs delete). The event is
    **always** stored + fanned out (D3) — the gate is only on the delete.
  - **In-process node witnesses (RED-on-revert recorded for the spine):**
    - **WE1** (headline) — attach a blob in a `self`-thread space → redact → the `blobs_dir` file is
      gone → a subsequent `BlobStore::get`/fetch returns absent (`10003`). RED-on-revert: skip the
      delete → present → RED.
    - **WE2** (Retained refusal, **RED-on-revert D2**) — author with stored `Retention::Retained` →
      redact → blob **kept** + redactor gets `10004`; `Erasable`/absent author → blob **deleted**.
      RED-on-revert: drop the Retention check → Retained content erased → RED.
    - **WE3** (convergence, **RED-on-revert D3**) — the redact event is stored + in the fanout set
      **identically** regardless of the side-effect outcome. RED-on-revert: move the gate to
      *admission* (reject the redact on Retained) → two-node divergence repro → RED.
    - **WE4** (federated erasure) — a redact arriving `ReceivedViaFederation` at a node holding a
      cached copy (G12) fires the same hook → the cache is deleted. RED-on-revert: gate the hook to
      `LocallySubmitted` only → cache survives → RED.
    - **WE5** (shared-fate safety, D8) — two `message.file` sends of identical plaintext → **two**
      physical blobs (fresh-key, §3) → redact one → the other survives. RED-on-revert: a fixture that
      forces one shared copy + delete → the live blob vanishes → RED.

- **C3 — the `redact` verb + minimal client tombstone + box-gated e2e.**
  - `xgen-client`: `ops::redact` (build + sign + send a `message.redact` into a space/room targeting an
    event_id) + clap `RedactArgs` (V7) wired through **all four** D-092 arms (CLI `app.rs` · run-path ·
    `batch.rs` · `aicontrol` via `reconstruct_argv`) + an **Appendix F** entry (`docs/xgen_appendix_f_en.md`,
    the J-323 thin-verb obligation; a new `redact` row + a Session bump).
  - the **minimal client tombstone** (V8/D6): the history/self read suppresses a redacted `message.file`.
  - **box-gated real-binary e2e** (`#[ignore]`, sibling to `m12_2a_self_thread_e2e` / M12.3 C3): two
    same-identity clients + a real node — `register` → `self` → `send --attach` → `redact <target>` →
    the second client's `fetch` of the redacted blob returns `10003`; the tombstone hides it. **Joe runs
    the box-gated RUN separately.**

**(Chat) close-bridge = M12.4 close = M12 close** — canonical flips (CLAUDE PLAY, JOURNAL J-389,
ROADMAP, design/audit/runbook status), the WE6 D3-boundary close-claim recorded, the §3 forward-
constraint noted, D-093 honored; **M12 CLOSED → Round-2 final pre-UI gate next.**

---

## §6 Witnesses (RED-on-revert; WE6 is a close-claim, not a test)

WE1 blob-bytes erasure (headline) · WE2 F2b Retained refusal (RED-on-revert **D2**) · WE3 convergence
(RED-on-revert **D3**) · WE4 federated erasure · WE5 shared-fate safety (D8) — all C2 in-process; WE1
also at C3 box-gated. **WE6 (close-claim):** M12.4 erases the blob **content** (bytes, every reachable
copy) + refuses-on-Retained + tombstones for display; it does **NOT** crypto-shred the DAG-resident
residue (the `message.file` event existence + the plaintext descriptor key + any text body) — that is
**D3** (the descriptor `enc:`-wrap + the destroy-to-erase storage op, per A-05 / D-088 cascade). Stated
honestly at the witness set and the close, exactly as M12.1 stated W2's boundary.

---

## §7 Definition of Done + push blocks

**Per-commit gate (each of C1–C3):** `cargo build --workspace` 0-error · `cargo clippy --workspace
--all-targets --all-features -- -D warnings` clean · `cargo test --workspace` green (**1448/0** +
the commit's new tests) · for **C2**, RED-on-revert recorded in the commit message for the spine
(**D2 Retained refusal + D3 convergence gate**). The C3 box-gated e2e is `#[ignore]` (in-suite count
unaffected); **Joe runs the box-gated RUN separately.** Build target
`CARGO_TARGET_DIR=C:/cargo-targets/XGenProtocol`.

**Two-seat:** I commit code; Chat authors the canonical-record close-bridge. **Joe pushes each commit.**
PowerShell push block per commit (I hand it after the commit lands):

```powershell
git push        # ships <hash>  feat(M12.4): C# — <subject>
```

*(No "commit pushed" DoD line — unflippable inside its own commit; `Status: COMPLETED` on the
close-bridge is the shipped signal. Joe pushes.)*

---

## §8 Out of scope (later / reserved — do NOT pull in)

- **Crypto-shred of the DAG residue** — the descriptor `enc:`-wrap + the destroy-to-erase storage op
  (the shared text-path D3 / M8.7 S+L arc). M12.4 erases bytes, not the DAG event.
- **WORM / legal-hold production backend** — operator/module (D-093 clause 2); M12.4 builds only the
  protocol-layer **refusal**, reserves the hook.
- **Policy-keyed dedup / blob reverse-index / a forward feature** — reserved future optimizations (D8 /
  §3 forward-constraint); M12.4 v1 = no attachment dedup, no forward.
- **Richer redaction UI** — deferred to UI (D6 = minimal tombstone only).
- **Any change to M12.1 blob crypto maturity** (R-1) — untouched.

---

## §9 Sequence + entry (Rule 0)

this runbook → **Joe locks §3 (the D8 mechanism pick) + §4 (V1–V8)** → implement C1→C3 (spine-first) →
hand Joe the push per commit → Chat close-bridge → **M12.4 close = M12 close** → Round-2 final pre-UI
gate → UI → Streams.

**Entry (Rule 0):** `CLAUDE.md` PLAY → `JOURNAL.md` J-388 → `tasks/M12_4_ERASURE_DESIGN.md`
(M12.4-D1..D9) → `tasks/M12_4_ERASURE_PHASE0_AUDIT.md` → this runbook →
`tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (R-1 + the built primitives) → `DECISIONS.md` D-093 / D-088 /
AH-D1 → `docs/ROADMAP.md` (M12).
