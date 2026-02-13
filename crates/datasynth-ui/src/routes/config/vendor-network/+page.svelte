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

  function getClusterTotal(): number {
    if (!$config?.vendor_network?.clusters) return 0;
    const c = $config.vendor_network.clusters;
    return c.reliable_strategic + c.standard_operational + c.transactional + c.problematic;
  }
</script>

<div class="page">
  <ConfigPageHeader title="Vendor Network" description="Configure multi-tier supply chain network and vendor clustering" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Network Settings" description="Enable and configure the vendor supply chain network">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.vendor_network.enabled}
              label="Enable Vendor Network"
              description="Generate multi-tier supply chain networks with vendor clustering and dependency modeling"
            />

            {#if $config.vendor_network.enabled}
              <FormGroup
                label="Supply Chain Depth"
                htmlFor="network-depth"
                helpText="Number of supply chain tiers (1 = Tier 1 only, up to 5)"
                error={getError('vendor_network.depth')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="network-depth"
                      bind:value={$config.vendor_network.depth}
                      min="1"
                      max="5"
                      step="1"
                    />
                    <span>{$config.vendor_network.depth} tiers</span>
                  </div>
                {/snippet}
              </FormGroup>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      {#if $config.vendor_network.enabled}
        <FormSection title="Tier 1 Vendors" description="Direct suppliers to your organization">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Minimum Count"
                htmlFor="t1-min"
                helpText="Minimum number of Tier 1 vendors"
                error={getError('vendor_network.tiers.tier1.count_min')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="t1-min"
                    bind:value={$config.vendor_network.tiers.tier1.count_min}
                    min="1"
                    max="10000"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Maximum Count"
                htmlFor="t1-max"
                helpText="Maximum number of Tier 1 vendors"
                error={getError('vendor_network.tiers.tier1.count_max')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="t1-max"
                    bind:value={$config.vendor_network.tiers.tier1.count_max}
                    min="1"
                    max="10000"
                  />
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Tier 2 Vendors" description="Suppliers to your Tier 1 vendors">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Min per Parent"
                htmlFor="t2-min"
                helpText="Minimum Tier 2 vendors per Tier 1 parent"
                error={getError('vendor_network.tiers.tier2.count_per_parent_min')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="t2-min"
                    bind:value={$config.vendor_network.tiers.tier2.count_per_parent_min}
                    min="1"
                    max="100"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Max per Parent"
                htmlFor="t2-max"
                helpText="Maximum Tier 2 vendors per Tier 1 parent"
                error={getError('vendor_network.tiers.tier2.count_per_parent_max')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="t2-max"
                    bind:value={$config.vendor_network.tiers.tier2.count_per_parent_max}
                    min="1"
                    max="100"
                  />
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Tier 3 Vendors" description="Suppliers to your Tier 2 vendors">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Min per Parent"
                htmlFor="t3-min"
                helpText="Minimum Tier 3 vendors per Tier 2 parent"
                error={getError('vendor_network.tiers.tier3.count_per_parent_min')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="t3-min"
                    bind:value={$config.vendor_network.tiers.tier3.count_per_parent_min}
                    min="1"
                    max="50"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Max per Parent"
                htmlFor="t3-max"
                helpText="Maximum Tier 3 vendors per Tier 2 parent"
                error={getError('vendor_network.tiers.tier3.count_per_parent_max')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="t3-max"
                    bind:value={$config.vendor_network.tiers.tier3.count_per_parent_max}
                    min="1"
                    max="50"
                  />
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Vendor Cluster Distribution" description="Distribution of vendors across behavioral clusters (should sum to 100%)">
          {#snippet children()}
            <div class="distribution-grid">
              <div class="distribution-item">
                <label>Reliable Strategic</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.vendor_network.clusters.reliable_strategic}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.vendor_network.clusters.reliable_strategic * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Standard Operational</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.vendor_network.clusters.standard_operational}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.vendor_network.clusters.standard_operational * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Transactional</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.vendor_network.clusters.transactional}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.vendor_network.clusters.transactional * 100).toFixed(0)}%</span>
                </div>
              </div>

              <div class="distribution-item">
                <label>Problematic</label>
                <div class="slider-with-value">
                  <input
                    type="range"
                    bind:value={$config.vendor_network.clusters.problematic}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                  <span>{($config.vendor_network.clusters.problematic * 100).toFixed(0)}%</span>
                </div>
              </div>
            </div>

            <div class="distribution-total" class:warning={Math.abs(getClusterTotal() - 1.0) > 0.01}>
              Total: {(getClusterTotal() * 100).toFixed(0)}%
              {#if Math.abs(getClusterTotal() - 1.0) > 0.01}
                <span class="warning-text">(should sum to 100%)</span>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Vendor Dependencies" description="Concentration limits for supply chain risk management">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Max Single Vendor Concentration"
                htmlFor="max-single"
                helpText="Maximum spend share for any single vendor (0-1)"
                error={getError('vendor_network.dependencies.max_single_vendor_concentration')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="max-single"
                      bind:value={$config.vendor_network.dependencies.max_single_vendor_concentration}
                      min="0.01"
                      max="0.5"
                      step="0.01"
                    />
                    <span>{($config.vendor_network.dependencies.max_single_vendor_concentration * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Top 5 Concentration"
                htmlFor="top-5"
                helpText="Maximum combined spend share for top 5 vendors (0-1)"
                error={getError('vendor_network.dependencies.top_5_concentration')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="top-5"
                      bind:value={$config.vendor_network.dependencies.top_5_concentration}
                      min="0.05"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.vendor_network.dependencies.top_5_concentration * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Multi-Tier Supply Chain</h4>
          <p>Model Tier 1, Tier 2, and Tier 3 supplier relationships with cascading tier generation from parent vendors to create realistic supply chain depth.</p>
        </div>
        <div class="info-card">
          <h4>Vendor Clusters</h4>
          <p>Four behavioral profiles classify vendors: Reliable Strategic (long-term partners), Standard Operational (routine suppliers), Transactional (spot purchases), and Problematic (quality/delivery issues).</p>
        </div>
        <div class="info-card">
          <h4>Concentration Risk</h4>
          <p>Configurable limits for single-vendor and top-5 vendor concentration model realistic supply chain risk and procurement diversification policies.</p>
        </div>
        <div class="info-card">
          <h4>Supply Chain Depth</h4>
          <p>Each tier generates child vendors from its parents, creating branching supply chains up to 5 levels deep with configurable count ranges per tier.</p>
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
