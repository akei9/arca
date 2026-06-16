<script lang="ts">
  let {
    value,
    min = 0,
    max = 100,
    step = 1,
    ariaLabel,
    valueLabel,
    oninput,
    onchange,
    class: className = '',
    ...rest
  } = $props<{
    value: number;
    min?: number;
    max?: number;
    step?: number;
    ariaLabel?: string;
    valueLabel?: string;
    oninput?: (value: number) => void;
    onchange?: (value: number) => void;
    class?: string;
    [key: string]: unknown;
  }>();

  const classes = $derived(['slider', className].filter(Boolean).join(' '));
  const percent = $derived(max <= min ? 0 : ((value - min) / (max - min)) * 100);

  function readValue(raw: string): number {
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : value;
  }
</script>

<div {...rest} class={classes}>
  <div class="slider__track">
    <div class="slider__fill" style:width={`${percent}%`}></div>
    <div class="slider__knob" style:left={`${percent}%`}></div>
    <input
      class="slider__native"
      type="range"
      aria-label={ariaLabel}
      {min}
      {max}
      {step}
      {value}
      oninput={(event) => oninput?.(readValue(event.currentTarget.value))}
      onchange={(event) => onchange?.(readValue(event.currentTarget.value))}
    />
  </div>
  <div class="slider__val">{valueLabel ?? value}</div>
</div>
