<script>
  // app_sampler.svelte — SAMPLER matrix. Plain-JS app shell (bare $state, no TS
  // annotations — N-041). Mounts all built `core` components live in a
  // semantic-group x state grid; each cell is a real `envelope`-registered instance
  // (`{type}#{state}`) so CDP `ids()` enumerates the matrix. 16 built `core` components.
  //
  // M-RP3.2: the matrix is now tabbed by class x arity (di/dd x atomic/composite).
  // ALL panels stay MOUNTED; inactive panels are hidden via CSS display:none
  // (class:hidden), NEVER {#if} — `envelope` registers only while mounted, so
  // unmounting a panel would drop its ids from window.__XGEN_DEBUG__ and break the
  // CDP matrix-count invariant (D-097). DI-atomic holds the current 44 cells with
  // INTERACTIVE / DISPLAY / NAVIGATION sub-headers; the other three panels are empty
  // placeholders (di-composite's first occupant is status-indicator, M-RP2.22).
  //
  // State-map is RAGGED on purpose (honest, not a forced uniform grid):
  //   default  — all
  //   disabled — interactive only (display/navigation di have none); NOTE `toggle` has
  //              no `disabled` prop (atomic gap, N-045) -> shown as `toggle#switch` instead
  //   invalid  — only textfield (bad email) + number (out-of-range) + date; no faked columns
  //   variants — toggle checked/switch, button toggle-mode, textfield password, textarea \n
  // No focus column: focus is transient; a static focus cell would be a lie (verify live).
  import { onMount } from 'svelte';
  import Toggle from '$core/components/data-independent/toggle.svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import TextField from '$core/components/data-independent/textfield.svelte';
  import Select from '$core/components/data-independent/select.svelte';
  import TextArea from '$core/components/data-independent/textarea.svelte';
  import NumberField from '$core/components/data-independent/number.svelte';
  import Range from '$core/components/data-independent/range.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import Paragraph from '$core/components/data-independent/paragraph.svelte';
  import Img from '$core/components/data-independent/image.svelte';
  import DateField from '$core/components/data-independent/date.svelte'; // not `Date` (global)
  import ColorField from '$core/components/data-independent/color.svelte';
  import FileField from '$core/components/data-independent/file.svelte';
  import SelectMultiple from '$core/components/data-independent/select-multiple.svelte';
  import Led from '$core/components/data-independent/led.svelte';
  import Icon from '$core/components/data-independent/icon.svelte'; // 28th core, first SVG-rooted (M-RP6.1a)
  import Separator from '$core/components/data-independent/separator.svelte'; // 29th core, first value-less (M-RP6.1b)
  import Link from '$core/components/data-independent/link.svelte';
  import StatusIndicator from '$core/components/data-independent/status-indicator.svelte';
  import StatusBar from '$core/components/data-independent/status-bar.svelte'; // status-bar di composite (M-RP6.1e-A)
  import Shelf from '$core/components/data-independent/shelf.svelte'; // ordered command strip (M-RP6.1i)
  import PasswordField from '$core/components/data-independent/password-field.svelte';
  import StarRating from '$core/components/data-independent/star-rating.svelte';
  import FileFieldComposite from '$core/components/data-independent/file-field.svelte';
  import Combobox from '$core/components/data-independent/combobox.svelte';
  import Chip from '$core/components/data-independent/chip.svelte';
  import TagSelect from '$core/components/data-independent/tag-select.svelte';
  import ColorPicker from '$core/components/data-independent/color-picker.svelte';
  import ConverterField from '$core/components/data-independent/converter-field.svelte';
  import Meter from '$core/components/data-independent/meter.svelte';
  import Section from '$core/components/data-independent/section.svelte';
  import Dialog from '$core/components/data-independent/dialog.svelte'; // 31st core, di composite modal (M-RP6.1e-C1)
  import EntityAvatar from '$core/components/data-dependent/entity-avatar.svelte'; // first dd-atomic (M-RP5.0)
  import EntityItem from '$core/components/data-dependent/entity-item.svelte'; // first dd-composite (M-RP5.1)
  import Status from '$core/components/data-dependent/status.svelte'; // self-status dd-atomic (M-RP5.1a)
  import EntityPanel from '$core/components/data-dependent/entity-panel.svelte'; // last dd-composite (M-RP5.2)
  import Message from '$core/components/data-dependent/message.svelte'; // message dd sub-family opener (M-RP5.5 A)
  import MessageStream from '$core/components/data-dependent/message-stream.svelte'; // message-stream shell (M-RP5.6 A)
  import FixtureWidget from './fixture-widget.svelte'; // sampler-local stub for the message details socket
  import FixtureRegWidget from './fixture-reg-widget.svelte'; // its REGISTERING sibling (M-RP6.9) — the instrument for the N×M question

  // Processor (common infra, M-RP4.0/M-RP4.2): the kind-1 transformer attachment, fed from the
  // source-agnostic substitutions store. The atomic forwards {...rest}; processor(...) returns a
  // symbol-keyed attachment that lands on the inner <textarea>. Adds NO registry entry.
  // The store is hydrated on mount from the sampler host's get_substitutions Tauri command
  // (xgen-sampler_config.toml [substitutions], generated from the seed on start — the REAL load
  // path, mirroring app_client.svelte / J-437; M-RP4.4 / D-101). No literal seed.
  import { processor } from '$common/components/processor/processor';
  import { clamp } from '$common/components/processor/clamp';
  import { intlNumber } from '$common/components/processor/transform';
  import { substitutions } from '$common/components/processor/store.svelte';
  import SubstitutionsEditor from '$common/components/widgets/substitutions-editor.svelte';
  import EntityContextMenu from '$common/components/widgets/entity-context-menu.svelte'; // 2nd widget, first W-11 dd-socket consumer (M-RP5.3)

  // Runtime client<->node skin-swap (D-098): flipping [data-shell] re-aliases --accent*
  // live, so the whole grid re-themes at once. Replaces "run in both real shells".
  let shell = $state('client');
  function applyShell(s) { shell = s; document.documentElement.dataset.shell = s; }

  // Hydrate the user-owned substitution pairs from the sampler host's REAL load path (M-RP4.4):
  // xgen-sampler_config.toml [substitutions] -> get_substitutions -> store. Mirrors
  // app_client.svelte (J-437). Async, so the processor-host cell starts with empty rules and
  // re-attaches when setRules lands (source-agnostic store + attachment lifecycle, D-099).
  onMount(async () => {
    applyShell('client');
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      substitutions.setRules(await invoke('get_substitutions'));
    } catch (_) {
      // Outside Tauri (plain-browser Vite preview) — store stays empty; the morph's canonical
      // home is the Tauri window (D-098), where CDP verifies it.
    }
  });

  // Tab container (M-RP3.2): outer class x arity axis. Inactive panels are CSS-hidden,
  // NOT unmounted (class:hidden), so every component stays registered for CDP `ids()`.
  let activeTab = $state('di-atomic'); // 'di-atomic' | 'di-composite' | 'dd-atomic' | 'dd-composite'
  const tabs = [
    { id: 'di-atomic',    label: 'DI Atomics' },
    { id: 'di-composite', label: 'DI Composites' },
    { id: 'dd-atomic',    label: 'DD Atomics' },
    { id: 'dd-composite', label: 'DD Composites' },
    { id: 'widget',       label: 'Widgets' },
  ];

  // Bound values (bare $state — plain-JS shell). One per interactive cell.
  let tglDefault = $state(false);
  let tglChecked = $state(true);
  let tglSwitch = $state(false);
  let btnPressed = $state(true);
  let tfDefault = $state('hello');
  let tfDisabled = $state('disabled text');
  let tfInvalid = $state('not-an-email');
  let tfPassword = $state('secret');
  let selDefault = $state('two');
  let selDisabled = $state('one');
  let taDefault = $state('line one\nline two');
  let taDisabled = $state('inert');
  let taProcessed = $state(''); // M-RP4.0/4.2: processor-host cell; store-sourced user rules morph on input
  let numDefault = $state(42);
  let numDisabled = $state(7);
  let numInvalid = $state(50); // outside [0,10] -> :invalid (rangeOverflow)
  let numClamped = $state(5); // clamp-host (M-RP4.1): kind-3 guard coerces to [0,10] on change
  // converter-field (M-RP4.5): kind-2 bridge. Typed value = number; display = Intl-formatted string.
  // Stable converter identity (defined once, NOT inline) so the prop doesn't churn per render.
  const numConv = intlNumber({ maximumFractionDigits: 2 });
  let cvDefault = $state(1234.5); // shows "1,234.5"; edit raw, blur reformats
  let cvDisabled = $state(99.9);
  let rngDefault = $state(50);
  let rngDisabled = $state(30);
  // date — string bind:value for every type (empty would be ''); seeded valid here.
  let dtDefault = $state('2026-06-28');
  let dtTime = $state('13:45');
  let dtDatetime = $state('2026-06-28T13:45');
  let dtMonth = $state('2026-06');
  let dtWeek = $state('2026-W26');
  let dtDisabled = $state('2026-06-28');
  let dtInvalid = $state('2030-01-01'); // outside [2026-01-01, 2026-12-31] -> :invalid (rangeOverflow)
  // color — always a string hex #rrggbb (never empty); seeded with the per-shell accents.
  let colDefault = $state('#9a6a30');
  let colDisabled = $state('#2a6090');
  // file — FileList | null; unsettable from markup (browser security), so all start null.
  let fDefault = $state(null);
  let fMultiple = $state(null);
  let fDisabled = $state(null);
  // select-multiple — string[]; empty = [] (NOT null). Arrays CAN seed from markup (unlike file).
  let smDefault = $state([]);
  let smSeeded = $state(['a', 'c']);
  let smDisabled = $state(['b']);
  // password-field (M-RP2.23) — string bind:value; the child textfield is redactValue'd.
  let pwDefault = $state('hunter2');
  let pwDisabled = $state('inert');
  let pwRevealed = $state('shown');
  // star-rating (M-RP2.24) — numeric bind:value; 0 = unrated. readonly cell shows a fixed rating.
  let srDefault = $state(0);
  let srRated = $state(3);
  let srReadonly = $state(4);
  // file-field (M-RP2.25) — FileList|null; unsettable from markup (browser security), all start null.
  let ffDefault = $state(null);
  let ffMultiple = $state(null);
  let ffDisabled = $state(null);
  // combobox (M-RP2.26) — rich owned-popup; string bind:value = shown label. preset seeded.
  let cbDefault = $state('');
  let cbPreset = $state('Online');
  let cbDisabled = $state('');
  // chip (M-RP2.27) — display token, no bind. onRemove spy for CDP.
  let chipRemoved = $state(0);
  // status-bar (M-RP6.1e-A) — the resize-grip seam is inert in the sampler (no Tauri); a
  // counter spy makes the onResizeGrip callback CDP-observable on pointerdown (the real
  // startResizeDragging wiring is 6.1e-B, real client).
  let sbGripFired = $state(0);
  // shelf (M-RP6.1i) — the ordered command strip. onCommand writes the dispatched commandId into a
  // DOM probe ([data-probe]) so the CDP can read it back (DoD leg 4: click/Enter → the commandId).
  // The three faces carry the REAL frame commands they will serve in the client (6.1k/6.1l): gear →
  // widget.manager, diskette → layout.save, load → layout.load. Enabled here (a test-bed);
  // disabled-in-the-client is 6.1j's separate, honest story.
  let shelfLastCommand = $state('');
  const shelfBottomItems = [
    { icon: 'gear', label: 'Widget manager', command: 'widget.manager' },
    { icon: 'diskette', label: 'Save layout', command: 'layout.save' },
    { icon: 'load', label: 'Load layout', command: 'layout.load' },
  ];
  // one disabled face among enabled — the disabled branch exercised HERE, so 6.1j inherits a
  // proven state (activation inert; still keyboard-reachable).
  const shelfMixedItems = [
    { icon: 'gear', label: 'Widget manager', command: 'widget.manager' },
    { icon: 'diskette', label: 'Save layout', command: 'layout.save', disabled: true },
    { icon: 'load', label: 'Load layout', command: 'layout.load' },
  ];
  // tag-select (M-RP2.28) — string[] bind:value. default seeds 4 (proves +N overflow at
  // COLLAPSE_AT=3); max caps at 2 (3rd pick no-ops + data-full dim); create allows freeform.
  let tsDefault = $state(['important', 'work', 'personal', 'urgent']);
  let tsMax = $state(['important', 'work']);
  let tsCreate = $state([]);
  let tsManaged = $state(0); // gear spy (onManage) for CDP
  // color-picker (M-RP2.29) — string bind:value #rrggbbaa; seeded with the per-shell accents (padded ff).
  let cpDefault = $state('#9a6a30ff');
  let cpDisabled = $state('#2a6090ff');
  // dialog (M-RP6.1e-C1): closed on mount; the s-note triggers below flip these to open.
  let dlgDefault = $state(false);
  let dlgLabelled = $state(false);
  let dlgClosed = $state(0); // onClose spy for CDP (fires on button + Esc)
  const tagOptions = ['important', 'work', 'personal', 'urgent', 'later'];
  const cbOptions = [
    { value: 'online', label: 'Online', status: 'active' },
    { value: 'away', label: 'Away', status: 'idle' },
    { value: 'busy', label: 'Busy', status: 'DND' },
    { value: 'offline', label: 'Offline', status: 'off', disabled: true },
  ];

  const selOptions = [
    { value: 'one', label: 'One' },
    { value: 'two', label: 'Two' },
    { value: 'three', label: 'Three' },
  ];

  // entity-avatar (M-RP5.0, first dd-atomic) — source-agnostic EntityDescriptor view-models
  // (the shell owns the protocol -> descriptor map; core imports no IdentityRecord/SpaceState).
  // kind identity + DM space -> circle; non-DM space -> rounded-square. name absent -> xgid-tail
  // fallback initials. flags drive the isAi spark badge + revoked greyed/slash overlay.
  const eaIdentity = { kind: 'identity', name: 'Alice Ng', id: 'xgen://identity/alice-7f3a', flags: {} };
  const eaSpace = { kind: 'space', name: 'Dev Team', id: 'xgen://space/dev-2b11', flags: {} };
  const eaDm = { kind: 'space', name: 'Bob Lee', id: 'xgen://space/dm-bob-9c04', flags: { isDm: true } };
  const eaAnon = { kind: 'identity', id: 'xgen://identity/anon-55az', flags: {} }; // no name -> "AZ"
  const eaRevoked = { kind: 'identity', name: 'Mallory', id: 'xgen://identity/mal-1', flags: { revoked: true } };
  const eaAi = { kind: 'identity', name: 'Aria Bot', id: 'xgen://identity/aria-ai', flags: { isAi: true } };
  // room (M-RP5.0c) — a location peer to space; shape branches to hexagon (clip-path).
  const eaRoom = { kind: 'room', name: 'general', id: 'xgen://room/general-1a', flags: {} };
  let eaActivated = $state(0); // onActivate spy (reserved menu seam) for CDP

  // entity-context-menu (M-RP5.3, 2nd widget) — the gesture-agnostic overlay. The sampler wires a
  // trigger button to ecmRef.open(anchorEl); onSelect is the stubbed consumer dispatch (pure layer).
  const ecmEntity = { kind: 'identity', name: 'Alice Ng', id: 'xgen://identity/alice-7f3a', flags: {} };
  const ecmStatus = { emoji: '🎧', text: 'in a meeting', updatedAt: new Date(Date.now() - 5 * 60000).toISOString() };
  let ecmRef; // component instance (bind:this) exposing open()/close()
  let ecmSelected = $state('—'); // last dispatched item id (dispatch spy for CDP)
  const ecmSelect = (itemId) => { ecmSelected = itemId; };

  // entity-item (M-RP5.1, first dd-composite) — composes the real entity-avatar (single-knob
  // derive: row->list, card->card, nav->labeled, inline->presence). secondary/status/meta are
  // caller slots (source-agnostic; the shell would map protocol + Track A state.status here).
  const eiStatus = { emoji: '🟢', text: 'online' }; // Track A self-status slot (card only)
  let eiActivated = $state(0); // onActivate spy for CDP (row activation)

  // status (M-RP5.1a, self-status dd-atomic) — source-agnostic view-model {emoji?,text?,updatedAt?,
  // expiresAt?} (shell maps Track A StatusRecord). Timestamps are computed relative to render so
  // `full`'s "updated Nm ago" and the expired case are deterministic (browser Date.now(), fine in
  // the live sampler). expired → mounted-but-empty (data-empty), still registers.
  const nowMs = Date.now();
  const isoAt = (deltaMs) => new Date(nowMs + deltaMs).toISOString();
  const stEmoji = { emoji: '🎯' };                                       // emoji-only
  const stEmojiText = { emoji: '🌙', text: 'sleeping' };                 // emoji + text
  const stTextOnly = { text: 'on vacation' };                            // no emoji (tooltip/line)
  const stFull = { emoji: '🎧', text: 'in a meeting', updatedAt: isoAt(-5 * 60 * 1000) };
  const stExpired = { emoji: '💤', text: 'away', expiresAt: isoAt(-60 * 1000) }; // expired → empty
  const stAvatar = { emoji: '🟢', text: 'online', updatedAt: isoAt(-2 * 60 * 1000) }; // avatar corner

  // entity-panel (M-RP5.2, last dd-composite) — EntityItemInput[] view-models (descriptor + caller
  // secondary/status/meta). Rows are variant="row"; per the "status once per variant" rule (M-RP5.2
  // entity-item amendment) a row forwards status to the avatar CORNER badge (card would show the
  // inline line instead). Spaces = square avatars; DMs = circle; Aria (isAi + status) proves the
  // corner-fix H no-overlap (isAi spark top-right, status badge bottom-right on one glyph).
  const epSpaces = [
    { descriptor: { kind: 'space', name: 'Dev Team', id: 'xgen://space/dev-2b11', flags: {} }, meta: '3' },
    { descriptor: { kind: 'space', name: 'Design', id: 'xgen://space/design-7a2', flags: {} }, meta: '12' },
    { descriptor: { kind: 'space', name: 'Ops', id: 'xgen://space/ops-9f1', flags: {} } },
  ];
  const epDms = [
    { descriptor: { kind: 'space', name: 'Bob Lee', id: 'xgen://space/dm-bob-9c04', flags: { isDm: true } }, status: { emoji: '🟢', text: 'online' }, meta: '1' },
    { descriptor: { kind: 'identity', name: 'Alice Ng', id: 'xgen://identity/alice-7f3a', flags: {} }, meta: '5' },
    { descriptor: { kind: 'identity', name: 'Aria Bot', id: 'xgen://identity/aria-ai', flags: { isAi: true } }, status: { emoji: '🤖', text: 'thinking' } },
  ];
  const epArchived = [
    { descriptor: { kind: 'space', name: 'Archive A', id: 'xgen://space/arch-a', flags: {} } },
    { descriptor: { kind: 'space', name: 'Archive B', id: 'xgen://space/arch-b', flags: {} } },
  ];
  let epSelSpaces = $state('xgen://space/design-7a2'); // pre-selected 2nd row (bind:selected proof)
  let epSelDms = $state();
  let epCollapsed = $state(true);
  let epActivated = $state(0); // onActivate spy for CDP
  // room-kind panel (M-RP5.0c) — proves item/panel ripple hexagon avatars with no item/panel code.
  const epRooms = [
    { descriptor: { kind: 'room', name: 'general', id: 'xgen://room/general-1a', flags: {} }, meta: '9' },
    { descriptor: { kind: 'room', name: 'dev', id: 'xgen://room/dev-2b', flags: {} } },
  ];
  let epSelRooms = $state();

  // unresolved (M-RP-IDENTITY-RESOLUTION §4/§5a) — the two states a member row can be in when the
  // client holds NO identity record. ⚠️ The names are xgid TAILS on purpose: with no book record
  // `toDescriptor` already falls back to `tail(identity_id)`, so a real ③/④ row has NO display
  // name (Phase-0 §5b). A fixture with a human name would be tuning a case the product cannot
  // produce. The first row is RESOLVED and is the control — a mark is only readable against one.
  // ⚠️ TAIL LENGTH IS LOAD-BEARING: `tail()` returns the whole final segment (`ed25519:` + a
  // 44-char key) and `.ei-name` is LEFT-anchored, so the real render KEEPS the constant head and
  // ellipsises the distinguishing bytes (`M_RP_MEMBERS.md` §6a). A short fixture tail would not
  // clip, and would show Joe a more legible string than the product can produce.
  const epUnresolved = [
    { descriptor: { kind: 'identity', name: 'Bob Lee', id: 'xgen://identity/bob-9c04', flags: {} } },
    { descriptor: { kind: 'identity', name: 'ed25519:9xK2vN8pLdA3QmR47bTfHs1YnE6cZkW5jU0gMpQaXrBc', id: 'xgen://identity/unasked-1', flags: {} }, unresolved: 'unasked' },
    { descriptor: { kind: 'identity', name: 'ed25519:Zk9WbT5cH1sYnE6f7QmR4xK2vN8pLdA3jU0gMpQaXrBd', id: 'xgen://identity/erased-1', flags: {} }, unresolved: 'erased' },
  ];
  // §5a-i — the erased row is the DM counterpart and is pre-SELECTED, because the lock (the L16
  // highlight SURVIVES the mark) is unobservable unless something renders the two together.
  // 🛑 INERT + ONE-WAY, MIRRORING `members-panel.svelte:164` EXACTLY (`interactive={false}`,
  // `selected=` not `bind:selected=`). Interactive with a binding, a click would write the
  // selection away and SILENTLY DESTROY the very demonstration this cell exists for.

  // message (M-RP5.5 A, message dd sub-family opener) — MessageDescriptor fixtures. `author` REUSES
  // the entity dd-socket (EntityDescriptor); the shell would map protocol → descriptor here. The
  // `details` widget socket resolves widgetId → component via a sampler-supplied registry; an
  // unknown id is DROPPED on render (W-13). `text-own` sets isOwn → the row mirrors to the right.
  const eaSelf = { kind: 'identity', name: 'You', id: 'xgen://identity/self-0', flags: {} };
  const msgWidgets = { 'fixture.time': FixtureWidget, 'fixture.badge': FixtureWidget, 'fixture.reg': FixtureRegWidget };
  const msgOther = { kind: 'text', id: 'm-1', author: eaIdentity, body: 'Hey — did the fan-out converge on your node? 🤔', timestamp: '2026-07-07T12:03:00Z' };
  const msgOwn = { kind: 'text', id: 'm-2', author: eaSelf, body: 'Yep — 19s, clean 🚀 Pushing the tag now.', timestamp: '2026-07-07T12:04:00Z', isOwn: true };
  const msgDetails = { kind: 'text', id: 'm-3', author: eaIdentity, body: 'Two details mounts render inline in the header.', timestamp: '2026-07-07T12:05:00Z', details: [{ widgetId: 'fixture.time', props: { label: '12:05' } }, { widgetId: 'fixture.badge', props: { label: 'pinned', tone: 'muted' } }] };
  const msgUnknownWidget = { kind: 'text', id: 'm-4', author: eaIdentity, body: 'One known mount + one unknown id (dropped).', timestamp: '2026-07-07T12:06:00Z', details: [{ widgetId: 'fixture.time', props: { label: '12:06' } }, { widgetId: 'does.not.exist', props: { label: 'GHOST' } }] };

  // M-RP5.5 B — text render-states. `grouped` is a PROP (stream-computed, M-RP5.6), not a descriptor
  // field; `edited` / `deleted` are descriptor fields. `text-grouped-edited` proves the combination
  // (header suppressed by grouped + "(edited)" shown because not deleted).
  const msgGrouped = { kind: 'text', id: 'm-5', author: eaIdentity, body: 'A continuation line — same author, header suppressed but avatar column reserved. 👍🏽', timestamp: '2026-07-07T12:07:00Z' };
  const msgEdited = { kind: 'text', id: 'm-6', author: eaIdentity, body: 'This line was tweaked after it was sent ✏️', timestamp: '2026-07-07T12:08:00Z', edited: true };
  const msgDeleted = { kind: 'text', id: 'm-7', author: eaIdentity, body: 'Original text — must NOT render (tombstone).', timestamp: '2026-07-07T12:09:00Z', deleted: true, details: [{ widgetId: 'fixture.time', props: { label: '12:09' } }] };
  const msgGroupedEdited = { kind: 'text', id: 'm-8', author: eaIdentity, body: 'Grouped + edited: header gone, (edited) still shown.', timestamp: '2026-07-07T12:10:00Z', edited: true };

  // M-RP6.9 — the `bodyExtras` socket. Below the body, OUTSIDE the header guard, so it survives a
  // grouped continuation (`text-extras-grouped` is the cell that proves it, and the reason the
  // socket was chosen over perforating grouping). Each cell mounts THREE stubs: two non-registering
  // (`fixture.time` / `fixture.badge` → FixtureWidget, the control) and one REGISTERING
  // (`fixture.reg` → FixtureRegWidget), so the N×M registration question is measured on the
  // registry rather than read off the source. Every mount is a text stub — zero assets, zero URLs.
  //
  // `mountKey` (D-2) is supplied on `text-body-extras` only, so one cell exercises the stable-key
  // path and the others keep the legacy index fallback. Both must work; that is the point of the
  // fallback being byte-identical to the old key.
  const xTime = { widgetId: 'fixture.time', props: { label: '👍 3' } };
  const xBadge = { widgetId: 'fixture.badge', props: { label: '🎉 1', tone: 'muted' } };
  const xReg = { widgetId: 'fixture.reg', props: { label: '🚀 2' } };
  const msgBodyExtras = { kind: 'text', id: 'm-11', author: eaIdentity, body: 'Three mounts sit below this body — the strip is the socket 📎', timestamp: '2026-07-19T12:00:00Z', bodyExtras: [{ ...xTime, mountKey: 'k-time' }, { ...xBadge, mountKey: 'k-badge' }, { ...xReg, mountKey: 'k-reg' }] };
  const msgExtrasGrouped = { kind: 'text', id: 'm-12', author: eaIdentity, body: 'A continuation row: no header, no avatar — and the strip is still here 🎯', timestamp: '2026-07-19T12:01:00Z', bodyExtras: [xTime, xBadge, xReg] };
  const msgExtrasUnknown = { kind: 'text', id: 'm-13', author: eaIdentity, body: 'Two known mounts + one unknown id (dropped, W-13).', timestamp: '2026-07-19T12:02:00Z', bodyExtras: [xTime, { widgetId: 'does.not.exist', props: { label: 'GHOST' } }, xReg] };
  const msgExtrasDeleted = { kind: 'text', id: 'm-14', author: eaIdentity, body: 'Original text — must NOT render (tombstone).', timestamp: '2026-07-19T12:03:00Z', deleted: true, bodyExtras: [xTime, xReg] };

  // M-RP5.5 C — system kind: authorless centered notice (no avatar / header / name / details).
  // `system-long` wraps to ≥2 lines to prove the centered wrap stays symmetric.
  const msgSystemNotice = { kind: 'system', id: 'm-9', body: 'alice joined the room', timestamp: '2026-07-07T12:11:00Z' };
  const msgSystemLong = { kind: 'system', id: 'm-10', body: 'bob renamed the room from “general” to “protocol-dev” — everyone in the room was notified of the change.', timestamp: '2026-07-07T12:12:00Z' };

  // message-stream (M-RP5.6 A, shell) — ordered MessageDescriptor arrays. Timestamps are computed
  // RELATIVE to load time so the day-divider "Today"/"Yesterday" bands are stable regardless of when
  // CDP runs (the divider label is a fn of `new Date()`). The sampler is dev tooling → Date.now() ok.
  // (`nowMs` reuses the existing declaration above.)
  const minMs = 60 * 1000;
  const dayMs = 86_400_000;
  const iso = (ms) => new Date(ms).toISOString();
  const atNoon = (ms) => { const d = new Date(ms); d.setHours(12, 0, 0, 0); return d.getTime(); };

  // stream-basic — same local day, two authors + a system notice. Proves: consecutive same-author
  // collapse (sb-2 grouped) + different author breaks (sb-3) + a system message breaks a run (sb-5).
  const streamBasic = [
    { kind: 'text', id: 'sb-1', author: eaIdentity, body: 'First line from Alice 👋', timestamp: iso(nowMs - 4 * minMs) },
    { kind: 'text', id: 'sb-2', author: eaIdentity, body: 'Second line, same author within 5 min → grouped (header suppressed).', timestamp: iso(nowMs - 3 * minMs) },
    { kind: 'text', id: 'sb-3', author: eaSelf, body: 'Different author → not grouped; mirrors right ✅', timestamp: iso(nowMs - 2 * minMs), isOwn: true },
    { kind: 'system', id: 'sb-4', body: 'alice changed the topic', timestamp: iso(nowMs - 90 * 1000) },
    { kind: 'text', id: 'sb-5', author: eaIdentity, body: 'After a system notice → run broken, not grouped.', timestamp: iso(nowMs - 60 * 1000) },
  ];
  // stream-days — five distinct local days (oldest heads the stream un-dividered) → four day-CHANGE
  // dividers exhibiting all four label bands, and each post-divider row is grouped=false.
  const streamDays = [
    { kind: 'text', id: 'sd-1', author: eaIdentity, body: 'Oldest day (10 days ago) — top of stream, no divider above.', timestamp: iso(atNoon(nowMs - 10 * dayMs)) },
    { kind: 'text', id: 'sd-2', author: eaIdentity, body: '8 days ago → date-only divider above me (≥7 band).', timestamp: iso(atNoon(nowMs - 8 * dayMs)) },
    { kind: 'text', id: 'sd-3', author: eaSelf, body: '3 days ago → weekday+date divider (2–6 band).', timestamp: iso(atNoon(nowMs - 3 * dayMs)), isOwn: true },
    { kind: 'text', id: 'sd-4', author: eaIdentity, body: 'Yesterday → "Yesterday (…)" divider.', timestamp: iso(atNoon(nowMs - 1 * dayMs)) },
    { kind: 'text', id: 'sd-5', author: eaIdentity, body: 'Today → "Today (…)" divider.', timestamp: iso(atNoon(nowMs)) },
  ];
  // background / empty — the wallpaper layer resolves widgetId → component (drop-unknown W-13), same
  // registry as message.details; a KNOWN mount shows through when empty (no fallback paragraph), an
  // UNKNOWN one drops (backgroundMountCount 0) but is still "declared" (still no fallback paragraph).
  const streamWidgets = { 'fixture.wallpaper': FixtureWidget, 'fixture.time': FixtureWidget, 'fixture.badge': FixtureWidget, 'fixture.reg': FixtureRegWidget };

  // stream-extras (M-RP6.9) — the N×M bench, and the only place the registration question can
  // actually be answered: FOUR rows each carrying THREE mounts, one of them registering. Grouping is
  // stream-computed, so the run is built to yield exactly two grouped rows (alice, alice → grouped;
  // self breaks the run; self, self → grouped), which puts the socket on both row types in one
  // fixture. Same local day throughout ⇒ no day dividers to break the runs.
  // Expect: 4 registering mounts, ids `stream-extras__m-<msgid>__x-<key>`, all distinct.
  const sxMounts = [
    { widgetId: 'fixture.time', props: { label: '👍 4' } },
    { widgetId: 'fixture.badge', props: { label: '❤️ 2', tone: 'muted' } },
    { widgetId: 'fixture.reg', props: { label: '👀' } },
  ];
  const streamExtras = [
    { kind: 'text', id: 'sx-1', author: eaIdentity, body: 'Row 1 — head of the run, header + avatar shown.', timestamp: iso(nowMs - 4 * minMs), bodyExtras: sxMounts },
    { kind: 'text', id: 'sx-2', author: eaIdentity, body: 'Row 2 — grouped continuation. No header, no avatar. Strip still renders.', timestamp: iso(nowMs - 3 * minMs), bodyExtras: sxMounts },
    { kind: 'text', id: 'sx-3', author: eaSelf, body: 'Row 3 — different author breaks the run; mirrors right.', timestamp: iso(nowMs - 2 * minMs), isOwn: true, bodyExtras: sxMounts },
    { kind: 'text', id: 'sx-4', author: eaSelf, body: 'Row 4 — grouped continuation on an own row.', timestamp: iso(nowMs - 1 * minMs), isOwn: true, bodyExtras: sxMounts },
  ];
  const streamBgKnown = [{ widgetId: 'fixture.wallpaper', props: { label: 'wallpaper', tone: 'muted' } }];
  const streamBgUnknown = [{ widgetId: 'does.not.exist', props: { label: 'GHOST' } }];
  let streamBgLive = $state(true); // sampler control → backgroundLive
  let streamSel = $state(); // bind:selected proof (click-select)

  // stream-scroll (M-RP5.6 B) — the ONE live fixture for the scroll machine. A MUTABLE $state array
  // seeded with 10 `text` messages (1 min apart, mostly same author → grouped runs, one `eaSelf` to
  // break) — enough to overflow max-height:340px so the viewport is actually scrollable. append /
  // prepend / reset reassign the array (`[...arr, m]` / `[m, ...arr]`) so the stream's `$derived`
  // recompute + the B classifier (count-grew + first-id) fire on the honest user path (CDP clicks the
  // real buttons). The four static fixtures above (basic/days/empty/bg) are UNTOUCHED (they proved A).
  const ssSeed = () => {
    const arr = [];
    for (let i = 9; i >= 0; i--) {
      const own = i === 4;
      arr.push({
        kind: 'text',
        id: `ss-seed-${i}`,
        author: own ? eaSelf : eaIdentity,
        body: `Seed line ${10 - i} — enough rows to overflow the 340px viewport so the scroll machine has something to scroll.`,
        timestamp: iso(nowMs - i * minMs),
        ...(own ? { isOwn: true } : {}),
      });
    }
    return arr;
  };
  let ssCounter = $state(0);
  let streamScroll = $state(ssSeed());
  function ssAppend() {
    const last = streamScroll[streamScroll.length - 1];
    ssCounter++;
    streamScroll = [
      ...streamScroll,
      {
        kind: 'text',
        id: `ss-app-${ssCounter}`,
        author: last?.author ?? eaIdentity,
        body: `Appended #${ssCounter} at load-now (extends the last author's group).`,
        timestamp: iso(nowMs),
        ...(last?.isOwn ? { isOwn: true } : {}),
      },
    ];
  }
  function ssPrepend() {
    const first = streamScroll[0];
    ssCounter++;
    const firstMs = first ? Date.parse(first.timestamp) : nowMs;
    streamScroll = [
      {
        kind: 'text',
        id: `ss-pre-${ssCounter}`,
        author: eaIdentity,
        body: `Prepended #${ssCounter} — older (10 min before the current top).`,
        timestamp: iso(firstMs - 10 * minMs),
      },
      ...streamScroll,
    ];
  }
  function ssReset() {
    ssCounter = 0;
    streamScroll = ssSeed();
  }

  // ── M-RP6.3 Leg C1 fixtures ──────────────────────────────────────────────────────────────────
  // stream-fit — the SIZED-HOST fill proof (F-2). The stream now imposes NO height of its own, so a
  // host must supply one; `streamFitH` toggles TWO heights (V2 — one height could be a coincidence of
  // a hardcoded number). Seeded to overflow at BOTH so the viewport self-scrolls (V3). NB: this is a
  // fixture problem, not a component one — DO NOT put a cap back inside the component (§3.1).
  const streamFit = [];
  for (let i = 15; i >= 0; i--) {
    const own = i % 4 === 0;
    streamFit.push({
      kind: 'text',
      id: `sf-${i}`,
      author: own ? eaSelf : eaIdentity,
      body: `Fit line ${16 - i} — enough rows to overflow both 240px and 640px so the viewport self-scrolls, not the document.`,
      timestamp: iso(nowMs - i * minMs),
      ...(own ? { isOwn: true } : {}),
    });
  }
  let streamFitH = $state(240); // sized-host height, toggled 240 ↔ 640 (V2)

  // stream-status — the connection-gap row (F-4), fed by a MUTABLE status ARRAY (history accumulates;
  // "the live widget IS the historical marker, matured"). `gap-1` cycles through all four phases
  // WITHOUT its id changing (V6 — the row matures in place, key stays `status-gap-1`). The two
  // same-author messages straddle the gap so the gap BREAKS the run (V7). `status` carries DATA only;
  // the row renders as unstyled inline chrome (`<div class="stream-status" data-phase>`), appearance
  // is Ms Design's — so nothing is visible here yet by design.
  const streamStatusMsgs = [
    { kind: 'text', id: 'st-1', author: eaIdentity, body: 'Before the gap.', timestamp: iso(nowMs - 3 * minMs) },
    { kind: 'text', id: 'st-2', author: eaIdentity, body: 'After the gap — same author within window; would group but for the status break.', timestamp: iso(nowMs - 1 * minMs) },
  ];
  const gap1 = () => ({ id: 'gap-1', phase: 'counting-down', timestamp: nowMs - 2 * minMs, attempt: 2, maxAttempts: 10, remainingMs: 6000 });
  let streamStatuses = $state([gap1()]);
  // Mount toggle ({#if}, unlike the tab panels' display:none) so the interval+DEV-hook `$effect`
  // cleanup is drivable: unmount → `stream-status` gone from __XGEN_STREAM__ (proxy for clearInterval
  // running — same cleanup; the 60s firing itself is unproven-by-design in a CDP session).
  let streamStatusMounted = $state(true);
  function setStatusPhase(phase) {
    streamStatuses = streamStatuses.map((s, i) => (i === 0 ? { ...s, phase } : s));
  }
  function addStatusEpisode() {
    streamStatuses = [
      ...streamStatuses,
      { id: `gap-${streamStatuses.length + 1}`, phase: 'counting-down', timestamp: nowMs - 30 * 1000, attempt: 1, maxAttempts: 10, remainingMs: 8000 },
    ];
  }
  function resetStatuses() {
    streamStatuses = [gap1()];
  }

  // select-multiple shares the N-034 options-prop shape (carried over from `select`).
  const smOptions = [
    { value: 'a', label: 'Alpha' },
    { value: 'b', label: 'Beta' },
    { value: 'c', label: 'Gamma' },
  ];

  // led — caller-supplied state→colour map (hex OR var(--token)); colour is data, not accent.
  const ledStates = { ON: '#22c55e', OFF: 'var(--t4)', ERR: 'var(--err)' };

  // Inline data-URI so image#default renders with no file linkage (Joe's call):
  // the actual ui/assets/img-placeholder.svg artwork (mountains+sun glyph), hardcoded
  // here with explicit 72x72 so the sizeless-viewBox SVG renders tidy in the cell.
  const imgSrc =
    "data:image/svg+xml,%3Csvg viewBox='0 0 120 120' width='72' height='72' xmlns='http://www.w3.org/2000/svg' role='img' aria-label='Image placeholder'%3E%3Crect width='120' height='120' fill='%23c6c6c6'/%3E%3Crect x='30' y='40' width='60' height='43' rx='4' fill='none' stroke='%23e6e6e6' stroke-width='3.6' stroke-linejoin='round'/%3E%3Ccircle cx='43' cy='54' r='6' fill='%23e6e6e6'/%3E%3Cpath d='M33 79 L49 56 L60 68 L74 50 L87 79 Z' fill='%23e6e6e6'/%3E%3C/svg%3E";
