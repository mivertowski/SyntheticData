<script lang="ts">
  import { configStore, COA_COMPLEXITIES } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle, InputNumber } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
</script>

<div class="page">
  <ConfigPageHeader title="Chart of Accounts" description="Configure account hierarchy and structure" />

  {#if $config}
    <div class="sections">
      <FormSection
        title="Account Structure"
        description="Configure the chart of accounts complexity and hierarchy"
      >
        <div class="section-content">
          <FormGroup
            label="Complexity Level"
            htmlFor="coa-complexity"
            helpText="Determines the number of accounts generated"
          >
            <select id="coa-complexity" bind:value={$config.chart_of_accounts.complexity}>
              {#each COA_COMPLEXITIES as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </FormGroup>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Industry-Specific Accounts</span>
              <span class="toggle-description">
                Include accounts specific to the selected industry sector
              </span>
            </div>
            <Toggle bind:checked={$config.chart_of_accounts.industry_specific} />
          </div>

          <div class="form-grid">
            <FormGroup
              label="Min Hierarchy Depth"
              htmlFor="min-depth"
              helpText="Minimum levels in account hierarchy (1-10)"
            >
              <InputNumber
                id="min-depth"
                bind:value={$config.chart_of_accounts.min_hierarchy_depth}
                min={1}
                max={10}
                step={1}
              />
            </FormGroup>

            <FormGroup
              label="Max Hierarchy Depth"
              htmlFor="max-depth"
              helpText="Maximum levels in account hierarchy (1-10)"
            >
              <InputNumber
                id="max-depth"
                bind:value={$config.chart_of_accounts.max_hierarchy_depth}
                min={1}
                max={10}
                step={1}
              />
            </FormGroup>
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Account Types"
        description="Overview of generated account categories"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="info-card">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4M12 8h.01" />
            </svg>
            <div class="info-content">
              <strong>Standard Account Categories</strong>
              <p>The chart of accounts includes the following account types:</p>
              <ul>
                <li><strong>Assets:</strong> Cash, receivables, inventory, fixed assets, prepaid expenses</li>
                <li><strong>Liabilities:</strong> Payables, accruals, loans, deferred revenue</li>
                <li><strong>Equity:</strong> Common stock, retained earnings, AOCI</li>
                <li><strong>Revenue:</strong> Sales, service income, other income</li>
                <li><strong>Expenses:</strong> COGS, operating expenses, depreciation, interest</li>
              </ul>
            </div>
          </div>

          <div class="complexity-info">
            <h4>Complexity Levels</h4>
            <div class="complexity-grid">
              <div class="complexity-card">
                <span class="complexity-name">Small</span>
                <span class="complexity-count">~100 accounts</span>
                <p>Basic structure suitable for small businesses</p>
              </div>
              <div class="complexity-card">
                <span class="complexity-name">Medium</span>
                <span class="complexity-count">~400 accounts</span>
                <p>Standard structure for mid-size companies</p>
              </div>
              <div class="complexity-card">
                <span class="complexity-name">Large</span>
                <span class="complexity-count">~2500 accounts</span>
                <p>Detailed structure for enterprise organizations</p>
              </div>
            </div>
          </div>
        </div>
      </FormSection>
    </div>
  {:else}
    <div class="loading">Loading configuration...</div>
  {/if}
</div>

<style>
  .page {
    max-width: 900px;
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .section-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-4);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .toggle-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .toggle-description {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .info-card {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .info-card > svg {
    width: 24px;
    height: 24px;
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .info-content {
    flex: 1;
  }

  .info-content strong {
    display: block;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .info-content p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0 0 var(--space-2);
  }

  .info-content ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .info-content li {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    padding-left: var(--space-3);
    position: relative;
  }

  .info-content li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0.5em;
    width: 4px;
    height: 4px;
    background-color: var(--color-accent);
    border-radius: 50%;
  }

  .info-content li strong {
    display: inline;
    font-size: inherit;
    margin: 0;
  }

  .complexity-info h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-3);
  }

  .complexity-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
  }

  .complexity-card {
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    text-align: center;
  }

  .complexity-name {
    display: block;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .complexity-count {
    display: block;
    font-size: 0.75rem;
    color: var(--color-accent);
    margin-bottom: var(--space-2);
  }

  .complexity-card p {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  select {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  select:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  @media (max-width: 640px) {
    .complexity-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
