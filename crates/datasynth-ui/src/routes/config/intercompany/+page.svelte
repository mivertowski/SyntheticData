<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle, InputNumber } from '$lib/components/forms';
  import DistributionEditor from '$lib/components/forms/DistributionEditor.svelte';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const transferPricingMethods = [
    { value: 'cost_plus', label: 'Cost Plus', description: 'Cost plus markup percentage' },
    { value: 'comparable_uncontrolled', label: 'Comparable Uncontrolled', description: 'Market price comparison' },
    { value: 'resale_price', label: 'Resale Price', description: 'Resale price minus margin' },
    { value: 'transactional_net_margin', label: 'Transactional Net Margin', description: 'Net margin comparison' },
    { value: 'profit_split', label: 'Profit Split', description: 'Split based on contribution' },
  ];

  const transactionTypeLabels: Record<string, string> = {
    goods_sale: 'Goods Sales',
    service_provided: 'Services',
    loan: 'Loans',
    dividend: 'Dividends',
    management_fee: 'Management Fees',
    royalty: 'Royalties',
    cost_sharing: 'Cost Sharing',
  };

  const transactionTypeDescriptions: Record<string, string> = {
    goods_sale: 'Sale of goods between related entities',
    service_provided: 'Inter-company service arrangements',
    loan: 'Inter-company financing transactions',
    dividend: 'Dividend distributions to parent',
    management_fee: 'Corporate overhead allocations',
    royalty: 'IP licensing and royalty payments',
    cost_sharing: 'Shared service cost allocations',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Intercompany" description="Configure intercompany transactions and transfer pricing" />

  {#if $config}
    <div class="sections">
      <FormSection
        title="Intercompany Settings"
        description="Enable and configure intercompany transaction generation"
      >
        <div class="section-content">
          <div class="toggle-row highlight">
            <div class="toggle-info">
              <span class="toggle-label">Enable Intercompany Transactions</span>
              <span class="toggle-description">
                Generate matched intercompany transaction pairs between related entities
              </span>
            </div>
            <Toggle bind:checked={$config.intercompany.enabled} />
          </div>

          <div class="form-grid">
            <FormGroup
              label="IC Transaction Rate"
              htmlFor="ic-rate"
              helpText="Percentage of transactions that are intercompany (0-100%)"
            >
              <InputNumber
                id="ic-rate"
                bind:value={$config.intercompany.ic_transaction_rate}
                min={0}
                max={1}
                step={0.01}
              />
            </FormGroup>

            <FormGroup
              label="Markup Percent"
              htmlFor="markup"
              helpText="Transfer pricing markup for cost-plus method"
            >
              <InputNumber
                id="markup"
                bind:value={$config.intercompany.markup_percent}
                min={0}
                max={1}
                step={0.01}
              />
            </FormGroup>
          </div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate Matched Pairs</span>
              <span class="toggle-description">
                Create offsetting entries in both entities (seller and buyer)
              </span>
            </div>
            <Toggle bind:checked={$config.intercompany.generate_matched_pairs} />
          </div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate Eliminations</span>
              <span class="toggle-description">
                Create consolidation elimination entries for IC balances
              </span>
            </div>
            <Toggle bind:checked={$config.intercompany.generate_eliminations} />
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Transfer Pricing Method"
        description="Select the transfer pricing methodology"
      >
        <div class="section-content">
          <div class="method-grid">
            {#each transferPricingMethods as method}
              <button
                class="method-card"
                class:selected={$config.intercompany.transfer_pricing_method === method.value}
                onclick={() => $config.intercompany.transfer_pricing_method = method.value}
              >
                <span class="method-name">{method.label}</span>
                <span class="method-desc">{method.description}</span>
              </button>
            {/each}
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Transaction Type Distribution"
        description="Distribution of intercompany transaction types"
      >
        <div class="section-content">
          <DistributionEditor
            bind:distribution={$config.intercompany.transaction_type_distribution}
            labels={transactionTypeLabels}
            descriptions={transactionTypeDescriptions}
          />
        </div>
      </FormSection>

      <FormSection
        title="Intercompany Process"
        description="How intercompany transactions work"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="process-flow">
            <div class="flow-step">
              <div class="step-number">1</div>
              <div class="step-content">
                <h4>Transaction Initiation</h4>
                <p>One entity initiates an intercompany transaction (sale, service, loan, etc.)</p>
              </div>
            </div>
            <div class="flow-arrow">→</div>
            <div class="flow-step">
              <div class="step-number">2</div>
              <div class="step-content">
                <h4>Matched Pair Creation</h4>
                <p>System generates offsetting entries in both the seller and buyer entity</p>
              </div>
            </div>
            <div class="flow-arrow">→</div>
            <div class="flow-step">
              <div class="step-number">3</div>
              <div class="step-content">
                <h4>Transfer Price</h4>
                <p>Amounts are calculated using the selected transfer pricing method</p>
              </div>
            </div>
            <div class="flow-arrow">→</div>
            <div class="flow-step">
              <div class="step-number">4</div>
              <div class="step-content">
                <h4>Elimination</h4>
                <p>At consolidation, elimination entries remove IC balances</p>
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
    grid-template-columns: repeat(2, 1fr);
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

  .toggle-row.highlight {
    border: 1px solid var(--color-border);
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

  .method-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: var(--space-3);
  }

  .method-card {
    padding: var(--space-3);
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .method-card:hover {
    border-color: var(--color-accent);
  }

  .method-card.selected {
    border-color: var(--color-accent);
    background-color: rgba(99, 102, 241, 0.1);
  }

  .method-name {
    display: block;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .method-desc {
    display: block;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .process-flow {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    overflow-x: auto;
    padding: var(--space-2) 0;
  }

  .flow-step {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    min-width: 180px;
    flex-shrink: 0;
  }

  .step-number {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-accent);
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .step-content h4 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 var(--space-1);
  }

  .step-content p {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .flow-arrow {
    color: var(--color-text-muted);
    font-size: 1.25rem;
    flex-shrink: 0;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  @media (max-width: 768px) {
    .form-grid {
      grid-template-columns: 1fr;
    }

    .process-flow {
      flex-direction: column;
      align-items: stretch;
    }

    .flow-step {
      min-width: unset;
    }

    .flow-arrow {
      transform: rotate(90deg);
      text-align: center;
    }
  }
</style>
