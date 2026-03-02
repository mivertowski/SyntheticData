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

  const QUALITY_LEVELS = [
    { value: 'none', label: 'None', description: 'No quality validation applied' },
    { value: 'lenient', label: 'Lenient', description: 'Loose thresholds, warnings only' },
    { value: 'default', label: 'Default', description: 'Standard validation thresholds' },
    { value: 'strict', label: 'Strict', description: 'Tight thresholds for production data' },
  ];
</script>

<div class="page">
  <ConfigPageHeader title="Quality Gates" description="Configure validation quality thresholds for generated data" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Quality Gate Settings" description="Enable and configure data quality validation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.quality_gates.enabled}
              label="Enable Quality Gates"
              description="Validate generated data against configurable quality thresholds"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.quality_gates.enabled}
        <FormSection title="Validation Level" description="Choose a preset validation strictness level">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Quality Level"
                htmlFor="quality-level"
                helpText="Preset validation strictness level"
              >
                {#snippet children()}
                  <div class="level-selector">
                    {#each QUALITY_LEVELS as level}
                      <label class="level-option" class:selected={$config.quality_gates.level === level.value}>
                        <input
                          type="radio"
                          name="quality-level"
                          value={level.value}
                          bind:group={$config.quality_gates.level}
                        />
                        <span class="level-label">{level.label}</span>
                        <span class="level-desc">{level.description}</span>
                      </label>
                    {/each}
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Threshold Configuration" description="Fine-tune individual quality thresholds">
          {#snippet children()}
            <div class="form-stack">
              <div class="form-grid">
                <FormGroup
                  label="Benford Threshold (MAD)"
                  htmlFor="benford-threshold"
                  helpText="Maximum acceptable Mean Absolute Deviation from Benford's Law (0-0.1)"
                  error={getError('quality_gates.benford_threshold')}
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input
                        type="number"
                        id="benford-threshold"
                        bind:value={$config.quality_gates.benford_threshold}
                        min="0"
                        max="0.1"
                        step="0.001"
                      />
                      <span class="suffix">{($config.quality_gates.benford_threshold * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Balance Tolerance"
                  htmlFor="balance-tolerance"
                  helpText="Maximum allowable balance discrepancy (0-0.1)"
                  error={getError('quality_gates.balance_tolerance')}
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input
                        type="number"
                        id="balance-tolerance"
                        bind:value={$config.quality_gates.balance_tolerance}
                        min="0"
                        max="0.1"
                        step="0.001"
                      />
                      <span class="suffix">{($config.quality_gates.balance_tolerance * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Completeness Threshold"
                  htmlFor="completeness-threshold"
                  helpText="Minimum data completeness ratio required (0-1)"
                  error={getError('quality_gates.completeness_threshold')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="completeness-threshold"
                        bind:value={$config.quality_gates.completeness_threshold}
                        min="0"
                        max="1"
                        step="0.01"
                      />
                      <span>{($config.quality_gates.completeness_threshold * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>

              <Toggle
                bind:checked={$config.quality_gates.fail_on_violation}
                label="Fail on Violation"
                description="Stop generation if any quality gate threshold is exceeded (otherwise warn only)"
              />
            </div>
          {/snippet}
        </FormSection>
        <FormSection title="v0.11 Evaluator Thresholds" description="Thresholds for new evaluation dimensions">
          {#snippet children()}
            <div class="form-stack">
              <div class="form-grid">
                <FormGroup
                  label="Multi-Period Coherence"
                  htmlFor="multi-period-coherence"
                  helpText="Minimum acceptable coherence score across periods (0-1)"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="multi-period-coherence"
                        bind:value={$config.quality_gates.multi_period_coherence}
                        min="0"
                        max="1"
                        step="0.01"
                      />
                      <span>{(($config.quality_gates.multi_period_coherence ?? 0.99) * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="OCEL Enrichment Coverage"
                  htmlFor="ocel-enrichment"
                  helpText="Minimum state transition coverage for OCEL enrichment (0-1)"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="ocel-enrichment"
                        bind:value={$config.quality_gates.ocel_enrichment_coverage}
                        min="0"
                        max="1"
                        step="0.01"
                      />
                      <span>{(($config.quality_gates.ocel_enrichment_coverage ?? 0.95) * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Fraud Pack Effectiveness"
                  htmlFor="fraud-effectiveness"
                  helpText="Minimum detection rate for fraud pack evaluation (0-1)"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="fraud-effectiveness"
                        bind:value={$config.quality_gates.fraud_pack_effectiveness}
                        min="0"
                        max="1"
                        step="0.01"
                      />
                      <span>{(($config.quality_gates.fraud_pack_effectiveness ?? 0.80) * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Intervention Magnitude Tolerance"
                  htmlFor="intervention-tolerance"
                  helpText="Acceptable deviation from expected intervention effect (0-1)"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="intervention-tolerance"
                        bind:value={$config.quality_gates.intervention_magnitude_tolerance}
                        min="0"
                        max="0.5"
                        step="0.01"
                      />
                      <span>{(($config.quality_gates.intervention_magnitude_tolerance ?? 0.10) * 100).toFixed(0)}%</span>
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
          <h4>Benford's Law</h4>
          <p>Validates that the first-digit distribution of amounts follows Benford's Law, a key indicator of naturally occurring financial data.</p>
        </div>
        <div class="info-card">
          <h4>Balance Validation</h4>
          <p>Ensures that Assets = Liabilities + Equity and that all journal entries have balanced debits and credits within tolerance.</p>
        </div>
        <div class="info-card">
          <h4>Completeness</h4>
          <p>Checks that all required fields are populated and that referential integrity is maintained across document chains.</p>
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
  .level-selector { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-2); }
  .level-option { display: flex; flex-direction: column; padding: var(--space-3); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .level-option:hover { border-color: var(--color-accent); }
  .level-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .level-option input { display: none; }
  .level-label { font-weight: 600; font-size: 0.875rem; color: var(--color-text-primary); margin-bottom: var(--space-1); }
  .level-desc { font-size: 0.75rem; color: var(--color-text-secondary); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid, .level-selector { grid-template-columns: 1fr; } .event-item { grid-template-columns: 1fr; } }
</style>
