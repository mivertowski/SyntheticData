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

  function getLifecycleTotal(): number {
    if (!$config?.customer_segmentation?.lifecycle) return 0;
    const l = $config.customer_segmentation.lifecycle;
    return (
      l.prospect_rate +
      l.new_rate +
      l.growth_rate +
      l.mature_rate +
      l.at_risk_rate +
      l.churned_rate +
      l.won_back_rate
    );
  }
</script>

<div class="page">
  <ConfigPageHeader title="Customer Segmentation" description="Configure customer value segments, lifecycle stages, and network effects" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Segmentation Settings" description="Enable customer segmentation and value-based modeling">
        {#snippet children()}
          <Toggle
            bind:checked={$config.customer_segmentation.enabled}
            label="Enable Customer Segmentation"
            description="Generate segmented customers with value tiers, lifecycle stages, and network positions"
          />
        {/snippet}
      </FormSection>

      {#if $config.customer_segmentation.enabled}
        <FormSection title="Enterprise Segment" description="High-value enterprise customers">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Revenue Share"
                htmlFor="ent-revenue"
                helpText="Proportion of total revenue from enterprise customers"
                error={getError('customer_segmentation.value_segments.enterprise.revenue_share')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="ent-revenue"
                      bind:value={$config.customer_segmentation.value_segments.enterprise.revenue_share}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.customer_segmentation.value_segments.enterprise.revenue_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Customer Share"
                htmlFor="ent-customer"
                helpText="Proportion of customers in this segment"
                error={getError('customer_segmentation.value_segments.enterprise.customer_share')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="ent-customer"
                      bind:value={$config.customer_segmentation.value_segments.enterprise.customer_share}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span>{($config.customer_segmentation.value_segments.enterprise.customer_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Min"
                htmlFor="ent-order-min"
                helpText="Minimum average order value"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="ent-order-min"
                      bind:value={$config.customer_segmentation.value_segments.enterprise.avg_order_min}
                      min="0"
                      step="1000"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Max"
                htmlFor="ent-order-max"
                helpText="Maximum average order value (optional)"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="ent-order-max"
                      bind:value={$config.customer_segmentation.value_segments.enterprise.avg_order_max}
                      min="0"
                      step="1000"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Mid-Market Segment" description="Mid-sized business customers">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Revenue Share"
                htmlFor="mm-revenue"
                helpText="Proportion of total revenue from mid-market customers"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="mm-revenue"
                      bind:value={$config.customer_segmentation.value_segments.mid_market.revenue_share}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.customer_segmentation.value_segments.mid_market.revenue_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Customer Share"
                htmlFor="mm-customer"
                helpText="Proportion of customers in this segment"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="mm-customer"
                      bind:value={$config.customer_segmentation.value_segments.mid_market.customer_share}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span>{($config.customer_segmentation.value_segments.mid_market.customer_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Min"
                htmlFor="mm-order-min"
                helpText="Minimum average order value"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="mm-order-min"
                      bind:value={$config.customer_segmentation.value_segments.mid_market.avg_order_min}
                      min="0"
                      step="500"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Max"
                htmlFor="mm-order-max"
                helpText="Maximum average order value"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="mm-order-max"
                      bind:value={$config.customer_segmentation.value_segments.mid_market.avg_order_max}
                      min="0"
                      step="500"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="SMB Segment" description="Small and medium business customers">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Revenue Share"
                htmlFor="smb-revenue"
                helpText="Proportion of total revenue from SMB customers"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="smb-revenue"
                      bind:value={$config.customer_segmentation.value_segments.smb.revenue_share}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.customer_segmentation.value_segments.smb.revenue_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Customer Share"
                htmlFor="smb-customer"
                helpText="Proportion of customers in this segment"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="smb-customer"
                      bind:value={$config.customer_segmentation.value_segments.smb.customer_share}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span>{($config.customer_segmentation.value_segments.smb.customer_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Min"
                htmlFor="smb-order-min"
                helpText="Minimum average order value"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="smb-order-min"
                      bind:value={$config.customer_segmentation.value_segments.smb.avg_order_min}
                      min="0"
                      step="100"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Max"
                htmlFor="smb-order-max"
                helpText="Maximum average order value"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="smb-order-max"
                      bind:value={$config.customer_segmentation.value_segments.smb.avg_order_max}
                      min="0"
                      step="100"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Consumer Segment" description="Individual consumer customers">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Revenue Share"
                htmlFor="con-revenue"
                helpText="Proportion of total revenue from consumer customers"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="con-revenue"
                      bind:value={$config.customer_segmentation.value_segments.consumer.revenue_share}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.customer_segmentation.value_segments.consumer.revenue_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Customer Share"
                htmlFor="con-customer"
                helpText="Proportion of customers in this segment"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="con-customer"
                      bind:value={$config.customer_segmentation.value_segments.consumer.customer_share}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span>{($config.customer_segmentation.value_segments.consumer.customer_share * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Min"
                htmlFor="con-order-min"
                helpText="Minimum average order value"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="con-order-min"
                      bind:value={$config.customer_segmentation.value_segments.consumer.avg_order_min}
                      min="0"
                      step="10"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Order Max"
                htmlFor="con-order-max"
                helpText="Maximum average order value"
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="con-order-max"
                      bind:value={$config.customer_segmentation.value_segments.consumer.avg_order_max}
                      min="0"
                      step="10"
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Customer Lifecycle Distribution" description="Distribution of customers across lifecycle stages (should sum to 100%)">
          {#snippet children()}
            <div class="distribution-grid">
              <div class="distribution-item">
                <label>Prospect</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.customer_segmentation.lifecycle.prospect_rate}
                    min="0"
                    max="1"
                    step="0.01"
                  />
                  <span>{($config.customer_segmentation.lifecycle.prospect_rate * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>New</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.customer_segmentation.lifecycle.new_rate}
                    min="0"
                    max="1"
                    step="0.01"
                  />
                  <span>{($config.customer_segmentation.lifecycle.new_rate * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Growth</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.customer_segmentation.lifecycle.growth_rate}
                    min="0"
                    max="1"
                    step="0.01"
                  />
                  <span>{($config.customer_segmentation.lifecycle.growth_rate * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Mature</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.customer_segmentation.lifecycle.mature_rate}
                    min="0"
                    max="1"
                    step="0.01"
                  />
                  <span>{($config.customer_segmentation.lifecycle.mature_rate * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>At Risk</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.customer_segmentation.lifecycle.at_risk_rate}
                    min="0"
                    max="1"
                    step="0.01"
                  />
                  <span>{($config.customer_segmentation.lifecycle.at_risk_rate * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Churned</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.customer_segmentation.lifecycle.churned_rate}
                    min="0"
                    max="1"
                    step="0.01"
                  />
                  <span>{($config.customer_segmentation.lifecycle.churned_rate * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Won Back</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.customer_segmentation.lifecycle.won_back_rate}
                    min="0"
                    max="1"
                    step="0.01"
                  />
                  <span>{($config.customer_segmentation.lifecycle.won_back_rate * 100).toFixed(0)}%</span>
                </div>
              </div>
            </div>

            <div class="distribution-total" class:warning={Math.abs(getLifecycleTotal() - 1.0) > 0.01}>
              Total: {(getLifecycleTotal() * 100).toFixed(0)}%
              {#if Math.abs(getLifecycleTotal() - 1.0) > 0.01}
                <span class="warning-text">(should sum to 100%)</span>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Customer Networks" description="Configure referral and corporate hierarchy networks">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.customer_segmentation.networks.referrals_enabled}
                label="Enable Referrals"
                description="Generate referral relationships between customers"
              />

              {#if $config.customer_segmentation.networks.referrals_enabled}
                <FormGroup
                  label="Referral Rate"
                  htmlFor="referral-rate"
                  helpText="Proportion of customers acquired through referrals"
                  error={getError('customer_segmentation.networks.referral_rate')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="referral-rate"
                        bind:value={$config.customer_segmentation.networks.referral_rate}
                        min="0"
                        max="0.5"
                        step="0.01"
                      />
                      <span>{($config.customer_segmentation.networks.referral_rate * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              {/if}

              <Toggle
                bind:checked={$config.customer_segmentation.networks.corporate_hierarchies_enabled}
                label="Enable Corporate Hierarchies"
                description="Generate parent-child corporate ownership structures"
              />

              {#if $config.customer_segmentation.networks.corporate_hierarchies_enabled}
                <FormGroup
                  label="Hierarchy Probability"
                  htmlFor="hierarchy-prob"
                  helpText="Probability that a customer belongs to a corporate hierarchy"
                  error={getError('customer_segmentation.networks.hierarchy_probability')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="hierarchy-prob"
                        bind:value={$config.customer_segmentation.networks.hierarchy_probability}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.customer_segmentation.networks.hierarchy_probability * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Value Segments</h4>
          <p>Four tiers model the revenue concentration curve: Enterprise (40% revenue, 5% customers), Mid-Market, SMB, and Consumer with configurable order value ranges.</p>
        </div>
        <div class="info-card">
          <h4>Lifecycle Stages</h4>
          <p>Seven stages track customer evolution: Prospect, New, Growth, Mature, At Risk, Churned, and Won Back. Distribution rates control the customer base composition.</p>
        </div>
        <div class="info-card">
          <h4>Customer Networks</h4>
          <p>Referral chains link customers who brought in other customers. Corporate hierarchies create parent-child ownership structures for enterprise accounts.</p>
        </div>
        <div class="info-card">
          <h4>Revenue Concentration</h4>
          <p>Models the classic 80/20 distribution where a small number of enterprise customers generate the majority of revenue, creating realistic revenue analytics data.</p>
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
  .suffix { font-size: 0.8125rem; color: var(--color-text-muted); font-family: var(--font-mono); }
  .distribution-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-4); }
  .distribution-item { display: flex; flex-direction: column; gap: var(--space-1); }
  .distribution-item label { font-size: 0.8125rem; font-weight: 500; color: var(--color-text-secondary); }
  .slider-with-value { display: flex; align-items: center; gap: var(--space-2); }
  .slider-with-value input[type='range'] { flex: 1; }
  .slider-with-value span { font-size: 0.8125rem; font-family: var(--font-mono); min-width: 44px; text-align: right; }
  .distribution-total { padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); font-size: 0.8125rem; background-color: var(--color-background); }
  .distribution-total.warning { background-color: rgba(234, 179, 8, 0.1); border: 1px solid #eab308; }
  .info-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .warning-text { font-family: var(--font-sans); margin-left: var(--space-2); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid { grid-template-columns: 1fr; } }
</style>
