<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle, InputNumber } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const STREAM_TARGETS = [
    { value: 'file', label: 'File (JSONL)', description: 'Write stream output to a JSONL file' },
    { value: 'http', label: 'HTTP', description: 'Stream to an HTTP endpoint' },
    { value: 'noop', label: 'NoOp', description: 'Discard output (for benchmarking)' },
  ];

  const BACKPRESSURE_STRATEGIES = [
    { value: 'block', label: 'Block', description: 'Block producer when buffer is full' },
    { value: 'drop_oldest', label: 'Drop Oldest', description: 'Drop oldest items when buffer is full' },
    { value: 'drop_newest', label: 'Drop Newest', description: 'Drop newest items when buffer is full' },
    { value: 'buffer', label: 'Unbounded Buffer', description: 'Grow buffer without limit (use with caution)' },
  ];

  const PHASES: { id: keyof import('$lib/stores/config').StreamingPhaseFilters; label: string }[] = [
    { id: 'master_data', label: 'Master Data' },
    { id: 'journal_entries', label: 'Journal Entries' },
    { id: 'document_flows', label: 'Document Flows' },
    { id: 'anomaly_injection', label: 'Anomaly Injection' },
    { id: 'ocpm', label: 'Process Mining' },
  ];

  function ensureStreamConfig() {
    if (!$config) return;
    if (!$config.streaming) {
      $config.streaming = {
        enabled: false,
        target: 'file',
        file_path: './output/stream.jsonl',
        buffer_size: 1000,
        backpressure: 'block',
        phase_filters: {
          master_data: true,
          journal_entries: true,
          document_flows: true,
          anomaly_injection: true,
          ocpm: true,
        },
      };
    }
  }
</script>

<div class="page">
  <ConfigPageHeader
    title="Streaming Pipeline"
    description="Configure real-time streaming output with buffering and backpressure control"
  />

  {#if $config}
    <div class="page-content">
      <FormSection title="Stream Configuration" description="Enable and configure the streaming output pipeline">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              checked={$config.streaming?.enabled ?? false}
              label="Enable Streaming"
              description="Stream generated records in real-time via the PhaseSink pipeline"
              onchange={() => {
                ensureStreamConfig();
                if ($config.streaming) $config.streaming.enabled = !$config.streaming.enabled;
              }}
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.streaming?.enabled}
        <FormSection title="Stream Target" description="Choose where streamed records are sent">
          {#snippet children()}
            <div class="target-selector">
              {#each STREAM_TARGETS as target}
                <label class="target-option" class:selected={$config.streaming?.target === target.value}>
                  <input
                    type="radio"
                    name="stream-target"
                    value={target.value}
                    checked={$config.streaming?.target === target.value}
                    onchange={() => {
                      if ($config.streaming) $config.streaming.target = target.value;
                    }}
                  />
                  <span class="target-label">{target.label}</span>
                  <span class="target-desc">{target.description}</span>
                </label>
              {/each}
            </div>
          {/snippet}
        </FormSection>

        {#if $config.streaming?.target === 'file'}
          <FormSection title="File Output" description="Configure JSONL file output path">
            {#snippet children()}
              <div class="form-stack">
                <FormGroup label="Output File Path" htmlFor="stream-path" helpText="Path for the JSONL stream output file">
                  {#snippet children()}
                    <input
                      type="text"
                      id="stream-path"
                      bind:value={$config.streaming.file_path}
                      placeholder="./output/stream.jsonl"
                      class="text-input"
                    />
                  {/snippet}
                </FormGroup>
              </div>
            {/snippet}
          </FormSection>
        {/if}

        <FormSection title="Buffer & Backpressure" description="Configure stream buffering and backpressure behavior">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup label="Buffer Size" htmlFor="buffer-size" helpText="Number of records to buffer before applying backpressure">
                {#snippet children()}
                  <InputNumber bind:value={$config.streaming.buffer_size} id="buffer-size" min={100} max={100000} step={100} />
                {/snippet}
              </FormGroup>

              <FormGroup label="Backpressure Strategy" htmlFor="backpressure" helpText="What to do when the buffer is full">
                {#snippet children()}
                  <div class="strategy-selector">
                    {#each BACKPRESSURE_STRATEGIES as strategy}
                      <label class="strategy-option" class:selected={$config.streaming?.backpressure === strategy.value}>
                        <input
                          type="radio"
                          name="backpressure"
                          value={strategy.value}
                          checked={$config.streaming?.backpressure === strategy.value}
                          onchange={() => {
                            if ($config.streaming) $config.streaming.backpressure = strategy.value;
                          }}
                        />
                        <span class="strategy-label">{strategy.label}</span>
                        <span class="strategy-desc">{strategy.description}</span>
                      </label>
                    {/each}
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Phase Filters" description="Select which generation phases to include in the stream">
          {#snippet children()}
            <div class="form-stack">
              {#each PHASES as phase}
                <Toggle
                  bind:checked={$config.streaming.phase_filters[phase.id]}
                  label={phase.label}
                  description={`Include ${phase.label.toLowerCase()} records in stream output`}
                />
              {/each}
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
  .target-selector { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-3); }
  .target-option { display: flex; flex-direction: column; padding: var(--space-3); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .target-option:hover { border-color: var(--color-accent); }
  .target-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .target-option input { display: none; }
  .target-label { font-weight: 600; font-size: 0.875rem; color: var(--color-text-primary); margin-bottom: var(--space-1); }
  .target-desc { font-size: 0.75rem; color: var(--color-text-secondary); }
  .strategy-selector { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-2); }
  .strategy-option { display: flex; flex-direction: column; padding: var(--space-3); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .strategy-option:hover { border-color: var(--color-accent); }
  .strategy-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .strategy-option input { display: none; }
  .strategy-label { font-weight: 600; font-size: 0.8125rem; color: var(--color-text-primary); }
  .strategy-desc { font-size: 0.75rem; color: var(--color-text-secondary); }
  .text-input { width: 100%; padding: var(--space-2) var(--space-3); border: 1px solid var(--color-border); border-radius: var(--radius-md); background-color: var(--color-surface); color: var(--color-text-primary); font-family: var(--font-mono); font-size: 0.8125rem; }
  @media (max-width: 768px) { .target-selector, .strategy-selector { grid-template-columns: 1fr; } }
</style>
