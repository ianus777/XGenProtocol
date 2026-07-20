// echo-status.ts — the PURE send-status rules (M-RP6.3 Leg D2/D3). Extracted from the echo store for the
// same reason `derive.ts` / `grouping.ts` / `mounts.ts` / `transform.ts` were: the store is a `.svelte.ts`,
// and the node vitest harness cannot load one. These three functions encode locks #6 and #7 — the rules that
// decide what a user is TOLD about a message and whether they are handed a button that can double-post it —
// so they are worth more as tests than as comments.
//
// No runes, no store imports, no DOM. Pure functions over plain values.

/**
 * The four honest outcomes plus `pending`.
 *
 * ⚠️ `pending` IS NOT A FIFTH OUTCOME — §9.11.1 is explicit that in-flight is "the state before one exists".
 * It is modelled because a row must render during the wait, and rendering nothing would be worse.
 */
export type SendStatus = 'pending' | 'accepted' | 'rejected' | 'timed_out' | 'failed';

/**
 * Narrow the wire string ONCE so no consumer ever branches on a bare string.
 *
 * 🔑 AN UNRECOGNISED STATUS MAPS TO `timed_out`, AND THE CHOICE IS D6, NOT TIDINESS. The four arms are
 * exhaustive in Rust today (`resident.rs`), so this can only fire on a version skew — but the fallback still
 * has to be the SAFE claim, and the two candidates are NOT symmetric:
 *
 *   - `failed` claims "never reached the wire" AND RETRIES FREELY (lock #7). A wrong guess there hands the
 *     user a retry button that can place a PERMANENT, IDENTITY-ATTRIBUTED DUPLICATE on the federated
 *     network — the one irreversible harm §3.1 exists to prevent.
 *   - `timed_out` claims "we do not know", which is exactly true of a status we do not recognise, and
 *     carries NO retry affordance.
 *
 * Under uncertainty, say you do not know.
 */
export function narrowStatus(status: string): SendStatus {
  if (status === 'accepted' || status === 'rejected' || status === 'timed_out' || status === 'failed') {
    return status;
  }
  return 'timed_out';
}

/**
 * 🔒 LOCK #6 — THREE VISUAL STATES, NOT TWO (plus `pending`, the quiet fourth):
 *
 *   accepted          -> 'sent'
 *   timed_out         -> 'unresolved'   its OWN state: the node MAY hold it
 *   rejected + failed -> 'not-sent'     one state, DIFFERENT copy (the causes differ)
 *   pending           -> 'pending'      before an outcome exists
 *
 * Collapsing `timed_out` into either neighbour is the D6 lie verbatim — which is why it is a function with
 * a test rather than a ternary someone can "simplify" later.
 */
export function toneOf(status: SendStatus): 'sent' | 'unresolved' | 'not-sent' | 'pending' {
  if (status === 'accepted') return 'sent';
  if (status === 'timed_out') return 'unresolved';
  if (status === 'pending') return 'pending';
  return 'not-sent';
}

/**
 * 🔒 LOCK #7 AS NARROWED AT §3.1 — `failed` ONLY.
 *
 *   failed    -> true   never reached the wire, so a retry CANNOT duplicate anything
 *   rejected  -> false  a deterministic refusal; it will be refused again
 *   timed_out -> false  ⚠️ THE ONE THAT MATTERS. The node may already hold the message, so a retry can put
 *                       a second copy on the federated network, permanently, under the user's identity.
 *                       Not building the affordance is reversible; shipping it and letting the habit form
 *                       is not.
 *   pending   -> false  there is nothing to retry yet
 *
 * ONE predicate, read by BOTH the store's refusal and the widget's button (N-126: a highlighted affordance
 * and the committed action must be the same function call, not two that are supposed to agree).
 */
export function isRetryable(status: SendStatus): boolean {
  return status === 'failed';
}
