<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle, DistributionEditor } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  // Default vendor distribution settings
  const defaultDistribution = {
    payment_terms_net30: 0.40,
    payment_terms_net60: 0.30,
    payment_terms_net90: 0.20,
    payment_terms_immediate: 0.10,
  };

  // Ensure distribution exists
  $effect(() => {
    if ($config?.master_data?.vendors && Object.keys($config.master_data.vendors.distribution).length === 0) {
      $config.master_data.vendors.distribution = { ...defaultDistribution };
    }
  });

  // Payment terms labels
  const paymentTermsLabels: Record<string, string> = {
    payment_terms_net30: 'Net 30 Days',
    payment_terms_net60: 'Net 60 Days',
    payment_terms_net90: 'Net 90 Days',
    payment_terms_immediate: 'Immediate Payment',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Vendor Configuration" description="Configure supplier and service provider master data generation" />

  {#if $config}
    <div class="sections">
      <!-- Entity Count -->
      <FormSection
        title="Vendor Count"
        description="Number of vendor master records to generate"
      >
        <div class="section-content">
          <FormGroup
            label="Number of Vendors"
            htmlFor="vendor-count"
            helpText="Typical range: 50-500 for medium-sized company"
          >
            <input
              id="vendor-count"
              type="number"
              min="1"
              max="10000"
              step="1"
              bind:value={$config.master_data.vendors.count}
            />
          </FormGroup>

          <div class="quick-presets">
            <span class="preset-label">Quick presets:</span>
            <button type="button" onclick={() => $config.master_data.vendors.count = 50}>
              Small (50)
            </button>
            <button type="button" onclick={() => $config.master_data.vendors.count = 200}>
              Medium (200)
            </button>
            <button type="button" onclick={() => $config.master_data.vendors.count = 500}>
              Large (500)
            </button>
            <button type="button" onclick={() => $config.master_data.vendors.count = 2000}>
              Enterprise (2000)
            </button>
          </div>
        </div>
      </FormSection>

      <!-- Payment Terms Distribution -->
      <FormSection
        title="Payment Terms Distribution"
        description="Distribution of payment terms across vendors"
      >
        <div class="section-content">
          <p class="section-intro">
            Configure how payment terms are distributed among vendors.
            Net 30 is most common, with larger vendors often negotiating longer terms.
          </p>

          <DistributionEditor
            label="Payment Terms"
            bind:distribution={$config.master_data.vendors.distribution}
            labels={paymentTermsLabels}
            helpText="Typical: 40% Net 30, 30% Net 60, 20% Net 90, 10% Immediate"
          />
        </div>
      </FormSection>

      <!-- Vendor Characteristics -->
      <FormSection
        title="Vendor Characteristics"
        description="Behavioral patterns and attributes"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="info-card">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4M12 8h.01" />
            </svg>
            <div class="info-content">
              <strong>Vendor Behavior Types</strong>
              <p>
                Vendors are classified into behavior types that affect payment timing and invoice patterns:
              </p>
              <ul>
                <li><strong>Strict:</strong> Consistent invoicing, exact amounts, on-time expectations</li>
                <li><strong>Flexible:</strong> Variable timing, occasional adjustments, negotiable terms</li>
                <li><strong>Problematic:</strong> Frequent errors, duplicate invoices, inconsistent data</li>
              </ul>
            </div>
          </div>

          <div class="characteristics-grid">
            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                  </svg>
                </span>
                <span class="characteristic-title">Intercompany Vendors</span>
              </div>
              <p class="characteristic-description">
                A percentage of vendors will be flagged as intercompany for IC transaction generation.
                Typically 5-15% of vendors in a multi-entity group.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M17 9V7a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2m2 4h10a2 2 0 0 0 2-2v-6a2 2 0 0 0-2-2H9a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2z" />
                  </svg>
                </span>
                <span class="characteristic-title">Bank Account Diversity</span>
              </div>
              <p class="characteristic-description">
                Vendors are assigned bank accounts from various institutions with realistic
                SWIFT/BIC codes and account number formats.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
                  </svg>
                </span>
                <span class="characteristic-title">Tax Identification</span>
              </div>
              <p class="characteristic-description">
                Each vendor receives appropriate tax IDs (VAT, EIN, etc.) based on their
                country of registration for compliance reporting.
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

  .info-card {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .info-card > svg {
    width: 24px;
    height: 24px;
    color: var(--color-accent);
    flex-shrink: 0;
    margin-top: 2px;
  }

  .info-content {
    flex: 1;
  }

  .info-content strong {
    display: block;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .info-content p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0 0 var(--space-2);
    line-height: 1.5;
  }

  .info-content ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .info-content li {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    padding-left: var(--space-3);
    position: relative;
  }

  .info-content li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0.5em;
    width: 4px;
    height: 4px;
    background-color: var(--color-accent);
    border-radius: 50%;
  }

  .info-content li strong {
    display: inline;
    font-size: inherit;
    margin: 0;
    color: var(--color-text-primary);
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
