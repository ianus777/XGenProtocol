# Arc H — End-to-End Encryption (PG-05) — Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Scope, preconditions, status

Implementation runbook for **Arc H / PG-05**, executing the locks in `tasks/ARC_H_E2E_DESIGN.md` v1.1 against the audit `tasks/ARC_H_E2E_AUDIT.md` v1.0. **Awaiting Joe approval — no code until approved.**

**Scope:** operationalise the Phase-2 epoch-key scheme onto the live `message.*` path + the **content-blindness proof** + the **envelope key-granularity substrate** (AH-D1, a D-088 amendment). **Fenced:** openmls/RFC 9420 (= D3); no-E2E client indicator (UI); destroy-to-erase op (erasure-impl arc). **PG-05 closes interface-locked / impl-deferred behind D3**, cascade `D-088 content-erasure → PG-05 real crypto → D3`.

**Two C-commits + a doc-only close.** Suite baseline: **1131**/0/2 (J-252). Build out: `C:/cargo-targets/XGenProtocol`.

## §1 — Session discipline (every pickup)

- **Rule 0:** CLAUDE.md PLAY → latest JOURNAL → this runbook §2/§3. Runbook is item 4, never item 1.
- All `E:\` writes via **Filesystem tools**; one writer per file per commit; **Clair implements, Joe pushes** — Claude never pushes.
- **D-074:** canonical-record changes ride the same commit as the work they record (the D-088 amendment + §3.10.7 ride C1).
- DoD checklists below **omit "commit pushed"** by convention; `Status: COMPLETED` on this doc is the shipped signal.

---

## §2 — C1: encryption boundary + the guarantee (the load-bearing commit)

Ships the boundary, the envelope substrate, the content-blindness proof, **and** the D-088 amendment + §3.10.7 extension (same commit, D-074).

### §2.1 — Work items

1. **Space encryption mode (AH-D2).** `SpaceState.e2e_encryption: bool`, set-once at create, default `false`. Thread through the three Space constructors + `build_space_create_event` + the DM-create path (the sites Arc G's `jurisdiction` threads through — AG-D4; **CP-5** pins them at pickup). No applier, no `state_key` arm — rides `SpaceState` `PartialEq` (M8-free). ch3 §3.10.8 field schema + Appendix C Space class.

2. **Group-init anchor (AH-D3).** Add `EventType::MlsGroupInit` → `"state.mls_group_init"` in `xgen-common/src/wire.rs` (`as_str`/`from_str` + the FC-D1 round-trip sweep — mirror the existing `mls.*` rows). `RoomState.mls_epoch: Option<u64>` (genesis 0); applier sets genesis; **Node-readable, never encrypted** (§3.10.1). Emitted by the creating client only when `space.e2e_encryption == true`. `state_key` shape = **CP-2**.

3. **Envelope encrypt send-path (AH-D1).** In `xgen-core/src/encryption/client_mls.rs`, add the **`enc:` v2 envelope**: `enc:<base64url( version_1 ‖ wrapped_CK ‖ epoch_8le ‖ nonce_12 ‖ ciphertext )>`.
   - generate `CK` = random 32B (`OsRng`) **per message** — **never** KDF'd from the epoch secret;
   - encrypt plaintext under `CK` (ChaCha20Poly1305);
   - wrap `CK` under the current epoch key;
   - emit the v2 envelope; keep v1 decode for back-compat; `version` byte discriminates.
   - `is_encrypted_content` / `event_trace` blind-substitution still key off the `enc:` prefix — **unchanged**.
   - Wire the client send-path: `message.*` content in an E2E Space becomes a v2 envelope **before** dispatch. Exact insertion site in `xgen-client` = **CP-4**.

4. **Node DS blind-route wiring (AH-A4).** Wire `MlsDeliveryService` + `handle_encrypted_content` (pass-through) + `is_encrypted_content` into the live `xgen-node` ingest/fan-out for `message.*` in E2E Spaces (Arc F's `app.rs` first-message-branch is the precedent). Node stores the opaque blob byte-identical; never decrypts. DS wiring site = **CP-4**.

5. **Content-blindness proof (AH-D5) — the headline test.** In-process, one E2E Space, one `message.text`:
   - (a) Node store byte-identical (`handle_encrypted_content` in == out);
   - (b) `event_trace` blind-substitutes the `enc:` content on the live path;
   - (c) DS routes `mls.welcome`/`mls.commit` as opaque bytes;
   - (d) Node-side content read = ciphertext only; a member client unwraps `CK` → original plaintext;
   - (e) **erasability invariant (= AH-D1(2), one invariant):** destroying the wrapped `CK` makes content unrecoverable **even given the epoch secret**; the Node (no epoch secret) is strictly weaker. **Test form:** hold the epoch secret, destroy the wrapped `CK`, decryption MUST fail.
   - **Name it precisely: a *content-blindness* proof.** Assert in the test/module docs the **metadata fence** — who/when/volume/which-Space stay legible; **no metadata/traffic-analysis blindness is claimed.**

6. **D-088 amendment + §3.10.7 (mechanical, this commit — D-074).**
   - **DECISIONS.md:** **append** a dated block to the **existing D-088 entry** — header e.g. `### Amendment (2026-..) — AH-D1 envelope granularity`. **Do not rewrite the original decision.** Record: (i) per-message random-`CK` envelope chosen; (ii) why — D-088 crypto-shred needs a per-erasure-unit key, epoch-only keys cannot give it; (iii) the threat-defended invariant (erasure defeats a holder of the epoch secret) + the non-weakening-of-MLS invariant.
   - **ch3 §3.10.7:** extend single-layer epoch encryption → the envelope (`CK` wrapped under the epoch secret), marked as the D-088-amendment substrate.

