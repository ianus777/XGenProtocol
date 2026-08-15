<script lang="ts">
  // message-intro — the DM welcome intro as it renders ON A MESSAGE ROW (M-RP-INTRO Leg 3, §4.3). The THIRD
  // tenant of the `bodyExtras` socket M-RP6.9 built, after `send-status`.
  //
  // 🛑 THIS IS NOT `dm-intro`, AND THEY MUST NOT BE MERGED (§7.4 / `G-8a`). `dm-intro` is the PRE-send draft
  // page in the `above` socket — the "start of conversation" placeholder the sender sees before a DM exists.
  // This is the POST-send artefact: a payload that travelled, attributed to its author, sitting inside their
  // first message. Two different objects wearing one word.
  //
  // 🔒 WHY A COMPONENT AND NOT A PROCESSED STRING (§7.3, the `dm-intro` reasoning at its strongest). The
  // payload is authored by a person the reader has NEVER MET and arrives on first contact. `{headline}` and
  // `{blurb}` stay TEXT NODES, escaped by construction — NO `{@html}`, NO sanitiser, NO markup path. Markup
  // belongs to `M-RP-PROCESSOR-RENDER`, a separate milestone whose own note says it must not be scoped in a
  // single sitting. Opening a sanitiser surface here would be doing that milestone's most dangerous half by
  // accident, on the one surface where the content is least trusted.
  //
  // 🔒 IT RENDERS INSIDE MESSAGE CHROME, NEVER AS SYSTEM CHROME (`I1`, `D-113` S-5, ruled at J-701). It sits
  // in `bodyExtras` — below the body, OUTSIDE the header guard — so it is attributed to its sender, survives
  // a grouped continuation row, and never borrows the system's voice for a stranger's words.
  //
  // ⚠️ NOTHING TYPE-CHECKS THE PROP BAG (`B-8`): the socket types its registry as components taking NO
  // props, while `WidgetMount.props` is `Record<string, unknown>`. So `intro` is declared `unknown` and
  // re-validated HERE, at runtime, with the SAME rule the projection used. That is not belt-and-braces — it
  // is the only thing standing between a malformed mount and a broken row, and it makes the empty render
  // below a REACHABLE branch rather than dead code (N-091, the `send-status` `localId` precedent).
  import { envelope } from '$common/components/base/envelope';
  import { normaliseIntro } from './stream/derive';

  // `id` is supplied by `resolveMounts` from the host's own prefix; the widget never invents one.
  let { intro = undefined, id }: { intro?: unknown; id?: string } = $props();

  // ⚠️ BOUND THE RENDERED LENGTH. An unbounded stranger-authored blurb is a layout weapon on first contact,
  // and it cannot be bounded at the input instead: `textfield` does not forward `{...rest}`, so the composer
  // cannot set a `maxlength` without a `core` change — and a composer bound would only ever constrain OUR
  // OWN sends, never a peer's. Bounding at the RENDER covers both directions with one rule.
  // 🔓 THE VALUES ARE JOE'S (`D-138`) — shipped plausible rather than blank, because something that does not
  // render cannot be looked at. PROVISIONAL → `M-RP-SKIN`.
  const HEADLINE_MAX = 120;
  const BLURB_MAX = 600;

  // Truncation is a plain slice with an ellipsis: no word-boundary cleverness, because a payload in a script
  // with no spaces would defeat it and the honest failure is a visibly cut line, not a silently dropped word.
  const clamp = (s: string, max: number): string => (s.length > max ? `${s.slice(0, max)}…` : s);

  const safe = $derived(normaliseIntro(intro));
  const headline = $derived(safe?.headline ? clamp(safe.headline, HEADLINE_MAX) : '');
  const blurb = $derived(safe?.blurb ? clamp(safe.blurb, BLURB_MAX) : '');

  const debug = () => ({
    // `valid` distinguishes "no intro was passed" from "an intro was passed and rejected" — the two are
    // indistinguishable on screen, which is exactly why the getter must tell them apart (the `N-099` shape).
    present: intro !== undefined && intro !== null,
    valid: safe !== null,
    hasHeadline: headline !== '',
    hasBlurb: blurb !== '',
    headlineTruncated: (safe?.headline?.length ?? 0) > HEADLINE_MAX,
    blurbTruncated: (safe?.blurb?.length ?? 0) > BLURB_MAX,
  });
</script>

<!-- Root class supplied by `envelope` (`message-intro`) — no literal class here, so `mergeClasses` does not
  double it (the shipped no-dedupe defect, the `dm-intro` precedent). The root is ALWAYS mounted while the
  mount resolves, so a rejected payload renders an EMPTY box rather than a broken one — and the row above it
  still carries the sender's sentence, which is the whole of 1-bis. Children carry descriptive hooks for
  Joe's skin. -->
<div use:envelope={{ name: 'message-intro', id, debug }}>
  {#if headline}<div class="message-intro-headline">{headline}</div>{/if}
  {#if blurb}<p class="message-intro-blurb">{blurb}</p>{/if}
</div>

<style>
  /* STRUCTURAL ONLY, and PROVISIONAL — discharged at `M-RP-SKIN` (the `composer-panel` / `stream-panel`
     precedent). The intro must stay INSIDE the message row: `min-width: 0` lets it shrink below its content
     and `overflow-wrap: anywhere` breaks an unbreakable stranger-authored token (a long URL-shaped string)
     so it wraps rather than pushing the row wider or migrating a horizontal scrollbar to the document.
     NO colour, NO font-size, NO weight, NO border — `.message-intro*` is Joe's to skin (`D-138`: the
     mechanism is here, the values are his). */
  .message-intro-headline,
  .message-intro-blurb {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .message-intro-blurb {
    margin: 0;
  }
</style>