</script>

<div class="sampler-bar">
  <span class="sampler-title">XGen Sampler</span>
  <div class="sampler-seg" role="group" aria-label="skin">
    <button class:active={shell === 'client'} onclick={() => applyShell('client')}>client</button>
    <button class:active={shell === 'node'} onclick={() => applyShell('node')}>node</button>
  </div>
</div>

<div class="sampler-tabs" role="tablist" aria-label="component class x arity">
  {#each tabs as t}
    <button
      role="tab"
      aria-selected={activeTab === t.id}
      class:active={activeTab === t.id}
      onclick={() => (activeTab = t.id)}
    >{t.label}</button>
  {/each}
</div>

<!-- Scroll region (M-RP4.9): the header block above (bar + tabs) is fixed; only this body
  scrolls (flex:1 + overflow-y:auto). All panels stay mounted inside it (N-053). -->
<div class="sampler-scroll">
<!-- DI · atomic — the current 44-cell grid; INTERACTIVE / DISPLAY / NAVIGATION sub-headers -->
<div class="sampler-panel" class:hidden={activeTab !== 'di-atomic'}>
  <div class="sampler-body">
    <div class="s-section-title">Interactive</div>

    <div class="s-row">
      <div class="s-rowname">toggle</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">toggle#default</span><Toggle bind:checked={tglDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">toggle#checked</span><Toggle bind:checked={tglChecked} id="checked" /></div>
        <div class="s-cell"><span class="s-id">toggle#switch</span><Toggle bind:checked={tglSwitch} id="switch" shape="switch" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">button</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">button#default</span><Button label="Action" id="default" /></div>
        <div class="s-cell"><span class="s-id">button#disabled</span><Button label="Action" id="disabled" disabled /></div>
        <div class="s-cell"><span class="s-id">button#toggle</span><Button label="Toggle" id="toggle" mode="toggle" bind:pressed={btnPressed} /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">textfield</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">textfield#default</span><TextField bind:value={tfDefault} id="default" placeholder="text" /></div>
        <div class="s-cell"><span class="s-id">textfield#disabled</span><TextField bind:value={tfDisabled} id="disabled" disabled /></div>
        <div class="s-cell"><span class="s-id">textfield#invalid</span><TextField type="email" bind:value={tfInvalid} id="invalid" /></div>
        <div class="s-cell"><span class="s-id">textfield#password</span><TextField type="password" bind:value={tfPassword} id="password" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">select</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">select#default</span><Select bind:value={selDefault} id="default" options={selOptions} placeholder="Pick one" /></div>
        <div class="s-cell"><span class="s-id">select#disabled</span><Select bind:value={selDisabled} id="disabled" options={selOptions} disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">select-multiple</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">select-multiple#default</span><SelectMultiple bind:value={smDefault} id="default" options={smOptions} /></div>
        <div class="s-cell"><span class="s-id">select-multiple#seeded</span><SelectMultiple bind:value={smSeeded} id="seeded" options={smOptions} /></div>
        <div class="s-cell"><span class="s-id">select-multiple#disabled</span><SelectMultiple bind:value={smDisabled} id="disabled" options={smOptions} disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">textarea</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">textarea#default</span><TextArea bind:value={taDefault} id="default" rows={3} /></div>
        <div class="s-cell"><span class="s-id">textarea#disabled</span><TextArea bind:value={taDisabled} id="disabled" rows={3} disabled /></div>
        <div class="s-cell"><span class="s-id">textarea#processed</span><TextArea {...processor(substitutions.rules, { trusted: true })} bind:value={taProcessed} id="processed" rows={3} placeholder="type -> <- :) <3 :( -- to morph" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">number</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">number#default</span><NumberField bind:value={numDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">number#disabled</span><NumberField bind:value={numDisabled} id="disabled" disabled /></div>
        <div class="s-cell"><span class="s-id">number#invalid</span><NumberField bind:value={numInvalid} id="invalid" min={0} max={10} /></div>
        <div class="s-cell"><span class="s-id">number#clamped</span><NumberField {...clamp({ min: 0, max: 10 })} bind:value={numClamped} id="clamped" min={0} max={10} /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">converter-field</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">converter-field#default</span><ConverterField converter={numConv} bind:value={cvDefault} id="default" placeholder="number" /></div>
        <div class="s-cell"><span class="s-id">converter-field#disabled</span><ConverterField converter={numConv} bind:value={cvDisabled} id="disabled" disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">range</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">range#default</span><Range bind:value={rngDefault} id="default" min={0} max={100} step={1} /></div>
        <div class="s-cell"><span class="s-id">range#disabled</span><Range bind:value={rngDisabled} id="disabled" min={0} max={100} step={1} disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">date</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">date#default</span><DateField bind:value={dtDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">date#time</span><DateField type="time" bind:value={dtTime} id="time" /></div>
        <div class="s-cell"><span class="s-id">date#datetime</span><DateField type="datetime-local" bind:value={dtDatetime} id="datetime" /></div>
        <div class="s-cell"><span class="s-id">date#month</span><DateField type="month" bind:value={dtMonth} id="month" /></div>
        <div class="s-cell"><span class="s-id">date#week</span><DateField type="week" bind:value={dtWeek} id="week" /></div>
        <div class="s-cell"><span class="s-id">date#disabled</span><DateField bind:value={dtDisabled} id="disabled" disabled /></div>
        <div class="s-cell"><span class="s-id">date#invalid</span><DateField bind:value={dtInvalid} id="invalid" min="2026-01-01" max="2026-12-31" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">color</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">color#default</span><ColorField bind:value={colDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">color#disabled</span><ColorField bind:value={colDisabled} id="disabled" disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">file</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">file#default</span><FileField bind:files={fDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">file#multiple</span><FileField bind:files={fMultiple} id="multiple" multiple /></div>
        <div class="s-cell"><span class="s-id">file#disabled</span><FileField bind:files={fDisabled} id="disabled" disabled /></div>
      </div>
    </div>

    <div class="s-section-title">Display</div>

    <div class="s-row">
      <div class="s-rowname">label</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">label#default</span><Label text="A field caption" id="default" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">paragraph</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">paragraph#default</span><Paragraph text="A single paragraph of prose, rendered read-only as a text node." id="default" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">image</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">image#default</span><Img src={imgSrc} alt="sample placeholder" id="default" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">led</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">led#default</span><Led states={ledStates} state="ON" id="default" /></div>
        <div class="s-cell"><span class="s-id">led#off</span><Led states={ledStates} state="OFF" id="off" /></div>
        <div class="s-cell"><span class="s-id">led#pulse</span><Led states={ledStates} state="ERR" pulse id="pulse" /></div>
        <div class="s-cell"><span class="s-id">led#unknown</span><Led states={ledStates} state="???" id="unknown" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">icon</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">icon#default</span><Icon name="caret-down" id="default" /></div>
        <div class="s-cell"><span class="s-id">icon#s16</span><Icon name="dot" size={16} id="s16" /></div>
        <div class="s-cell"><span class="s-id">icon#s20</span><Icon name="dot" size={20} id="s20" /></div>
        <div class="s-cell"><span class="s-id">icon#s24</span><Icon name="dot" size={24} id="s24" /></div>
        <div class="s-cell"><span class="s-id">icon#tinted</span><Icon name="square" tint="var(--accent2)" id="tinted" /></div>
        <div class="s-cell"><span class="s-id">icon#labelled</span><Icon name="caret-down" label="collapse" id="labelled" /></div>
        <div class="s-cell"><span class="s-id">icon#raw</span><Icon path="M5 5h14v14H5z" id="raw" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">separator</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">separator#horizontal</span><div style="width:120px"><Separator id="horizontal" /></div></div>
        <div class="s-cell"><span class="s-id">separator#vertical</span><div style="display:flex; align-items:stretch; gap:8px; height:20px"><Label text="A" /><Separator orientation="vertical" id="vertical" /><Label text="B" /></div></div>
        <div class="s-cell"><span class="s-id">separator#double</span><div style="width:120px"><Separator variant="double" id="double" /></div></div>
        <div class="s-cell"><span class="s-id">separator#gap</span><div style="width:120px"><Separator variant="gap" id="gap" /></div></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">meter</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">meter#optimum</span><Meter value={35} min={0} max={100} optimum={20} low={50} high={80} id="optimum" /></div>
        <div class="s-cell"><span class="s-id">meter#caution</span><Meter value={65} min={0} max={100} optimum={20} low={50} high={80} id="caution" /></div>
        <div class="s-cell"><span class="s-id">meter#danger</span><Meter value={94} min={0} max={100} optimum={20} low={50} high={80} id="danger" /></div>
        <div class="s-cell"><span class="s-id">meter#neutral</span><Meter value={50} min={0} max={100} id="neutral" /></div>
        <div class="s-cell"><span class="s-id">meter#fixed</span><Meter value={70} min={0} max={100} optimum={20} low={50} high={80} width="120px" id="fixed" /></div>
        <div class="s-cell"><span class="s-id">meter#disabled</span><Meter value={40} min={0} max={100} disabled id="disabled" /></div>
        <div class="s-cell"><span class="s-id">meter#custom</span><Meter value={70} min={0} max={100} optimum={20} low={50} high={80} fill="var(--accent2)" id="custom" /></div>
        <div class="s-cell"><span class="s-id">meter#neutral2</span><Meter value={50} min={0} max={100} fill="var(--t3)" id="neutral2" /></div>
      </div>
    </div>

    <div class="s-section-title">Container</div>

    <div class="s-row">
      <div class="s-rowname">section</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">section#plain</span><Section title="Spaces" id="plain">Body content here.</Section></div>
        <div class="s-cell"><span class="s-id">section#badged</span><Section title="Direct messages" badge="2/5" id="badged">alice, bob</Section></div>
        <div class="s-cell"><span class="s-id">section#collapsible</span><Section title="Rooms" collapsible id="collapsible">#general, #dev</Section></div>
        <div class="s-cell"><span class="s-id">section#bare</span><Section id="bare">No header — bare container.</Section></div>
        <div class="s-cell"><span class="s-id">section#nested</span><Section title="Outer" collapsible id="nested"><Section title="Inner" id="nested-inner">nested body</Section></Section></div>
        <div class="s-cell"><span class="s-id">section#fixed</span><Section title="Fixed width" width="320px" id="fixed">width=320px</Section></div>
      </div>
    </div>

    <div class="s-section-title">Navigation</div>

    <div class="s-row">
      <div class="s-rowname">link</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">link#default</span><Link href="#settings" text="Settings" id="default" /></div>
        <div class="s-cell"><span class="s-id">link#external</span><Link href="https://xgen.example" text="xgen.example" external ariaLabel="XGen site (opens externally)" id="external" /></div>
        <div class="s-cell"><span class="s-id">link#disabled</span><Link href="#x" text="Unavailable" disabled id="disabled" /></div>
      </div>
    </div>

    <div class="s-section-title">Token</div>

    <div class="s-row">
      <div class="s-rowname">chip</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">chip#default</span><Chip label="rust" id="default" onRemove={() => chipRemoved++} /></div>
        <div class="s-cell"><span class="s-id">chip#static</span><Chip label="svelte" removable={false} id="static" /></div>
        <div class="s-cell"><span class="s-id">chip#long</span><Chip label="a-very-long-keyword-that-truncates" id="long" onRemove={() => chipRemoved++} /></div>
      </div>
    </div>
  </div>
</div>

<!-- DI · composite — first occupant: status-indicator (M-RP2.22) -->
<div class="sampler-panel" class:hidden={activeTab !== 'di-composite'}>
  <div class="sampler-body">
    <div class="s-section-title">Composite</div>

    <div class="s-row">
      <div class="s-rowname">status-indicator</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">status-indicator#default</span><StatusIndicator states={ledStates} state="ON" caption="Connected" id="default" /></div>
        <div class="s-cell"><span class="s-id">status-indicator#withlink</span><StatusIndicator states={ledStates} state="OFF" caption="Disconnected" linkHref="https://xgen.example/status" linkText="Status page" linkExternal id="withlink" /></div>
        <div class="s-cell"><span class="s-id">status-indicator#pulse</span><StatusIndicator states={ledStates} state="ERR" pulse caption="Error" linkHref="#logs" linkText="View logs →" id="pulse" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">status-bar</div>
      <div class="s-cells">
        <div class="s-cell" style="flex:0 0 340px"><span class="s-id">status-bar#default</span><div style="width:340px"><StatusBar states={ledStates} state="ON" caption="Connected" id="default" onResizeGrip={() => sbGripFired++} /></div></div>
        <div class="s-cell" style="flex:0 0 340px"><span class="s-id">status-bar#secondary</span><div style="width:340px"><StatusBar states={ledStates} state="ERR" pulse caption="Reconnecting" secondaryText="v0.10.3" id="secondary" onResizeGrip={() => sbGripFired++} /></div></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">shelf</div>
      <div class="s-cells">
        <div class="s-cell" style="flex:0 0 220px"><span class="s-id">shelf#bottom</span><div style="width:220px"><Shelf items={shelfBottomItems} position="bottom" ariaLabel="System commands" onCommand={(c) => (shelfLastCommand = c)} id="bottom" /></div></div>
        <div class="s-cell" style="flex:0 0 220px"><span class="s-id">shelf#mixed</span><div style="width:220px"><Shelf items={shelfMixedItems} position="bottom" ariaLabel="Mixed shelf" onCommand={(c) => (shelfLastCommand = c)} id="mixed" /></div></div>
        <div class="s-cell" style="flex:0 0 220px"><span class="s-id">shelf#top-empty</span><div style="width:220px"><Shelf items={[]} position="top" ariaLabel="Favourites" id="top-empty" /></div></div>
        <div class="s-cell"><span class="s-id">lastCommand</span><output data-probe="shelf-last-command">{shelfLastCommand}</output></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">password-field</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">password-field#default</span><PasswordField bind:value={pwDefault} id="default" placeholder="password" autocomplete="current-password" /></div>
        <div class="s-cell"><span class="s-id">password-field#disabled</span><PasswordField bind:value={pwDisabled} id="disabled" disabled /></div>
        <div class="s-cell"><span class="s-id">password-field#revealed</span><PasswordField bind:value={pwRevealed} id="revealed" revealedByDefault /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">star-rating</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">star-rating#default</span><StarRating bind:value={srDefault} id="default" ariaLabel="Rate" /></div>
        <div class="s-cell"><span class="s-id">star-rating#rated</span><StarRating bind:value={srRated} id="rated" ariaLabel="Rate" /></div>
        <div class="s-cell"><span class="s-id">star-rating#readonly</span><StarRating bind:value={srReadonly} id="readonly" readonly ariaLabel="Rating" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">file-field</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">file-field#default</span><FileFieldComposite bind:files={ffDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">file-field#multiple</span><FileFieldComposite bind:files={ffMultiple} id="multiple" multiple /></div>
        <div class="s-cell"><span class="s-id">file-field#disabled</span><FileFieldComposite bind:files={ffDisabled} id="disabled" disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">combobox</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">combobox#default</span><Combobox bind:value={cbDefault} id="default" options={cbOptions} placeholder="Type or pick" /></div>
        <div class="s-cell"><span class="s-id">combobox#preset</span><Combobox bind:value={cbPreset} id="preset" options={cbOptions} /></div>
        <div class="s-cell"><span class="s-id">combobox#disabled</span><Combobox bind:value={cbDisabled} id="disabled" options={cbOptions} disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">tag-select</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">tag-select#default</span><TagSelect bind:value={tsDefault} id="default" options={tagOptions} width="260px" onManage={() => tsManaged++} placeholder="Add tags" /></div>
        <div class="s-cell"><span class="s-id">tag-select#max</span><TagSelect bind:value={tsMax} id="max" options={tagOptions} max={2} onManage={() => tsManaged++} placeholder="max 2" /></div>
        <div class="s-cell"><span class="s-id">tag-select#create</span><TagSelect bind:value={tsCreate} id="create" options={tagOptions} allowCreate onManage={() => tsManaged++} placeholder="type + Enter" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">color-picker</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">color-picker#default</span><ColorPicker bind:value={cpDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">color-picker#disabled</span><ColorPicker bind:value={cpDisabled} id="disabled" disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">dialog (onClose fired: {dlgClosed})</div>
      <div class="s-cells">
        <div class="s-cell">
          <span class="s-id">dialog#default</span>
          <button type="button" class="s-note" onclick={() => (dlgDefault = true)}>Open dialog</button>
          <Dialog title="Confirm action" bind:open={dlgDefault} id="default" onClose={() => dlgClosed++}>
            This is a modal dialog. Close it with the button below or the Esc key — the backdrop does not dismiss.
          </Dialog>
        </div>
        <div class="s-cell">
          <span class="s-id">dialog#labelled</span>
          <button type="button" class="s-note" onclick={() => (dlgLabelled = true)}>Open (custom label)</button>
          <Dialog title="About XGen" bind:open={dlgLabelled} closeLabel="Done" id="labelled" onClose={() => dlgClosed++}>
            A dialog exercising a non-default closeLabel ("Done", not the "Close" default).
          </Dialog>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- DD · atomic — first occupant: entity-avatar (M-RP5.0). kind x variant grid + edge cells -->
<div class="sampler-panel" class:hidden={activeTab !== 'dd-atomic'}>
  <div class="sampler-body">
    <div class="s-section-title">Atomic</div>

    <div class="s-row">
      <div class="s-rowname">entity-avatar · identity</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-avatar#identity-presence</span><EntityAvatar descriptor={eaIdentity} variant="presence" onActivate={() => eaActivated++} id="identity-presence" /></div>
        <div class="s-cell"><span class="s-id">entity-avatar#identity-list</span><EntityAvatar descriptor={eaIdentity} variant="list" onActivate={() => eaActivated++} id="identity-list" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-avatar · space</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-avatar#space-presence</span><EntityAvatar descriptor={eaSpace} variant="presence" id="space-presence" /></div>
        <div class="s-cell"><span class="s-id">entity-avatar#space-list</span><EntityAvatar descriptor={eaSpace} variant="list" id="space-list" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-avatar · DM</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-avatar#dm-presence</span><EntityAvatar descriptor={eaDm} variant="presence" id="dm-presence" /></div>
        <div class="s-cell"><span class="s-id">entity-avatar#dm-list</span><EntityAvatar descriptor={eaDm} variant="list" id="dm-list" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-avatar · edge</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-avatar#absent</span><EntityAvatar descriptor={eaAnon} variant="list" id="absent" /></div>
        <div class="s-cell"><span class="s-id">entity-avatar#revoked</span><EntityAvatar descriptor={eaRevoked} variant="list" id="revoked" /></div>
        <div class="s-cell"><span class="s-id">entity-avatar#ai</span><EntityAvatar descriptor={eaAi} variant="list" id="ai" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-avatar · room (M-RP5.0c)</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-avatar#room-presence</span><EntityAvatar descriptor={eaRoom} variant="presence" id="room-presence" /></div>
        <div class="s-cell"><span class="s-id">entity-avatar#room-list</span><EntityAvatar descriptor={eaRoom} variant="list" id="room-list" /></div>
        <div class="s-cell"><span class="s-id">entity-avatar#room-status</span><EntityAvatar descriptor={eaRoom} variant="list" status={stAvatar} id="room-status" /></div>
      </div>
    </div>

    <div class="s-section-title">Self-status (M-RP5.1a)</div>

    <div class="s-row">
      <div class="s-rowname">status · badge</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">status#badge-emoji</span><Status status={stEmoji} variant="badge" id="badge-emoji" /></div>
        <div class="s-cell"><span class="s-id">status#badge-noemoji</span><Status status={stTextOnly} variant="badge" id="badge-noemoji" /></div>
        <div class="s-cell"><span class="s-id">status#expired</span><Status status={stExpired} variant="badge" id="expired" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">status · line / full</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">status#line</span><Status status={stEmojiText} variant="line" id="line" /></div>
        <div class="s-cell"><span class="s-id">status#line-noemoji</span><Status status={stTextOnly} variant="line" id="line-noemoji" /></div>
        <div class="s-cell"><span class="s-id">status#full</span><Status status={stFull} variant="full" id="full" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-avatar · with status (M-RP5.1b)</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-avatar#with-status</span><EntityAvatar descriptor={eaIdentity} variant="list" status={stAvatar} id="with-status" /></div>
      </div>
    </div>
  </div>
</div>

<!-- DD · composite — first occupant: entity-item (M-RP5.1). variant x kind grid + edge cells.
  Each cell yields TWO registry entries: the composite (entity-item#id) + its self-registering
  avatar child (entity-avatar#id__avatar) — the matrix multiplies (status-indicator precedent). -->
<div class="sampler-panel" class:hidden={activeTab !== 'dd-composite'}>
  <div class="sampler-body">
    <div class="s-section-title">Composite</div>

    <div class="s-row">
      <div class="s-rowname">entity-item · variant × kind</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-item#row-identity</span><EntityItem descriptor={eaIdentity} variant="row" meta="3" onActivate={() => eiActivated++} id="row-identity" /></div>
        <div class="s-cell"><span class="s-id">entity-item#card-space</span><EntityItem descriptor={eaSpace} variant="card" secondary="#general · 42 members" status={eiStatus} onActivate={() => eiActivated++} id="card-space" /></div>
        <div class="s-cell"><span class="s-id">entity-item#nav-dm</span><EntityItem descriptor={eaDm} variant="nav" onActivate={() => eiActivated++} id="nav-dm" /></div>
        <div class="s-cell"><span class="s-id">entity-item#inline-identity</span><EntityItem descriptor={eaIdentity} variant="inline" id="inline-identity" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-item · edge</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">entity-item#selected</span><EntityItem descriptor={eaIdentity} variant="row" meta="7" selected onActivate={() => eiActivated++} id="selected" /></div>
        <div class="s-cell"><span class="s-id">entity-item#card-plain</span><EntityItem descriptor={eaSpace} variant="card" id="card-plain" /></div>
        <div class="s-cell"><span class="s-id">entity-item#fixed</span><EntityItem descriptor={eaIdentity} variant="row" meta="1" width="280px" id="fixed" /></div>
      </div>
    </div>

    <div class="s-section-title">Panel (M-RP5.2, last dd-composite)</div>

    <div class="s-row">
      <div class="s-rowname">entity-panel · spaces / DMs</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#spaces</span><EntityPanel items={epSpaces} title="Spaces" badge="3" collapsible bind:selected={epSelSpaces} onActivate={() => epActivated++} id="spaces" /></div>
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#dms</span><EntityPanel items={epDms} title="Direct messages" bind:selected={epSelDms} onActivate={() => epActivated++} id="dms" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-panel · empty / collapsed</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#empty</span><EntityPanel items={[]} title="Blocked" emptyText="No blocked users." id="empty" /></div>
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#collapsed</span><EntityPanel items={epArchived} title="Archived" badge="2" collapsible bind:collapsed={epCollapsed} id="collapsed" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-panel · inert (M-RP-PANEL-INERT)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#inert</span><EntityPanel items={epSpaces} title="Members (inert)" interactive={false} selected="xgen://space/dev-2b11" id="inert" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">entity-panel · unresolved (M-RP-IDENTITY-RESOLUTION)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#unresolved</span><EntityPanel items={epUnresolved} interactive={false} selected="xgen://identity/erased-1" id="unresolved" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">room ripple (M-RP5.0c)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-item#room-item</span><EntityItem descriptor={eaRoom} variant="row" meta="9" id="room-item" /></div>
        <div class="s-cell" style="width: 300px; align-self: flex-start"><span class="s-id">entity-panel#rooms</span><EntityPanel items={epRooms} title="Rooms" bind:selected={epSelRooms} id="rooms" /></div>
      </div>
    </div>

    <div class="s-section-title">Message (M-RP5.5 A/B/C — text full + states + system)</div>

    <div class="s-row">
      <div class="s-rowname">message · text full × isOwn</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-other</span><Message descriptor={msgOther} id="text-other" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-own</span><Message descriptor={msgOwn} id="text-own" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message · details socket</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-details</span><Message descriptor={msgDetails} widgets={msgWidgets} id="text-details" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-unknown-widget</span><Message descriptor={msgUnknownWidget} widgets={msgWidgets} id="text-unknown-widget" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message · states (B): grouped / edited / deleted</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-grouped</span><Message descriptor={msgGrouped} grouped id="text-grouped" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-edited</span><Message descriptor={msgEdited} id="text-edited" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-deleted</span><Message descriptor={msgDeleted} widgets={msgWidgets} id="text-deleted" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-grouped-edited</span><Message descriptor={msgGroupedEdited} grouped id="text-grouped-edited" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message · bodyExtras socket (M-RP6.9)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-body-extras</span><Message descriptor={msgBodyExtras} widgets={msgWidgets} id="text-body-extras" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-extras-grouped</span><Message descriptor={msgExtrasGrouped} widgets={msgWidgets} grouped id="text-extras-grouped" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-extras-unknown</span><Message descriptor={msgExtrasUnknown} widgets={msgWidgets} id="text-extras-unknown" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#text-extras-deleted</span><Message descriptor={msgExtrasDeleted} widgets={msgWidgets} id="text-extras-deleted" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message · system (C)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#system-notice</span><Message descriptor={msgSystemNotice} id="system-notice" /></div>
        <div class="s-cell" style="width: 340px; align-self: flex-start"><span class="s-id">message#system-long</span><Message descriptor={msgSystemLong} id="system-long" /></div>
      </div>
    </div>

    <div class="s-section-title">Message-stream (M-RP5.6 A — shell)</div>

    <div class="s-row">
      <div class="s-rowname">message-stream · basic (grouping + system-break)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-basic</span><MessageStream messages={streamBasic} widgets={streamWidgets} bind:selected={streamSel} id="stream-basic" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message-stream · bodyExtras N×M bench (M-RP6.9)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-extras</span><MessageStream messages={streamExtras} widgets={streamWidgets} id="stream-extras" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message-stream · days (four divider bands)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-days</span><MessageStream messages={streamDays} id="stream-days" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message-stream · empty / background (backgroundLive: <button type="button" class="s-note" onclick={() => (streamBgLive = !streamBgLive)}>{streamBgLive}</button>)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-empty</span><MessageStream messages={[]} id="stream-empty" /></div>
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-bg</span><MessageStream messages={[]} background={streamBgKnown} widgets={streamWidgets} backgroundLive={streamBgLive} id="stream-bg" /></div>
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-bg-unknown</span><MessageStream messages={[]} background={streamBgUnknown} widgets={streamWidgets} id="stream-bg-unknown" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message-stream · scroll (M-RP5.6 B — live: <button type="button" class="s-note" onclick={ssAppend}>append</button> <button type="button" class="s-note" onclick={ssPrepend}>prepend</button> <button type="button" class="s-note" onclick={ssReset}>reset</button>)</div>
      <div class="s-cells">
        <!-- C1: max-height is gone from the component (F-2), so this fixture supplies a sized host to
          stay scrollable (fixture hygiene — the cap belongs on the host, never back in the component). -->
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-scroll</span><div style="height: 340px"><MessageStream messages={streamScroll} id="stream-scroll" /></div></div>
      </div>
    </div>

    <div class="s-section-title">Message-stream (M-RP6.3 Leg C1 — region fitness)</div>

    <div class="s-row">
      <div class="s-rowname">message-stream · fit (F-2 sized host — height: <button type="button" class="s-note" onclick={() => (streamFitH = streamFitH === 240 ? 640 : 240)}>{streamFitH}px</button>; fills + self-scrolls)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-fit</span><div style="height: {streamFitH}px"><MessageStream messages={streamFit} id="stream-fit" /></div></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">message-stream · status (F-4 — gap-1 phase: <button type="button" class="s-note" onclick={() => setStatusPhase('counting-down')}>counting-down</button> <button type="button" class="s-note" onclick={() => setStatusPhase('dialling')}>dialling</button> <button type="button" class="s-note" onclick={() => setStatusPhase('resolved')}>resolved</button> <button type="button" class="s-note" onclick={() => setStatusPhase('exhausted')}>exhausted</button> · <button type="button" class="s-note" onclick={addStatusEpisode}>+episode</button> <button type="button" class="s-note" onclick={resetStatuses}>reset</button> · <button type="button" class="s-note" onclick={() => (streamStatusMounted = !streamStatusMounted)}>{streamStatusMounted ? 'unmount' : 'mount'}</button>)</div>
      <div class="s-cells">
        <div class="s-cell" style="width: 380px; align-self: flex-start"><span class="s-id">message-stream#stream-status</span><div style="height: 240px">{#if streamStatusMounted}<MessageStream messages={streamStatusMsgs} status={streamStatuses} id="stream-status" />{/if}</div></div>
      </div>
    </div>
  </div>
</div>

<!-- WIDGET (Level-2 UI-plugin, D-102) -->
<div class="sampler-panel" class:hidden={activeTab !== 'widget'}>
  <div class="sampler-body">
    <div class="s-group">
      <div class="s-rowname">substitutions-editor</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">substitutions-editor#demo</span><SubstitutionsEditor id="demo" /></div>
      </div>
    </div>
    <div class="s-group">
      <div class="s-rowname">entity-context-menu</div>
      <div class="s-cells">
        <div class="s-cell" style="align-items:flex-start;">
          <span class="s-id">entity-context-menu#demo</span>
          <button type="button" class="ecm-trigger" onclick={(e) => ecmRef.open(e.currentTarget)}>Open menu ▾</button>
          <EntityContextMenu bind:this={ecmRef} descriptor={ecmEntity} status={ecmStatus} portal={true} onSelect={ecmSelect} id="demo" />
          <span class="s-note">selected: {ecmSelected}</span>
        </div>
      </div>
    </div>
  </div>
</div>
</div><!-- /.sampler-scroll -->
