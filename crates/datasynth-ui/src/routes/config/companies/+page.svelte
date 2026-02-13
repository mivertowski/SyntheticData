<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
  const validationErrors = configStore.validationErrors;

  const TRANSACTION_VOLUMES = [
    { value: 'ten_k', label: '10K', count: 10000 },
    { value: 'hundred_k', label: '100K', count: 100000 },
    { value: 'one_m', label: '1M', count: 1000000 },
    { value: 'ten_m', label: '10M', count: 10000000 },
    { value: 'hundred_m', label: '100M', count: 100000000 },
  ];

  const CURRENCIES = ['USD', 'EUR', 'GBP', 'JPY', 'CNY', 'CHF', 'CAD', 'AUD', 'INR', 'BRL'];
  const COUNTRIES = [
    { code: 'US', name: 'United States' },
    { code: 'DE', name: 'Germany' },
    { code: 'GB', name: 'United Kingdom' },
    { code: 'JP', name: 'Japan' },
    { code: 'CN', name: 'China' },
    { code: 'CH', name: 'Switzerland' },
    { code: 'CA', name: 'Canada' },
    { code: 'AU', name: 'Australia' },
    { code: 'IN', name: 'India' },
    { code: 'BR', name: 'Brazil' },
    { code: 'FR', name: 'France' },
    { code: 'NL', name: 'Netherlands' },
    { code: 'SG', name: 'Singapore' },
  ];

  let editingIndex = $state<number | null>(null);
  let showAddForm = $state(false);
  let newCompany = $state({
    code: '',
    name: '',
    currency: 'USD',
    country: 'US',
    fiscal_year_variant: 'K4',
    annual_transaction_volume: 'hundred_k',
    volume_weight: 1.0,
  });

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  function addCompany() {
    if (!$config) return;
    $config.companies = [...$config.companies, { ...newCompany }];
    configStore.set($config);
    showAddForm = false;
    newCompany = {
      code: '',
      name: '',
      currency: 'USD',
      country: 'US',
      fiscal_year_variant: 'K4',
      annual_transaction_volume: 'hundred_k',
      volume_weight: 1.0,
    };
  }

  function removeCompany(index: number) {
    if (!$config) return;
    $config.companies = $config.companies.filter((_, i) => i !== index);
    configStore.set($config);
  }

  function getTotalVolume(): number {
    if (!$config?.companies) return 0;
    return $config.companies.reduce((sum, c) => {
      const vol = TRANSACTION_VOLUMES.find(v => v.value === c.annual_transaction_volume);
      return sum + (vol?.count ?? 0) * c.volume_weight;
    }, 0);
  }

  function formatVolume(vol: number): string {
    if (vol >= 1e9) return (vol / 1e9).toFixed(1) + 'B';
    if (vol >= 1e6) return (vol / 1e6).toFixed(1) + 'M';
    if (vol >= 1e3) return (vol / 1e3).toFixed(0) + 'K';
    return vol.toString();
  }
</script>

