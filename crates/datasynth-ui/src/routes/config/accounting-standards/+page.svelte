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

  function getFairValueTotal(): number {
    if (!$config?.accounting_standards?.fair_value) return 0;
    const fv = $config.accounting_standards.fair_value;
    return fv.level1_percent + fv.level2_percent + fv.level3_percent;
  }

  const frameworks = [
    { value: 'us_gaap', label: 'US GAAP', description: 'United States Generally Accepted Accounting Principles' },
    { value: 'ifrs', label: 'IFRS', description: 'International Financial Reporting Standards' },
    { value: 'dual_reporting', label: 'Dual Reporting', description: 'Generate data compliant with both US GAAP and IFRS' },
  ];

  const testFrequencies = [
    { value: 'annual', label: 'Annual' },
    { value: 'quarterly', label: 'Quarterly' },
    { value: 'triggered', label: 'Triggered' },
  ];
</script>

<div class="page">
  <ConfigPageHeader title="Accounting Standards" description="Configure accounting framework and standards-based generation" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Standards Settings" description="Enable accounting standards compliance in generated data">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.accounting_standards.enabled}
              label="Enable Accounting Standards"
              description="Generate data that conforms to accounting framework requirements (ASC/IFRS)"
            />

            {#if $config.accounting_standards.enabled}
              <FormGroup
                label="Accounting Framework"
                htmlFor="framework"
                helpText="Select the accounting standards framework for data generation"
                error={getError('accounting_standards.framework')}
              >
                {#snippet children()}
                  <div class="framework-selector">
                    {#each frameworks as fw}
                      <label class="framework-option" class:selected={$config.accounting_standards.framework === fw.value}>
                        <input
                          type="radio"
                          name="framework"
                          value={fw.value}
                          bind:group={$config.accounting_standards.framework}
                        />
                        <span class="framework-label">{fw.label}</span>
                        <span class="framework-desc">{fw.description}</span>
                      </label>
                    {/each}
                  </div>
                {/snippet}
              </FormGroup>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      {#if $config.accounting_standards.enabled}
        <FormSection title="Revenue Recognition" description="ASC 606 / IFRS 15 revenue recognition standards">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.accounting_standards.revenue_recognition.enabled}
                label="Enable Revenue Recognition"
                description="Generate customer contracts with performance obligations and recognition schedules"
              />

              {#if $config.accounting_standards.revenue_recognition.enabled}
                <div class="form-grid">
                  <div class="form-stack">
                    <Toggle
                      bind:checked={$config.accounting_standards.revenue_recognition.generate_contracts}
                      label="Generate Contracts"
                      description="Create detailed customer contract records"
                    />
                  </div>

                  <FormGroup
                    label="Avg Obligations per Contract"
                    htmlFor="avg-obligations"
                    helpText="Average number of performance obligations per contract"
                    error={getError('accounting_standards.revenue_recognition.avg_obligations_per_contract')}
                  >
                    {#snippet children()}
                      <input
                        type="number"
                        id="avg-obligations"
                        bind:value={$config.accounting_standards.revenue_recognition.avg_obligations_per_contract}
                        min="1"
                        max="20"
                        step="0.5"
                      />
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Leases" description="ASC 842 / IFRS 16 lease accounting standards">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.accounting_standards.leases.enabled}
                label="Enable Lease Accounting"
                description="Generate lease records with ROU assets and lease liabilities"
              />

              {#if $config.accounting_standards.leases.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Lease Count"
                    htmlFor="lease-count"
                    helpText="Total number of leases to generate"
                    error={getError('accounting_standards.leases.lease_count')}
                  >
                    {#snippet children()}
                      <input
                        type="number"
                        id="lease-count"
                        bind:value={$config.accounting_standards.leases.lease_count}
                        min="1"
                        max="10000"
                      />
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Finance Lease Percent"
                    htmlFor="finance-lease-pct"
                    helpText="Proportion of leases classified as finance leases (0-1)"
                    error={getError('accounting_standards.leases.finance_lease_percent')}
                  >
                    {#snippet children()}
                      <div class="slider-with-value">
                        <input
                          type="range"
                          id="finance-lease-pct"
                          bind:value={$config.accounting_standards.leases.finance_lease_percent}
                          min="0"
                          max="1"
                          step="0.05"
                        />
                        <span>{($config.accounting_standards.leases.finance_lease_percent * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Fair Value" description="ASC 820 / IFRS 13 fair value measurement hierarchy">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.accounting_standards.fair_value.enabled}
                label="Enable Fair Value Measurements"
                description="Generate fair value measurements across the three-level hierarchy"
              />

              {#if $config.accounting_standards.fair_value.enabled}
                <div class="distribution-grid">
                  <div class="distribution-item">
                    <label>Level 1 (Quoted Prices)</label>
                    <div class="slider-with-value">
                      <input
                        type="range"
                        bind:value={$config.accounting_standards.fair_value.level1_percent}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.accounting_standards.fair_value.level1_percent * 100).toFixed(0)}%</span>
                    </div>
                  </div>

                  <div class="distribution-item">
                    <label>Level 2 (Observable Inputs)</label>
                    <div class="slider-with-value">
                      <input
                        type="range"
                        bind:value={$config.accounting_standards.fair_value.level2_percent}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.accounting_standards.fair_value.level2_percent * 100).toFixed(0)}%</span>
                    </div>
                  </div>

                  <div class="distribution-item">
                    <label>Level 3 (Unobservable Inputs)</label>
                    <div class="slider-with-value">
                      <input
                        type="range"
                        bind:value={$config.accounting_standards.fair_value.level3_percent}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.accounting_standards.fair_value.level3_percent * 100).toFixed(0)}%</span>
                    </div>
                  </div>
                </div>

                <div class="distribution-total" class:warning={Math.abs(getFairValueTotal() - 1.0) > 0.01}>
                  Total: {(getFairValueTotal() * 100).toFixed(0)}%
                  {#if Math.abs(getFairValueTotal() - 1.0) > 0.01}
                    <span class="warning-text">(should sum to 100%)</span>
                  {/if}
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Impairment" description="ASC 360 / IAS 36 asset impairment testing">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.accounting_standards.impairment.enabled}
                label="Enable Impairment Testing"
                description="Generate impairment test records for long-lived assets and goodwill"
              />

              {#if $config.accounting_standards.impairment.enabled}
                <FormGroup
                  label="Test Frequency"
                  htmlFor="test-frequency"
                  helpText="How often impairment tests are performed"
                  error={getError('accounting_standards.impairment.test_frequency')}
                >
                  {#snippet children()}
                    <select id="test-frequency" bind:value={$config.accounting_standards.impairment.test_frequency}>
                      {#each testFrequencies as freq}
                        <option value={freq.value}>{freq.label}</option>
                      {/each}
                    </select>
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Revenue Recognition (ASC 606 / IFRS 15)</h4>
          <p>Customer contracts with multi-element arrangements and performance obligations. Generates recognition schedules based on delivery milestones and time-based allocation.</p>
        </div>
        <div class="info-card">
          <h4>Lease Accounting (ASC 842 / IFRS 16)</h4>
          <p>Right-of-use assets and lease liabilities with finance vs. operating lease classification. Includes amortization schedules and interest expense calculations.</p>
        </div>
        <div class="info-card">
          <h4>Fair Value Hierarchy (ASC 820 / IFRS 13)</h4>
          <p>Three-level measurement hierarchy: Level 1 (quoted market prices), Level 2 (observable inputs), and Level 3 (unobservable model-based inputs).</p>
        </div>
        <div class="info-card">
          <h4>Impairment Testing (ASC 360 / IAS 36)</h4>
          <p>Periodic asset impairment assessments with recoverable amount calculations comparing carrying value to fair value less costs to sell or value in use.</p>
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

  .framework-selector { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-2); }
  .framework-option {
    display: flex;
    flex-direction: column;
    padding: var(--space-3);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: center;
  }
  .framework-option:hover { border-color: var(--color-accent); }
  .framework-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .framework-option input { display: none; }
  .framework-label { font-weight: 600; font-size: 0.875rem; color: var(--color-text-primary); }
  .framework-desc { font-size: 0.6875rem; color: var(--color-text-secondary); margin-top: var(--space-1); }

  select {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
  }
  select:focus { outline: none; border-color: var(--color-accent); }

  @media (max-width: 768px) { .form-grid, .distribution-grid, .framework-selector { grid-template-columns: 1fr; } }
</style>
