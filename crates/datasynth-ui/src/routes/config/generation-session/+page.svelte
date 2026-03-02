<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle, InputNumber } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const MONTHS = [
    { value: 1, label: 'January' }, { value: 2, label: 'February' },
    { value: 3, label: 'March' }, { value: 4, label: 'April' },
    { value: 5, label: 'May' }, { value: 6, label: 'June' },
    { value: 7, label: 'July' }, { value: 8, label: 'August' },
    { value: 9, label: 'September' }, { value: 10, label: 'October' },
    { value: 11, label: 'November' }, { value: 12, label: 'December' },
  ];

  function ensureSessionConfig() {
    if (!$config) return;
    if (!$config.generation_session) {
      $config.generation_session = {
        mode: 'single',
        period_count: 12,
        fiscal_year_start_month: 1,
        incremental: false,
        append_months: 3,
        checkpoint_path: '',
      };
    }
  }
</script>

<div class="page">
  <ConfigPageHeader
    title="Generation Session"
    description="Configure single-run or multi-period generation with checkpoints and incremental appending"
  />

  {#if $config}
    <div class="page-content">
      <FormSection title="Session Mode" description="Choose between single-run and multi-period generation">
        {#snippet children()}
          <div class="form-stack">
            <div class="mode-selector">
              <label class="mode-option" class:selected={($config.generation_session?.mode ?? 'single') === 'single'}>
                <input
                  type="radio"
                  name="session-mode"
                  value="single"
                  checked={($config.generation_session?.mode ?? 'single') === 'single'}
                  onchange={() => { ensureSessionConfig(); if ($config.generation_session) $config.generation_session.mode = 'single'; }}
                />
                <span class="mode-label">Single Run</span>
                <span class="mode-desc">Generate all data in one pass</span>
              </label>
              <label class="mode-option" class:selected={$config.generation_session?.mode === 'multi_period'}>
                <input
                  type="radio"
                  name="session-mode"
                  value="multi_period"
                  checked={$config.generation_session?.mode === 'multi_period'}
                  onchange={() => { ensureSessionConfig(); if ($config.generation_session) $config.generation_session.mode = 'multi_period'; }}
                />
                <span class="mode-label">Multi-Period</span>
                <span class="mode-desc">Generate data across multiple fiscal periods with balance continuity</span>
              </label>
            </div>
          </div>
        {/snippet}
      </FormSection>

      {#if $config.generation_session?.mode === 'multi_period'}
        <FormSection title="Period Configuration" description="Configure the number of periods and fiscal year">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup label="Number of Periods" htmlFor="period-count" helpText="How many periods to generate">
                {#snippet children()}
                  <InputNumber bind:value={$config.generation_session.period_count} id="period-count" min={1} max={120} />
                {/snippet}
              </FormGroup>

              <FormGroup label="Fiscal Year Start" htmlFor="fy-start" helpText="Month when the fiscal year begins">
                {#snippet children()}
                  <select id="fy-start" bind:value={$config.generation_session.fiscal_year_start_month}>
                    {#each MONTHS as month}
                      <option value={month.value}>{month.label}</option>
                    {/each}
                  </select>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Incremental Generation" description="Append new periods to existing datasets">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.generation_session.incremental}
                label="Enable Incremental Generation"
                description="Append new months to previously generated data maintaining balance continuity"
              />

              {#if $config.generation_session.incremental}
                <FormGroup label="Months to Append" htmlFor="append-months" helpText="Number of new months to generate">
                  {#snippet children()}
                    <InputNumber bind:value={$config.generation_session.append_months} id="append-months" min={1} max={60} />
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Checkpoint" description="Save and restore generation state for multi-period runs">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup label="Checkpoint File Path" htmlFor="checkpoint-path" helpText="Path to save/load .dss checkpoint files">
                {#snippet children()}
                  <input
                    type="text"
                    id="checkpoint-path"
                    bind:value={$config.generation_session.checkpoint_path}
                    placeholder="./checkpoints/session.dss"
                    class="text-input"
                  />
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Balance Continuity</h4>
          <p>Multi-period mode carries forward closing balances as opening balances, ensuring GL continuity across fiscal periods.</p>
        </div>
        <div class="info-card">
          <h4>Checkpoints</h4>
          <p>Save generation state to .dss files for resuming interrupted runs or appending new periods later.</p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .page { max-width: 960px; }
  .page-content { display: flex; flex-direction: column; gap: var(--space-5); }
  .form-stack { display: flex; flex-direction: column; gap: var(--space-4); }
  .form-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-4); }
  .mode-selector { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-3); }
  .mode-option { display: flex; flex-direction: column; padding: var(--space-4); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .mode-option:hover { border-color: var(--color-accent); }
  .mode-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .mode-option input { display: none; }
  .mode-label { font-weight: 600; font-size: 0.875rem; color: var(--color-text-primary); margin-bottom: var(--space-1); }
  .mode-desc { font-size: 0.75rem; color: var(--color-text-secondary); }
  .text-input { width: 100%; padding: var(--space-2) var(--space-3); border: 1px solid var(--color-border); border-radius: var(--radius-md); background-color: var(--color-surface); color: var(--color-text-primary); font-family: var(--font-mono); font-size: 0.8125rem; }
  .info-cards { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  @media (max-width: 768px) { .form-grid, .mode-selector, .info-cards { grid-template-columns: 1fr; } }
</style>
