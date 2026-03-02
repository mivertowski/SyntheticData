<script lang="ts">
  import { configStore, PRIVACY_LEVELS } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
  const validationErrors = configStore.validationErrors;

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  function getPrivacyDetails(level: string): { epsilon: number; k: number; description: string } {
    const found = PRIVACY_LEVELS.find(p => p.value === level);
    return found || { epsilon: 1.0, k: 5, description: 'Unknown' };
  }
</script>

<div class="page">
  <ConfigPageHeader title="Fingerprinting" description="Extract statistical fingerprints from real data for privacy-preserving synthesis" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Fingerprint Mode" description="Enable fingerprint-based generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.fingerprint.enabled}
              label="Enable Fingerprint Mode"
              description="Generate synthetic data based on extracted fingerprint from real data"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.fingerprint.enabled}
        <FormSection title="Privacy Level" description="Select the privacy protection level for fingerprint extraction">
          {#snippet children()}
            <div class="privacy-level-selector">
              {#each PRIVACY_LEVELS as level}
                <label
                  class="privacy-level-option"
                  class:selected={$config.fingerprint.privacy_level === level.value}
                >
                  <input
                    type="radio"
                    name="privacy-level"
                    value={level.value}
                    bind:group={$config.fingerprint.privacy_level}
                  />
                  <div class="privacy-level-content">
                    <span class="privacy-level-label">{level.label}</span>
                    <span class="privacy-level-desc">{level.description}</span>
                    <div class="privacy-level-params">
                      <span class="param">ε = {level.epsilon}</span>
                      <span class="param">k = {level.k}</span>
                    </div>
                  </div>
                </label>
              {/each}
            </div>

            <div class="privacy-info">
              <div class="privacy-detail">
                <span class="detail-label">Epsilon (ε)</span>
                <span class="detail-value">{getPrivacyDetails($config.fingerprint.privacy_level).epsilon}</span>
                <span class="detail-desc">Lower = more privacy, less utility</span>
              </div>
              <div class="privacy-detail">
                <span class="detail-label">k-Anonymity</span>
                <span class="detail-value">{getPrivacyDetails($config.fingerprint.privacy_level).k}</span>
                <span class="detail-desc">Higher = more privacy, rare values suppressed</span>
              </div>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Generation Settings" description="Configure how synthetic data is generated from fingerprint">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Scale Factor"
                htmlFor="scale"
                helpText="Multiply the row count by this factor (1.0 = same size as original)"
                error={getError('fingerprint.scale')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="scale"
                      bind:value={$config.fingerprint.scale}
                      min="0.1"
                      max="10"
                      step="0.1"
                    />
                    <span class="slider-value">{$config.fingerprint.scale.toFixed(1)}x</span>
                  </div>
                {/snippet}
              </FormGroup>

              <Toggle
                bind:checked={$config.fingerprint.preserve_correlations}
                label="Preserve Correlations"
                description="Use Gaussian copula to preserve multivariate correlations"
              />

              <Toggle
                bind:checked={$config.fingerprint.streaming}
                label="Streaming Mode"
                description="Generate data in streaming mode for large datasets"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="File Paths" description="Configure input and output paths for fingerprint files">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Input Path"
                htmlFor="input-path"
                helpText="Path to input data file or directory (CSV, JSON, Parquet)"
              >
                {#snippet children()}
                  <input
                    type="text"
                    id="input-path"
                    bind:value={$config.fingerprint.input_path}
                    placeholder="./data/input.csv or ./data/"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Output Path"
                htmlFor="output-path"
                helpText="Path to save fingerprint file (.dsf format)"
              >
                {#snippet children()}
                  <input
                    type="text"
                    id="output-path"
                    bind:value={$config.fingerprint.output_path}
                    placeholder="./fingerprints/output.dsf"
                  />
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
        <FormSection title="Evaluation & Privacy" description="Configure fingerprint evaluation and privacy settings">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.fingerprint.evaluation_mode}
                label="Evaluation Mode"
                description="Enable evaluation of fingerprint fidelity against source data"
              />
              <FormGroup
                label="Privacy Level"
                htmlFor="privacy-level-detail"
                helpText="Level of privacy protection applied during fingerprint extraction"
              >
                {#snippet children()}
                  <div class="privacy-selector">
                    {#each [
                      { value: 'basic', label: 'Basic', desc: 'Minimal noise, higher fidelity' },
                      { value: 'standard', label: 'Standard', desc: 'Balanced privacy and fidelity' },
                      { value: 'strict', label: 'Strict', desc: 'Maximum privacy, differential privacy guarantees' },
                    ] as level}
                      <label class="privacy-option" class:selected={$config.fingerprint?.privacy_level === level.value}>
                        <input
                          type="radio"
                          name="privacy-level-detail"
                          value={level.value}
                          bind:group={$config.fingerprint.privacy_level}
                        />
                        <span class="privacy-label">{level.label}</span>
                        <span class="privacy-desc">{level.desc}</span>
                      </label>
                    {/each}
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-section">
        <h2>About Fingerprinting</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>Privacy-Preserving Synthesis</h3>
            <p>
              Fingerprinting extracts statistical properties from real data without
              exposing individual records. The extracted fingerprint can be used to
              generate synthetic data that matches the original distribution.
            </p>
          </div>
          <div class="info-card">
            <h3>Privacy Levels</h3>
            <p>
              <strong>Minimal:</strong> Low privacy, high utility for testing.<br/>
              <strong>Standard:</strong> Balanced for most use cases.<br/>
              <strong>Maximum:</strong> Strongest privacy guarantees.
            </p>
          </div>
          <div class="info-card">
            <h3>DSF File Format</h3>
            <p>
              The .dsf (DataSynth Fingerprint) file is a ZIP archive containing
              schema, statistics, correlations, and privacy audit information.
            </p>
          </div>
          <div class="info-card">
            <h3>CLI Commands</h3>
            <p>
              <code>datasynth-data fingerprint extract --input ./data --output ./fp.dsf</code><br/>
              <code>datasynth-data fingerprint validate ./fp.dsf</code><br/>
              <code>datasynth-data fingerprint info ./fp.dsf</code>
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

  .privacy-level-selector {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-2);
  }

  .privacy-level-option {
    display: flex;
    flex-direction: column;
    padding: var(--space-3);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: center;
  }

  .privacy-level-option:hover {
    border-color: var(--color-accent);
  }

  .privacy-level-option.selected {
    border-color: var(--color-accent);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .privacy-level-option input {
    display: none;
  }

  .privacy-level-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .privacy-level-label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }

  .privacy-level-desc {
    font-size: 0.6875rem;
    color: var(--color-text-secondary);
  }

  .privacy-level-params {
    display: flex;
    justify-content: center;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .privacy-level-params .param {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
    background-color: var(--color-background);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .privacy-info {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .privacy-detail {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .detail-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .detail-value {
    font-size: 1.5rem;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--color-accent);
  }

  .detail-desc {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

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

  .info-card code {
    font-size: 0.6875rem;
    background-color: var(--color-surface);
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    display: block;
    margin: 2px 0;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  .privacy-selector { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-2); }
  .privacy-option { display: flex; flex-direction: column; padding: var(--space-3); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .privacy-option:hover { border-color: var(--color-accent); }
  .privacy-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .privacy-option input { display: none; }
  .privacy-label { font-weight: 600; font-size: 0.8125rem; color: var(--color-text-primary); }
  .privacy-desc { font-size: 0.75rem; color: var(--color-text-secondary); }

  @media (max-width: 768px) {
    .privacy-level-selector {
      grid-template-columns: repeat(2, 1fr);
    }

    .privacy-info,
    .privacy-selector,
    .info-grid {
      grid-template-columns: 1fr;
    }

  }
</style>
