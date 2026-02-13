<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';
  import DistributionEditor from '$lib/components/forms/DistributionEditor.svelte';

  const config = configStore.config;

  // Line item distribution labels
  const lineItemLabels: Record<string, string> = {
    '2': '2 Lines (Two-Liner)',
    '3': '3 Lines',
    '4': '4 Lines',
    '5': '5 Lines',
    '6': '6 Lines',
    '7-9': '7-9 Lines',
    '10-99': '10+ Lines',
  };

  // Source distribution labels
  const sourceLabels: Record<string, string> = {
    manual: 'Manual Entry',
    interface: 'Interface/API',
    batch: 'Batch Upload',
    recurring: 'Recurring/Scheduled',
  };

  // Format number for display
  function formatAmount(value: number): string {
    if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`;
    if (value >= 1000) return `${(value / 1000).toFixed(0)}K`;
    return value.toFixed(2);
  }
</script>

<div class="page">
  <ConfigPageHeader title="Transaction Settings" description="Configure journal entry characteristics and distributions" />

  {#if $config}
    <div class="sections">
      <!-- Line Item Distribution -->
      <FormSection
        title="Line Item Distribution"
        description="Distribution of line counts per journal entry based on academic research"
      >
        <div class="section-content">
          <p class="section-intro">
            Real ERP data shows ~61% of journal entries have exactly 2 lines (two-liners),
            with an 88% preference for even line counts. This distribution is based on
            empirical research of actual GL data.
          </p>

          <DistributionEditor
            label="Line Count Distribution"
            bind:distribution={$config.transactions.line_item_distribution}
            labels={lineItemLabels}
            helpText="Adjust the slider or enter exact percentages. Values auto-normalize to 100%."
          />
        </div>
      </FormSection>

      <!-- Amount Distribution -->
      <FormSection
        title="Amount Distribution"
        description="Configure transaction amounts using log-normal distribution"
      >
        <div class="section-content">
          <div class="form-grid">
            <FormGroup
              label="Minimum Amount"
              htmlFor="min-amount"
              helpText="Smallest allowed transaction amount"
            >
              <input
                id="min-amount"
                type="number"
                min="0.01"
                step="0.01"
                bind:value={$config.transactions.amount_distribution.min_amount}
              />
            </FormGroup>

            <FormGroup
              label="Maximum Amount"
              htmlFor="max-amount"
              helpText="Largest allowed transaction amount"
            >
              <input
                id="max-amount"
                type="number"
                min="1"
                step="1000"
                bind:value={$config.transactions.amount_distribution.max_amount}
              />
            </FormGroup>
          </div>

          <div class="subsection">
            <h4>Log-Normal Parameters</h4>
            <p class="subsection-description">
              Transaction amounts follow a log-normal distribution, which is common in financial data.
              Higher mu shifts amounts larger, higher sigma increases spread.
            </p>
            <div class="form-grid">
              <FormGroup
                label="Mu (μ)"
                htmlFor="lognormal-mu"
                helpText="Location parameter (log scale). Default 7.0 ≈ median of ~$1,100"
              >
                <input
                  id="lognormal-mu"
                  type="number"
                  min="0"
                  max="20"
                  step="0.1"
                  bind:value={$config.transactions.amount_distribution.lognormal_mu}
                />
              </FormGroup>

              <FormGroup
                label="Sigma (σ)"
                htmlFor="lognormal-sigma"
                helpText="Scale parameter. Higher values = more spread. Default 2.5"
              >
                <input
                  id="lognormal-sigma"
                  type="number"
                  min="0.1"
                  max="5"
                  step="0.1"
                  bind:value={$config.transactions.amount_distribution.lognormal_sigma}
                />
              </FormGroup>
            </div>
          </div>

          <div class="subsection">
            <h4>Number Preferences</h4>
            <p class="subsection-description">
              Human-entered amounts tend to be round numbers. These settings add realistic bias.
            </p>
            <div class="form-grid">
              <FormGroup
                label="Round Number Probability"
                htmlFor="round-prob"
                helpText="Probability of amounts like $100, $1000, $5000"
              >
                <div class="input-with-unit">
                  <input
                    id="round-prob"
                    type="number"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={$config.transactions.amount_distribution.round_number_probability}
                  />
                  <span class="unit">{($config.transactions.amount_distribution.round_number_probability * 100).toFixed(0)}%</span>
                </div>
              </FormGroup>

              <FormGroup
                label="Nice Number Probability"
                htmlFor="nice-prob"
                helpText="Probability of amounts like $99.99, $499.95"
              >
                <div class="input-with-unit">
                  <input
                    id="nice-prob"
                    type="number"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={$config.transactions.amount_distribution.nice_number_probability}
                  />
                  <span class="unit">{($config.transactions.amount_distribution.nice_number_probability * 100).toFixed(0)}%</span>
                </div>
              </FormGroup>
            </div>
          </div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Benford's Law Compliance</span>
              <span class="toggle-description">
                First digits follow Benford's distribution P(d) = log₁₀(1 + 1/d).
                Required for realistic financial data and fraud detection testing.
              </span>
            </div>
            <Toggle bind:checked={$config.transactions.amount_distribution.benford_compliance} />
          </div>
        </div>
      </FormSection>

      <!-- Source Distribution -->
      <FormSection
        title="Transaction Source Distribution"
        description="How transactions enter the system"
      >
        <div class="section-content">
          <p class="section-intro">
            Different transaction sources have different error profiles. Manual entries
            have more human errors, while interface/batch sources may have systematic issues.
          </p>

          <DistributionEditor
            label="Source Type Distribution"
            bind:distribution={$config.transactions.source_distribution}
            labels={sourceLabels}
            helpText="Typical enterprise: 10% manual, 30% interface, 40% batch, 20% recurring"
          />
        </div>
      </FormSection>

      <!-- Seasonality -->
      <FormSection
        title="Seasonality Patterns"
        description="Configure period-end volume spikes"
      >
        <div class="section-content">
          <p class="section-intro">
            Financial activity increases at period boundaries. Month-end, quarter-end,
            and year-end show predictable volume spikes due to closing activities.
          </p>

          <div class="seasonality-settings">
            <div class="seasonality-row">
              <div class="toggle-row">
                <div class="toggle-info">
                  <span class="toggle-label">Month-End Spike</span>
                  <span class="toggle-description">Increase volume in last 3 days of each month</span>
                </div>
                <Toggle bind:checked={$config.transactions.seasonality.month_end_spike} />
              </div>
              {#if $config.transactions.seasonality.month_end_spike}
                <FormGroup
                  label="Multiplier"
                  htmlFor="month-multiplier"
                >
                  <div class="input-with-unit">
                    <input
                      id="month-multiplier"
                      type="number"
                      min="1"
                      max="10"
                      step="0.1"
                      bind:value={$config.transactions.seasonality.month_end_multiplier}
                    />
                    <span class="unit">×</span>
                  </div>
                </FormGroup>
              {/if}
            </div>

            <div class="seasonality-row">
              <div class="toggle-row">
                <div class="toggle-info">
                  <span class="toggle-label">Quarter-End Spike</span>
                  <span class="toggle-description">Additional increase for Mar, Jun, Sep, Dec</span>
                </div>
                <Toggle bind:checked={$config.transactions.seasonality.quarter_end_spike} />
              </div>
              {#if $config.transactions.seasonality.quarter_end_spike}
                <FormGroup
                  label="Multiplier"
                  htmlFor="quarter-multiplier"
                >
                  <div class="input-with-unit">
                    <input
                      id="quarter-multiplier"
                      type="number"
                      min="1"
                      max="15"
                      step="0.1"
                      bind:value={$config.transactions.seasonality.quarter_end_multiplier}
                    />
                    <span class="unit">×</span>
                  </div>
                </FormGroup>
              {/if}
            </div>

            <div class="seasonality-row">
              <div class="toggle-row">
                <div class="toggle-info">
                  <span class="toggle-label">Year-End Spike</span>
                  <span class="toggle-description">Highest volume for annual close (December)</span>
                </div>
                <Toggle bind:checked={$config.transactions.seasonality.year_end_spike} />
              </div>
              {#if $config.transactions.seasonality.year_end_spike}
                <FormGroup
                  label="Multiplier"
                  htmlFor="year-multiplier"
                >
                  <div class="input-with-unit">
                    <input
                      id="year-multiplier"
                      type="number"
                      min="1"
                      max="20"
                      step="0.1"
                      bind:value={$config.transactions.seasonality.year_end_multiplier}
                    />
                    <span class="unit">×</span>
                  </div>
                </FormGroup>
              {/if}
            </div>

            <div class="toggle-row standalone">
              <div class="toggle-info">
                <span class="toggle-label">Day of Week Patterns</span>
                <span class="toggle-description">
                  Weekday activity patterns (more on Mon-Thu, less on Fri, minimal on weekends)
                </span>
              </div>
              <Toggle bind:checked={$config.transactions.seasonality.day_of_week_patterns} />
            </div>
          </div>
        </div>
      </FormSection>
    </div>
  {:else}
    <div class="loading">Loading configuration...</div>
  {/if}
</div>

<style>
  .page {
    max-width: 900px;
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .section-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .section-intro {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-4);
  }

  .subsection {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .subsection h4 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .subsection-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .input-with-unit {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .input-with-unit input {
    flex: 1;
  }

  .unit {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    min-width: 40px;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .toggle-row.standalone {
    margin-top: var(--space-2);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .toggle-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .toggle-description {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .seasonality-settings {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .seasonality-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .seasonality-row .toggle-row {
    padding: 0;
    background: none;
  }

  .seasonality-row :global(.form-group) {
    max-width: 200px;
    margin-left: auto;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  input[type="number"] {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }

  input[type="number"]:focus {
    outline: none;
    border-color: var(--color-accent);
  }
</style>
