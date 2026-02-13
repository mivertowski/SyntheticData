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
  <ConfigPageHeader title="Process Mining (OCPM)" description="Generate OCEL 2.0 compatible event logs for object-centric process mining" />

  {#if $config}
    <div class="page-content">
      <FormSection title="OCPM Generation" description="Enable object-centric process mining event log generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.ocpm.enabled}
              label="Enable OCPM Event Logs"
              description="Generate OCEL 2.0 compatible event logs with many-to-many object relationships"
            />

            {#if $config.ocpm.enabled}
              <Toggle
                bind:checked={$config.ocpm.generate_lifecycle_events}
                label="Generate Lifecycle Events"
                description="Generate Start/Complete pairs instead of atomic events"
              />

              <Toggle
                bind:checked={$config.ocpm.include_object_relationships}
                label="Include Object Relationships"
                description="Include object-to-object relationships in output"
              />

              <Toggle
                bind:checked={$config.ocpm.compute_variants}
                label="Compute Process Variants"
                description="Compute and export distinct execution patterns"
              />

              {#if $config.ocpm.compute_variants}
                <FormGroup
                  label="Max Variants"
                  htmlFor="max-variants"
                  helpText="Maximum variants to track (0 = unlimited)"
                >
                  {#snippet children()}
                    <input
                      type="number"
                      id="max-variants"
                      bind:value={$config.ocpm.max_variants}
                      min="0"
                    />
                  {/snippet}
                </FormGroup>
              {/if}
            {/if}
          </div>
        {/snippet}
      </FormSection>

      {#if $config.ocpm.enabled}
        <FormSection title="P2P Process Configuration" description="Procure-to-Pay process behavior settings">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Rework Probability"
                htmlFor="p2p-rework"
                helpText="Probability of rework loops in the process"
                error={getError('ocpm.p2p_process.rework_probability')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="p2p-rework"
                      bind:value={$config.ocpm.p2p_process.rework_probability}
                      min="0"
                      max="0.3"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.ocpm.p2p_process.rework_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Skip Step Probability"
                htmlFor="p2p-skip"
                helpText="Probability of skipping process steps"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="p2p-skip"
                      bind:value={$config.ocpm.p2p_process.skip_step_probability}
                      min="0"
                      max="0.2"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.ocpm.p2p_process.skip_step_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Out-of-Order Probability"
                htmlFor="p2p-ooo"
                helpText="Probability of steps occurring out of order"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="p2p-ooo"
                      bind:value={$config.ocpm.p2p_process.out_of_order_probability}
                      min="0"
                      max="0.2"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.ocpm.p2p_process.out_of_order_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="O2C Process Configuration" description="Order-to-Cash process behavior settings">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Rework Probability"
                htmlFor="o2c-rework"
                helpText="Probability of rework loops in the process"
                error={getError('ocpm.o2c_process.rework_probability')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="o2c-rework"
                      bind:value={$config.ocpm.o2c_process.rework_probability}
                      min="0"
                      max="0.3"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.ocpm.o2c_process.rework_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Skip Step Probability"
                htmlFor="o2c-skip"
                helpText="Probability of skipping process steps"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="o2c-skip"
                      bind:value={$config.ocpm.o2c_process.skip_step_probability}
                      min="0"
                      max="0.2"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.ocpm.o2c_process.skip_step_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Out-of-Order Probability"
                htmlFor="o2c-ooo"
                helpText="Probability of steps occurring out of order"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="o2c-ooo"
                      bind:value={$config.ocpm.o2c_process.out_of_order_probability}
                      min="0"
                      max="0.2"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.ocpm.o2c_process.out_of_order_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Output Formats" description="Select which output formats to generate">
          {#snippet children()}
            <div class="output-toggles">
              <Toggle
                bind:checked={$config.ocpm.output.ocel_json}
                label="OCEL 2.0 JSON"
                description="Standard OCEL 2.0 JSON format"
              />

              <Toggle
                bind:checked={$config.ocpm.output.ocel_xml}
                label="OCEL 2.0 XML"
                description="OCEL 2.0 XML format"
              />

              <Toggle
                bind:checked={$config.ocpm.output.flattened_csv}
                label="Flattened CSV"
                description="Flattened CSV for each object type"
              />

              <Toggle
                bind:checked={$config.ocpm.output.event_object_csv}
                label="Event-Object CSV"
                description="Event-object relationship table"
              />

              <Toggle
                bind:checked={$config.ocpm.output.object_relationship_csv}
                label="Object Relationship CSV"
                description="Object-to-object relationship table"
              />

              <Toggle
                bind:checked={$config.ocpm.output.variants_csv}
                label="Process Variants CSV"
                description="Process variants summary"
                disabled={!$config.ocpm.compute_variants}
              />
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-section">
        <h2>About OCPM</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>Object-Centric Process Mining</h3>
            <p>
              Traditional process mining assumes one object per case. OCPM (OCEL 2.0)
              supports many-to-many relationships between events and objects,
              enabling more accurate process analysis.
            </p>
          </div>
          <div class="info-card">
            <h3>Output Formats</h3>
            <p>
              <strong>OCEL 2.0:</strong> Standard format for process mining tools.<br/>
              <strong>CSV:</strong> Flattened tables for direct analysis.<br/>
              <strong>Variants:</strong> Summary of distinct execution patterns.
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

  .output-toggles {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-3);
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

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid,
    .output-toggles,
    .info-grid {
      grid-template-columns: 1fr;
    }

  }
</style>
