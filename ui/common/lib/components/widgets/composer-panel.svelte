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
  // 🔒 OFFLINE STILL SENDS, AND ON PURPOSE. The shell's lifecycle guard turns a send during an outage into
  // an immediate `failed` — which lock #7 lets the user RETRY FREELY, because `failed` means it never
  // reached the wire. So the sentence is PRESERVED, visible and retryable, instead of sitting in a textarea
  // the user may close. §3.1's own table names `failed` "the common outage path", which only happens if the
  // send is allowed through. Disabling the button offline would look tidier and would lose the words.
  import { envelope } from '$common/components/base/envelope';
  import Textarea from '$core/components/data-independent/textarea.svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import { echo } from '$common/stores/echo-state.svelte';

  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  let draft = $state('');

  // Lock #12's ONE predicate, read from the shared latch so R5 and R6 can never disagree about which room
  // is active (a second copy of this rule is the D-067 drift the lift exists to prevent).
  const canSend = $derived(roomLatch.canSend);
  const hasText = $derived(draft.trim().length > 0);
  const sendEnabled = $derived(canSend && hasText);

  // Functional copy, PROVISIONAL (appearance and final phrasing -> M-RP-SKIN; the stream-panel precedent).
  // The two placeholders say DIFFERENT things because the truths are different (N-091): no room selected is
  // not the same as an empty box in a real room.
  const PLACEHOLDER = 'Write a message…';
  const PLACEHOLDER_NO_ROOM = 'Select a room to send a message.';

  function submit(): void {
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
    draftLength: draft.length,
    roomId: roomLatch.effectiveRoomId,
    spaceId: roomLatch.effectiveSpaceId,
    wired: echo.wired,
    echoCount: echo.forRoom(roomLatch.effectiveRoomId).length,
  });
</script>

<!-- Widget root (the stream-panel / rooms-panel precedent: `data-tier="widget"` + envelope). The textarea is
  never disabled — only the button is (lock #12). -->
<div class="composer-panel" data-tier="widget" use:envelope={{ name: 'composer-panel', id, debug }}>
  <Textarea
    bind:value={draft}
    placeholder={canSend ? PLACEHOLDER : PLACEHOLDER_NO_ROOM}
    rows={2}
    id={cid('input')}
    onkeydown={onKeydown}
  />
  <div class="composer-actions">
    <Button label="Send" disabled={!sendEnabled} onclick={submit} id={cid('send')} />
  </div>
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
</style>
