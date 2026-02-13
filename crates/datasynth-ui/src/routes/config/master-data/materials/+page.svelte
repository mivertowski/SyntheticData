<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, DistributionEditor } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  // Default material distribution settings
  const defaultDistribution = {
    raw_material: 0.30,
    semi_finished: 0.20,
    finished_goods: 0.35,
    trading_goods: 0.10,
    services: 0.05,
  };

  // Ensure distribution exists
  $effect(() => {
    if ($config?.master_data?.materials && Object.keys($config.master_data.materials.distribution).length === 0) {
      $config.master_data.materials.distribution = { ...defaultDistribution };
    }
  });

  // Material type labels
  const materialTypeLabels: Record<string, string> = {
    raw_material: 'Raw Materials (ROH)',
    semi_finished: 'Semi-Finished (HALB)',
    finished_goods: 'Finished Goods (FERT)',
    trading_goods: 'Trading Goods (HAWA)',
    services: 'Services (DIEN)',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Materials Configuration" description="Configure products, raw materials, and inventory items" />

  {#if $config}
    <div class="sections">
      <!-- Entity Count -->
      <FormSection
        title="Material Count"
        description="Number of material master records to generate"
      >
        <div class="section-content">
          <FormGroup
            label="Number of Materials"
            htmlFor="material-count"
            helpText="Typical range: 100-5000 depending on industry"
          >
            <input
              id="material-count"
              type="number"
              min="1"
              max="100000"
              step="1"
              bind:value={$config.master_data.materials.count}
            />
          </FormGroup>

          <div class="quick-presets">
            <span class="preset-label">Quick presets:</span>
            <button type="button" onclick={() => $config.master_data.materials.count = 100}>
              Small (100)
            </button>
            <button type="button" onclick={() => $config.master_data.materials.count = 500}>
              Medium (500)
            </button>
            <button type="button" onclick={() => $config.master_data.materials.count = 2000}>
              Large (2000)
            </button>
            <button type="button" onclick={() => $config.master_data.materials.count = 10000}>
              Manufacturing (10K)
            </button>
          </div>
        </div>
      </FormSection>

      <!-- Material Type Distribution -->
      <FormSection
        title="Material Type Distribution"
        description="Distribution of material types (SAP material types)"
      >
        <div class="section-content">
          <p class="section-intro">
            Material types determine valuation class, account determination, and
            inventory management procedures. Manufacturing typically has more raw
            materials and semi-finished goods, while retail focuses on trading goods.
          </p>

          <DistributionEditor
            label="Material Types"
            bind:distribution={$config.master_data.materials.distribution}
            labels={materialTypeLabels}
            helpText="Distribution affects inventory movements and COGS posting"
          />
        </div>
      </FormSection>

      <!-- Material Characteristics -->
      <FormSection
        title="Material Characteristics"
        description="Valuation and inventory management settings"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="characteristics-grid">
            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
                  </svg>
                </span>
                <span class="characteristic-title">Valuation Methods</span>
              </div>
              <p class="characteristic-description">
                Materials use Standard Price (S) or Moving Average Price (V) valuation.
                Standard is typical for finished goods, moving average for raw materials.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                  </svg>
                </span>
                <span class="characteristic-title">Bill of Materials</span>
              </div>
              <p class="characteristic-description">
                Finished and semi-finished goods have BOMs that reference raw materials.
                BOM depth and component count are configurable per industry.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                    <line x1="3" y1="9" x2="21" y2="9" />
                    <line x1="9" y1="21" x2="9" y2="9" />
                  </svg>
                </span>
                <span class="characteristic-title">Account Determination</span>
              </div>
              <p class="characteristic-description">
                Material and valuation class combinations determine which GL accounts
                are posted during inventory transactions.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                    <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
                  </svg>
                </span>
                <span class="characteristic-title">Batch Management</span>
              </div>
              <p class="characteristic-description">
                Certain materials (pharmaceuticals, chemicals) are batch-managed,
                requiring lot tracking through the supply chain.
              </p>
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

  .section-intro {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .quick-presets {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .preset-label {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .quick-presets button {
    padding: var(--space-1) var(--space-3);
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .quick-presets button:hover {
    background-color: var(--color-surface);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .characteristics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-4);
  }

  .characteristic-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .characteristic-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .characteristic-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-surface);
    border-radius: var(--radius-md);
    color: var(--color-accent);
  }

  .characteristic-icon svg {
    width: 18px;
    height: 18px;
  }

  .characteristic-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .characteristic-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  input[type="number"] {
    width: 100%;
    max-width: 200px;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }

  input[type="number"]:focus {
    outline: none;
    border-color: var(--color-accent);
  }
</style>
