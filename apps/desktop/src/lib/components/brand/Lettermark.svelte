<script lang="ts">
  let {
    size = 22,
    color = 'currentColor',
    title,
    class: className = '',
    ...rest
  } = $props<{
    size?: number;
    color?: string;
    title?: string;
    class?: string;
    [key: string]: unknown;
  }>();

  const width = $derived(size * (540 / 700));
  const path =
    'M 390 140 L 540 140 L 540 380 A 270 320 0 0 1 0 380 L 390 380 Z ' +
    'M 110 490 L 390 490 A 140 130 0 0 1 110 490 Z';
</script>

<svg
  {...rest}
  class={className}
  width={width}
  height={size}
  viewBox="0 0 540 700"
  role={title ? 'img' : undefined}
  aria-hidden={title ? undefined : true}
  style:display="block"
  style:flex-shrink="0"
>
  {#if title}
    <title>{title}</title>
  {/if}
  <path fill={color} fill-rule="evenodd" d={path} />
</svg>
