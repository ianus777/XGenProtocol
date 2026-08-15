<script lang="ts">
  // composer-panel — R6 (M-RP6.3 Leg D2). A `kind: system` region widget; the swap is DERIVED via a
  // `CLIENT_PLUGINS` descriptor (`surface: 'region'`, `regionId: 'composer'`), exactly like stream-panel /
  // rooms-panel — there is NO `app_client` register line (F-1 / N-116: that anchor went stale a generation
  // ago and `buildWidgetRegistry` picks the descriptor up).
  //
  // ⚠️ THIS IS A BUILD, NOT A WRAP. R6 did not exist. And the milestone is not "add a text box": measured at
  // J-559, `sendMessage()` had exactly ONE caller — the DEV bridge — so the whole send path tree-shook out
  // of production. IN A RELEASE BUILD BEFORE THIS WIDGET, THE CLIENT COULD NOT SEND A MESSAGE AT ALL.
  //
  // W-3 HOLDS AND IS WHAT SHAPES THE FILE. `region-node` passes a region widget ONLY its `regionId`, and
  // there are ZERO `@tauri-apps` imports under `ui/common` (verified). So the composer cannot receive the
  // send function as a prop and cannot import Tauri: the transport is STORE-MEDIATED (the shell injects it
  // into the echo store at boot). That is N-096 — store-mediation is the only channel, not a preference.
  //
  // 🔒 THE ROOM COMES FROM THE SHARED LATCH, NOT THE BUS (§2.1). Reading `selection.current` would grey this
  // widget out while the user is still looking at the conversation — click a Space in the tree, the stream
  // keeps showing its latched room, and the composer would refuse with nothing on screen explaining why.
  // Sharing R5's latch means THE MESSAGE ALWAYS GOES TO THE CONVERSATION YOU ARE LOOKING AT.
  //
  // 🔒 LOCK #12 — TYPING YES, SENDING NO. With no room latched the textarea stays live and the send button
  // is disabled. Silently accepting a sentence that goes nowhere is the worst of the three options; killing
  // the textarea mid-thought is the second worst (D6: block send, NEVER block typing).
  //
  // 🔒 THE DM DRAFT IS THE SECOND SEND TARGET (§5.2/§5.3). A member with no existing DM opens a draft
  // (dmDraft.active); its room does not exist yet, so `canSend` is false FOR IT and the gate widens to
  // `canSend || dmDraft.active`. Two things follow. (1) The draft's text routes through `dmDraft` (keyed by
  // counterpart, survives navigation) via a function binding, so the DM draft and the room never share one
  // buffer — otherwise a private draft would surface in the room's composer the instant the draft closed.
  // (2) The submit() draft branch sits ABOVE the early return: opening a draft leaves the latch pointing at
  // the group room you came from (v1.3), so the normal path would `echo.send` INTO that stale room — the
  // draft must be intercepted first.
  //
  // 🔒 C-bis-4 — THE DRAFT BRANCH NOW CREATES THE DM (§5.2). It runs `create_dm_space` (injected into
  // `dmDraft` by the shell, W-3), latches the real room the result carries, echoes the sent text into it,
  // and clears the draft. On FAILURE (`create` returns null) it latches nothing, sends nothing, clears
  // nothing — so NOTHING implies the DM exists — and the draft stays open with its text (§5.4, D-065), the
  // cause in `dmDraft.error`.
  //
  // 🔒 OFFLINE STILL SENDS, AND ON PURPOSE. The shell's lifecycle guard turns a send during an outage into
  // an immediate `failed` — which lock #7 lets the user RETRY FREELY, because `failed` means it never
  // reached the wire. So the sentence is PRESERVED, visible and retryable, instead of sitting in a textarea
  // the user may close. §3.1's own table names `failed` "the common outage path", which only happens if the
  // send is allowed through. Disabling the button offline would look tidier and would lose the words.
  //
  // 🔒 THE TEXT PROCESSOR IS WIRED HERE, AND HERE ONLY (M-RP-PROCESSOR-WIRE Leg B, §3.5). Of the two
  // text-input call sites in the whole client+common tree, this is the one that gets it. The other —
  // `substitutions-editor`'s own textarea — is refused permanently: processing the box where you AUTHOR
  // substitution rules is a feedback loop, where typing `:)` into the rule list rewrites the rule you are
  // in the middle of writing. The policy is default OFF everywhere, opt in per call site, behind two
  // gates: the atomic must forward `{...rest}` (`password-field` and `textfield` do not, so processing
  // them is structurally impossible), and the consumer must ask. The criterion is COMPOSING vs
  // CONFIGURING — composing is prose written in flow, where the transformation is visible and correctable
  // as it happens; configuring is a value stored and reused, where the rewrite lands silently and nobody
  // is watching. A message is the composing case, which is the whole reason this widget qualifies.
  //
  // 🔒 M-RP-INTRO LEG 2a — THE INTRO IS AUTHORED HERE, AND IT IS OPT-IN, NEVER AUTOMATIC (§2.2/§3.0). The
  // affordance is a disclosure that exists ONLY while a DM draft is active: the intro is a FIRST-CONTACT
  // artefact, so the surface that authors it is the surface that opens a first contact. A send with the
  // disclosure untouched — or opened and left empty — is BYTE-IDENTICAL to today's: no key at all, not an
  // empty one (`N-182`). ⚠️ An intro the user did not choose to send would be stranger-authored content
  // their own client authored FOR them, which is a different object from the one J-701 ruled on.
  //
  // 🛑 AND `text` IS AUTHORED INDEPENDENTLY AND STAYS REQUIRED (1-bis). Writing an intro does not populate,
  // replace or derive the sentence, and a user who writes an intro and NO sentence cannot send: `hasText`
  // reads the SENTENCE buffer only, so Send stays disabled — the composer says so before `desktop.rs:313`
  // has to refuse it silently.
  import { envelope } from '$common/components/base/envelope';
  import Textarea from '$core/components/data-independent/textarea.svelte';
  import Textfield from '$core/components/data-independent/textfield.svelte';
  import Toggle from '$core/components/data-independent/toggle.svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import { spaceLatch } from '$common/stores/space-latch.svelte';
  import { echo } from '$common/stores/echo-state.svelte';
  import { dmDraft } from '$common/stores/dm-draft.svelte';
  import { substitutions } from '$common/components/processor/store.svelte';
  import { processor } from '$common/components/processor/processor';
  // M-RP-INTRO Leg 2a — the ONE validation rule, shared with the read side (`derive.ts`). The composer calls
  // it at the SEAM, never per keystroke.
  import { normaliseIntro, type MessageIntro } from './stream/derive';

  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // The ROOM composing buffer — a single unkeyed string, the pre-C-bis behaviour unchanged (text persists
  // across room switches; not keyed by room). It is used ONLY when no DM draft is active.
  let draft = $state('');

  // C-bis-3 — THE TEXT ROUTES THROUGH `dmDraft` WHILE A DRAFT IS ACTIVE (§5.3). The composer has ONE editable
  // buffer; a DM draft and a room share this widget, so without routing the two would collide — a private
  // draft's text would sit in the room's composer the moment the draft closes, and vice versa. A function
  // binding (get/set) keeps `bind:value` — and therefore the processor attachment — untouched, and makes the
  // buffer PULL from the right source instead of syncing two states with a bidirectional effect (N-136 warns
  // about exactly that read-modify-write shape). Active ⇒ the store's per-counterpart text (survives
  // navigation, keyed by id, restored on re-open); otherwise ⇒ the local room buffer.
  const composerText = (): string => (dmDraft.active ? dmDraft.text : draft);
  const setComposerText = (v: string): void => {
    if (dmDraft.active) dmDraft.setText(v);
    else draft = v;
  };

  // ── M-RP-INTRO Leg 2a: the intro disclosure ────────────────────────────────────────────────────────────
  // Whether the disclosure is EXPANDED. Widget-local and deliberately NOT keyed by counterpart: it is a view
  // preference, not content. The CONTENT is keyed by counterpart in `dmDraft` (the `_texts` shape), so
  // switching drafts repopulates the fields with that person's intro while the panel stays as the user left
  // it. 🛑 Collapsed by DEFAULT — that is what makes the intro opt-in rather than a form to dismiss.
  let introOpen = $state(false);

  // Function bindings, the `composerText` shape one block up: the fields PULL from the store rather than
  // syncing two states with a bidirectional effect (N-136 warns about exactly that read-modify-write).
  // ⚠️ THE BUFFER HOLDS RAW TYPED TEXT. `normaliseIntro` runs at the SEAM, not here — stripping blanks per
  // keystroke would eat a leading space as the user typed it.
  const introHeadline = (): string => dmDraft.intro?.headline ?? '';
  const introBlurb = (): string => dmDraft.intro?.blurb ?? '';
  function writeIntro(headline: string, blurb: string): void {
    // An entirely empty form stores NOTHING (absent-not-empty, N-182) — so opening the disclosure, typing,
    // then clearing it again leaves a send byte-identical to one that never opened it.
    dmDraft.setIntro(headline === '' && blurb === '' ? null : { headline, blurb });
  }
  const setIntroHeadline = (v: string): void => writeIntro(v, introBlurb());
  const setIntroBlurb = (v: string): void => writeIntro(introHeadline(), v);

  // What would actually go on the wire, validated by the SAME rule the receiver validates with. `null` ⇒ no
  // key. Read by `submit()` and by the getter, so the verify pass sees exactly what the send will carry.
  const sendableIntro = $derived(dmDraft.active ? normaliseIntro(dmDraft.intro) : null);

  // Functional copy, PROVISIONAL (wording and appearance are Joe's — M-RP-SKIN; the PLACEHOLDER precedent
  // three lines down). Shipped with plausible values rather than blank, because something that does not
  // render cannot be looked at (D-138).
  const INTRO_DISCLOSURE = 'Add an intro';
  const INTRO_HEADLINE_PLACEHOLDER = 'Headline (optional)';
  const INTRO_BLURB_PLACEHOLDER = 'A sentence or two about you (optional)';

  // Lock #12's ONE predicate, read from the shared latch so R5 and R6 can never disagree about which room
  // is active (a second copy of this rule is the D-067 drift the lift exists to prevent). C-bis-3 widens the
  // send gate to `canSend || dmDraft.active`: a draft has no latched room (its room does not exist yet), so
  // `canSend` is false FOR THE DRAFT'S OWN ROOM — but the draft is a legitimate send target. The latch that
  // IS still set points at the group room the user came from, which is exactly why the draft must never ride
  // the normal path (see submit()).
  const canSend = $derived(roomLatch.canSend);
  const hasText = $derived(composerText().trim().length > 0);
  const sendEnabled = $derived((canSend || dmDraft.active) && hasText);

  // Functional copy, PROVISIONAL (appearance and final phrasing -> M-RP-SKIN; the stream-panel precedent).
  // The two placeholders say DIFFERENT things because the truths are different (N-091): no room selected is
  // not the same as an empty box in a real room.
  const PLACEHOLDER = 'Write a message…';
  const PLACEHOLDER_NO_ROOM = 'Select a room to send a message.';

  function submit(): void {
    // ⚠️ SAFETY-CRITICAL ORDERING (C-bis-3, §5.2): the draft branch goes ABOVE the early return. During a
    // draft the latch STAYS SET at the group room the user came from (v1.3 — opening a draft never moves the
    // latch), so `roomLatch.effective*` are NON-null and the normal path below would `echo.send` the draft's
    // text INTO that stale room. Intercepting here is the only thing preventing that leak — a disabled button
    // is a courtesy, never a guarantee, so it cannot be left to `sendEnabled`. No fabricated roomId.
    if (dmDraft.active) {
      const draftText = composerText().trim();
      if (!draftText) return;
      const counterpart = dmDraft.counterpart;
      if (counterpart == null) return; // active ⇒ counterpart set; defensive, and it narrows the type
      // C-bis-4 (§5.2) — the real sequence, in an async helper. Capture the counterpart + text NOW: `create`
      // is awaited, and a navigation mid-create must not swap either out from under the send. Fire and
      // return — submit() stays sync so the key/click handler is responsive while the node round-trips.
      // M-RP-INTRO Leg 2a — the intro is captured HERE for the same reason and validated HERE, at the seam:
      // `sendableIntro` is `null` unless the user actually typed one, so the ordinary DM send is unchanged.
      void createDraftDm(counterpart, draftText, sendableIntro);
      return;
    }
    const text = draft.trim();
    const spaceId = roomLatch.effectiveSpaceId;
    const roomId = roomLatch.effectiveRoomId;
    // Re-check at the moment of action, not only at render: the latch can move between the last paint and
    // this call, and a disabled button is a courtesy, never a guarantee (the M-RP7.6 handler-refusal shape).
    if (!text || spaceId == null || roomId == null) return;
    draft = '';
    // Deliberately NOT awaited: the row is appended synchronously by `echo.send`, so the sentence is on
    // screen before the network is consulted, and every later state change arrives through the store.
    void echo.send(spaceId, roomId, text);
  }

  // C-bis-4 (§5.2) — turn the draft into a real DM. `create_dm_space` is injected into `dmDraft` by the
  // shell (W-3: this widget imports no Tauri and cannot receive the transport as a prop — store-mediation is
  // the only channel, the echo precedent). The verb signs and sends the DM's three-event chain and returns
  // the new Space/Room ids, after which the SHIPPED machinery takes over: latch the real room, echo the sent
  // text into it, clear the draft (which drops the counterpart AND its text — it was sent). The order is
  // §5.2's: create → latch → send → clear.
  //
  // ⚠️ FAILURE (§5.4, D-065): `create` returns null → latch nothing, send nothing, clear nothing, so NOTHING
  // on screen implies the DM exists; the draft stays open with its text (`create` kept both) and the cause
  // sits in `dmDraft.error` — the single call site a visible failure surface reads once Joe rules on it.
  //
  // 🛑 DEVIATION FROM RUNBOOK §5.2, REPORTED NOT ABSORBED (Rule 6 / the J-516 precedent — ship the correct
  // path, flag it loudly): the runbook's sequence asserts "the latch becomes REAL, first time" but, measured
  // from source, `roomLatch.effectiveRoomId` resolves `_latched` against `spacesState.spaces`
  // (room-latch.svelte.ts:48-55), and `create_dm_space` writes the new DM only to the client-state FILE on
  // disk (ops.rs:1025-1041) — it never touches the in-memory `spacesState`. So a bare `latch(result.room_id)`
  // resolves to NULL and R5 shows "Select a room" instead of the new DM. The fix lives in the shell's
  // injected transport (which owns `loadSpaces`, the disk re-read); this widget's sequence is unchanged, and
  // by the time `create` resolves the store already carries the new DM. See app_client.svelte for the seam.
  //
  // 🔒 M-RP-INTRO Leg 2a — `intro` is CARRIED, NEVER DERIVED. It is already validated by the caller and is
  // `null` for an ordinary send, in which case `echo.send` writes no `intro` on the row and the Rust seam
  // attaches no key: byte-identical to today's output (`N-182`). `text` is passed exactly as before (1-bis).
  async function createDraftDm(
    counterpart: string,
    text: string,
    intro: MessageIntro | null,
  ): Promise<void> {
    const result = await dmDraft.create(counterpart);
    if (result == null) return; // create failed — draft stays open (dmDraft.create kept text + set `error`)
    roomLatch.latch(result.room_id);
    // 🛑 C-bis-6 — POST-CREATE ORIENTATION, and a REPORTED file-list DEVIATION (Rule 6, the C-bis-4 model).
    // The runbook §3 C-bis-6 lists ONLY members-panel + spaces-panel, but its step 1 requires the SPACE context
    // to move for BOTH branches — "the existing DM and the post-create path". The post-create `latch()` lives
    // HERE, not in members-panel, so this file is a forced third edit (exactly as C-bis-4 forced app_client).
    // `roomLatch.latch()` alone leaves `spaceLatch` on the Space you drafted from → after the create R2 draws
    // that Space's rooms while R5/R6 target the new DM's room, and R1 highlights the wrong Space (J-708 ①).
    // Moving the browsing latch to the new DM's Space makes R2 list the DM's room and R1 go unlit (Joe's rule).
    // The result already carries the new Space id — no lookup, no round trip. The store shortcut (roomLatch
    // moving spaceLatch) is forbidden by both latch headers, so the move is done at the caller, exactly as the
    // shell's one effect drives both `note()`s.
    spaceLatch.latch(result.space_id);
    // Not awaited — the echo row appends synchronously, so the sent message is on screen the instant the
    // draft clears; every later status change arrives through the echo store (the room-send precedent).
    // M-RP-INTRO Leg 2a — `at` is passed EXPLICITLY because `intro` is the trailing parameter. Its default
    // is `Date.now()` and this is the same instant, so lock #4's client-minted timestamp is unchanged; the
    // explicit argument is positional bookkeeping, not a new decision about time.
    void echo.send(result.space_id, result.room_id, text, Date.now(), intro);
    dmDraft.clear();
  }

  // Enter sends, Shift+Enter is a newline — the chat convention. `isComposing` is honoured so an IME
  // candidate-confirming Enter never sends a half-typed word (it fires with keyCode 229 mid-composition).
  // ⚠️ THE GUARD COMES BEFORE `preventDefault`, AND THE ORDER IS THE WHOLE POINT. Swallowing Enter while
  // sending is disabled would make the key do NOTHING — no send AND no newline — so a user with no room
  // selected would find their Enter key silently dead. That is blocking typing to enforce a send rule,
  // which is exactly what D6 forbids. When we will not send, Enter stays a plain newline.
  function onKeydown(e: KeyboardEvent): void {
    if (e.key !== 'Enter' || e.shiftKey || e.isComposing) return;
    if (!sendEnabled) return;
    e.preventDefault();
    submit();
  }

  const debug = () => ({
    canSend,
    hasText,
    sendEnabled,
    draftActive: dmDraft.active,
    draftCounterpart: dmDraft.counterpart ?? null,
    // `textLength` = the buffer CURRENTLY shown (draft text while active, else the room buffer).
    // `roomBufferLength` = the local room buffer regardless. The two are reported separately so the verify
    // pass can prove a draft's text never bleeds into the room buffer, nor the reverse (§5.3, the leak).
    textLength: composerText().length,
    roomBufferLength: draft.length,
    // C-bis-4 — the create failure surface (§5.4), read here so the verify pass can assert "failure ⇒ draft
    // open, text kept, error set" without reaching into __XGEN_DRAFT__ separately.
    draftError: dmDraft.error,
    // M-RP-INTRO Leg 2a — the authoring surface, exposed so the verify pass can assert the OPT-IN property
    // directly: `introSendable` is what will actually ride the wire, validated by the same rule the receiver
    // uses. `false` with the disclosure open and the fields blank is the load-bearing reading — that is the
    // state which must still produce a send byte-identical to today's.
    introOpen,
    introSendable: sendableIntro !== null,
    introHeadlineLength: introHeadline().length,
    introBlurbLength: introBlurb().length,
    roomId: roomLatch.effectiveRoomId,
    spaceId: roomLatch.effectiveSpaceId,
    wired: echo.wired,
    echoCount: echo.forRoom(roomLatch.effectiveRoomId).length,
  });
