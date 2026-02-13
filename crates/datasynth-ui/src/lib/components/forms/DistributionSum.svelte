<script lang="ts">
  let {
    total,
    target = 1.0,
    tolerance = 0.01,
    label = 'Total',
    suffix = '%',
    multiplier = 100,
    decimals = 0,
  }: {
    total: number;
    target?: number;
    tolerance?: number;
    label?: string;
    suffix?: string;
    multiplier?: number;
    decimals?: number;
  } = $props();

  let isValid = $derived(Math.abs(total - target) <= tolerance);
</script>

<div class="distribution-sum" class:warning={!isValid}>
  <span class="sum-label">{label}:</span>
  <span class="sum-value">{(total * multiplier).toFixed(decimals)}{suffix}</span>
  {#if !isValid}
    <span class="sum-warning">(should sum to {(target * multiplier).toFixed(decimals)}{suffix})</span>
  {/if}
</div>

<style>
  .distribution-sum {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    background-color: var(--color-background);
  }

  .distribution-sum.warning {
    background-color: var(--color-warning-bg, rgba(234, 179, 8, 0.1));
    border: 1px solid var(--color-warning, #eab308);
  }

  .sum-label {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .sum-value {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .sum-warning {
    color: var(--color-warning, #eab308);
    font-size: 0.75rem;
  }
</style>
