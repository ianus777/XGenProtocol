# XGen Protocol — Arc I (GDPR Erasure / Right-to-be-Forgotten, PG-02) Design
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Frame

Design for PG-02, backed by `ARC_I_ERASURE_AUDIT.md` v1.0. **Design-only arc**: the deliverable is the canonical XGen erasure *architecture* + a protocol-wide DECISIONS stance (**candidate D-088**, §4) + a deferred-implementation sketch (§5). No C1/C2 code; the content-erasure *implementation* is substrate-gated on PG-05 (Arc H). PG-02 closes to **design-locked / impl-deferred**, not ✅ DONE (D-065). AI-D# are arc-local; the cross-arc invariant lives in D-088.

## §2 — The erasure architecture (the spine)

Three orthogonal questions, three settled answers:

1. **Content** (data *inside* events). Mechanism = **crypto-shredding**: content encrypted under an erasable key; the DAG retains ciphertext; the signature signs ciphertext and stays valid; key-destruction makes content unrecoverable **with the chain and all signatures intact, nothing mutated**. Requires the PG-05 boundary → implementation deferred. Blank-at-rest rejected as default (integrity scar + D-076 conflict); survives only as a named last-resort for legacy pre-PG-05 plaintext (§7), unbuilt. *Whether* content erasure is permitted is tier-graded (axis 3).

2. **Identity** (the pubkey↔natural-person binding). Mechanism = **orphan the binding** — remove the PII record (display name, Tier-claims / legal attestation) from the registry cache. **Touches no events**: pubkey persists as `sender`, every signature keeps verifying, chain untouched. Residue = orphaned pseudonymous key (anonymized data, outside Art.17). No verify-skip, no integrity change.

3. **Permission to erase is tier-graded — protocol binds the endpoints, the Auth Module declares the interior (keystone; OPEN-Q-I1 RESOLVED via AI-D8).** The `AuthTier` ladder is a monotonic erasability gradient (erasability decreases as identity-verification strength increases). The protocol fixes the endpoints and enforces the gradient; the module that vouches for an identity declares the in-between policy, because the module is the bearer of the real legal/policing relationship and the only party that knows the retention obligation.
   - **T1 (loosest, no module)** — protocol-default **max-erasable**: identity anonymized by binding-orphan (this *is* "delete"); content erasable. No module exists to declare otherwise, so the endpoint holds structurally.
   - **T4 (strictest, legal identity)** — protocol **binds** any module at this class to **destruction of no record (content or binding)**; total retention under Art.17(3) lawful basis. A module declaring a T4 identity erasable would not be issuing legal identity. Endpoint holds as a protocol constraint on what a module *may* declare.
   - **T2 (e.g. LinkedIn-type org) / T3** — **Auth-Module-declared**, bounded by the endpoints (AI-D8). Not a protocol constant; carried on the Trust Assertion.

**Residuals (stated, not solved):** (a) a non-complying / offline / out-of-jurisdiction replica still holds the cached binding — the protocol cannot compel another operator (§2.2-shape); (b) identity-orphan and content-shred are **complementary** — an orphaned key still links a person's events, so un-erased content / out-of-band correlates can re-identify; complete erasure needs both (and the AI-D9 materialization-layer scrub).

## §3 — Locked design decisions (AI-D#)

**AI-D1 — One uniform `erasure.request` event**, target discriminant `content` (EventXgid[s]) | `identity` (identity id). Carries requester authorization (AI-D6) + a lawful-basis tag for audit.

**AI-D2 — Content path = crypto-shred over PG-05; inert pre-substrate; tier-gated.** Specified but no destruction backend until PG-05; pre-PG-05 honestly inert (queued/rejected, never silently "done"). Permission tier-graded (AI-D4) — at T4 content destruction refused outright (legal hold), substrate-independent. Blank-at-rest not the default (audit AI-A2b).

**AI-D3 — Identity path = binding-orphan in the registry.** Remove PII fields from the self-certifying cache record; pubkey + signatures untouched. PG-05-independent (§5).

**AI-D4 — Tier-graded permission gate: principle + endpoints locked; interior module-declared.** Gate applies to **both** axes (content + identity). Locked: **T1 = max-erasable; T4 = no record destruction**; erasability monotonic in tier. Interior (T2/T3) = Auth-Module-declared (AI-D8). Refusal emits a defined `erasure_refused_legal_basis` error (code assigned at impl per Arc-E/F renumber discipline — do not guess).

**AI-D5 — Federated propagate / confirm / audit = reuse existing infra.** Erasure event flows through the federation push path (applier-reuse, Arc F/G-shape); **Tier 3+ requires delivery-confirmation acks**; every erasure + outcome writes to the **M6 protocol-audit-log**. Residual = non-complying replica (recorded unconfirmed).

**AI-D6 — Authorization model.** Origins: (a) data subject self-signed; (b) operator / legal authority compliance erasure. The tier-gate (AI-D4) holds regardless of origin — at T4 a legal authority may *compel disclosure* but the protocol still won't destroy the record (lawful-basis retention). Exact predicate = impl-time CP.

**AI-D7 — Complementarity is normative documentation.** Complete erasure requires orphan + shred (+ AI-D9 scrub); neither alone is represented as full Art.17 compliance.

