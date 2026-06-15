<script lang="ts">
  import { onMount } from 'svelte';
  import { writeClipboardText } from '../../clipboard';
  import { generatePassword, type GeneratedPassword } from '../../ipc';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, IconButton, Tag } from '../primitives';

  let length = $state(24);
  let uppercase = $state(true);
  let lowercase = $state(true);
  let digits = $state(true);
  let symbols = $state(true);
  let excludeAmbiguous = $state(false);
  let generated = $state<GeneratedPassword | null>(null);
  let isRevealed = $state(false);
  let busy = $state(false);
  let copied = $state(false);
  let errorMessage = $state('');
  let copyTimer: ReturnType<typeof setTimeout> | null = null;
  let generationCounter = 0;

  const hasCharacterSet = $derived(uppercase || lowercase || digits || symbols);
  const entropyLabel = $derived(generated ? `${Math.round(generated.entropyBits)} bits` : 'pending');
  const strengthLabel = $derived(strengthFromEntropy(generated?.entropyBits ?? 0));
  const outputValue = $derived(
    generated ? (isRevealed ? generated.password : maskPassword(generated.password)) : '••••••••••••••••••••••••',
  );

  onMount(() => {
    return () => {
      if (copyTimer) {
        clearTimeout(copyTimer);
      }
    };
  });

  async function generate() {
    const currentGeneration = ++generationCounter;

    if (!hasCharacterSet) {
      generated = null;
      isRevealed = false;
      copied = false;
      errorMessage = 'Select at least one character set';
      return;
    }

    busy = true;
    copied = false;
    isRevealed = false;
    errorMessage = '';

    try {
      const nextGenerated = await generatePassword({
        length,
        uppercase,
        lowercase,
        digits,
        symbols,
        excludeAmbiguous,
      });

      if (currentGeneration !== generationCounter) {
        return;
      }

      generated = nextGenerated;
    } catch (error) {
      if (currentGeneration !== generationCounter) {
        return;
      }

      generated = null;
      isRevealed = false;
      errorMessage = messageFromError(error);
    } finally {
      if (currentGeneration === generationCounter) {
        busy = false;
      }
    }
  }

  async function copyGenerated() {
    if (!generated?.password) {
      return;
    }

    copied = await writeClipboardText(generated.password);

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    copyTimer = setTimeout(() => {
      copied = false;
      copyTimer = null;
    }, 1500);
  }

  function resetGenerated() {
    generationCounter += 1;
    generated = null;
    isRevealed = false;
    copied = false;
    busy = false;
    errorMessage = '';
  }

  function toggleReveal() {
    isRevealed = !isRevealed;
  }

  function maskPassword(value: string): string {
    return '•'.repeat(Math.max(value.length, 8));
  }

  function strengthFromEntropy(entropy: number): string {
    if (entropy >= 120) {
      return 'excellent';
    }

    if (entropy >= 80) {
      return 'strong';
    }

    if (entropy >= 50) {
      return 'fair';
    }

    return generated ? 'weak' : 'pending';
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to generate password';
  }
</script>

<section class="generator-panel" aria-labelledby="generator-title">
  <div class="detail-head">
    <div class="detail__title-wrap">
      <div class="detail__crumbs mono">vault &nbsp;/&nbsp; <b>generator</b></div>
      <h1 id="generator-title" class="detail__title">generate<em>.</em></h1>
      <div class="detail__meta mono">
        <Tag value={strengthLabel} />
        <span>vault · <b>{vaultState.vaultName || 'local'}</b></span>
        <span>entropy · <b>{entropyLabel}</b></span>
      </div>
    </div>
    <div class="detail__actions">
      <Button variant="primary" size="sm" onclick={generate} disabled={busy || !hasCharacterSet}>
        <Icon name="refresh" size={12} />
        {busy ? 'generating' : 'generate'}
      </Button>
    </div>
  </div>

  <div class="generator-body">
    <section class="generator-output" aria-labelledby="generator-output-title">
      <div class="generator-output__head">
        <span id="generator-output-title">password</span>
        <div class="generator-output__actions">
          <IconButton
            label={isRevealed ? 'Hide generated password' : 'Reveal generated password'}
            onclick={toggleReveal}
            disabled={!generated?.password}
          >
            <Icon name="eye" size={13} />
          </IconButton>
          <IconButton label="Copy generated password" onclick={copyGenerated} disabled={!generated?.password}>
            <Icon name="copy" size={13} />
          </IconButton>
        </div>
      </div>
      <output class="generator-output__value">{outputValue}</output>
      <div class="generator-output__meta mono">
        <span>length · <b>{length}</b></span>
        <span>entropy · <b>{entropyLabel}</b></span>
        <span>copy · <b>{copied ? 'done' : 'ready'}</b></span>
      </div>
      {#if errorMessage}
        <div class="generator-error mono error" role="alert">{errorMessage}</div>
      {/if}
    </section>

    <section class="generator-controls" aria-labelledby="generator-controls-title">
      <div class="generator-controls__head">
        <p class="generator-controls__eyebrow mono">policy</p>
        <h2 id="generator-controls-title">random</h2>
      </div>

      <label class="generator-slider">
        <span>length</span>
        <input bind:value={length} type="range" min="8" max="128" step="1" oninput={resetGenerated} />
        <b>{length}</b>
      </label>

      <div class="generator-options">
        <label>
          <input bind:checked={uppercase} type="checkbox" onchange={resetGenerated} />
          <span>uppercase</span>
        </label>
        <label>
          <input bind:checked={lowercase} type="checkbox" onchange={resetGenerated} />
          <span>lowercase</span>
        </label>
        <label>
          <input bind:checked={digits} type="checkbox" onchange={resetGenerated} />
          <span>digits</span>
        </label>
        <label>
          <input bind:checked={symbols} type="checkbox" onchange={resetGenerated} />
          <span>symbols</span>
        </label>
        <label>
          <input bind:checked={excludeAmbiguous} type="checkbox" onchange={resetGenerated} />
          <span>no_ambiguous</span>
        </label>
      </div>
    </section>
  </div>
</section>
