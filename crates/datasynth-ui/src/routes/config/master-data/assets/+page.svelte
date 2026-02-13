<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, DistributionEditor } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  // Default asset distribution settings
  const defaultDistribution = {
    buildings: 0.10,
    machinery: 0.30,
    vehicles: 0.15,
    furniture: 0.20,
    it_equipment: 0.20,
    intangibles: 0.05,
  };

  // Ensure distribution exists
  $effect(() => {
    if ($config?.master_data?.assets && Object.keys($config.master_data.assets.distribution).length === 0) {
      $config.master_data.assets.distribution = { ...defaultDistribution };
    }
  });

  // Asset class labels
  const assetClassLabels: Record<string, string> = {
    buildings: 'Buildings & Structures',
    machinery: 'Machinery & Equipment',
    vehicles: 'Vehicles',
    furniture: 'Furniture & Fixtures',
    it_equipment: 'IT Equipment',
    intangibles: 'Intangible Assets',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Fixed Assets Configuration" description="Configure capital equipment and depreciation settings" />

  {#if $config}
    <div class="sections">
      <!-- Entity Count -->
      <FormSection
        title="Asset Count"
        description="Number of fixed asset master records to generate"
      >
        <div class="section-content">
          <FormGroup
            label="Number of Assets"
            htmlFor="asset-count"
            helpText="Typical range: 50-500 for medium-sized company"
          >
            <input
              id="asset-count"
              type="number"
              min="1"
              max="10000"
              step="1"
              bind:value={$config.master_data.assets.count}
            />
          </FormGroup>

          <div class="quick-presets">
            <span class="preset-label">Quick presets:</span>
            <button type="button" onclick={() => $config.master_data.assets.count = 25}>
              Small (25)
            </button>
            <button type="button" onclick={() => $config.master_data.assets.count = 100}>
              Medium (100)
            </button>
            <button type="button" onclick={() => $config.master_data.assets.count = 500}>
              Large (500)
            </button>
            <button type="button" onclick={() => $config.master_data.assets.count = 2000}>
              Enterprise (2000)
            </button>
          </div>
        </div>
      </FormSection>

      <!-- Asset Class Distribution -->
      <FormSection
        title="Asset Class Distribution"
        description="Distribution of assets across classes"
      >
        <div class="section-content">
          <p class="section-intro">
            Asset classes determine depreciation methods, useful life, and GL account
            assignments. Manufacturing typically has more machinery, while services
            focus on IT equipment and furniture.
          </p>

          <DistributionEditor
            label="Asset Classes"
            bind:distribution={$config.master_data.assets.distribution}
            labels={assetClassLabels}
            helpText="Distribution affects depreciation expense patterns"
          />
        </div>
      </FormSection>

      <!-- Asset Characteristics -->
      <FormSection
        title="Asset Characteristics"
        description="Depreciation and lifecycle settings"
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
              <strong>Depreciation Methods</strong>
              <p>
                Assets are depreciated using standard methods based on class:
              </p>
              <ul>
                <li><strong>Straight-Line:</strong> Even depreciation over useful life (buildings, furniture)</li>
                <li><strong>Declining Balance:</strong> Accelerated depreciation (machinery, IT)</li>
                <li><strong>Units of Production:</strong> Usage-based depreciation (vehicles)</li>
              </ul>
            </div>
          </div>

          <div class="characteristics-grid">
            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                    <line x1="16" y1="2" x2="16" y2="6" />
                    <line x1="8" y1="2" x2="8" y2="6" />
                    <line x1="3" y1="10" x2="21" y2="10" />
                  </svg>
                </span>
                <span class="characteristic-title">Useful Life</span>
              </div>
              <p class="characteristic-description">
                Buildings: 30-40 years, Machinery: 7-15 years, Vehicles: 5-7 years,
                IT Equipment: 3-5 years, Furniture: 7-10 years.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10" />
                    <polyline points="12,6 12,12 16,14" />
                  </svg>
                </span>
                <span class="characteristic-title">Acquisition Timing</span>
              </div>
              <p class="characteristic-description">
                Assets are acquired throughout the generation period with realistic
                timing patterns (more at year-end due to budget cycles).
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3,6 5,6 6,10 14,10 15,6 17,6" />
                    <circle cx="6" cy="15" r="2" />
                    <circle cx="14" cy="15" r="2" />
                  </svg>
                </span>
                <span class="characteristic-title">Disposals</span>
              </div>
              <p class="characteristic-description">
                A percentage of fully depreciated assets are disposed, generating
                gain/loss entries and retirement postings.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 2v20M2 12h20" />
                  </svg>
                </span>
                <span class="characteristic-title">Additions</span>
              </div>
              <p class="characteristic-description">
                Asset additions (improvements, capitalized repairs) increase asset
                value and may extend useful life.
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
