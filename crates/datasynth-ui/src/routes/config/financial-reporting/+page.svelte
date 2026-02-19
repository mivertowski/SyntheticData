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
  <ConfigPageHeader title="Financial Reporting" description="Configure financial statement generation and KPIs" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Financial Reporting Module" description="Enable financial statement and KPI generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.financial_reporting.enabled}
              label="Enable Financial Reporting"
              description="Generate financial statements, KPIs, and budget variance reports"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.financial_reporting.enabled}
        <FormSection title="Statement Types" description="Select which financial statements to generate">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.financial_reporting.balance_sheet}
                label="Balance Sheet"
                description="Generate balance sheet with assets, liabilities, and equity"
              />

              <Toggle
                bind:checked={$config.financial_reporting.income_statement}
                label="Income Statement"
                description="Generate profit and loss statement with revenue and expenses"
              />

              <Toggle
                bind:checked={$config.financial_reporting.cash_flow}
                label="Cash Flow Statement"
                description="Generate statement of cash flows (operating, investing, financing)"
              />

              <Toggle
                bind:checked={$config.financial_reporting.equity_changes}
                label="Changes in Equity"
                description="Generate statement of changes in stockholders' equity"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="KPIs & Budgets" description="Configure management KPI and budget generation">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.financial_reporting.kpis}
                label="Management KPIs"
                description="Generate key performance indicators (current ratio, ROE, margins, etc.)"
              />

              <Toggle
                bind:checked={$config.financial_reporting.budgets}
                label="Budgets & Variance"
                description="Generate budget line items and variance analysis reports"
              />

              <div class="form-grid">
                <FormGroup
                  label="Budget Variance Threshold"
                  htmlFor="budget-variance"
                  helpText="Threshold for flagging budget variances (0-100%)"
                  error={getError('financial_reporting.budget_variance_threshold')}
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input
                        type="number"
                        id="budget-variance"
                        bind:value={$config.financial_reporting.budget_variance_threshold}
                        min="0"
                        max="1"
                        step="0.01"
                        disabled={!$config.financial_reporting.budgets}
                      />
                      <span class="suffix">{($config.financial_reporting.budget_variance_threshold * 100).toFixed(1)}%</span>
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
          <h4>Financial Statements</h4>
          <p>
            Generates GAAP / IFRS / PCG compliant financial statements derived from
            the generated journal entries and trial balances. Statements are
            internally consistent with the underlying transaction data.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates balance_sheet.csv, income_statement.csv,
            cash_flow_statement.csv, changes_in_equity.csv,
            financial_kpis.csv, and budget_variance.csv.
          </p>
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
  .input-with-suffix input { flex: 1; }
  .suffix { font-size: 0.8125rem; color: var(--color-text-muted); font-family: var(--font-mono); }
  .info-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid { grid-template-columns: 1fr; } }
</style>
