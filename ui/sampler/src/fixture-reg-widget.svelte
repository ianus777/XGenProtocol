<script>
  // Sampler-local REGISTERING fixture stub (M-RP6.9, S-4). The sibling of `fixture-widget.svelte`
  // and its opposite number: that one deliberately does NOT register (it is the control, and it
  // stays untouched); this one DOES.
  //
  // WHY IT EXISTS. §1.1 established by READING that a mounted widget is invisible to the registry
  // today — `cid()` is only ever called with hardcoded literals and no `id` reached a mount, so
  // "N mounts × M rows" was neither N×M new ids nor N×M collisions but sixty zeros. That is a
  // correct answer and an unmeasured one, and it stops being true the moment a real tenant arrives:
  // an interactive reaction WILL want to register. This stub is the instrument that turns the read
  // answer into a measured one — it takes the `id` the container hands down (D-1) and registers
  // under it, so the N×M question is answered on the registry rather than in an argument.
  //
  // It is also the negative-space proof for D-4: on a tombstone the socket is not rendered, so this
  // stub does not mount, so no `x-` id appears — an assertion that is only meaningful because this
  // component would otherwise have produced one.
  //
  // NOT a real widget, and it holds no meaning: it renders its label and nothing else. The fence
  // (§0) is that the container never learns what a mount IS; this stub is careful to be nothing.
  import { envelope } from '$common/components/base/envelope';

  let { label = 'reg', id = undefined } = $props();

  const debug = () => ({ label });
</script>

<!-- Appearance lives in `skin.css` (§ SAMPLER FIXTURE STUBS, M-RP6.9) — shared with the control stub,
  so the twins are now identical BY CONSTRUCTION rather than by hand-synced copies. `data-reg` carries
  the single intentional divergence (the dotted outline). -->
<span class="fixture-widget" data-reg="true" use:envelope={{ name: 'fixture-reg', id, debug }}>{label}</span>
