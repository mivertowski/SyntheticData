<script lang="ts">
  let {
    value = $bindable(0),
    min = 0,
    max = 1,
    step = 0.01,
    suffix = '%',
    disabled = false,
    label = '',
    multiplier = 100,
    decimals = 0,
  }: {
    value: number;
    min?: number;
    max?: number;
    step?: number;
    suffix?: string;
    disabled?: boolean;
    label?: string;
    multiplier?: number;
    decimals?: number;
  } = $props();
</script>

<div class="slider-with-value">
  {#if label}
    <span class="slider-label">{label}</span>
  {/if}
  <input
    type="range"
    bind:value
    {min}
    {max}
    {step}
    {disabled}
  />
  <span class="slider-value">{(value * multiplier).toFixed(decimals)}{suffix}</span>
</div>

<style>
  .slider-with-value {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .slider-label {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
    min-width: 120px;
  }

  input[type='range'] {
    flex: 1;
    min-width: 80px;
  }

  .slider-value {
    font-size: 0.8125rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    min-width: 48px;
    text-align: right;
  }
</style>
