<script lang="ts">
  // dm-intro — the DM draft "start of conversation" page (M-RP-MEMBER-ACT Leg C-bis).
  //
  // 🛑 STRUCTURAL PLACEHOLDER ONLY — THIS FILE IS JOE'S, on the skin.css precedent. Leg C-bis-1 lands the
  // three structural slots (avatar · heading · paragraph, IN THAT ORDER), fed by props, so the `above`
  // socket on `stream-panel` has a tenant to mount and measure. Leg C-bis-2 feeds it from the `dmDraft`
  // store. NO copy is decided here (the locked sentence, §5.5, is Joe's to place), NO skin rule is added,
  // NO layout opinion is taken — Joe authors the real markup, wording, avatar wiring, and skin.
  //
  // WHY A COMPONENT, NOT A PROCESSED STRING (§5.8): `{counterpart_display_name}` is WIRE DATA authored by
  // a person you have never met, up to MAX_DISPLAY_NAME_LEN bytes, rendered on first contact. A component
  // makes every field a TEXT NODE, escaped by construction — no `{@html}`, no sanitiser surface (N-032).
  import { envelope } from '$common/components/base/envelope';

  // Props-fed placeholder contract (Leg C-bis-2 supplies these from the draft). `heading`/`body` are text
  // nodes; the avatar slot is a structural box Joe wires (entity-avatar or otherwise). Optional so the
  // never-mounted C-bis-1 state and any unfed field render honestly rather than throwing (N-091).
  let {
    heading = '',
    body = '',
    id,
  }: { heading?: string; body?: string; id?: string } = $props();

  const debug = () => ({ heading, hasBody: body !== '' });
</script>

<!-- Root class supplied by `envelope` (`dm-intro`) — no literal class here, so mergeClasses does not double
  it (the shipped no-dedupe defect). Children carry descriptive hooks for Joe's skin. -->
<div use:envelope={{ name: 'dm-intro', id, debug }}>
  <div class="dm-intro-avatar" aria-hidden="true"></div>
  <div class="dm-intro-heading">{heading}</div>
  <p class="dm-intro-body">{body}</p>
</div>
