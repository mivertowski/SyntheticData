<script lang="ts">
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';
  import { configStore, INDUSTRIES, COA_COMPLEXITIES } from '$lib/stores/config';

  const config = configStore.config;
  const loading = configStore.loading;
  const error = configStore.error;
  const validationErrors = configStore.validationErrors;

  // Get validation error for a field
  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }
</script>

<div class="page">
  <ConfigPageHeader title="Global Settings" description="Configure industry, time period, and performance settings" />

  {#if $error}
    <div class="alert alert-error">
      <p>{$error}</p>
    </div>
  {/if}

  {#if $config}
    <div class="page-content">
      <FormSection title="Industry & Time Period" description="Define the scope of data generation">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Industry"
              htmlFor="industry"
              helpText="Affects chart of accounts structure and business processes"
              error={getError('global.industry')}
            >
              {#snippet children()}
                <select id="industry" bind:value={$config.global.industry}>
                  {#each INDUSTRIES as ind}
                    <option value={ind.value}>{ind.label}</option>
                  {/each}
                </select>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="CoA Complexity"
              htmlFor="complexity"
              helpText="Number of accounts in the chart of accounts"
              error={getError('chart_of_accounts.complexity')}
            >
              {#snippet children()}
                <select id="complexity" bind:value={$config.chart_of_accounts.complexity}>
                  {#each COA_COMPLEXITIES as c}
                    <option value={c.value}>{c.label}</option>
                  {/each}
                </select>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Start Date"
              htmlFor="start-date"
              helpText="First day of the generation period (YYYY-MM-DD)"
              error={getError('global.start_date')}
              required
            >
              {#snippet children()}
                <input
                  type="date"
                  id="start-date"
                  bind:value={$config.global.start_date}
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Period (Months)"
              htmlFor="period-months"
              helpText="Number of months to generate data for (1-120)"
              error={getError('global.period_months')}
              required
            >
              {#snippet children()}
                <input
                  type="number"
                  id="period-months"
                  bind:value={$config.global.period_months}
                  min="1"
                  max="120"
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Group Currency"
              htmlFor="group-currency"
              helpText="Reporting currency for consolidation"
            >
              {#snippet children()}
                <input
                  type="text"
                  id="group-currency"
                  bind:value={$config.global.group_currency}
                  maxlength="3"
                  placeholder="USD"
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Random Seed"
              htmlFor="seed"
              helpText="Optional seed for reproducible generation"
            >
              {#snippet children()}
                <input
                  type="number"
                  id="seed"
                  bind:value={$config.global.seed}
                  placeholder="Leave empty for random"
                />
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Performance" description="Control generation speed and resource usage">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.global.parallel}
              label="Parallel Generation"
              description="Use multiple CPU cores for faster generation"
            />

            <div class="form-grid">
              <FormGroup
                label="Worker Threads"
                htmlFor="workers"
                helpText="Number of threads (0 = auto-detect)"
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="workers"
                    bind:value={$config.global.worker_threads}
                    min="0"
                    max="64"
                    disabled={!$config.global.parallel}
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Memory Limit (MB)"
                htmlFor="memory"
                helpText="Maximum memory usage (0 = unlimited)"
                error={getError('global.memory_limit_mb')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="memory"
                    bind:value={$config.global.memory_limit_mb}
                    min="0"
                    step="256"
                    placeholder="0"
                  />
                {/snippet}
              </FormGroup>
            </div>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Chart of Accounts" description="Account hierarchy settings">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.chart_of_accounts.industry_specific}
              label="Industry-Specific Accounts"
              description="Include accounts tailored to the selected industry"
            />

            <div class="form-grid">
              <FormGroup
                label="Min Hierarchy Depth"
                htmlFor="min-depth"
                helpText="Minimum account hierarchy levels"
                error={getError('chart_of_accounts.min_hierarchy_depth')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="min-depth"
                    bind:value={$config.chart_of_accounts.min_hierarchy_depth}
                    min="1"
                    max="10"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Max Hierarchy Depth"
                htmlFor="max-depth"
                helpText="Maximum account hierarchy levels"
                error={getError('chart_of_accounts.max_hierarchy_depth')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="max-depth"
                    bind:value={$config.chart_of_accounts.max_hierarchy_depth}
                    min="1"
                    max="10"
                  />
                {/snippet}
              </FormGroup>
            </div>
          </div>
        {/snippet}
      </FormSection>
    </div>
  {:else if $loading}
    <div class="loading">
      <p>Loading configuration...</p>
    </div>
  {:else}
    <div class="loading">
      <p>Configuration not available</p>
      <button class="btn-primary" onclick={() => configStore.load()}>
        Retry
      </button>
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

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .form-stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .alert {
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-4);
  }

  .alert-error {
    background-color: rgba(220, 53, 69, 0.1);
    border: 1px solid var(--color-danger);
    color: var(--color-danger);
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