**AI-D8 — Erasability interior is an Auth-Module-declared property of the Trust Assertion, carried in an *extensible* module-policy descriptor.** The module declares its tier's erasure/retention policy (bounded by the protocol-fixed endpoints of AI-D4); the protocol enforces what is declared. The descriptor is **forward-extensible by design** (§8): erasability is its *first* member, not its only one; unknown members are **preserved verbatim** (the shipped `EventType::Unknown` forward-compat posture). Concrete hook: a **TrustAssertion schema extension** (Arc E's primitive gains the descriptor) — sibling to the assertion's TTL and Arc G's jurisdiction, and the natural first exercise of the **Tier-1 auth-module rebuild**. This **resolves OPEN-Q-I1 by delegation** — the threshold gets its rightful owner instead of a guessed constant. Residual: the module is *trusted* to declare honestly within the endpoints, the same trust already placed in it to verify identity (§2.2-shape).

**AI-D9 — Erasure enrichments are implementation-within-frame, materialization-layer only.** Custom text filters, residual-identifier / correlate scrubbing, etc. operate on the **rebuildable materialization/display layer** (D-080 split: SQLite cache + client render), **never** the immutable DAG — so they are integrity-safe (not blank-at-rest) and **PG-05-independent**. The protocol defines the *capability*; the implementation chooses *thoroughness*. This is the implementation remedy for the AI-A6 re-identification residual.

## §4 — Candidate DECISIONS entry (cross-arc — to Joe-lock)

**D-088 — XGen erasure model: crypto-shred content, orphan identity, monotonic tier-graded permission (protocol binds endpoints, Auth Module declares interior).** Proposed text:

> Right-to-erasure in XGen's no-anonymity append-only federated model resolves along three axes. **Content** is erased by crypto-shredding over the encryption boundary (PG-05): the immutable DAG retains ciphertext, signatures stay valid, key-destruction makes content unrecoverable without mutating the log — no event deleted, no integrity invariant weakened. **Identity** is erased by orphaning the pubkey↔person binding in the registry cache; the pubkey persists as an anonymous token, all signatures keep verifying, the DAG is untouched. **Permission to erase is monotonic in identity-verification strength: the protocol fixes the endpoints, the Auth Module declares the interior.** T1 (no module) = max-erasable (binding-orphan + content); T4 (legal identity) = destruction of no record at all, retained under Art.17(3) lawful basis (the conscious counterpart to the no-anonymity pillar); T2/T3 = the issuing Auth Module's declared retention policy, carried on the Trust Assertion within the fixed endpoints — modules being the bearers of the real legal/policing function. The module-policy descriptor is forward-extensible (erasability is its first member; unknown members preserved verbatim) so unknown future module requirements have a home without a protocol change. Erasure *enrichments* (display-layer scrubbing/filters) are implementation-within-frame on the rebuildable materialization layer, never the DAG. Residual exposure (non-complying replicas; re-identification via correlates) is acknowledged and out of in-protocol scope, mirroring the jurisdiction stance. Blank-at-rest event mutation is rejected as a default mechanism (would weaken universal signature-verifiability; conflicts with D-076).

Protocol-wide invariant. D-088 is the next free number (current highest = D-087).

## §5 — Deferred-implementation sketch (post-PG-05)

- **Content-erasure (PG-05-gated):** wire the crypto-shred backend into AI-D2's path, behind the tier-gate.
- **Identity-erasure (PG-05-INDEPENDENT — buildable now):** AI-D3/D4/D5/D8 (binding-orphan + tier-gate + module-declared descriptor + federated propagate/confirm/audit) depend on none of PG-05; a deployment needing GDPR identity-erasure before E2E could implement this half as its own arc, riding the Tier-1 auth-module rebuild (which supplies the AI-D8 descriptor). This arc builds nothing (design-only, Joe-locked) but records the option.
- **AI-D9 enrichments:** materialization-layer, buildable any time.

Sequencing: **Arc H (PG-05) precedes the content-erasure build.**

## §6 — Canonical-doc reconciliation (close plan, doc-only)

- **ch1 L879–883** → point the "uniform tier-graded deletion Event" requirement at D-088 (event + monotonic gradient + endpoints + module-declared interior).
- **Appendix D §3.3** → replace "acknowledged-but-unsolved" with the resolved architecture + residuals.
- **ch3** → specify the `erasure.request` schema (AI-D1) + tier-permission endpoints (AI-D4) normative; the TrustAssertion module-policy descriptor (AI-D8) marked as the auth-module-rebuild hook; content destruction impl-deferred.
- **gap-audit §5** → PG-02 = **design-locked / impl-deferred behind PG-05** (NOT ✅ DONE); register annotated.
- **DECISIONS.md** → add D-088.

## §7 — Scope fence & honesty posture (D-065)

- OUT: content-destruction impl (PG-05); blank-at-rest as default (rejected); compelling external modules / non-complying replicas (operator/legal); **legacy pre-PG-05 plaintext** (named residual, last-resort blank-at-rest specified *with integrity cost stated*, builds nothing).
- The arc closes the **architectural** gap; content-erasure mechanism is explicitly not built; PG-02 does not flip to DONE. Stating T4 zero-erasability plainly *is* a deliverable.

## §8 — Open-doors principle (module-extensibility)

Auth-tier modules will very probably arrive with special requirements, limitations, and functions not yet known. The protocol and implementation must therefore keep **extension points open**, not pre-enumerate module behaviour: the AI-D8 module-policy descriptor is extensible with verbatim-preserved unknown members; the module interface stays the pluggable seam (institutional-independence); the materialization layer (AI-D9) absorbs deployment-specific behaviour without protocol change. Recorded here as a **design principle** and a **recurrence-candidate** for the Tier-1 auth-module rebuild milestone — *not* minted as a numbered DECISIONS principle now (promotion discipline: 3+ genuine instances). Flagged so it is not lost.

No DECISIONS change is made by this doc; D-088 is *proposed* for Joe-lock at close (D-069/D-074). Doc-only — suite unchanged at J-252's 1131/0/2, not re-run.
