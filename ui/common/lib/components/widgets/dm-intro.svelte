<script lang="ts">
  // dm-intro — the DM draft "start of conversation" page (M-RP-MEMBER-ACT Leg C-bis).
  //
  // 🔒 JOE'S FILE, on the skin.css precedent. C-bis-1 landed the three structural slots (avatar · heading ·
  // paragraph, IN THAT ORDER). C-bis-2 PLACES the locked content under Joe's 2026-08-09 amendment (runbook
  // §0 / task doc §5.5): "make a placeholder, but with content. i will check it independently afterwards
  // and maybe i do some updates manually or leave it as is." This PLACES a locked string; it does NOT
  // author one — §5.5's sentence is Joe's, ruled verbatim at J-703. No skin rule is added (the page ships
  // UNSKINNED — §5.5's truncation is skin, and lands when Joe skins it).
  //
  // THE COPY, VERBATIM (§5.5): the heading is the counterpart's name (or `…`+last-8 for a nameless one,
  // §5.6-bis); the paragraph is Joe's locked sentence with the SAME name interpolated ⇒ the name appears
  // TWICE (Discord's shape, PROVISIONAL). The word "stream" is load-bearing, NOT a paraphrase of Discord's
  // "history": the client holds no message history (J-598 ③), so "history" would be a false statement.
  //
  // 🔑 R-3 — stream-panel resolves the LABEL and passes it as DATA (`name`); the SENTENCE is composed HERE,
  // so the copy lives in the file that owns it and Joe edits ONE place (stream-panel carries no wording).
  //
  // WHY A COMPONENT, NOT A PROCESSED STRING (§5.8): `name` is WIRE DATA authored by a person you have never
  // met, up to MAX_DISPLAY_NAME_LEN bytes, rendered on first contact. `{name}` stays a TEXT NODE, escaped
  // by construction — no `{@html}`, no sanitiser surface (N-032). This is the entire reason the page is a
  // component and not a processed string.
  //
  // NO xgid row (§5.6-bis) · NO control whose verb does not exist (§5.7) · the avatar is an aria-hidden
  // structural box (there is no avatar source to wire, and none is invented). Joe adds later elements here.
  import { envelope } from '$common/components/base/envelope';

  // `name` = the counterpart's resolved display label (or `…`+last-8), passed as DATA by stream-panel
  // (C-bis-2). Optional so any unfed state renders honestly rather than throwing (N-091). NOTE: unrelated
  // to the envelope's `name: 'dm-intro'` below — that is a literal object key, not this variable.
  let { name = '', id }: { name?: string; id?: string } = $props();

  const debug = () => ({ name, hasName: name !== '' });
</script>

<!-- Root class supplied by `envelope` (`dm-intro`) — no literal class here, so mergeClasses does not double
  it (the shipped no-dedupe defect). Children carry descriptive hooks for Joe's skin. -->
<div use:envelope={{ name: 'dm-intro', id, debug }}>
  <div class="dm-intro-avatar" aria-hidden="true"></div>
  <div class="dm-intro-heading">{name}</div>
  <p class="dm-intro-body">This is the start of private direct message stream with {name}.</p>
</div>
