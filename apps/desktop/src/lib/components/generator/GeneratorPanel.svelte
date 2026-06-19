<script lang="ts">
  import { onMount } from 'svelte';
  import { writeConfiguredClipboardText } from '../../clipboard';
  import { generatePassword, type GeneratedPassword } from '../../ipc';
  import { uiState } from '../../stores/ui.svelte';
  import { clearEntryDraft, setEntryDraft, vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, Entropy, IconButton, Slider, Toggle } from '../primitives';

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
  const entropyBits = $derived(generated ? Math.round(generated.entropyBits) : 0);
  const strengthLabel = $derived(strengthFromEntropy(entropyBits));
  const entropyFilled = $derived(generated ? Math.max(1, Math.min(16, Math.round(entropyBits / 8))) : 0);
  const metaEntropy = $derived(generated ? `${entropyBits} bits` : 'pending');
  const displayedPassword = $derived(
    generated ? (isRevealed ? generated.password : '•'.repeat(Math.max(generated.password.length, 12))) : '',
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
      isRevealed = true;
    } catch (error) {
      if (currentGeneration !== generationCounter) {
        return;
      }

      generated = null;
      errorMessage = messageFromError(error);
    } finally {
      if (currentGeneration === generationCounter) {
        busy = false;
      }
    }
  }

  async function copyGenerated() {
    if (busy || !generated?.password) {
      return;
    }

    copied = await writeConfiguredClipboardText(generated.password);

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    copyTimer = setTimeout(() => {
      copied = false;
      copyTimer = null;
    }, 1400);
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
    if (!generated?.password) {
      return;
    }

    isRevealed = !isRevealed;
  }

  function useInNewEntry() {
    if (!generated?.password) {
      return;
    }

    setEntryDraft({ password: generated.password });
    vaultState.selectedEntry = null;
    uiState.view = 'edit';
  }

  function openBlankEntry() {
    clearEntryDraft();
    vaultState.selectedEntry = null;
    uiState.view = 'edit';
  }

  function renderParts(value: string) {
    return Array.from(value).map((character) => ({
      character,
      tone: /\d/.test(character) ? 'c-num' : /[^a-zA-Z0-9]/.test(character) ? 'c-sym' : '',
    }));
  }

  function strengthFromEntropy(entropy: number): string {
    if (!generated) {
      return 'pending';
    }

    if (entropy >= 120) {
      return 'excellent';
    }

    if (entropy >= 80) {
      return 'strong';
    }

    if (entropy >= 50) {
      return 'fair';
    }

    return 'weak';
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to generate password';
  }
</script>

<section class="page generator-page" aria-labelledby="generator-title">
  <div class="page__head">
    <span class="page__hash mono">#</span>
    <h1 id="generator-title" class="page__title">generate<em>.</em></h1>
    <div class="page__meta mono">
      <span>vault · <b>{vaultState.vaultName || 'vault.local'}</b></span>
      <span>entropy · <b>{metaEntropy}</b></span>
    </div>
  </div>

  <div class="gen">
    <div class="gen__output">
      <div class="gen__pw" aria-live="polite">
        {#if generated}
          {#if isRevealed}
            {#each renderParts(displayedPassword) as part, index (`${part.character}-${index}`)}
              <span class={part.tone}>{part.character}</span>
            {/each}
          {:else}
            {displayedPassword}
          {/if}
        {:else}
          <span class="gen__pw-placeholder">press generate to mint a fresh password.</span>
        {/if}
      </div>

      <div class="gen__pw-foot">
        <Entropy filled={entropyFilled} bits={entropyBits} strength={strengthLabel} />
        <span class="status__spacer"></span>
        <Button variant="ghost" size="sm" onclick={generate} disabled={busy || !hasCharacterSet}>
          <Icon name="refresh" size={12} />
          {busy ? 'generating' : generated ? 'regenerate' : 'generate'}
        </Button>
        <IconButton
          label={isRevealed ? 'Hide generated password' : 'Reveal generated password'}
          onclick={toggleReveal}
          disabled={!generated?.password}
        >
          <Icon name="eye" size={13} />
        </IconButton>
        <Button variant={copied ? 'vault' : 'primary'} size="sm" onclick={copyGenerated} disabled={!generated?.password}>
          {#if copied}
            copied
          {:else}
            <Icon name="copy" size={12} />
            copy
          {/if}
        </Button>
      </div>

      {#if errorMessage}
        <div class="generator-error mono error" role="alert">{errorMessage}</div>
      {/if}
    </div>

    <div class="gen__controls">
      <div class="gen__ctrl">
        <div class="gen__ctrl-k">length<small>characters</small></div>
        <Slider
          value={length}
          min={8}
          max={48}
          step={1}
          ariaLabel="Password length"
          valueLabel={String(length)}
          oninput={(next) => {
            length = next;
            resetGenerated();
          }}
        />
      </div>

      <div class="gen__ctrl">
        <div class="gen__ctrl-k">uppercase<small>A–Z</small></div>
        <div></div>
        <Toggle
          label="Uppercase letters"
          checked={uppercase}
          ontoggle={(next) => {
            uppercase = next;
            resetGenerated();
          }}
        />
      </div>

      <div class="gen__ctrl">
        <div class="gen__ctrl-k">lowercase<small>a–z</small></div>
        <div></div>
        <Toggle
          label="Lowercase letters"
          checked={lowercase}
          ontoggle={(next) => {
            lowercase = next;
            resetGenerated();
          }}
        />
      </div>

      <div class="gen__ctrl">
        <div class="gen__ctrl-k">numbers<small>0–9</small></div>
        <div></div>
        <Toggle
          label="Numbers"
          checked={digits}
          ontoggle={(next) => {
            digits = next;
            resetGenerated();
          }}
        />
      </div>

      <div class="gen__ctrl">
        <div class="gen__ctrl-k">symbols<small>!@#$</small></div>
        <div></div>
        <Toggle
          label="Symbols"
          checked={symbols}
          ontoggle={(next) => {
            symbols = next;
            resetGenerated();
          }}
        />
      </div>

      <div class="gen__ctrl">
        <div class="gen__ctrl-k">exclude ambiguous<small>0 O 1 l I</small></div>
        <div></div>
        <Toggle
          label="Exclude ambiguous characters"
          checked={excludeAmbiguous}
          variant="vault"
          ontoggle={(next) => {
            excludeAmbiguous = next;
            resetGenerated();
          }}
        />
      </div>
    </div>

    <div class="gen__actions">
      <Button variant="primary" onclick={useInNewEntry} disabled={!generated?.password}>
        <Icon name="plus" size={11} sw={2} />
        use in new entry
      </Button>
      <Button variant="bare" onclick={openBlankEntry}>blank entry</Button>
    </div>
  </div>
</section>
