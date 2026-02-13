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
  <ConfigPageHeader title="HR / Payroll" description="Configure payroll, time tracking, and expense generation" />

  {#if $config}
    <div class="page-content">
      <FormSection title="HR Module" description="Enable HR and payroll data generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.hr.enabled}
              label="Enable HR / Payroll"
              description="Generate payroll runs, time entries, and expense reports"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.hr.enabled}
        <FormSection title="Process Modules" description="Enable or disable individual HR sub-processes">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.hr.time_tracking}
                label="Time Tracking"
                description="Generate employee time entry records"
              />

              <Toggle
                bind:checked={$config.hr.expenses}
                label="Expense Reports"
                description="Generate employee expense reports with line items"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Payroll Settings" description="Configure payroll frequency and rates">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Payroll Frequency"
                htmlFor="payroll-freq"
                helpText="How often payroll is processed"
                error={getError('hr.payroll_frequency')}
              >
                {#snippet children()}
                  <select id="payroll-freq" bind:value={$config.hr.payroll_frequency}>
                    <option value="weekly">Weekly</option>
                    <option value="bi_weekly">Bi-Weekly</option>
                    <option value="semi_monthly">Semi-Monthly</option>
                    <option value="monthly">Monthly</option>
                  </select>
                {/snippet}
              </FormGroup>

              <div class="form-grid">
                <FormGroup
                  label="Overtime Rate"
                  htmlFor="overtime-rate"
                  helpText="Overtime multiplier as a proportion (e.g. 0.50 = 1.5x)"
                  error={getError('hr.overtime_rate')}
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input
                        type="number"
                        id="overtime-rate"
                        bind:value={$config.hr.overtime_rate}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span class="suffix">{($config.hr.overtime_rate * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Benefits Rate"
                  htmlFor="benefits-rate"
                  helpText="Benefits cost as a proportion of base salary"
                  error={getError('hr.benefits_rate')}
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input
                        type="number"
                        id="benefits-rate"
                        bind:value={$config.hr.benefits_rate}
                        min="0"
                        max="1"
                        step="0.01"
                      />
                      <span class="suffix">{($config.hr.benefits_rate * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Expense Settings" description="Configure expense report parameters">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Avg Expense Amount"
                htmlFor="avg-expense"
                helpText="Average amount per expense line item"
                error={getError('hr.avg_expense_amount')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="avg-expense"
                      bind:value={$config.hr.avg_expense_amount}
                      min="0"
                      step="10"
                      disabled={!$config.hr.expenses}
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Expense Approval Threshold"
                htmlFor="expense-threshold"
                helpText="Amount above which expenses require additional approval"
                error={getError('hr.expense_approval_threshold')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="expense-threshold"
                      bind:value={$config.hr.expense_approval_threshold}
                      min="0"
                      step="100"
                      disabled={!$config.hr.expenses}
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Payroll Processing</h4>
          <p>
            Generates payroll runs with line items including base salary,
            overtime, deductions, taxes, and benefits. Supports configurable
            pay frequencies and overtime calculations.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates payroll_runs.csv, payslips.csv, time_entries.csv,
            expense_reports.csv, and expense_line_items.csv.
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
