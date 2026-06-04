# Arc H — End-to-End Encryption (PG-05) — Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Scope, method, vocabulary

**Arc H = PG-05** (E2E encryption boundary; ch2 L2148/2357, ch3 §3.10). The last Round-1 D-071 arc. Audit-first, no code until Joe approves a runbook.

**Locked scope (Joe, this session):** operationalise the Phase-2 scheme onto the live event path + ship the server-blindness proof + make the **key-granularity decision** IN. **Fenced:** openmls / RFC 9420 real-crypto integration (= D3) and the no-E2E client indicator (UI milestone). PG-05 closes **interface-locked / impl-deferred behind D3**, not ✅ DONE.

**Method.** Each finding grounded against live code (`Select-String`/read on the workspace, `.claude` worktrees excluded) and the ch3 §3.10 normative surface. Verdict vocabulary per gap-audit §0.4 (NO-GAP / GAP-CONFIRMED / SPEC-DRIFT / NEEDS-DESIGN). Findings carry stable IDs **AH-A#**.

**The audit's first job (Joe-directed):** the honest *buildable-now vs D3-gated* determination — §1.

---

## §1 — The buildable-now vs D3-gated determination (FIRST JOB)

PG-05 splits cleanly into two layers along the D-052 line. The split is the whole arc.

### §1.1 — Layer 1: Phase-2 epoch-key scheme — **BUILT, UNWIRED → buildable now (AH-A1)**

A complete `xgen-core/src/encryption/` module exists (`mod.rs` + four submodules), implementing the full MLS *interface* with a Phase-2 key schedule (`ChaCha20Poly1305 + SHA-256`), per D-052:

- `client_mls.rs` — `ClientMlsGroup` (epoch counter + members + `epoch_secret`), `add_member`/`remove_member` (advance epoch), `derive_epoch_key`, `encrypt_message`/`decrypt_message`, `EncryptedContent` (`enc:<base64url(epoch_8le‖nonce_12‖ciphertext)>`). Unit-tested: round-trip, post-removal isolation, epoch isolation, wrong-key failure.
- `delivery_service.rs` — `MlsDeliveryService` (per-room route/drain queue), `handle_encrypted_content` (opaque pass-through), `is_encrypted_content` (`enc:` detector). The Node-blind primitives.
- `key_package.rs` — `KeyPackageStore` (FIFO, single-use `consume`, ≥3-pool intent), `StoredKeyPackage`.
- `group.rs` — Node-side epoch tracking (opaque counter, no key material).

**Grounding — zero live callers (the keystone, AH-A1).** `Select-String` for `encryption::|ClientMlsGroup|derive_epoch_key|EncryptedContent|DeliveryService` across the workspace (worktrees excluded) returns only: (a) the module's own unit tests; (b) `xgen-client/src/app.rs:3873/4268/4796` — the multiparty **test fixtures** (`s1_groups`, `dm_group_eve`). No production send-path encrypts; the Node never routes through `MlsDeliveryService`; `KeyPackageStore` has no upload/consume handler. **This is the SR-F1 / AF-A1 pattern: rich machinery, no production seam.** No new crypto dependency is required to wire Layer 1. **Verdict: GAP-CONFIRMED (wiring), buildable now, self-contained.**

### §1.2 — Layer 2: RFC 9420 / openmls real crypto — **GATED = D3 (AH-A2)**

- **openmls is absent from every manifest.** `Select-String` for `openmls|mls-rs` across all `Cargo.toml` (worktrees excluded) → **zero hits**. Present crypto deps: `ed25519-dalek 2`, `sha2 0.10`, `chacha20poly1305 0.10` only. No HPKE, no x25519, no TreeKEM.
- **D3 is an explicit independent/parallel workstream.** `docs/ROADMAP.md` L766: *“D3 — MLS operationalisation. Wire shape already specced (Ch3 §3.10, Appendix I Part X.6); openmls integration pending. Runs as an independent parallel workstream alongside the M-series per D-066. Timing is open.”* (🟡). L277 lists it under “Parallel or blocked.” `DECISIONS.md` D-052 records the deferral.

**Verdict: GATED.** Arc H neither forces nor depends on D3. Layer-1 wiring is forward-compatible with the eventual swap *because* D-052 built the Phase-2 interface to equal openmls’s (only the key schedule changes). **Conclusion: Arc H operationalises Layer 1 + proves server-blindness + locks the wire/interface; Layer 2 stays D3.**

---

## §2 — The missing live seam (what “operationalise” means concretely)

