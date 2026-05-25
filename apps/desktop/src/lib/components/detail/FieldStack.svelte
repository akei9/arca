<script lang="ts">
  import type { EntryDto } from '../../ipc';
  import { Icon } from '../icons';
  import { Entropy, IconButton, Kbd } from '../primitives';

  let {
    entry,
  } = $props<{
    entry: EntryDto;
  }>();

  const passwordMask = '●●●●●●●●●●●●●●●●●●●●●●●●';
  const url = $derived(entry.url?.trim() || 'not_set');
  const username = $derived(entry.username.trim() || 'not_set');
  const entropyBits = $derived(Math.max(72, Math.min(112, entry.title.length * 6 + entry.username.length * 4 + 48)));
  const entropyFilled = $derived(Math.max(10, Math.min(16, Math.round(entropyBits / 7))));
</script>

<div class="fields">
  <div class="field">
    <div class="field__k">url</div>
    <div class={entry.url ? 'field__v field__v--url' : 'field__v'}>{url}</div>
    <div class="field__actions">
      <IconButton label={`Open ${entry.title} URL`} disabled={!entry.url}>
        <Icon name="external" size={13} />
      </IconButton>
    </div>
  </div>

  <div class="field">
    <div class="field__k">user <Kbd value="U" /></div>
    <div class="field__v">{username}</div>
    <div class="field__actions">
      <IconButton label={`Copy ${entry.title} username`}>
        <Icon name="copy" size={13} />
      </IconButton>
    </div>
  </div>

  <div class="field field--focus">
    <div class="field__k">password <Kbd value="C" /></div>
    <div class="field__v field__v--mask">{passwordMask}</div>
    <div class="field__actions">
      <IconButton label={`Reveal ${entry.title} password`}>
        <Icon name="eye" size={13} />
      </IconButton>
      <IconButton variant="accent" label={`Copy ${entry.title} password`}>✓</IconButton>
    </div>
  </div>

  <div class="field">
    <div class="field__k">entropy</div>
    <div>
      <Entropy filled={entropyFilled} bits={entropyBits} />
    </div>
    <div></div>
  </div>
</div>
