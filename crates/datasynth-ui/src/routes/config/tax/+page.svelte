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
  <ConfigPageHeader title="Tax Accounting" description="Configure VAT/GST, sales tax, withholding, and tax provisions" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Tax Module" description="Enable tax accounting data generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.tax.enabled}
              label="Enable Tax Accounting"
              description="Generate tax jurisdiction records, VAT/GST, withholding tax, and provisions"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.tax.enabled}
        <FormSection title="Tax Jurisdictions" description="Configure tax types by jurisdiction">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.tax.vat_gst.enabled}
                label="VAT / GST"
                description="Generate Value Added Tax or Goods & Services Tax records"
              />

              {#if $config.tax.vat_gst.enabled}
                <Toggle
                  bind:checked={$config.tax.vat_gst.reverse_charge}
                  label="Reverse Charge Mechanism"
                  description="Enable reverse charge for cross-border B2B transactions"
                />
              {/if}

              <Toggle
                bind:checked={$config.tax.sales_tax.enabled}
                label="Sales Tax"
                description="Generate US-style sales tax with nexus calculations"
              />

              <Toggle
                bind:checked={$config.tax.payroll_tax.enabled}
                label="Payroll Tax"
                description="Generate payroll tax withholding and employer contributions"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Withholding Tax" description="Configure withholding tax rates and treaty networks">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.tax.withholding.enabled}
                label="Enable Withholding Tax"
                description="Generate withholding tax records for cross-border payments"
              />

              {#if $config.tax.withholding.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Default Rate"
                    htmlFor="wht-default-rate"
                    helpText="Standard withholding tax rate for non-treaty countries"
                    error={getError('tax.withholding.default_rate')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input
                          type="number"
                          id="wht-default-rate"
                          bind:value={$config.tax.withholding.default_rate}
                          min="0"
                          max="1"
                          step="0.01"
                        />
                        <span class="suffix">{($config.tax.withholding.default_rate * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Treaty Reduced Rate"
                    htmlFor="wht-treaty-rate"
                    helpText="Reduced rate applied under tax treaties"
                    error={getError('tax.withholding.treaty_reduced_rate')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input
                          type="number"
                          id="wht-treaty-rate"
                          bind:value={$config.tax.withholding.treaty_reduced_rate}
                          min="0"
                          max="1"
                          step="0.01"
                        />
                        <span class="suffix">{($config.tax.withholding.treaty_reduced_rate * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>

                <Toggle
                  bind:checked={$config.tax.withholding.treaty_network}
                  label="Treaty Network"
                  description="Generate treaty-based reduced rates across country pairs"
                />
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Tax Provisions" description="Configure tax provision and uncertain tax position settings">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.tax.provisions.enabled}
                label="Enable Tax Provisions"
                description="Generate current/deferred tax provisions and effective tax rate reconciliation"
              />

              {#if $config.tax.provisions.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Statutory Rate"
                    htmlFor="statutory-rate"
                    helpText="Corporate statutory tax rate"
                    error={getError('tax.provisions.statutory_rate')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input
                          type="number"
                          id="statutory-rate"
                          bind:value={$config.tax.provisions.statutory_rate}
                          min="0"
                          max="1"
                          step="0.01"
                        />
                        <span class="suffix">{($config.tax.provisions.statutory_rate * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>

                <Toggle
                  bind:checked={$config.tax.provisions.uncertain_positions}
                  label="Uncertain Tax Positions"
                  description="Generate ASC 740-10 / IAS 12 uncertain tax position records"
                />
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Anomaly Rate" description="Tax-specific anomaly injection">
          {#snippet children()}
            <FormGroup
              label="Anomaly Rate"
              htmlFor="tax-anomaly"
              helpText="Rate of anomalous tax records (0-100%)"
              error={getError('tax.anomaly_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="tax-anomaly"
                    bind:value={$config.tax.anomaly_rate}
                    min="0"
                    max="1"
                    step="0.005"
                  />
                  <span class="suffix">{($config.tax.anomaly_rate * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Tax Compliance</h4>
          <p>
            Generates multi-jurisdiction tax data including VAT/GST with reverse charge,
            US sales tax with nexus, withholding tax with treaty networks, payroll taxes,
            and ASC 740 / IAS 12 tax provisions with uncertain positions.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates tax_jurisdictions.csv, vat_returns.csv, sales_tax_returns.csv,
            withholding_certificates.csv, tax_provisions.csv, and uncertain_tax_positions.csv.
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
