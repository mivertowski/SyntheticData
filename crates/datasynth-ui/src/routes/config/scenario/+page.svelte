<script lang="ts">
  import { configStore, SCENARIO_PROFILES } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  let newTag = $state('');
  let newMetaKey = $state('');
  let newMetaValue = $state('');

  function addTag() {
    if (!$config?.scenario || !newTag.trim()) return;
    if (!$config.scenario.tags.includes(newTag.trim())) {
      $config.scenario.tags = [...$config.scenario.tags, newTag.trim()];
    }
    newTag = '';
  }

  function removeTag(tag: string) {
    if (!$config?.scenario) return;
    $config.scenario.tags = $config.scenario.tags.filter(t => t !== tag);
  }

  function addMetadata() {
    if (!$config?.scenario || !newMetaKey.trim()) return;
    $config.scenario.metadata = {
      ...$config.scenario.metadata,
      [newMetaKey.trim()]: newMetaValue.trim()
    };
    newMetaKey = '';
    newMetaValue = '';
  }

  function removeMetadata(key: string) {
    if (!$config?.scenario) return;
    const { [key]: _, ...rest } = $config.scenario.metadata;
    $config.scenario.metadata = rest;
  }
</script>

<div class="page">
  <ConfigPageHeader title="Scenario Configuration" description="Configure metadata, tagging, and ML training setup for this generation run" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Tags" description="Categorize and filter datasets with tags">
        {#snippet children()}
          <div class="form-stack">
            <div class="tag-input-row">
              <input
                type="text"
                placeholder="Add a tag (e.g., fraud_detection, retail)"
                bind:value={newTag}
                onkeydown={(e) => e.key === 'Enter' && addTag()}
              />
              <button class="btn-secondary" onclick={addTag} disabled={!newTag.trim()}>
                Add
              </button>
            </div>

            {#if $config.scenario.tags.length > 0}
              <div class="tag-list">
                {#each $config.scenario.tags as tag}
                  <span class="tag">
                    {tag}
                    <button class="tag-remove" onclick={() => removeTag(tag)}>&times;</button>
                  </span>
                {/each}
              </div>
            {:else}
              <p class="hint-text">No tags added yet. Tags help organize and filter generated datasets.</p>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Data Quality Profile" description="Select a preset profile or leave custom">
        {#snippet children()}
          <div class="form-stack">
            <FormGroup
              label="Profile Preset"
              htmlFor="profile"
              helpText="Presets control default data quality settings (missing values, typos, duplicates)"
            >
              {#snippet children()}
                <select id="profile" bind:value={$config.scenario.profile}>
                  <option value={null}>Custom (no preset)</option>
                  {#each SCENARIO_PROFILES as profile}
                    <option value={profile.value}>{profile.label} - {profile.description}</option>
                  {/each}
                </select>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Description"
              htmlFor="description"
              helpText="Human-readable description of the scenario purpose"
            >
              {#snippet children()}
                <textarea
                  id="description"
                  bind:value={$config.scenario.description}
                  placeholder="Describe the purpose of this generation run..."
                  rows="3"
                ></textarea>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="ML Training" description="Configure settings for machine learning training datasets">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.scenario.ml_training}
              label="Enable ML Training Mode"
              description="Enables balanced labeling and optimized output for ML training"
            />

            {#if $config.scenario.ml_training}
              <FormGroup
                label="Target Anomaly Ratio"
                htmlFor="anomaly-ratio"
                helpText="Target ratio of anomalies in the dataset (leave empty for natural distribution)"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="anomaly-ratio"
                      bind:value={$config.scenario.target_anomaly_ratio}
                      min="0"
                      max="0.5"
                      step="0.01"
                      placeholder="e.g., 0.1 for 10%"
                    />
                    <span class="suffix">
                      {$config.scenario.target_anomaly_ratio != null
                        ? `${($config.scenario.target_anomaly_ratio * 100).toFixed(0)}%`
                        : 'Auto'}
                    </span>
                  </div>
                {/snippet}
              </FormGroup>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Custom Metadata" description="Add key-value pairs for tracking and filtering">
        {#snippet children()}
          <div class="form-stack">
            <div class="metadata-input-row">
              <input
                type="text"
                placeholder="Key"
                bind:value={newMetaKey}
              />
              <input
                type="text"
                placeholder="Value"
                bind:value={newMetaValue}
                onkeydown={(e) => e.key === 'Enter' && addMetadata()}
              />
              <button class="btn-secondary" onclick={addMetadata} disabled={!newMetaKey.trim()}>
                Add
              </button>
            </div>

            {#if Object.keys($config.scenario.metadata).length > 0}
              <div class="metadata-list">
                {#each Object.entries($config.scenario.metadata) as [key, value]}
                  <div class="metadata-item">
                    <span class="metadata-key">{key}</span>
                    <span class="metadata-value">{value}</span>
                    <button class="metadata-remove" onclick={() => removeMetadata(key)}>&times;</button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="hint-text">No metadata added yet. Metadata helps track generation runs.</p>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      <div class="info-section">
        <h2>About Scenario Configuration</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>Tags</h3>
            <p>
              Tags help organize and filter datasets across multiple runs.
              Common examples: "fraud_detection", "month_end_stress", "retail".
            </p>
          </div>
          <div class="info-card">
            <h3>ML Training Mode</h3>
            <p>
              When enabled, the generator optimizes output for ML training by
              ensuring balanced labels and including all necessary metadata.
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

  .tag-input-row,
  .metadata-input-row {
    display: flex;
    gap: var(--space-2);
  }

  .tag-input-row input,
  .metadata-input-row input {
    flex: 1;
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    background-color: var(--color-accent);
    color: white;
    border-radius: var(--radius-full);
    font-size: 0.8125rem;
  }

  .tag-remove {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: 0;
    opacity: 0.8;
  }

  .tag-remove:hover {
    opacity: 1;
  }

  .metadata-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .metadata-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .metadata-key {
    font-weight: 600;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--color-text-primary);
  }

  .metadata-value {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .metadata-remove {
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: var(--space-1);
  }

  .metadata-remove:hover {
    color: var(--color-error);
  }

  .hint-text {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    font-style: italic;
  }

  .input-with-suffix {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .input-with-suffix input {
    flex: 1;
  }

  .input-with-suffix .suffix {
    font-size: 0.8125rem;
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
    min-width: 50px;
    text-align: right;
  }

  textarea {
    width: 100%;
    padding: var(--space-2);
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    resize: vertical;
    font-family: inherit;
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

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .info-grid {
      grid-template-columns: 1fr;
    }

    .metadata-input-row {
      flex-direction: column;
    }
  }
</style>