What Layer-1 wiring must add (each grounded as absent):

| # | Seam | As-built | Verdict |
|---|------|----------|---------|
| AH-A3 | **Client send-path encryption** | `message.*` content sent as plaintext JSON; `encrypt_message` has no live caller | GAP-CONFIRMED |
| AH-A4 | **Node DS routing + blind-store** | `MlsDeliveryService.route`/`drain_for_recipient` + `handle_encrypted_content`/`is_encrypted_content` exist but are unwired into `xgen-node` connection-handling / fan-out; server-blindness is *designed, not enforced on the live path* | GAP-CONFIRMED |
| AH-A5 | **KeyPackage upload/distribute** | seven `mls.*` transport messages **are backed + round-trip-tested** (`xgen-common/src/wire.rs:131–141, 222–229, 315–322, 703–716`: `MlsKeyPackage{,Ack,Request,Response}`, `MlsCommit/Welcome/Proposal`), but no node handler binds them to `KeyPackageStore` | GAP-CONFIRMED (handlers; wire types present) |
| AH-A6 | **`state.mls_group_init` DAG anchor** | §3.10.10 registers it as an EventType; `Select-String mls_group_init\|MlsGroupInit` → **zero hits**. No DAG event records “Room R has an MLS group at epoch 0” | GAP-CONFIRMED |
| AH-A7 | **Epoch-advance ↔ membership events** | `ClientMlsGroup::add_member/remove_member` advance epochs in isolation; nothing ties an epoch advance to the actual `membership.*` events on the DAG (§3.10.5/.6 “MUST advance to a new epoch”) | GAP-CONFIRMED |
| AH-A8 | **Space encryption-mode field** | §3.10.8 mandates an `e2e_encryption` field, immutable after Space creation; `Select-String e2e_encryption\|encryption_mode\|EncryptionMode` → **zero hits**. Which Spaces encrypt is undefined in code | GAP-CONFIRMED |

**Scope note on AH-A8:** §3.10’s E2E scope is **`message.*` only** (`message.text/image/file/reaction/edit/delete`); `state.*` / `membership.*` / `system.*` / `federation.*` are explicitly NOT encrypted (Nodes must read them to validate/route/enforce). This bounds the encrypt seam to message content and is what makes server-blindness *checkable*: a Node validates signatures over the envelope + reads metadata, but the `content` blob is opaque.

---

## §3 — First-class finding: key granularity vs D-088 crypto-shred (AH-A9)

**The crux (Joe-flagged first-class).** As-built, message content is encrypted **directly under the per-epoch key** — `encrypt_message(epoch_key, epoch, plaintext)` → `enc:<epoch‖nonce‖ciphertext>` (`client_mls.rs:150`). There is exactly **one key per epoch**. ch3 §3.10.7 itself is epoch-keyed: *“encrypts using the current MLS epoch’s application secret, producing an MLS PrivateMessage.”*

**The mismatch.** D-088 (Arc I) deferred GDPR content-erasure to “PG-05’s boundary” via **crypto-shred**, which assumes a **per-erasure-unit erasable key**. With epoch-only keys, destroying a key erases the **entire epoch**, not one event — so an Arc-H that ships epoch-only keys with no per-unit seam forces the later erasure arc to retrofit the substrate. That is precisely the *“build on the wrong substrate”* trap D-088 exists to avoid. **Fence the erasure build; do NOT fence the granularity decision.**

**Candidate resolution to GROUND in design (not assumed here): envelope / layered keys.** MLS (epoch key) handles access + confidentiality and distributes a **per-message (or per-erasure-unit) content key `CK`**; content is encrypted under `CK`; the wrapped `CK` rides with the message; **erasure = destroy `CK`** — granular crypto-shred without weakening MLS forward secrecy. Separates MLS’s forward-secrecy concern from erasure’s retention concern.

**Seam viability (grounded, design to confirm):** the insertion point is clean — `encrypt_message` is a free function and `EncryptedContent` is a `String` with an extensible `enc:` envelope, so a layered format (`enc:` v2 carrying `wrapped_CK ‖ epoch ‖ nonce ‖ ciphertext`) is addable without disturbing the `is_encrypted_content` / event_trace blind-substitution convention. **But** §3.10.7’s current normative is single-layer epoch encryption — the envelope scheme is an **extension** to the spec, not a reading of it. Design must reconcile §3.10.7 + verify the `client_mls.rs` surfaces before committing.

