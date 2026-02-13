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
  <ConfigPageHeader title="Procure to Pay (P2P)" description="Configure the procurement-to-payment document flow" />

  {#if $config?.document_flows?.p2p}
    {@const p2p = $config.document_flows.p2p}
    <div class="page-content">
      <div class="flow-diagram">
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2" />
              <path d="M9 5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2 2 2 0 0 0-2-2h-2a2 2 0 0 0-2 2z" />
            </svg>
          </div>
          <span class="step-label">Purchase Order</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{p2p.average_po_to_gr_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
              <path d="M3.27 6.96L12 12.01l8.73-5.05M12 22.08V12" />
            </svg>
          </div>
          <span class="step-label">Goods Receipt</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{p2p.average_gr_to_invoice_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <path d="M14 2v6h6M16 13H8M16 17H8M10 9H8" />
            </svg>
          </div>
          <span class="step-label">Invoice Receipt</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{p2p.average_invoice_to_payment_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
            </svg>
          </div>
          <span class="step-label">Payment</span>
        </div>
      </div>

      <FormSection title="General Settings" description="Enable or disable P2P flow generation">
        {#snippet children()}
          <Toggle
            bind:checked={p2p.enabled}
            label="Enable P2P Flow"
            description="Generate complete Procure-to-Pay document chains"
          />
        {/snippet}
      </FormSection>

      <FormSection title="Three-Way Matching" description="Configure PO-GR-Invoice matching rates">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Three-Way Match Rate"
              htmlFor="three-way-match"
              helpText="Percentage of invoices that perfectly match PO and GR (0-100%)"
              error={getError('document_flows.p2p.three_way_match_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="three-way-match"
                    bind:value={p2p.three_way_match_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.three_way_match_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Price Variance Rate"
              htmlFor="price-variance"
              helpText="Rate of invoices with price differences from PO"
              error={getError('document_flows.p2p.price_variance_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="price-variance"
                    bind:value={p2p.price_variance_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.price_variance_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Max Price Variance"
              htmlFor="max-price-variance"
              helpText="Maximum price deviation when variances occur"
              error={getError('document_flows.p2p.max_price_variance_percent')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="max-price-variance"
                    bind:value={p2p.max_price_variance_percent}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.max_price_variance_percent * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Quantity Variance Rate"
              htmlFor="qty-variance"
              helpText="Rate of invoices with quantity differences"
              error={getError('document_flows.p2p.quantity_variance_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="qty-variance"
                    bind:value={p2p.quantity_variance_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.quantity_variance_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Delivery Settings" description="Configure goods receipt behavior">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Partial Delivery Rate"
              htmlFor="partial-delivery"
              helpText="Percentage of orders with partial deliveries"
              error={getError('document_flows.p2p.partial_delivery_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="partial-delivery"
                    bind:value={p2p.partial_delivery_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.partial_delivery_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Timing Configuration" description="Average days between document steps">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="PO to Goods Receipt"
              htmlFor="po-to-gr"
              helpText="Average days from purchase order to goods receipt"
              error={getError('document_flows.p2p.average_po_to_gr_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="po-to-gr"
                    bind:value={p2p.average_po_to_gr_days}
                    min="0"
                    max="365"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="GR to Invoice"
              htmlFor="gr-to-invoice"
              helpText="Average days from goods receipt to invoice receipt"
              error={getError('document_flows.p2p.average_gr_to_invoice_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="gr-to-invoice"
                    bind:value={p2p.average_gr_to_invoice_days}
                    min="0"
                    max="365"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Invoice to Payment"
              htmlFor="invoice-to-payment"
              helpText="Average days from invoice to payment (payment terms)"
              error={getError('document_flows.p2p.average_invoice_to_payment_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="invoice-to-payment"
                    bind:value={p2p.average_invoice_to_payment_days}
                    min="0"
                    max="365"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Line Items" description="Configure PO line count distribution">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Minimum Lines"
              htmlFor="min-lines"
              helpText="Minimum number of lines per PO"
              error={getError('document_flows.p2p.line_count_distribution.min_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="min-lines"
                  bind:value={p2p.line_count_distribution.min_lines}
                  min="1"
                  max="100"
                  disabled={!p2p.enabled}
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Maximum Lines"
              htmlFor="max-lines"
              helpText="Maximum number of lines per PO"
              error={getError('document_flows.p2p.line_count_distribution.max_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="max-lines"
                  bind:value={p2p.line_count_distribution.max_lines}
                  min="1"
                  max="100"
                  disabled={!p2p.enabled}
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Most Common (Mode)"
              htmlFor="mode-lines"
              helpText="Most common line count"
              error={getError('document_flows.p2p.line_count_distribution.mode_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="mode-lines"
                  bind:value={p2p.line_count_distribution.mode_lines}
                  min="1"
                  max="100"
                  disabled={!p2p.enabled}
                />
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Payment Behavior" description="Configure realistic payment patterns including late payments and corrections">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Late Payment Rate"
              htmlFor="late-payment-rate"
              helpText="Percentage of payments made after due date"
              error={getError('document_flows.p2p.payment_behavior.late_payment_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="late-payment-rate"
                    bind:value={p2p.payment_behavior.late_payment_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.payment_behavior.late_payment_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Partial Payment Rate"
              htmlFor="partial-payment-rate"
              helpText="Percentage of invoices paid in multiple installments"
              error={getError('document_flows.p2p.payment_behavior.partial_payment_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="partial-payment-rate"
                    bind:value={p2p.payment_behavior.partial_payment_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.payment_behavior.partial_payment_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Payment Correction Rate"
              htmlFor="correction-rate"
              helpText="Percentage of payments requiring correction (NSF, chargebacks)"
              error={getError('document_flows.p2p.payment_behavior.payment_correction_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="correction-rate"
                    bind:value={p2p.payment_behavior.payment_correction_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!p2p.enabled}
                  />
                  <span class="suffix">{(p2p.payment_behavior.payment_correction_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Late Payment Distribution" description="Distribution of how late payments are made">
        {#snippet children()}
          <div class="distribution-grid">
            <div class="distribution-item">
              <FormGroup
                label="1-7 Days Late"
                htmlFor="late-1-7"
                helpText="Slightly late payments"
                error={getError('document_flows.p2p.payment_behavior.late_payment_days_distribution.slightly_late_1_to_7')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="late-1-7"
                      bind:value={p2p.payment_behavior.late_payment_days_distribution.slightly_late_1_to_7}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!p2p.enabled}
                    />
                    <span class="suffix">{(p2p.payment_behavior.late_payment_days_distribution.slightly_late_1_to_7 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <div class="distribution-item">
              <FormGroup
                label="8-14 Days Late"
                htmlFor="late-8-14"
                helpText="Moderately late"
                error={getError('document_flows.p2p.payment_behavior.late_payment_days_distribution.late_8_to_14')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="late-8-14"
                      bind:value={p2p.payment_behavior.late_payment_days_distribution.late_8_to_14}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!p2p.enabled}
                    />
                    <span class="suffix">{(p2p.payment_behavior.late_payment_days_distribution.late_8_to_14 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <div class="distribution-item">
              <FormGroup
                label="15-30 Days Late"
                htmlFor="late-15-30"
                helpText="Very late payments"
                error={getError('document_flows.p2p.payment_behavior.late_payment_days_distribution.very_late_15_to_30')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="late-15-30"
                      bind:value={p2p.payment_behavior.late_payment_days_distribution.very_late_15_to_30}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!p2p.enabled}
                    />
                    <span class="suffix">{(p2p.payment_behavior.late_payment_days_distribution.very_late_15_to_30 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <div class="distribution-item">
              <FormGroup
                label="31-60 Days Late"
                htmlFor="late-31-60"
                helpText="Severely late"
                error={getError('document_flows.p2p.payment_behavior.late_payment_days_distribution.severely_late_31_to_60')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="late-31-60"
                      bind:value={p2p.payment_behavior.late_payment_days_distribution.severely_late_31_to_60}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!p2p.enabled}
                    />
                    <span class="suffix">{(p2p.payment_behavior.late_payment_days_distribution.severely_late_31_to_60 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <div class="distribution-item">
              <FormGroup
                label="Over 60 Days Late"
                htmlFor="late-over-60"
                helpText="Extremely late"
                error={getError('document_flows.p2p.payment_behavior.late_payment_days_distribution.extremely_late_over_60')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="late-over-60"
                      bind:value={p2p.payment_behavior.late_payment_days_distribution.extremely_late_over_60}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!p2p.enabled}
                    />
                    <span class="suffix">{(p2p.payment_behavior.late_payment_days_distribution.extremely_late_over_60 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          </div>
          <p class="distribution-note">
            Distribution must sum to 100%
          </p>
        {/snippet}
      </FormSection>
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

  .flow-diagram {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-5);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    flex-wrap: wrap;
  }

  .flow-step {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }

  .step-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    color: var(--color-accent);
  }

  .step-icon svg {
    width: 24px;
    height: 24px;
  }

  .step-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-align: center;
    white-space: nowrap;
  }

  .flow-arrow {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
    color: var(--color-text-muted);
  }

  .flow-arrow svg {
    width: 32px;
    height: 32px;
  }

  .days-label {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
    color: var(--color-accent);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .distribution-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: var(--space-3);
  }

  .distribution-item {
    min-width: 0;
  }

  .distribution-note {
    margin-top: var(--space-3);
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-align: center;
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

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid {
      grid-template-columns: 1fr;
    }

    .flow-diagram {
      flex-direction: column;
    }

    .flow-arrow {
      transform: rotate(90deg);
    }
  }
</style>