<div class="page">
  <ConfigPageHeader title="Companies" description="Configure company codes, currencies, and transaction volumes" />

  {#if $config?.companies}
    <div class="summary-cards">
      <div class="summary-card">
        <span class="card-value">{$config.companies.length}</span>
        <span class="card-label">Companies</span>
      </div>
      <div class="summary-card">
        <span class="card-value">{formatVolume(getTotalVolume())}</span>
        <span class="card-label">Total Volume / Year</span>
      </div>
      <div class="summary-card">
        <span class="card-value">{new Set($config.companies.map(c => c.currency)).size}</span>
        <span class="card-label">Currencies</span>
      </div>
    </div>

    <div class="company-list">
      {#each $config.companies as company, index}
        <div class="company-card" class:editing={editingIndex === index}>
          {#if editingIndex === index}
            <div class="company-form">
              <div class="form-row">
                <FormGroup label="Code" htmlFor={`code-${index}`} error={getError(`companies[${index}].code`)}>
                  {#snippet children()}
                    <input
                      type="text"
                      id={`code-${index}`}
                      bind:value={company.code}
                      maxlength="4"
                      placeholder="ABCD"
                    />
                  {/snippet}
                </FormGroup>
                <FormGroup label="Name" htmlFor={`name-${index}`}>
                  {#snippet children()}
                    <input
                      type="text"
                      id={`name-${index}`}
                      bind:value={company.name}
                      placeholder="Company Name"
                    />
                  {/snippet}
                </FormGroup>
              </div>
              <div class="form-row">
                <FormGroup label="Currency" htmlFor={`currency-${index}`}>
                  {#snippet children()}
                    <select id={`currency-${index}`} bind:value={company.currency}>
                      {#each CURRENCIES as curr}
                        <option value={curr}>{curr}</option>
                      {/each}
                    </select>
                  {/snippet}
                </FormGroup>
                <FormGroup label="Country" htmlFor={`country-${index}`}>
                  {#snippet children()}
                    <select id={`country-${index}`} bind:value={company.country}>
                      {#each COUNTRIES as c}
                        <option value={c.code}>{c.code} - {c.name}</option>
                      {/each}
                    </select>
                  {/snippet}
                </FormGroup>
              </div>
              <div class="form-row">
                <FormGroup label="Annual Volume" htmlFor={`volume-${index}`}>
                  {#snippet children()}
                    <select id={`volume-${index}`} bind:value={company.annual_transaction_volume}>
                      {#each TRANSACTION_VOLUMES as vol}
                        <option value={vol.value}>{vol.label} transactions</option>
                      {/each}
                    </select>
                  {/snippet}
                </FormGroup>
                <FormGroup label="Volume Weight" htmlFor={`weight-${index}`} helpText="Relative weight (1.0 = normal)">
                  {#snippet children()}
                    <input
                      type="number"
                      id={`weight-${index}`}
                      bind:value={company.volume_weight}
                      min="0.1"
                      max="10"
                      step="0.1"
                    />
                  {/snippet}
                </FormGroup>
              </div>
              <div class="form-actions">
                <button class="btn-secondary" onclick={() => { editingIndex = null; }}>
                  Done
                </button>
              </div>
            </div>
          {:else}
            <div class="company-header">
              <div class="company-code">{company.code}</div>
              <div class="company-info">
                <span class="company-name">{company.name}</span>
                <span class="company-details">
                  {company.currency} | {company.country} | {TRANSACTION_VOLUMES.find(v => v.value === company.annual_transaction_volume)?.label ?? '?'}/yr
                </span>
              </div>
              <div class="company-weight">
                <span class="weight-value">{company.volume_weight.toFixed(1)}x</span>
                <span class="weight-label">weight</span>
              </div>
              <div class="company-actions">
                <button class="btn-icon" onclick={() => { editingIndex = index; }} title="Edit">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                    <path d="M18.5 2.5a2.12 2.12 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                  </svg>
                </button>
                <button
                  class="btn-icon danger"
                  onclick={() => removeCompany(index)}
                  disabled={$config.companies.length <= 1}
                  title="Remove"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  </svg>
                </button>
              </div>
            </div>
          {/if}
        </div>
      {/each}

      {#if showAddForm}
        <div class="company-card add-form">
          <div class="company-form">
            <h3>Add New Company</h3>
            <div class="form-row">
              <FormGroup label="Code" htmlFor="new-code">
                {#snippet children()}
                  <input
                    type="text"
                    id="new-code"
                    bind:value={newCompany.code}
                    maxlength="4"
                    placeholder="ABCD"
                  />
                {/snippet}
              </FormGroup>
              <FormGroup label="Name" htmlFor="new-name">
                {#snippet children()}
                  <input
                    type="text"
                    id="new-name"
                    bind:value={newCompany.name}
                    placeholder="Company Name"
                  />
                {/snippet}
              </FormGroup>
            </div>
            <div class="form-row">
              <FormGroup label="Currency" htmlFor="new-currency">
                {#snippet children()}
                  <select id="new-currency" bind:value={newCompany.currency}>
                    {#each CURRENCIES as curr}
                      <option value={curr}>{curr}</option>
                    {/each}
                  </select>
                {/snippet}
              </FormGroup>
              <FormGroup label="Country" htmlFor="new-country">
                {#snippet children()}
                  <select id="new-country" bind:value={newCompany.country}>
                    {#each COUNTRIES as c}
                      <option value={c.code}>{c.code} - {c.name}</option>
                    {/each}
                  </select>
                {/snippet}
              </FormGroup>
            </div>
            <div class="form-row">
              <FormGroup label="Annual Volume" htmlFor="new-volume">
                {#snippet children()}
                  <select id="new-volume" bind:value={newCompany.annual_transaction_volume}>
                    {#each TRANSACTION_VOLUMES as vol}
                      <option value={vol.value}>{vol.label} transactions</option>
                    {/each}
                  </select>
                {/snippet}
              </FormGroup>
              <FormGroup label="Volume Weight" htmlFor="new-weight">
                {#snippet children()}
                  <input
                    type="number"
                    id="new-weight"
                    bind:value={newCompany.volume_weight}
                    min="0.1"
                    max="10"
                    step="0.1"
                  />
                {/snippet}
              </FormGroup>
            </div>
            <div class="form-actions">
              <button class="btn-secondary" onclick={() => { showAddForm = false; }}>
                Cancel
              </button>
              <button
                class="btn-primary"
                onclick={addCompany}
                disabled={!newCompany.code || !newCompany.name}
              >
                Add Company
              </button>
            </div>
          </div>
        </div>
      {:else}
        <button class="add-company-btn" onclick={() => { showAddForm = true; }}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14M5 12h14" />
          </svg>
          Add Company
        </button>
      {/if}
    </div>

    <FormSection title="About Companies" description="How company configuration affects generation">
      {#snippet children()}
        <div class="info-content">
          <p>
            Each company code represents a separate legal entity in the generated data.
            Companies can have different currencies, countries, and transaction volumes.
          </p>
          <ul class="info-list">
            <li><strong>Code</strong> - 4-character identifier used in all transactions</li>
            <li><strong>Currency</strong> - Local currency for the company's transactions</li>
            <li><strong>Country</strong> - Affects tax codes, holiday calendars, and local regulations</li>
            <li><strong>Volume</strong> - Base number of transactions per year</li>
            <li><strong>Weight</strong> - Multiplier for transaction volume (higher = more transactions)</li>
          </ul>
        </div>
      {/snippet}
    </FormSection>
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

  .summary-cards {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-4);
    margin-bottom: var(--space-6);
  }

  .summary-card {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .card-value {
    font-family: var(--font-mono);
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .card-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: var(--space-1);
  }

  .company-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-bottom: var(--space-6);
  }

  .company-card {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .company-card.editing {
    border-color: var(--color-accent);
  }

  .company-card.add-form {
    border-style: dashed;
  }

  .company-header {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
  }

  .company-code {
    font-family: var(--font-mono);
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-accent);
    background-color: var(--color-background);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    min-width: 60px;
    text-align: center;
  }

  .company-info {
    flex: 1;
    min-width: 0;
  }

  .company-name {
    display: block;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .company-details {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .company-weight {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    padding: var(--space-2) var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .weight-value {
    font-family: var(--font-mono);
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .weight-label {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
  }

  .company-actions {
    display: flex;
    gap: var(--space-1);
  }

  .btn-icon {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-icon:hover:not(:disabled) {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .btn-icon.danger:hover:not(:disabled) {
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  .btn-icon:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-icon svg {
    width: 16px;
    height: 16px;
  }

  .company-form {
    padding: var(--space-4);
  }

  .company-form h3 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: var(--space-4);
  }

  .form-row {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-4);
    padding-top: var(--space-4);
    border-top: 1px solid var(--color-border);
  }

  .add-company-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-4);
    background: none;
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-lg);
    color: var(--color-text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .add-company-btn:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .add-company-btn svg {
    width: 20px;
    height: 20px;
  }

  .info-content {
    font-size: 0.875rem;
  }

  .info-content p {
    margin-bottom: var(--space-4);
  }

  .info-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .info-list li {
    color: var(--color-text-secondary);
    padding-left: var(--space-4);
    position: relative;
  }

  .info-list li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0.5em;
    width: 6px;
    height: 6px;
    background-color: var(--color-accent);
    border-radius: 50%;
  }

  .info-list li strong {
    color: var(--color-text-primary);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .summary-cards {
      grid-template-columns: 1fr;
    }

    .form-row {
      grid-template-columns: 1fr;
    }

    .company-header {
      flex-wrap: wrap;
    }

    .company-weight {
      order: -1;
      margin-left: auto;
    }

    .company-info {
      width: 100%;
      order: 1;
    }

    .company-actions {
      order: 2;
      margin-left: auto;
    }

  }
</style>