</script>

<!-- Widget root (the stream-panel / rooms-panel precedent: `data-tier="widget"` + envelope). The textarea is
  never disabled — only the button is (lock #12). -->
<div class="composer-panel" data-tier="widget" use:envelope={{ name: 'composer-panel', id, debug }}>
  <!-- The processor attachment (kind 1, D-099) rides `{...rest}` onto the inner <textarea>. NO options
    object: `trusted` already defaults to false (processor.ts) and the store has already validated the set
    as Tier-2, so an explicit `trusted: false` would change nothing and advertise a knob that must never be
    turned. The second validation inside processor() is deliberate and stays — it is what makes the
    no-crash claim true independently of the store. The attachment is keyed by a unique symbol, so it
    cannot collide with `onkeydown`, which rides the same `rest` under a string key. -->
  <Textarea
    {...processor(substitutions.rules)}
    bind:value={composerText, setComposerText}
    placeholder={canSend ? PLACEHOLDER : PLACEHOLDER_NO_ROOM}
    rows={2}
    id={cid('input')}
    onkeydown={onKeydown}
  />
  <!-- M-RP-INTRO Leg 2a — THE INTRO DISCLOSURE. Rendered ONLY while a DM draft is active: the intro is a
    first-contact artefact, so the affordance lives on the surface that opens a first contact, and the
    ordinary room composer is untouched. Collapsed by default (`introOpen = false`) — opening it is the
    opt-in, and leaving it empty still sends no key (`N-182`). The native <label> wrapping a core Toggle is
    the shipped `grid-plate-settings` shape. Appearance is PROVISIONAL → M-RP-SKIN; the structure is here so
    Joe has something to look at (D-138). -->
  {#if dmDraft.active}
    <div class="composer-intro">
      <label class="composer-intro-toggle">
        <Toggle bind:checked={introOpen} id={cid('intro-toggle')} />
        <span>{INTRO_DISCLOSURE}</span>
      </label>
      {#if introOpen}
        <!-- ⚠️ NO `maxlength` HERE, AND IT IS NOT AN OVERSIGHT: `textfield` does not forward `{...rest}`
          (the processor comment above says so), so bounding at the input needs a `core` change. The bound
          therefore lives at the RENDER (`message-intro.svelte`), which covers BOTH directions with one rule
          — our own typing and a stranger's payload — rather than only ours. -->
        <Textfield
          bind:value={introHeadline, setIntroHeadline}
          placeholder={INTRO_HEADLINE_PLACEHOLDER}
          id={cid('intro-headline')}
        />
        <Textarea
          bind:value={introBlurb, setIntroBlurb}
          placeholder={INTRO_BLURB_PLACEHOLDER}
          rows={2}
          id={cid('intro-blurb')}
        />
      {/if}
    </div>
  {/if}
  <div class="composer-actions">
    <Button label="Send" disabled={!sendEnabled} onclick={submit} id={cid('send')} />
  </div>
  <!-- C-bis-8 (§5.4) — THE CREATE-FAILURE SURFACE. One line under the actions rendering `dmDraft.error`
    VERBATIM: the node's own words at the throw path (`String(e)`, dm-draft.svelte.ts:142 — e.g. "failed to
    connect to Node … (os error 10061)"). NO label, NO prefix, NO wrapping sentence — a label would be
    authored copy, and copy is Joe's. `#if` so the line exists ONLY while an error is set; `create()` clears
    `_error` at its top (dm-draft.svelte.ts:134), so a successful retry SELF-CLEARS the line with no extra
    wiring. NO `use:envelope` — this registers nothing, so the client registry floor is unmoved. Appearance
    (`.composer-error`) is `skin.css`'s and currently unstyled — this leg ships structure only (M-RP-SKIN). -->
  {#if dmDraft.error}
    <div class="composer-error">{dmDraft.error}</div>
  {/if}
</div>

<style>
  /* Structural only (the fill chain; appearance is skin's, N-090/N-025 — and this block is PROVISIONAL,
     discharged at M-RP-SKIN). Like stream-panel, R6 must FILL its tile rather than sit as a short card, so
     it carries a minimal structural block (N-094: the question is "could a skinner want to retune this?" —
     a fill contract is not a look). `min-height: 0` rides every level so a long draft scrolls INSIDE the
     leaf and never migrates the scrollbar to the document (the J-499 D5 failure mode). */
  .composer-panel {
    display: flex;
    flex-direction: column;
    gap: 4px;
    height: 100%;
    min-height: 0;
    padding: 4px;
  }
  .composer-actions {
    display: flex;
    justify-content: flex-end;
    flex: 0 0 auto;
  }
  /* M-RP-INTRO Leg 2a — STRUCTURAL ONLY, and PROVISIONAL (discharged at M-RP-SKIN, like the block above).
     The disclosure is a fixed-size stack above the actions: `flex: 0 0 auto` so it never steals the
     textarea's growth, `min-width: 0` so a long field cannot push the composer wider than R6. NO colour, NO
     font-size, NO weight, NO spacing opinion beyond the gap that keeps the two fields from touching —
     `.composer-intro*` is Joe's to skin. */
  .composer-intro {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 0 0 auto;
    min-width: 0;
  }
  .composer-intro-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  /* C-bis-8 (§5.4) — STRUCTURAL ONLY (the block above is PROVISIONAL, discharged at M-RP-SKIN). The failure
     line must stay inside the tile: `min-width: 0` lets the flex item shrink below its content, and
     `overflow-wrap: anywhere` breaks a long unbreakable token (a URL, an os-error tail) so it wraps rather
     than pushing the composer wider than R6 or migrating a horizontal scrollbar. NO colour, NO font-size, NO
     weight — `.composer-error` is Joe's to skin (skin.css). */
  .composer-error {
    flex: 0 0 auto;
    min-width: 0;
    overflow-wrap: anywhere;
  }
</style>
