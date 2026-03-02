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

  const DIFFICULTY_LEVELS = [
    { value: 'easy', label: 'Easy', description: 'Obvious anomalies for baseline model training' },
    { value: 'medium', label: 'Medium', description: 'Moderate difficulty with some subtle patterns' },
    { value: 'hard', label: 'Hard', description: 'Subtle anomalies requiring advanced detection' },
    { value: 'expert', label: 'Expert', description: 'Near-invisible anomalies for expert-level models' },
  ];
</script>

<div class="page">
  <ConfigPageHeader title="Anomaly Injection" description="Configure fraud and anomaly injection patterns for ML training data" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Anomaly Injection Settings" description="Enable and configure anomaly injection for ML training">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.anomaly_injection.enabled}
              label="Enable Anomaly Injection"
              description="Inject labeled anomalies into generated data for supervised ML training"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.anomaly_injection.enabled}
        <FormSection title="Base Configuration" description="Core anomaly injection parameters">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Base Anomaly Rate"
                htmlFor="base-rate"
                helpText="Overall proportion of records that contain anomalies (0-0.1)"
                error={getError('anomaly_injection.base_rate')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="base-rate"
                      bind:value={$config.anomaly_injection.base_rate}
                      min="0"
                      max="0.1"
                      step="0.005"
                    />
                    <span>{($config.anomaly_injection.base_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Difficulty Level"
                htmlFor="difficulty-level"
                helpText="How subtle the injected anomalies should be"
              >
                {#snippet children()}
                  <div class="difficulty-selector">
                    {#each DIFFICULTY_LEVELS as level}
                      <label class="difficulty-option" class:selected={$config.anomaly_injection.difficulty_level === level.value}>
                        <input
                          type="radio"
                          name="difficulty-level"
                          value={level.value}
                          bind:group={$config.anomaly_injection.difficulty_level}
                        />
                        <span class="difficulty-label">{level.label}</span>
                        <span class="difficulty-desc">{level.description}</span>
                      </label>
                    {/each}
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Advanced Injection" description="Configure sophisticated anomaly patterns">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.anomaly_injection.multi_stage_schemes}
                label="Multi-Stage Schemes"
                description="Generate complex fraud schemes that span multiple transactions and time periods"
              />

              <Toggle
                bind:checked={$config.anomaly_injection.correlated_injection}
                label="Correlated Injection"
                description="Inject anomalies that are correlated across related records and entities"
              />

              <Toggle
                bind:checked={$config.anomaly_injection.near_miss_enabled}
                label="Near-Miss Anomalies"
                description="Generate borderline cases that are close to anomalous but technically within normal ranges"
              />

              {#if $config.anomaly_injection.near_miss_enabled}
                <FormGroup
                  label="Near-Miss Rate"
                  htmlFor="near-miss-rate"
                  helpText="Proportion of near-miss cases relative to true anomalies (0-1)"
                  error={getError('anomaly_injection.near_miss_rate')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="near-miss-rate"
                        bind:value={$config.anomaly_injection.near_miss_rate}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.anomaly_injection.near_miss_rate * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Quick Apply Fraud Packs" description="Quickly apply pre-configured fraud scenario packs">
          {#snippet children()}
            <div class="form-stack">
              <p class="hint-text">Click a pack to apply its settings. This enables anomaly injection and configures fraud patterns.</p>
              <div class="pack-buttons">
                <button class="pack-btn" onclick={() => {
                  if ($config) {
                    $config.anomaly_injection.enabled = true;
                    $config.anomaly_injection.base_rate = 0.03;
                    if (!$config.fraud_packs) $config.fraud_packs = { enabled: true, packs: [], fraud_rate_override: null };
                    $config.fraud_packs.enabled = true;
                    $config.fraud_packs.packs = ['revenue_fraud'];
                  }
                }}>Revenue Fraud</button>
                <button class="pack-btn" onclick={() => {
                  if ($config) {
                    $config.anomaly_injection.enabled = true;
                    $config.anomaly_injection.base_rate = 0.02;
                    if (!$config.fraud_packs) $config.fraud_packs = { enabled: true, packs: [], fraud_rate_override: null };
                    $config.fraud_packs.enabled = true;
                    $config.fraud_packs.packs = ['payroll_ghost'];
                  }
                }}>Payroll Ghost</button>
                <button class="pack-btn" onclick={() => {
                  if ($config) {
                    $config.anomaly_injection.enabled = true;
                    $config.anomaly_injection.base_rate = 0.025;
                    if (!$config.fraud_packs) $config.fraud_packs = { enabled: true, packs: [], fraud_rate_override: null };
                    $config.fraud_packs.enabled = true;
                    $config.fraud_packs.packs = ['vendor_kickback'];
                  }
                }}>Vendor Kickback</button>
                <button class="pack-btn" onclick={() => {
                  if ($config) {
                    $config.anomaly_injection.enabled = true;
                    $config.anomaly_injection.base_rate = 0.015;
                    if (!$config.fraud_packs) $config.fraud_packs = { enabled: true, packs: [], fraud_rate_override: null };
                    $config.fraud_packs.enabled = true;
                    $config.fraud_packs.packs = ['management_override'];
                  }
                }}>Management Override</button>
                <button class="pack-btn pack-btn-primary" onclick={() => {
                  if ($config) {
                    $config.anomaly_injection.enabled = true;
                    $config.anomaly_injection.base_rate = 0.05;
                    if (!$config.fraud_packs) $config.fraud_packs = { enabled: true, packs: [], fraud_rate_override: null };
                    $config.fraud_packs.enabled = true;
                    $config.fraud_packs.packs = ['comprehensive'];
                  }
                }}>Comprehensive</button>
              </div>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Fraud Patterns</h4>
          <p>Fictitious transactions, revenue manipulation, split transactions, round-tripping, ghost employees, and duplicate payments with full ground truth labels.</p>
        </div>
        <div class="info-card">
          <h4>Error Patterns</h4>
          <p>Duplicate entries, reversed amounts, wrong period postings, misclassified accounts, and missing references that mimic real accounting errors.</p>
        </div>
        <div class="info-card">
          <h4>Process Issues</h4>
          <p>Late postings, skipped approvals, threshold manipulation, and out-of-sequence documents that indicate control weaknesses.</p>
        </div>
        <div class="info-card">
          <h4>Statistical Anomalies</h4>
          <p>Unusual amounts, Benford violations, trend breaks, dormant account activity, and circular transactions for statistical detection models.</p>
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
  .difficulty-selector { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-2); }
  .difficulty-option { display: flex; flex-direction: column; padding: var(--space-3); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .difficulty-option:hover { border-color: var(--color-accent); }
  .difficulty-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .difficulty-option input { display: none; }
  .difficulty-label { font-weight: 600; font-size: 0.875rem; color: var(--color-text-primary); margin-bottom: var(--space-1); }
  .difficulty-desc { font-size: 0.75rem; color: var(--color-text-secondary); }
  .hint-text { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .pack-buttons { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .pack-btn { padding: var(--space-2) var(--space-3); border: 1px solid var(--color-border); border-radius: var(--radius-md); background-color: var(--color-surface); color: var(--color-text-primary); font-size: 0.8125rem; cursor: pointer; transition: all var(--transition-fast); }
  .pack-btn:hover { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .pack-btn-primary { background-color: var(--color-accent); color: white; border-color: var(--color-accent); }
  .pack-btn-primary:hover { background-color: var(--color-accent-hover, #0052cc); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid, .difficulty-selector { grid-template-columns: 1fr; } .event-item { grid-template-columns: 1fr; } }
</style>
