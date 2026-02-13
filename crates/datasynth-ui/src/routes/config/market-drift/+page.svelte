<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
  const validationErrors = configStore.validationErrors;

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }
</script>

<div class="page">
  <ConfigPageHeader title="Market Drift" description="Configure macroeconomic and market condition changes over time" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Market Drift Settings" description="Enable macroeconomic simulation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.market_drift.enabled}
              label="Enable Market Drift"
              description="Simulate macroeconomic and market condition changes that affect generated data"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.market_drift.enabled}
        <FormSection title="Economic Cycles" description="Configure cyclical macroeconomic patterns">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.market_drift.economic_cycle_enabled}
                label="Enable Economic Cycles"
                description="Add sinusoidal economic cycle patterns to volume and amount distributions"
              />

              {#if $config.market_drift.economic_cycle_enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Cycle Period (months)"
                    htmlFor="cycle-period"
                    helpText="Length of one complete economic cycle in months"
                    error={getError('market_drift.cycle_period_months')}
                  >
                    {#snippet children()}
                      <input
                        type="number"
                        id="cycle-period"
                        bind:value={$config.market_drift.cycle_period_months}
                        min="6"
                        max="120"
                      />
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Amplitude"
                    htmlFor="cycle-amplitude"
                    helpText="Strength of cyclical variation (0-1)"
                    error={getError('market_drift.amplitude')}
                  >
                    {#snippet children()}
                      <div class="slider-with-value">
                        <input
                          type="range"
                          id="cycle-amplitude"
                          bind:value={$config.market_drift.amplitude}
                          min="0"
                          max="1"
                          step="0.05"
                        />
                        <span>{($config.market_drift.amplitude * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Commodity & Industry" description="Configure commodity price and industry cycle effects">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Commodity Price Drift"
                htmlFor="commodity-drift"
                helpText="Rate of commodity price change per period (0-0.2)"
                error={getError('market_drift.commodity_price_drift')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="commodity-drift"
                      bind:value={$config.market_drift.commodity_price_drift}
                      min="0"
                      max="0.2"
                      step="0.005"
                    />
                    <span>{($config.market_drift.commodity_price_drift * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <Toggle
                bind:checked={$config.market_drift.industry_cycle_enabled}
                label="Enable Industry Cycles"
                description="Apply industry-specific cyclical patterns on top of general economic cycles"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Recession Modeling" description="Configure recession probability and impact">
          {#snippet children()}
            <div class="form-stack">
              <div class="form-grid">
                <FormGroup
                  label="Recession Probability"
                  htmlFor="recession-prob"
                  helpText="Probability of a recession occurring in any given period (0-1)"
                  error={getError('market_drift.recession_probability')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="recession-prob"
                        bind:value={$config.market_drift.recession_probability}
                        min="0"
                        max="1"
                        step="0.01"
                      />
                      <span>{($config.market_drift.recession_probability * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Recession Depth"
                  htmlFor="recession-depth"
                  helpText="Magnitude of recession impact on volumes and amounts (0-1)"
                  error={getError('market_drift.recession_depth')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="recession-depth"
                        bind:value={$config.market_drift.recession_depth}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.market_drift.recession_depth * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Economic Cycles</h4>
          <p>Sinusoidal patterns that model expansion and contraction phases, affecting transaction volumes, amounts, and entity activity levels over time.</p>
        </div>
        <div class="info-card">
          <h4>Commodity Prices</h4>
          <p>Gradual drift in commodity prices that affects material costs, purchase order amounts, and inventory valuations across the supply chain.</p>
        </div>
        <div class="info-card">
          <h4>Industry Cycles</h4>
          <p>Industry-specific cyclical patterns layered on top of general economic cycles, capturing sector-specific seasonality and trends.</p>
        </div>
        <div class="info-card">
          <h4>Recession Impact</h4>
          <p>Models recession events with configurable probability and depth, reducing transaction volumes and shifting payment patterns during downturns.</p>
        </div>
      </div>
    </div>
  {:else}
    <div class="loading">
      <p>Loading configuration...</p>
    </div>
  {/if}
</div>

<style>
  .page { max-width: 960px; }
  .page-content { display: flex; flex-direction: column; gap: var(--space-5); }
  .form-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-4); }
  .form-stack { display: flex; flex-direction: column; gap: var(--space-4); }
  .input-with-suffix { display: flex; align-items: center; gap: var(--space-2); }
  .suffix { font-size: 0.8125rem; color: var(--color-text-muted); font-family: var(--font-mono); }
  .distribution-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-4); }
  .distribution-item { display: flex; flex-direction: column; gap: var(--space-1); }
  .distribution-item label { font-size: 0.8125rem; font-weight: 500; color: var(--color-text-secondary); }
  .slider-with-value { display: flex; align-items: center; gap: var(--space-2); }
  .slider-with-value input[type='range'] { flex: 1; }
  .slider-with-value span { font-size: 0.8125rem; font-family: var(--font-mono); min-width: 44px; text-align: right; }
  .distribution-total { padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); font-size: 0.8125rem; background-color: var(--color-background); }
  .distribution-total.warning { background-color: rgba(234, 179, 8, 0.1); border: 1px solid #eab308; }
  .info-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .event-list { display: flex; flex-direction: column; gap: var(--space-3); }
  .event-item { display: grid; grid-template-columns: 1fr 1fr 2fr 1fr 1fr auto; gap: var(--space-2); align-items: center; padding: var(--space-3); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .event-item input, .event-item select { font-size: 0.8125rem; }
  .btn-danger { background-color: var(--color-error, #ef4444); color: white; border: none; padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); cursor: pointer; font-size: 0.75rem; }
  .btn-outline { background: none; border: 1px solid var(--color-border); padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); cursor: pointer; font-size: 0.8125rem; color: var(--color-text-secondary); }
  .btn-outline:hover { background-color: var(--color-background); color: var(--color-text-primary); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid { grid-template-columns: 1fr; } .event-item { grid-template-columns: 1fr; } }
</style>
