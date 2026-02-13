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
  <ConfigPageHeader title="Sales Quotes" description="Configure sales quote pipeline and conversion settings" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Sales Quotes Module" description="Enable sales quote pipeline generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.sales_quotes.enabled}
              label="Enable Sales Quotes"
              description="Generate sales quotes with line items, pricing, and conversion tracking"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.sales_quotes.enabled}
        <FormSection title="Pipeline Volume" description="Configure quote generation volume and timing">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Avg Quotes per Month"
                htmlFor="quotes-per-month"
                helpText="Average number of quotes generated per month"
                error={getError('sales_quotes.avg_quotes_per_month')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="quotes-per-month"
                    bind:value={$config.sales_quotes.avg_quotes_per_month}
                    min="1"
                    max="10000"
                    step="1"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Validity Days"
                htmlFor="validity-days"
                helpText="Number of days a quote remains valid"
                error={getError('sales_quotes.validity_days')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="validity-days"
                      bind:value={$config.sales_quotes.validity_days}
                      min="1"
                      max="365"
                      step="1"
                    />
                    <span class="suffix">days</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Line Items"
                htmlFor="avg-line-items"
                helpText="Average number of line items per quote"
                error={getError('sales_quotes.avg_line_items')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="avg-line-items"
                    bind:value={$config.sales_quotes.avg_line_items}
                    min="1"
                    max="100"
                    step="1"
                  />
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Conversion & Pricing" description="Configure conversion rates and discount settings">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Conversion Rate"
                htmlFor="conversion-rate"
                helpText="Proportion of quotes that convert to sales orders (0-100%)"
                error={getError('sales_quotes.conversion_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="conversion-rate"
                      bind:value={$config.sales_quotes.conversion_rate}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span class="suffix">{($config.sales_quotes.conversion_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Discount Rate"
                htmlFor="discount-rate"
                helpText="Average discount applied to quoted prices (0-100%)"
                error={getError('sales_quotes.discount_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="discount-rate"
                      bind:value={$config.sales_quotes.discount_rate}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span class="suffix">{($config.sales_quotes.discount_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Revision Rate"
                htmlFor="revision-rate"
                helpText="Proportion of quotes that undergo revision before decision (0-100%)"
                error={getError('sales_quotes.revision_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="revision-rate"
                      bind:value={$config.sales_quotes.revision_rate}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span class="suffix">{($config.sales_quotes.revision_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Quote Pipeline</h4>
          <p>
            Generates realistic sales quote pipelines with multi-line items,
            customer-specific pricing, discount negotiations, and quote
            revision tracking through to conversion or expiry.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates sales_quotes.csv and sales_quote_items.csv with
            full lifecycle tracking from draft through won, lost, or expired.
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
