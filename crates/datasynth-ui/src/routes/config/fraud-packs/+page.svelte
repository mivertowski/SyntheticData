<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle, RateSlider } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const FRAUD_PACKS = [
    {
      id: 'revenue_fraud',
      label: 'Revenue Fraud',
      description: 'Revenue manipulation, fictitious sales, channel stuffing, and premature revenue recognition patterns.',
    },
    {
      id: 'payroll_ghost',
      label: 'Payroll Ghost Employees',
      description: 'Ghost employees, phantom payroll entries, timesheet manipulation, and unauthorized pay changes.',
    },
    {
      id: 'vendor_kickback',
      label: 'Vendor Kickback',
      description: 'Vendor kickbacks, shell companies, inflated invoices, bid rigging, and duplicate payments.',
    },
    {
      id: 'management_override',
      label: 'Management Override',
      description: 'Override of controls, unauthorized journal entries, period-end adjustments, and SOD violations.',
    },
    {
      id: 'comprehensive',
      label: 'Comprehensive',
      description: 'All fraud patterns combined at calibrated rates for complete fraud detection model training.',
    },
  ];

  function togglePack(packId: string) {
    if (!$config) return;
    if (!$config.fraud_packs) {
      $config.fraud_packs = { enabled: true, packs: [], fraud_rate_override: null };
    }
    const packs = $config.fraud_packs.packs || [];
    const idx = packs.indexOf(packId);
    if (idx >= 0) {
      packs.splice(idx, 1);
    } else {
      packs.push(packId);
    }
    $config.fraud_packs.packs = [...packs];
  }

  function isPackEnabled(packId: string): boolean {
    return $config?.fraud_packs?.packs?.includes(packId) ?? false;
  }
</script>

<div class="page">
  <ConfigPageHeader
    title="Fraud Scenario Packs"
    description="Pre-configured fraud pattern bundles for ML training and audit testing"
  />

  {#if $config}
    <div class="page-content">
      <FormSection title="Fraud Packs" description="Select one or more fraud scenario packs to apply to your generated data">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              checked={$config.fraud_packs?.enabled ?? false}
              label="Enable Fraud Packs"
              description="Apply fraud scenario packs on top of standard generation"
              onchange={() => {
                if (!$config.fraud_packs) {
                  $config.fraud_packs = { enabled: true, packs: [], fraud_rate_override: null };
                } else {
                  $config.fraud_packs.enabled = !$config.fraud_packs.enabled;
                }
              }}
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.fraud_packs?.enabled}
        <FormSection title="Select Packs" description="Choose which fraud pattern bundles to inject">
          {#snippet children()}
            <div class="pack-grid">
              {#each FRAUD_PACKS as pack}
                <label class="pack-card" class:selected={isPackEnabled(pack.id)}>
                  <input
                    type="checkbox"
                    checked={isPackEnabled(pack.id)}
                    onchange={() => togglePack(pack.id)}
                  />
                  <div class="pack-content">
                    <span class="pack-label">{pack.label}</span>
                    <span class="pack-desc">{pack.description}</span>
                  </div>
                </label>
              {/each}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Rate Override" description="Optionally override the base fraud rate for all selected packs">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Fraud Rate Override"
                htmlFor="fraud-rate"
                helpText="Overall fraud rate applied to selected packs (0-100%)"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="fraud-rate"
                      bind:value={$config.fraud_packs.fraud_rate_override}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span class="mono">{(($config.fraud_packs.fraud_rate_override ?? 0) * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}
    </div>
  {/if}
</div>

<style>
  .page { max-width: 960px; }
  .page-content { display: flex; flex-direction: column; gap: var(--space-5); }
  .form-stack { display: flex; flex-direction: column; gap: var(--space-4); }
  .pack-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: var(--space-3); }
  .pack-card { display: flex; gap: var(--space-3); padding: var(--space-4); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .pack-card:hover { border-color: var(--color-accent); }
  .pack-card.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .pack-card input[type="checkbox"] { flex-shrink: 0; margin-top: 2px; }
  .pack-content { display: flex; flex-direction: column; gap: var(--space-1); }
  .pack-label { font-weight: 600; font-size: 0.875rem; color: var(--color-text-primary); }
  .pack-desc { font-size: 0.75rem; color: var(--color-text-secondary); line-height: 1.4; }
  .slider-with-value { display: flex; align-items: center; gap: var(--space-2); }
  .slider-with-value input[type="range"] { flex: 1; }
  .slider-with-value span { font-size: 0.8125rem; font-family: var(--font-mono); min-width: 44px; text-align: right; }
  .mono { font-family: var(--font-mono); }
</style>
