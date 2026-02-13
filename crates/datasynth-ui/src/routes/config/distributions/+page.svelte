<script lang="ts">
  import { configStore, DISTRIBUTION_TYPES, COPULA_TYPES, INDUSTRY_PROFILES, STATISTICAL_TEST_TYPES } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
  const validationErrors = configStore.validationErrors;

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  function addMixtureComponent() {
    if ($config) {
      $config.distributions.amounts.components = [
        ...$config.distributions.amounts.components,
        { weight: 0.1, mu: 7.0, sigma: 1.5, label: 'new' },
      ];
    }
  }

  function removeMixtureComponent(index: number) {
    if ($config && $config.distributions.amounts.components.length > 1) {
      $config.distributions.amounts.components = $config.distributions.amounts.components.filter((_, i) => i !== index);
    }
  }

  function normalizeWeights() {
    if ($config) {
      const total = $config.distributions.amounts.components.reduce((sum, c) => sum + c.weight, 0);
      if (total > 0) {
        $config.distributions.amounts.components = $config.distributions.amounts.components.map(c => ({
          ...c,
          weight: c.weight / total,
        }));
      }
    }
  }
</script>

<div class="page">
  <ConfigPageHeader title="Statistical Distributions" description="Configure advanced mixture models, correlations, and regime changes" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Distribution Settings" description="Enable advanced statistical distribution modeling">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.distributions.enabled}
              label="Enable Advanced Distributions"
              description="Use mixture models, correlations, and regime changes for realistic data generation"
            />

            {#if $config.distributions.enabled}
              <FormGroup
                label="Industry Profile"
                htmlFor="industry-profile"
                helpText="Pre-configured distribution patterns for your industry"
              >
                {#snippet children()}
                  <select id="industry-profile" bind:value={$config.distributions.industry_profile}>
                    <option value={null}>None (Custom)</option>
                    {#each INDUSTRY_PROFILES as profile}
                      <option value={profile.value}>{profile.label}</option>
                    {/each}
                  </select>
                {/snippet}
              </FormGroup>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      {#if $config.distributions.enabled}
        <FormSection title="Mixture Distribution" description="Multi-modal amount distribution with labeled components">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.distributions.amounts.enabled}
                label="Enable Mixture Model"
                description="Generate amounts from multiple distribution components"
              />

              {#if $config.distributions.amounts.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Distribution Type"
                    htmlFor="dist-type"
                    helpText="Base distribution for each component"
                  >
                    {#snippet children()}
                      <select id="dist-type" bind:value={$config.distributions.amounts.distribution_type}>
                        {#each DISTRIBUTION_TYPES as dt}
                          <option value={dt.value}>{dt.label}</option>
                        {/each}
                      </select>
                    {/snippet}
                  </FormGroup>

                  <div class="toggle-inline">
                    <Toggle
                      bind:checked={$config.distributions.amounts.benford_compliance}
                      label="Benford Compliance"
                      description="Ensure first digit distribution follows Benford's Law"
                    />
                  </div>
                </div>

                <div class="components-section">
                  <div class="components-header">
                    <h4>Mixture Components</h4>
                    <div class="components-actions">
                      <button class="btn-small" onclick={normalizeWeights}>Normalize</button>
                      <button class="btn-small btn-primary" onclick={addMixtureComponent}>+ Add</button>
                    </div>
                  </div>

                  <div class="components-list">
                    {#each $config.distributions.amounts.components as component, i}
                      <div class="component-row">
                        <div class="component-fields">
                          <div class="field">
                            <label>Weight</label>
                            <input type="number" bind:value={component.weight} min="0" max="1" step="0.05" />
                          </div>
                          <div class="field">
                            <label>Mu (log-scale)</label>
                            <input type="number" bind:value={component.mu} step="0.5" />
                          </div>
                          <div class="field">
                            <label>Sigma</label>
                            <input type="number" bind:value={component.sigma} min="0.1" step="0.1" />
                          </div>
                          <div class="field">
                            <label>Label</label>
                            <input type="text" bind:value={component.label} placeholder="e.g., routine" />
                          </div>
                        </div>
                        <button
                          class="btn-remove"
                          onclick={() => removeMixtureComponent(i)}
                          disabled={$config.distributions.amounts.components.length <= 1}
                        >
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M18 6L6 18M6 6l12 12" />
                          </svg>
                        </button>
                      </div>
                    {/each}
                  </div>

                  {#if getError('distributions.amounts.components')}
                    <p class="error-text">{getError('distributions.amounts.components')}</p>
                  {/if}
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Cross-Field Correlations" description="Model dependencies between related fields using copulas">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.distributions.correlations.enabled}
                label="Enable Correlation Modeling"
                description="Generate correlated values across multiple fields"
              />

              {#if $config.distributions.correlations.enabled}
                <FormGroup
                  label="Copula Type"
                  htmlFor="copula-type"
                  helpText="Mathematical model for dependency structure"
                >
                  {#snippet children()}
                    <div class="copula-selector">
                      {#each COPULA_TYPES as copula}
                        <label class="copula-option" class:selected={$config.distributions.correlations.copula_type === copula.value}>
                          <input
                            type="radio"
                            name="copula-type"
                            value={copula.value}
                            bind:group={$config.distributions.correlations.copula_type}
                          />
                          <span class="copula-label">{copula.label}</span>
                          <span class="copula-desc">{copula.description}</span>
                        </label>
                      {/each}
                    </div>
                  {/snippet}
                </FormGroup>

                <div class="correlation-matrix">
                  <h4>Correlation Matrix</h4>
                  <p class="matrix-hint">Values between -1 (negative correlation) and 1 (positive correlation). Diagonal must be 1.0.</p>
                  <div class="matrix-grid" style="--cols: {$config.distributions.correlations.fields.length + 1}">
                    <div class="matrix-header"></div>
                    {#each $config.distributions.correlations.fields as field}
                      <div class="matrix-header">{field.name}</div>
                    {/each}
                    {#each $config.distributions.correlations.matrix as row, i}
                      <div class="matrix-row-label">{$config.distributions.correlations.fields[i]?.name}</div>
                      {#each row as val, j}
                        <input
                          type="number"
                          class="matrix-cell"
                          class:diagonal={i === j}
                          value={val}
                          min="-1"
                          max="1"
                          step="0.05"
                          disabled={i === j}
                          oninput={(e) => {
                            const target = e.target;
                            if (target instanceof HTMLInputElement) {
                              const newVal = parseFloat(target.value);
                              $config.distributions.correlations.matrix[i][j] = newVal;
                              $config.distributions.correlations.matrix[j][i] = newVal;
                            }
                          }}
                        />
                      {/each}
                    {/each}
                  </div>
                  {#if getError('distributions.correlations.matrix')}
                    <p class="error-text">{getError('distributions.correlations.matrix')}</p>
                  {/if}
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Regime Changes" description="Simulate structural breaks and economic cycles">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.distributions.regime_changes.enabled}
                label="Enable Regime Changes"
                description="Model distribution shifts from business events"
              />

              {#if $config.distributions.regime_changes.enabled}
                <div class="economic-cycle">
                  <Toggle
                    bind:checked={$config.distributions.regime_changes.economic_cycle.enabled}
                    label="Enable Economic Cycles"
                    description="Add sinusoidal patterns with recession modeling"
                  />

                  {#if $config.distributions.regime_changes.economic_cycle.enabled}
                    <div class="form-grid">
                      <FormGroup
                        label="Cycle Period (months)"
                        htmlFor="cycle-period"
                        helpText="Length of one complete economic cycle"
                      >
                        {#snippet children()}
                          <input
                            type="number"
                            id="cycle-period"
                            bind:value={$config.distributions.regime_changes.economic_cycle.cycle_period_months}
                            min="12"
                            max="120"
                          />
                        {/snippet}
                      </FormGroup>

                      <FormGroup
                        label="Amplitude"
                        htmlFor="cycle-amplitude"
                        helpText="Strength of cyclical variation (0-1)"
                        error={getError('distributions.regime_changes.economic_cycle.amplitude')}
                      >
                        {#snippet children()}
                          <div class="slider-with-value">
                            <input
                              type="range"
                              id="cycle-amplitude"
                              bind:value={$config.distributions.regime_changes.economic_cycle.amplitude}
                              min="0"
                              max="0.5"
                              step="0.05"
                            />
                            <span class="slider-value">{($config.distributions.regime_changes.economic_cycle.amplitude * 100).toFixed(0)}%</span>
                          </div>
                        {/snippet}
                      </FormGroup>

                      <FormGroup
                        label="Recession Probability"
                        htmlFor="recession-prob"
                        helpText="Chance of recession occurring"
                        error={getError('distributions.regime_changes.economic_cycle.recession_probability')}
                      >
                        {#snippet children()}
                          <div class="slider-with-value">
                            <input
                              type="range"
                              id="recession-prob"
                              bind:value={$config.distributions.regime_changes.economic_cycle.recession_probability}
                              min="0"
                              max="0.3"
                              step="0.01"
                            />
                            <span class="slider-value">{($config.distributions.regime_changes.economic_cycle.recession_probability * 100).toFixed(0)}%</span>
                          </div>
                        {/snippet}
                      </FormGroup>

                      <FormGroup
                        label="Recession Depth"
                        htmlFor="recession-depth"
                        helpText="Magnitude of recession impact"
                      >
                        {#snippet children()}
                          <div class="slider-with-value">
                            <input
                              type="range"
                              id="recession-depth"
                              bind:value={$config.distributions.regime_changes.economic_cycle.recession_depth}
                              min="0.1"
                              max="0.5"
                              step="0.05"
                            />
                            <span class="slider-value">{($config.distributions.regime_changes.economic_cycle.recession_depth * 100).toFixed(0)}%</span>
                          </div>
                        {/snippet}
                      </FormGroup>
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Statistical Validation" description="Configure tests to verify distribution compliance">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.distributions.validation.enabled}
                label="Enable Validation"
                description="Run statistical tests during generation"
              />

              {#if $config.distributions.validation.enabled}
                <div class="validation-tests">
                  <h4>Enabled Tests</h4>
                  <div class="tests-list">
                    {#each STATISTICAL_TEST_TYPES as testType}
                      {@const isEnabled = $config.distributions.validation.tests.some(t => t.test_type === testType.value)}
                      <label class="test-option" class:enabled={isEnabled}>
                        <input
                          type="checkbox"
                          checked={isEnabled}
                          onchange={(e) => {
                            const target = e.target;
                            if (target instanceof HTMLInputElement) {
                              if (target.checked) {
                                $config.distributions.validation.tests = [
                                  ...$config.distributions.validation.tests,
                                  { test_type: testType.value, significance: 0.05, threshold_mad: null, target_distribution: null },
                                ];
                              } else {
                                $config.distributions.validation.tests = $config.distributions.validation.tests.filter(
                                  t => t.test_type !== testType.value
                                );
                              }
                            }
                          }}
                        />
                        <span class="test-label">{testType.label}</span>
                        <span class="test-desc">{testType.description}</span>
                      </label>
                    {/each}
                  </div>
                </div>

                <Toggle
                  bind:checked={$config.distributions.validation.fail_on_violation}
                  label="Fail on Violation"
                  description="Stop generation if validation tests fail (otherwise warn only)"
                />
              {/if}
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-section">
        <h2>About Statistical Distributions</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>Mixture Models</h3>
            <p>
              Transaction amounts often follow multi-modal distributions. Mixture models combine
              multiple components (e.g., routine, significant, major transactions) with different
              parameters for realistic amount generation.
            </p>
          </div>
          <div class="info-card">
            <h3>Copulas</h3>
            <p>
              Copulas model dependency between fields (e.g., larger transactions tend to have
              more line items). Different copula types capture different dependency patterns
              like tail dependencies for extreme events.
            </p>
          </div>
          <div class="info-card">
            <h3>Regime Changes</h3>
            <p>
              Real data shows structural breaks from acquisitions, policy changes, or economic
              cycles. Regime change modeling creates realistic temporal patterns with shifts
              in volume, amounts, and distributions.
            </p>
          </div>
          <div class="info-card">
            <h3>Validation</h3>
            <p>
              Statistical tests like Benford's Law, Anderson-Darling, and correlation checks
              verify that generated data matches expected patterns. Use these to ensure
              data quality for ML training or testing.
            </p>
          </div>
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
  .page {
    max-width: 900px;
  }

  .page-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .form-stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .toggle-inline {
    display: flex;
    align-items: center;
  }

  /* Mixture Components */
  .components-section {
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    padding: var(--space-4);
  }

  .components-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-3);
  }

  .components-header h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0;
  }

  .components-actions {
    display: flex;
    gap: var(--space-2);
  }

  .btn-small {
    padding: var(--space-1) var(--space-2);
    font-size: 0.75rem;
    border-radius: var(--radius-sm);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    cursor: pointer;
  }

  .btn-small:hover {
    background-color: var(--color-background);
  }

  .btn-small.btn-primary {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
    color: white;
  }

  .components-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .component-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2);
    background-color: var(--color-surface);
    border-radius: var(--radius-sm);
  }

  .component-fields {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-2);
  }

  .component-fields .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .component-fields label {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
  }

  .component-fields input {
    padding: var(--space-1) var(--space-2);
    font-size: 0.8125rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background-color: var(--color-background);
  }

  .btn-remove {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--color-text-muted);
  }

  .btn-remove:hover:not(:disabled) {
    color: var(--color-error);
    border-color: var(--color-error);
  }

  .btn-remove:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .btn-remove svg {
    width: 14px;
    height: 14px;
  }

  /* Copula Selector */
  .copula-selector {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-2);
  }

  .copula-option {
    display: flex;
    flex-direction: column;
    padding: var(--space-3);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .copula-option:hover {
    border-color: var(--color-accent);
  }

  .copula-option.selected {
    border-color: var(--color-accent);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .copula-option input {
    display: none;
  }

  .copula-label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .copula-desc {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  /* Correlation Matrix */
  .correlation-matrix {
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    padding: var(--space-4);
  }

  .correlation-matrix h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 var(--space-1);
  }

  .matrix-hint {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    margin: 0 0 var(--space-3);
  }

  .matrix-grid {
    display: grid;
    grid-template-columns: auto repeat(var(--cols, 3), 1fr);
    gap: var(--space-1);
  }

  .matrix-header {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-muted);
    padding: var(--space-1);
    text-align: center;
  }

  .matrix-row-label {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-muted);
    padding: var(--space-1);
    display: flex;
    align-items: center;
  }

  .matrix-cell {
    width: 100%;
    padding: var(--space-1);
    font-size: 0.8125rem;
    text-align: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background-color: var(--color-surface);
  }

  .matrix-cell.diagonal {
    background-color: var(--color-background);
    color: var(--color-text-muted);
  }

  /* Economic Cycle */
  .economic-cycle {
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  /* Sliders */
  .slider-with-value {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .slider-with-value input[type="range"] {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: var(--color-background);
    appearance: none;
    cursor: pointer;
  }

  .slider-with-value input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .slider-value {
    min-width: 50px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--color-text-primary);
  }

  /* Validation Tests */
  .validation-tests {
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    padding: var(--space-4);
  }

  .validation-tests h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 var(--space-3);
  }

  .tests-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .test-option {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-2);
    background-color: var(--color-surface);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .test-option:hover {
    background-color: rgba(59, 130, 246, 0.05);
  }

  .test-option.enabled {
    background-color: rgba(59, 130, 246, 0.1);
  }

  .test-option input {
    margin-top: 2px;
  }

  .test-label {
    font-weight: 500;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    min-width: 180px;
  }

  .test-desc {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  /* Select */
  select {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  select:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  /* Error */
  .error-text {
    font-size: 0.75rem;
    color: var(--color-error);
    margin: var(--space-2) 0 0;
  }

  /* Info Section */
  .info-section {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
  }

  .info-section h2 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: var(--space-4);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .info-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .info-card h3 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .info-card p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid,
    .copula-selector,
    .info-grid {
      grid-template-columns: 1fr;
    }

    .component-fields {
      grid-template-columns: repeat(2, 1fr);
    }

  }
</style>
