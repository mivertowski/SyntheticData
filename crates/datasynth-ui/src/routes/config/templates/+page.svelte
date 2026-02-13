<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle } from '$lib/components/forms';
  import DistributionEditor from '$lib/components/forms/DistributionEditor.svelte';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const cultureLabels: Record<string, string> = {
    western_us: 'Western/US',
    hispanic: 'Hispanic',
    german: 'German',
    french: 'French',
    chinese: 'Chinese',
    japanese: 'Japanese',
    indian: 'Indian',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Templates" description="Configure name, description, and reference generation templates" />

  {#if $config}
    <div class="sections">
      <FormSection
        title="Name Generation"
        description="Configure how user and entity names are generated"
      >
        <div class="section-content">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate Realistic Names</span>
              <span class="toggle-description">
                Use culturally-appropriate first and last name combinations
              </span>
            </div>
            <Toggle bind:checked={$config.templates.names.generate_realistic_names} />
          </div>

          <FormGroup
            label="Email Domain"
            htmlFor="email-domain"
            helpText="Domain used for generated email addresses"
          >
            <input
              id="email-domain"
              type="text"
              bind:value={$config.templates.names.email_domain}
              placeholder="company.com"
            />
          </FormGroup>

          <div class="distribution-section">
            <h4>Name Culture Distribution</h4>
            <p>Distribution of name origins for generated users</p>
            <DistributionEditor
              bind:distribution={$config.templates.names.culture_distribution}
              labels={cultureLabels}
            />
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Description Generation"
        description="Configure transaction description text"
      >
        <div class="section-content">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate Header Text</span>
              <span class="toggle-description">
                Generate realistic header descriptions for journal entries
              </span>
            </div>
            <Toggle bind:checked={$config.templates.descriptions.generate_header_text} />
          </div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate Line Text</span>
              <span class="toggle-description">
                Generate line item descriptions with account-specific context
              </span>
            </div>
            <Toggle bind:checked={$config.templates.descriptions.generate_line_text} />
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Reference Numbers"
        description="Configure document reference number formats"
      >
        <div class="section-content">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate References</span>
              <span class="toggle-description">
                Generate document reference numbers (invoices, POs, etc.)
              </span>
            </div>
            <Toggle bind:checked={$config.templates.references.generate_references} />
          </div>

          <div class="form-grid">
            <FormGroup
              label="Invoice Prefix"
              htmlFor="invoice-prefix"
              helpText="Prefix for invoice numbers"
            >
              <input
                id="invoice-prefix"
                type="text"
                bind:value={$config.templates.references.invoice_prefix}
                placeholder="INV"
              />
            </FormGroup>

            <FormGroup
              label="PO Prefix"
              htmlFor="po-prefix"
              helpText="Prefix for purchase order numbers"
            >
              <input
                id="po-prefix"
                type="text"
                bind:value={$config.templates.references.po_prefix}
                placeholder="PO"
              />
            </FormGroup>

            <FormGroup
              label="SO Prefix"
              htmlFor="so-prefix"
              helpText="Prefix for sales order numbers"
            >
              <input
                id="so-prefix"
                type="text"
                bind:value={$config.templates.references.so_prefix}
                placeholder="SO"
              />
            </FormGroup>
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
    grid-template-columns: repeat(3, 1fr);
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

  .distribution-section {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .distribution-section h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 var(--space-1);
  }

  .distribution-section p {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    margin: 0 0 var(--space-3);
  }

  input[type="text"] {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  input[type="text"]:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  @media (max-width: 640px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