**D-088 disposition (Joe-locked rule).** If Arc H picks a key granularity, that is a **D-088 amendment** — a full audit→design→lock re-walk in the design doc, **not a footnote**. This audit names the decision as first-class and records the amendment requirement; the design doc makes and locks the choice.

---

## §4 — PG-05 honest-close shape + cascade (AH-A10)

**Close shape.** With Layer 2 (openmls) fenced to D3, Arc H delivers the *operationalised scheme + server-blindness proof + wire/interface* — **not working RFC-9420 E2E**. Therefore **PG-05 closes design/interface-locked / impl-deferred behind D3**, the same honest shape as PG-02 (D-065 — not flipped to ✅ DONE). Be explicit in the gap-audit §5 row.

**The cascade (must be named).** PG-05-not-actually-built ⇒ D-088’s **content-erasure build stays gated behind D3 too**, not merely behind Arc H. The dependency chain is: `D-088 content-erasure → PG-05 real crypto → D3/openmls`. The **identity-orphan half** of D-088 remains PG-05-independent (could ride the Tier-1 auth-module rebuild), unchanged.

---

## §5 — Scope fences (locked)

- **IN:** operationalise Phase-2 on the live path (AH-A3–A8) + server-blindness proof + the **key-granularity decision** (AH-A9, D-088 amendment).
- **FENCE → D3:** openmls / RFC 9420 real crypto (AH-A2). Named, deferred, parallel, timing-open.
- **FENCE → UI milestone:** the no-E2E client indicator (§3.10.8 “clients MUST display a visible indicator”) — client/UI item, UI gated behind Round 2.
- **OUT (record):** multi-device per-identity KeyPackage management beyond the single-device happy path; Phase-3 nonce-counter management (`encrypt_message` notes a Phase-2 deterministic-nonce simplification — flag, do not fix here).

---

## §6 — Forks for the design doc (confirm-at-design, D-078)

- **AH-F1 (the granularity decision).** Envelope/layered `CK` vs epoch-only. Recommended: envelope. Lock as a D-088 amendment. **The arc’s central decision.**
- **AH-F2 — which Spaces encrypt.** Add the `e2e_encryption` Space field (AH-A8). All message Spaces by default, or DM-first then widen? Immutable-at-create per §3.10.8.
- **AH-F3 — group-init anchor.** Add `state.mls_group_init` EventType (AH-A6) vs derive group existence implicitly from membership. Spec registers the explicit event.
- **AH-F4 — epoch-advance trigger.** Where membership `add/remove` events drive `ClientMlsGroup` epoch advance (AH-A7), and how that interacts with M8 `derive_resolved` (encryption is client-side; the DAG carries ciphertext + `mls.commit`).
- **AH-F5 — server-blindness proof shape.** The deliverable test: a Node hosting an encrypting Space cannot recover `message.*` plaintext (assert opaque store + `event_trace` blind-substitution + DS pass-through), analogous to the migration two-node e2e.
- **AH-F6 — C-split.** Likely C1 = Space field + group-init + client encrypt send-path + Node DS blind-route + server-blind proof; C2 = KeyPackage upload/distribute + epoch-advance on membership + envelope `CK` granularity. Confirm at design.

---

## §7 — Loose threads (recorded, non-deraling)

- **Push status (resolved):** `git status -sb` = `## main...origin/main` (clean, no ahead-marker); `b712cc1` + `f55506c` are present in `git log` and **already pushed**. Nothing pending.
- **M8 number collision** (closed convergence M8 vs a pending multiparty pass) — parked for the Round-2 doc sweep. Not touched by Arc H.

---

## §8 — Conclusion

Arc H is a **wiring + interface-completion** arc, not a from-scratch crypto build. The Phase-2 scheme is fully built and unwired (AH-A1); openmls is genuinely gated (AH-A2 = D3). Operationalising Layer 1 ships a real, testable **server-blindness** guarantee while honestly leaving real-crypto to D3 — so **PG-05 closes interface-locked, with the named cascade to D-088’s content-erasure build** (AH-A10). The one decision that must be made *now* and not deferred is **key granularity** (AH-A9): an Arc-H envelope-key choice, locked as a D-088 amendment, prevents the wrong-substrate trap.

**Next (gated on Joe):** review forks §6 → write `tasks/ARC_H_E2E_DESIGN.md` (AH-D# locks; AH-F1 granularity = D-088 amendment re-walk) → Joe-lock → runbook → implement → close. No code until the runbook is approved.

**Audit complete (v1.0).**