### §2.2 — C1 DoD

- [ ] `e2e_encryption` set-once, default-OFF, threaded (CP-5); M8 convergence pin (permuted `derive_resolved` equal).
- [ ] `state.mls_group_init` round-trips; `RoomState.mls_epoch` genesis applier; emitted only in E2E Spaces.
- [ ] `enc:` v2 envelope: per-message random `CK`, wrapped under epoch key, v1 decode retained.
- [ ] Client send-path encrypts `message.*` in E2E Spaces; Node DS blind-routes + stores opaque.
- [ ] Content-blindness proof: 5 assertions incl. the threat-defended erasability invariant (e); metadata fence asserted in docs.
- [ ] DECISIONS.md D-088 **amendment block appended** (original intact); ch3 §3.10.7 extended — **same commit**.
- [ ] `cargo test --workspace` green (baseline +N); build all-targets 0; clippy clean (default **and** `--all-features`).

---

## §3 — C2: KeyPackage + epoch lifecycle

1. **KeyPackage upload/distribute (AH-A5).** Bind the already-backed `mls.*` transport messages (`MlsKeyPackage{,Ack,Request,Response}`) to `KeyPackageStore` in `xgen-node`: store on upload, `consume` (single-use) on add-member, **≥3-pool maintenance** + **expired-discard** (§3.10.3 MUSTs). Error codes from the **5001–5005 band** — **CP-3** (grep the band; do not guess — the 6007→6009 lesson).
2. **Epoch-advance on membership (AH-D4).** On `membership.{join,leave,kick,ban}` commit in an E2E Space, the affected client emits an `mls.commit` (Phase-2: advance `ClientMlsGroup` epoch secret); DS routes Welcome/Commit opaque. Downstream of resolved membership — **no new M8 surface**. **Concurrent commit-race fenced to D3** (single-committer happy path only; assert + comment the fence).
3. **Tests:** KeyPackage pool/expiry/single-use; epoch advance on add/remove; removed-member-cannot-decrypt-future on the wired path.

### §3.1 — C2 DoD

- [ ] `mls.*` handlers wired to `KeyPackageStore`; ≥3-pool + expiry + single-use enforced.
- [ ] Epoch advance fires on membership change in E2E Spaces; opaque Welcome/Commit routing.
- [ ] Commit-race fence to D3 explicit in code/comments.
- [ ] Suite green; build 0; clippy clean (default + `--all-features`).

---

## §4 — Close (doc-only, D-074)

- [ ] **gap-audit §5 PG-05 → interface-locked / impl-deferred (D3-gated)** — *not* ✅ DONE (D-065; PG-02 shape). Register: `Open 1/13` (PG-02 remains; PG-05 interface-locked).
- [ ] **Cascade note** recorded: `D-088 content-erasure → PG-05 real crypto → D3`; identity-orphan half stays PG-05-independent.
- [ ] ch3 §3.10 reconcile (§3.10.7 already done at C1) + Appendix C/I reconcile (KeyPackage, `state.mls_group_init` in the EventType registry).
- [ ] ROADMAP: Arc H ⚫ + D3 row updated (openmls still pending; interface now operationalised); paired CLAUDE.md PLAY flip.
- [ ] JOURNAL entry; CLAUDE.md PLAY block.
- [ ] **AH-D# promotion eval:** AH-D1 **promoted into D-088** (record explicitly that **D-088 now carries an amendment** so a future reader sees it); AH-D2…D6 stay **arc-local** (D-069).

---

## §5 — Confirm-at-pickup (D-078)

- **CP-1** — `enc:` v2 byte layout + `version` discriminator; `EncryptedContent` wrap/unwrap API shape.
- **CP-2** — `state.mls_group_init` `state_key` shape + `RoomState.mls_epoch`.
- **CP-3** — error-code mapping into the **5001–5005** band (grep first).
- **CP-4** — client send-path encrypt insertion site (`xgen-client`) + Node DS wiring site (`xgen-node/app.rs` ingest/fan-out).
- **CP-5** — `e2e_encryption` threading through the three Space constructors + `build_space_create_event` + DM-create (mirror AG-D4).

## §6 — Honest residue / fences (D-065)

D3 (real openmls + concurrent-commit resolution); UI no-E2E indicator (Round-2); destroy-to-erase op (erasure-impl arc); Phase-2 deterministic-nonce (less load-bearing under per-message `CK`, but proper counters = D3). **Content-blind ≠ blind** — metadata/traffic-analysis explicitly not claimed.

**Runbook v1.0 — awaiting Joe approval. No code until approved.**
