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
  <ConfigPageHeader title="Cross-Process Links" description="Enable cross-process entity linking for graph coherence" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Cross-Process Linking" description="Enable automatic linking of entities across business processes">
        {#snippet children()}
          <Toggle
            bind:checked={$config.cross_process_links.enabled}
            label="Enable Cross-Process Links"
            description="Create referential links between entities generated in different business processes"
          />
        {/snippet}
      </FormSection>

      {#if $config.cross_process_links.enabled}
        <FormSection title="Link Types" description="Select which cross-process links to generate">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.cross_process_links.inventory_p2p_o2c}
                label="Inventory P2P-O2C Links"
                description="Link GoodsReceipt to Delivery for inventory coherence across Procure-to-Pay and Order-to-Cash"
              />

              <Toggle
                bind:checked={$config.cross_process_links.payment_bank_reconciliation}
                label="Payment-Bank Reconciliation"
                description="Link payments to bank statement lines for automated reconciliation support"
              />

              <Toggle
                bind:checked={$config.cross_process_links.intercompany_bilateral}
                label="Intercompany Bilateral"
                description="Generate bilateral intercompany transaction pairs with matching elimination entries"
              />
            </div>
          {/snippet}
        </FormSection>

      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Inventory Linking</h4>
          <p>Connects P2P goods receipts to O2C deliveries, creating coherent inventory flow where items received from vendors can be traced through to customer deliveries.</p>
        </div>
        <div class="info-card">
          <h4>Payment Reconciliation</h4>
          <p>Links payments to bank statement lines for automated reconciliation support, including timing differences and batch processing effects.</p>
        </div>
        <div class="info-card">
          <h4>Intercompany Bilateral</h4>
          <p>Ensures intercompany transactions appear in both entities with matching elimination entries for consolidation and transfer pricing documentation.</p>
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
