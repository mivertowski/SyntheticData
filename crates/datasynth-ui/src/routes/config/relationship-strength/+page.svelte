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

  function getWeightsTotal(): number {
    if (!$config?.relationship_strength?.calculation) return 0;
    const c = $config.relationship_strength.calculation;
    return (
      c.transaction_volume_weight +
      c.transaction_count_weight +
      c.relationship_duration_weight +
      c.recency_weight +
      c.mutual_connections_weight
    );
  }
</script>

<div class="page">
  <ConfigPageHeader title="Relationship Strength" description="Configure how entity relationship strength is calculated" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Relationship Settings" description="Enable relationship strength calculation for entity graphs">
        {#snippet children()}
          <Toggle
            bind:checked={$config.relationship_strength.enabled}
            label="Enable Relationship Strength"
            description="Calculate weighted strength scores for entity-to-entity relationships"
          />
        {/snippet}
      </FormSection>

      {#if $config.relationship_strength.enabled}
        <FormSection title="Calculation Weights" description="Relative importance of each factor in strength calculation (should sum to 100%)">
          {#snippet children()}
            <div class="distribution-grid">
              <div class="distribution-item">
                <label>Transaction Volume</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.relationship_strength.calculation.transaction_volume_weight}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.relationship_strength.calculation.transaction_volume_weight * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Transaction Count</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.relationship_strength.calculation.transaction_count_weight}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.relationship_strength.calculation.transaction_count_weight * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Relationship Duration</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.relationship_strength.calculation.relationship_duration_weight}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.relationship_strength.calculation.relationship_duration_weight * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Recency</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.relationship_strength.calculation.recency_weight}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.relationship_strength.calculation.recency_weight * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Mutual Connections</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.relationship_strength.calculation.mutual_connections_weight}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.relationship_strength.calculation.mutual_connections_weight * 100).toFixed(0)}%</span>
                </div>
              </div>
            </div>

            <div class="distribution-total" class:warning={Math.abs(getWeightsTotal() - 1.0) > 0.01}>
              Total: {(getWeightsTotal() * 100).toFixed(0)}%
              {#if Math.abs(getWeightsTotal() - 1.0) > 0.01}
                <span class="warning-text">(should sum to 100%)</span>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Recency Decay" description="Configure how quickly recent interactions lose relevance">
          {#snippet children()}
            <FormGroup
              label="Recency Half-Life"
              htmlFor="half-life"
              helpText="Number of days after which recency score drops to 50% (exponential decay)"
              error={getError('relationship_strength.calculation.recency_half_life_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="half-life"
                    bind:value={$config.relationship_strength.calculation.recency_half_life_days}
                    min="1"
                    max="365"
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          {/snippet}
        </FormSection>

        <FormSection title="Strength Thresholds" description="Classification boundaries for relationship strength levels">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Strong Threshold"
                htmlFor="threshold-strong"
                helpText="Minimum score for a strong relationship (0-1)"
                error={getError('relationship_strength.thresholds.strong')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="threshold-strong"
                      bind:value={$config.relationship_strength.thresholds.strong}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{$config.relationship_strength.thresholds.strong.toFixed(2)}</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Moderate Threshold"
                htmlFor="threshold-moderate"
                helpText="Minimum score for a moderate relationship (0-1)"
                error={getError('relationship_strength.thresholds.moderate')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="threshold-moderate"
                      bind:value={$config.relationship_strength.thresholds.moderate}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{$config.relationship_strength.thresholds.moderate.toFixed(2)}</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Weak Threshold"
                htmlFor="threshold-weak"
                helpText="Minimum score for a weak relationship (below this is negligible)"
                error={getError('relationship_strength.thresholds.weak')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="threshold-weak"
                      bind:value={$config.relationship_strength.thresholds.weak}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{$config.relationship_strength.thresholds.weak.toFixed(2)}</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Calculation Weights</h4>
          <p>Five configurable factors determine relationship strength: transaction volume (log scale), transaction count (sqrt scale), relationship duration, recency (exponential decay), and mutual connections (Jaccard index).</p>
        </div>
        <div class="info-card">
          <h4>Recency Decay</h4>
          <p>Recent interactions contribute more to strength scores using exponential decay. The half-life parameter (default 90 days) controls how quickly past interactions lose relevance.</p>
        </div>
        <div class="info-card">
          <h4>Strength Thresholds</h4>
          <p>Relationships are classified into three levels: Strong (&gt;0.7), Moderate (0.4-0.7), and Weak (&lt;0.4). Scores below the weak threshold are considered negligible.</p>
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
  .warning-text { font-family: var(--font-sans); margin-left: var(--space-2); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid { grid-template-columns: 1fr; } }
</style>
