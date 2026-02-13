<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection } from '$lib/components/forms';
  import DistributionEditor from '$lib/components/forms/DistributionEditor.svelte';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const processLabels: Record<string, string> = {
    o2c_weight: 'Order-to-Cash (O2C)',
    p2p_weight: 'Procure-to-Pay (P2P)',
    r2r_weight: 'Record-to-Report (R2R)',
    h2r_weight: 'Hire-to-Retire (H2R)',
    a2r_weight: 'Acquire-to-Retire (A2R)',
  };

  const processDescriptions: Record<string, string> = {
    o2c_weight: 'Sales orders, deliveries, invoicing, and customer receipts',
    p2p_weight: 'Purchase orders, goods receipts, vendor invoices, and payments',
    r2r_weight: 'Journal entries, period close, and financial reporting',
    h2r_weight: 'Payroll, employee expenses, and HR transactions',
    a2r_weight: 'Fixed asset acquisitions, depreciation, and disposals',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Business Processes" description="Configure the distribution of transactions across business processes" />

  {#if $config}
    <div class="sections">
      <FormSection
        title="Process Distribution"
        description="Set the relative weight of each business process in transaction generation"
      >
        <div class="section-content">
          <div class="info-banner">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4M12 8h.01" />
            </svg>
            <span>Weights are automatically normalized to sum to 100%</span>
          </div>

          <DistributionEditor
            bind:distribution={$config.business_processes}
            labels={processLabels}
            descriptions={processDescriptions}
          />
        </div>
      </FormSection>

      <FormSection
        title="Process Details"
        description="Overview of each business process"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="process-cards">
            <div class="process-card">
              <div class="process-header">
                <span class="process-icon o2c">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M3 3h18v18H3zM9 9h6v6H9z" />
                  </svg>
                </span>
                <div>
                  <h4>Order-to-Cash (O2C)</h4>
                  <span class="process-weight">{(($config.business_processes.o2c_weight || 0) * 100).toFixed(0)}%</span>
                </div>
              </div>
              <p>Revenue cycle from customer order through cash collection:</p>
              <ul>
                <li>Sales Order → Delivery → Invoice → Receipt</li>
                <li>Credit checks, returns, and bad debt handling</li>
                <li>AR aging and cash application</li>
              </ul>
            </div>

            <div class="process-card">
              <div class="process-header">
                <span class="process-icon p2p">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
                    <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
                  </svg>
                </span>
                <div>
                  <h4>Procure-to-Pay (P2P)</h4>
                  <span class="process-weight">{(($config.business_processes.p2p_weight || 0) * 100).toFixed(0)}%</span>
                </div>
              </div>
              <p>Procurement cycle from requisition through payment:</p>
              <ul>
                <li>Purchase Order → Goods Receipt → Invoice → Payment</li>
                <li>Three-way matching and variance handling</li>
                <li>AP aging and payment terms</li>
              </ul>
            </div>

            <div class="process-card">
              <div class="process-header">
                <span class="process-icon r2r">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                    <path d="M14 2v6h6M16 13H8M16 17H8M10 9H8" />
                  </svg>
                </span>
                <div>
                  <h4>Record-to-Report (R2R)</h4>
                  <span class="process-weight">{(($config.business_processes.r2r_weight || 0) * 100).toFixed(0)}%</span>
                </div>
              </div>
              <p>Financial close and reporting cycle:</p>
              <ul>
                <li>Manual journal entries and adjustments</li>
                <li>Accruals, deferrals, and reclassifications</li>
                <li>Period close and consolidation</li>
              </ul>
            </div>

            <div class="process-card">
              <div class="process-header">
                <span class="process-icon h2r">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
                    <circle cx="9" cy="7" r="4" />
                    <path d="M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" />
                  </svg>
                </span>
                <div>
                  <h4>Hire-to-Retire (H2R)</h4>
                  <span class="process-weight">{(($config.business_processes.h2r_weight || 0) * 100).toFixed(0)}%</span>
                </div>
              </div>
              <p>Human resources and payroll cycle:</p>
              <ul>
                <li>Payroll processing and tax withholdings</li>
                <li>Employee expense reimbursements</li>
                <li>Benefits and compensation accruals</li>
              </ul>
            </div>

            <div class="process-card">
              <div class="process-header">
                <span class="process-icon a2r">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="2" y="7" width="20" height="14" rx="2" ry="2" />
                    <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" />
                  </svg>
                </span>
                <div>
                  <h4>Acquire-to-Retire (A2R)</h4>
                  <span class="process-weight">{(($config.business_processes.a2r_weight || 0) * 100).toFixed(0)}%</span>
                </div>
              </div>
              <p>Fixed asset lifecycle management:</p>
              <ul>
                <li>Asset acquisition and capitalization</li>
                <li>Depreciation and impairment</li>
                <li>Asset transfers and disposals</li>
              </ul>
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

  .info-banner {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3);
    background-color: rgba(99, 102, 241, 0.1);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .info-banner svg {
    width: 16px;
    height: 16px;
    color: var(--color-accent);
  }

  .process-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--space-4);
  }

  .process-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .process-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .process-icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    flex-shrink: 0;
  }

  .process-icon svg {
    width: 20px;
    height: 20px;
  }

  .process-icon.o2c { background-color: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .process-icon.p2p { background-color: rgba(59, 130, 246, 0.15); color: #3b82f6; }
  .process-icon.r2r { background-color: rgba(168, 85, 247, 0.15); color: #a855f7; }
  .process-icon.h2r { background-color: rgba(249, 115, 22, 0.15); color: #f97316; }
  .process-icon.a2r { background-color: rgba(236, 72, 153, 0.15); color: #ec4899; }

  .process-header h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .process-weight {
    font-size: 0.75rem;
    color: var(--color-accent);
    font-weight: 500;
  }

  .process-card p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0 0 var(--space-2);
  }

  .process-card ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .process-card li {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    padding-left: var(--space-3);
    position: relative;
    margin-bottom: var(--space-1);
  }

  .process-card li::before {
    content: '•';
    position: absolute;
    left: 0;
    color: var(--color-text-muted);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }
</style>
