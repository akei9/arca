<script lang="ts">
  let {
    filled = 13,
    segments = 16,
    bits = 94,
    strength = 'strong',
    class: className = '',
    ...rest
  } = $props<{
    filled?: number;
    segments?: number;
    bits?: number;
    strength?: string;
    class?: string;
    [key: string]: unknown;
  }>();

  const classes = $derived(['entropy', className].filter(Boolean).join(' '));
  const segmentCount = $derived(Math.max(0, Math.floor(segments)));
  const filledCount = $derived(Math.max(0, Math.min(Math.floor(filled), segmentCount)));
  const segmentClasses = $derived(
    Array.from({ length: segmentCount }, (_, index) => {
      if (index >= filledCount) {
        return '';
      }

      if (index < 5) {
        return ' entropy__seg--w';
      }

      if (index < 10) {
        return ' entropy__seg--m';
      }

      return ' entropy__seg--s';
    }),
  );
</script>

<div {...rest} class={classes}>
  <div class="entropy__bar" aria-hidden="true">
    {#each segmentClasses as segmentClass}
      <div class={`entropy__seg${segmentClass}`}></div>
    {/each}
  </div>
  <span class="entropy__bits">{bits} bits</span>
  <span class="entropy__cls">{strength}</span>
</div>
