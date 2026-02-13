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
  <ConfigPageHeader title="Order to Cash (O2C)" description="Configure the sales order-to-cash receipt document flow" />

  {#if $config?.document_flows?.o2c}
    {@const o2c = $config.document_flows.o2c}
    <div class="page-content">
      <div class="flow-diagram">
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
              <path d="M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1z" />
            </svg>
          </div>
          <span class="step-label">Sales Order</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{o2c.average_so_to_delivery_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 3h15v13H1zM16 8h4l3 3v5h-7V8z" />
              <circle cx="5.5" cy="18.5" r="2.5" />
              <circle cx="18.5" cy="18.5" r="2.5" />
            </svg>
          </div>
          <span class="step-label">Delivery</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{o2c.average_delivery_to_invoice_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <path d="M14 2v6h6M16 13H8M16 17H8M10 9H8" />
            </svg>
          </div>
          <span class="step-label">Customer Invoice</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{o2c.average_invoice_to_receipt_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
            </svg>
          </div>
          <span class="step-label">Cash Receipt</span>
        </div>
      </div>

      <FormSection title="General Settings" description="Enable or disable O2C flow generation">
        {#snippet children()}
          <Toggle
            bind:checked={o2c.enabled}
            label="Enable O2C Flow"
            description="Generate complete Order-to-Cash document chains"
          />
        {/snippet}
      </FormSection>

      <FormSection title="Credit & Risk" description="Configure credit check and bad debt behavior">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Credit Check Failure Rate"
              htmlFor="credit-fail"
              helpText="Percentage of orders that fail credit check"
              error={getError('document_flows.o2c.credit_check_failure_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="credit-fail"
                    bind:value={o2c.credit_check_failure_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.credit_check_failure_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Bad Debt Rate"
              htmlFor="bad-debt"
              helpText="Percentage of invoices written off as uncollectible"
              error={getError('document_flows.o2c.bad_debt_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="bad-debt"
                    bind:value={o2c.bad_debt_rate}
                    min="0"
                    max="1"
                    step="0.001"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.bad_debt_rate * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Fulfillment Settings" description="Configure delivery and returns behavior">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Partial Shipment Rate"
              htmlFor="partial-ship"
              helpText="Percentage of orders with partial shipments"
              error={getError('document_flows.o2c.partial_shipment_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="partial-ship"
                    bind:value={o2c.partial_shipment_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.partial_shipment_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Return Rate"
              htmlFor="return-rate"
              helpText="Percentage of orders with customer returns"
              error={getError('document_flows.o2c.return_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="return-rate"
                    bind:value={o2c.return_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.return_rate * 100).toFixed(0)}%</span>
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
              label="SO to Delivery"
              htmlFor="so-to-delivery"
              helpText="Average days from sales order to delivery"
              error={getError('document_flows.o2c.average_so_to_delivery_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="so-to-delivery"
                    bind:value={o2c.average_so_to_delivery_days}
                    min="0"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Delivery to Invoice"
              htmlFor="delivery-to-invoice"
              helpText="Average days from delivery to customer invoice"
              error={getError('document_flows.o2c.average_delivery_to_invoice_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="delivery-to-invoice"
                    bind:value={o2c.average_delivery_to_invoice_days}
                    min="0"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Invoice to Receipt"
              htmlFor="invoice-to-receipt"
              helpText="Average days from invoice to cash receipt"
              error={getError('document_flows.o2c.average_invoice_to_receipt_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="invoice-to-receipt"
                    bind:value={o2c.average_invoice_to_receipt_days}
                    min="0"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Cash Discounts" description="Early payment discount settings">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Eligible Rate"
              htmlFor="discount-eligible"
              helpText="Percentage of invoices eligible for early payment discount"
              error={getError('document_flows.o2c.cash_discount.eligible_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-eligible"
                    bind:value={o2c.cash_discount.eligible_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.cash_discount.eligible_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Discount Taken Rate"
              htmlFor="discount-taken"
              helpText="Percentage of customers who take the discount"
              error={getError('document_flows.o2c.cash_discount.taken_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-taken"
                    bind:value={o2c.cash_discount.taken_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.cash_discount.taken_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Discount Percentage"
              htmlFor="discount-percent"
              helpText="Discount percentage for early payment"
              error={getError('document_flows.o2c.cash_discount.discount_percent')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-percent"
                    bind:value={o2c.cash_discount.discount_percent}
                    min="0"
                    max="1"
                    step="0.005"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.cash_discount.discount_percent * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Discount Window"
              htmlFor="discount-days"
              helpText="Days within which discount must be taken"
              error={getError('document_flows.o2c.cash_discount.discount_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-days"
                    bind:value={o2c.cash_discount.discount_days}
                    min="0"
                    max="90"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Line Items" description="Configure SO line count distribution">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Minimum Lines"
              htmlFor="min-lines"
              helpText="Minimum number of lines per SO"
              error={getError('document_flows.o2c.line_count_distribution.min_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="min-lines"
                  bind:value={o2c.line_count_distribution.min_lines}
                  min="1"
                  max="100"
                  disabled={!o2c.enabled}
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Maximum Lines"
              htmlFor="max-lines"
              helpText="Maximum number of lines per SO"
              error={getError('document_flows.o2c.line_count_distribution.max_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="max-lines"
                  bind:value={o2c.line_count_distribution.max_lines}
                  min="1"
                  max="100"
                  disabled={!o2c.enabled}
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Most Common (Mode)"
              htmlFor="mode-lines"
              helpText="Most common line count"
              error={getError('document_flows.o2c.line_count_distribution.mode_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="mode-lines"
                  bind:value={o2c.line_count_distribution.mode_lines}
                  min="1"
                  max="100"
                  disabled={!o2c.enabled}
                />
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <!-- Payment Behavior Section -->
      <div class="section-divider">
        <h2>Payment Behavior</h2>
        <p>Configure realistic customer payment patterns</p>
      </div>

      <FormSection title="Dunning (Mahnungen)" description="Configure the dunning/reminder process for overdue invoices">
        {#snippet children()}
          <Toggle
            bind:checked={o2c.payment_behavior.dunning.enabled}
            label="Enable Dunning Process"
            description="Generate dunning letters for overdue invoices"
          />

          {#if o2c.payment_behavior.dunning.enabled}
            <div class="form-grid" style="margin-top: var(--space-4);">
              <FormGroup
                label="Level 1 (1st Reminder)"
                htmlFor="dunning-level-1"
                helpText="Days overdue before 1st reminder"
                error={getError('document_flows.o2c.payment_behavior.dunning.level_1_days_overdue')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="dunning-level-1"
                      bind:value={o2c.payment_behavior.dunning.level_1_days_overdue}
                      min="1"
                      max="365"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">days</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Level 2 (2nd Reminder)"
                htmlFor="dunning-level-2"
                helpText="Days overdue before 2nd reminder"
                error={getError('document_flows.o2c.payment_behavior.dunning.level_2_days_overdue')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="dunning-level-2"
                      bind:value={o2c.payment_behavior.dunning.level_2_days_overdue}
                      min="1"
                      max="365"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">days</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Level 3 (Final Notice)"
                htmlFor="dunning-level-3"
                helpText="Days overdue before final notice"
                error={getError('document_flows.o2c.payment_behavior.dunning.level_3_days_overdue')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="dunning-level-3"
                      bind:value={o2c.payment_behavior.dunning.level_3_days_overdue}
                      min="1"
                      max="365"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">days</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Collection Handover"
                htmlFor="dunning-collection"
                helpText="Days overdue before collection agency"
                error={getError('document_flows.o2c.payment_behavior.dunning.collection_days_overdue')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="dunning-collection"
                      bind:value={o2c.payment_behavior.dunning.collection_days_overdue}
                      min="1"
                      max="365"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">days</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Annual Interest Rate"
                htmlFor="dunning-interest"
                helpText="Interest charged on overdue amounts"
                error={getError('document_flows.o2c.payment_behavior.dunning.interest_rate_per_year')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="dunning-interest"
                      bind:value={o2c.payment_behavior.dunning.interest_rate_per_year}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">{(o2c.payment_behavior.dunning.interest_rate_per_year * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Dunning Charge"
                htmlFor="dunning-charge"
                helpText="Fixed charge per dunning letter"
                error={getError('document_flows.o2c.payment_behavior.dunning.dunning_charge')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="dunning-charge"
                      bind:value={o2c.payment_behavior.dunning.dunning_charge}
                      min="0"
                      max="1000"
                      step="1"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Dunning Block Rate"
                htmlFor="dunning-block"
                helpText="Rate of invoices blocked from dunning (disputes)"
                error={getError('document_flows.o2c.payment_behavior.dunning.dunning_block_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="dunning-block"
                      bind:value={o2c.payment_behavior.dunning.dunning_block_rate}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">{(o2c.payment_behavior.dunning.dunning_block_rate * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <h4 class="subsection-title">Payment Response Rates</h4>
            <p class="subsection-description">Percentage of customers who pay after each dunning level</p>
            <div class="form-grid">
              <FormGroup
                label="After Level 1"
                htmlFor="pay-after-1"
                helpText="Pay after 1st reminder"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="pay-after-1"
                      bind:value={o2c.payment_behavior.dunning.payment_after_dunning_rates.after_level_1}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">{(o2c.payment_behavior.dunning.payment_after_dunning_rates.after_level_1 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="After Level 2"
                htmlFor="pay-after-2"
                helpText="Pay after 2nd reminder"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="pay-after-2"
                      bind:value={o2c.payment_behavior.dunning.payment_after_dunning_rates.after_level_2}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">{(o2c.payment_behavior.dunning.payment_after_dunning_rates.after_level_2 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="After Level 3"
                htmlFor="pay-after-3"
                helpText="Pay after final notice"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="pay-after-3"
                      bind:value={o2c.payment_behavior.dunning.payment_after_dunning_rates.after_level_3}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">{(o2c.payment_behavior.dunning.payment_after_dunning_rates.after_level_3 * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="During Collection"
                htmlFor="pay-collection"
                helpText="Pay during collection"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="pay-collection"
                      bind:value={o2c.payment_behavior.dunning.payment_after_dunning_rates.during_collection}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">{(o2c.payment_behavior.dunning.payment_after_dunning_rates.during_collection * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Never Pay"
                htmlFor="never-pay"
                helpText="Becomes bad debt"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="never-pay"
                      bind:value={o2c.payment_behavior.dunning.payment_after_dunning_rates.never_pay}
                      min="0"
                      max="1"
                      step="0.01"
                      disabled={!o2c.enabled}
                    />
                    <span class="suffix">{(o2c.payment_behavior.dunning.payment_after_dunning_rates.never_pay * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
            <p class="distribution-note">Response rates must sum to 100%</p>
          {/if}
        {/snippet}
      </FormSection>

      <FormSection title="Partial Payments" description="Configure partial payment behavior">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Partial Payment Rate"
              htmlFor="partial-rate"
              helpText="Percentage of invoices paid in installments"
              error={getError('document_flows.o2c.payment_behavior.partial_payments.rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="partial-rate"
                    bind:value={o2c.payment_behavior.partial_payments.rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.payment_behavior.partial_payments.rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Days Until Remainder"
              htmlFor="remainder-days"
              helpText="Average days until remaining balance is paid"
              error={getError('document_flows.o2c.payment_behavior.partial_payments.avg_days_until_remainder')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="remainder-days"
                    bind:value={o2c.payment_behavior.partial_payments.avg_days_until_remainder}
                    min="1"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Short Payments" description="Configure unauthorized deductions and disputes">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Short Payment Rate"
              htmlFor="short-rate"
              helpText="Percentage of payments with unauthorized deductions"
              error={getError('document_flows.o2c.payment_behavior.short_payments.rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="short-rate"
                    bind:value={o2c.payment_behavior.short_payments.rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.payment_behavior.short_payments.rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Max Short Amount"
              htmlFor="max-short"
              helpText="Maximum deduction as percentage of invoice"
              error={getError('document_flows.o2c.payment_behavior.short_payments.max_short_percent')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="max-short"
                    bind:value={o2c.payment_behavior.short_payments.max_short_percent}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.payment_behavior.short_payments.max_short_percent * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="On-Account Payments" description="Configure unapplied customer payments">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="On-Account Rate"
              htmlFor="on-account-rate"
              helpText="Percentage of payments not matched to invoices"
              error={getError('document_flows.o2c.payment_behavior.on_account_payments.rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="on-account-rate"
                    bind:value={o2c.payment_behavior.on_account_payments.rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.payment_behavior.on_account_payments.rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Application Days"
              htmlFor="application-days"
              helpText="Average days until on-account payment is applied"
              error={getError('document_flows.o2c.payment_behavior.on_account_payments.avg_days_until_application')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="application-days"
                    bind:value={o2c.payment_behavior.on_account_payments.avg_days_until_application}
                    min="1"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Payment Corrections" description="Configure NSF, chargebacks, and reversals">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Correction Rate"
              htmlFor="correction-rate"
              helpText="Percentage of payments requiring correction"
              error={getError('document_flows.o2c.payment_behavior.payment_corrections.rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="correction-rate"
                    bind:value={o2c.payment_behavior.payment_corrections.rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.payment_behavior.payment_corrections.rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Resolution Days"
              htmlFor="resolution-days"
              helpText="Average days to resolve a correction"
              error={getError('document_flows.o2c.payment_behavior.payment_corrections.avg_resolution_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="resolution-days"
                    bind:value={o2c.payment_behavior.payment_corrections.avg_resolution_days}
                    min="1"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
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

  .section-divider {
    padding: var(--space-4) 0;
    border-top: 1px solid var(--color-border);
    margin-top: var(--space-4);
  }

  .section-divider h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: var(--space-1);
  }

  .section-divider p {
    color: var(--color-text-secondary);
    font-size: 0.875rem;
  }

  .subsection-title {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-top: var(--space-5);
    margin-bottom: var(--space-1);
  }

  .subsection-description {
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    margin-bottom: var(--space-3);
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
