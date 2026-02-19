<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
  const validationErrors = configStore.validationErrors;

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  $effect(() => {
    if ($config && !$config.country_packs) {
      $config.country_packs = {
        external_dir: null,
        overrides: {},
      };
    }
  });

  function overrideCount(): number {
    if (!$config?.country_packs?.overrides) return 0;
    return Object.keys($config.country_packs.overrides).length;
  }
</script>

<div class="page">
  <ConfigPageHeader
    title="Country Packs"
    description="Configure pluggable country packs for locale-specific data generation"
  />

  {#if $config}
    <div class="page-content">
      <FormSection title="External Directory" description="Path to a directory containing external JSON country pack files">
        {#snippet children()}
          <div class="form-stack">
            <FormGroup
              label="External Packs Directory"
              htmlFor="country-packs-dir"
              helpText="Absolute or relative path to a folder with custom country pack JSON files. Leave empty to use built-in packs only."
              error={getError('country_packs.external_dir')}
            >
              {#snippet children()}
                <input
                  type="text"
                  id="country-packs-dir"
                  value={$config.country_packs.external_dir ?? ''}
                  oninput={(e) => {
                    const val = (e.target as HTMLInputElement).value;
                    $config.country_packs.external_dir = val || null;
                  }}
                  placeholder="/path/to/country-packs"
                />
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Per-Country Overrides" description="Override specific sections of a country pack at generation time">
        {#snippet children()}
          <div class="form-stack">
            <div class="override-summary">
              {#if overrideCount() > 0}
                <p class="override-count">{overrideCount()} country override(s) configured</p>
                <div class="override-list">
                  {#each Object.keys($config.country_packs.overrides) as country}
                    <span class="override-badge">{country.toUpperCase()}</span>
                  {/each}
                </div>
              {:else}
                <p class="override-empty">No per-country overrides configured. Overrides can be added directly in the YAML configuration file under <code>country_packs.overrides</code>.</p>
              {/if}
            </div>
            <div class="override-help">
              <p>
                Per-country overrides let you customize specific sections of a country pack without replacing the entire file.
                Each override is keyed by ISO 3166-1 alpha-2 country code (e.g., <code>US</code>, <code>DE</code>, <code>GB</code>) and contains
                section-level overrides that are deep-merged with the pack defaults.
              </p>
            </div>
          </div>
        {/snippet}
      </FormSection>

      <div class="info-cards">
        <div class="info-card">
          <h4>Built-in Packs</h4>
          <p>
            Three country packs are included out of the box: US, DE, and GB. Each provides
            locale-specific defaults for tax rates, payment terms, currency formats,
            document numbering, and more.
          </p>
        </div>
        <div class="info-card">
          <h4>Merge Behavior</h4>
          <p>
            Country data is resolved in layers: built-in defaults, then the country pack
            file (built-in or external), then per-country overrides from this config. Each
            layer deep-merges into the previous one.
          </p>
        </div>
        <div class="info-card">
          <h4>16-Section Schema</h4>
          <p>
            Country packs can customize up to 16 data sections including tax rates, payment
            terms, currency formats, bank formats, address formats, regulatory requirements,
            and accounting standards.
          </p>
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
  .form-stack { display: flex; flex-direction: column; gap: var(--space-4); }
  .override-summary { padding: var(--space-3); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .override-count { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin: 0 0 var(--space-2) 0; }
  .override-empty { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .override-empty code { font-size: 0.75rem; background-color: var(--color-surface); padding: 1px 4px; border-radius: var(--radius-sm); }
  .override-list { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .override-badge { display: inline-block; padding: 2px 8px; font-size: 0.75rem; font-weight: 600; font-family: var(--font-mono); background-color: var(--color-accent); color: white; border-radius: var(--radius-sm); letter-spacing: 0.05em; }
  .override-help { font-size: 0.8125rem; color: var(--color-text-secondary); line-height: 1.5; }
  .override-help p { margin: 0; }
  .override-help code { font-size: 0.75rem; background-color: var(--color-background); padding: 1px 4px; border-radius: var(--radius-sm); }
  .info-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
</style>
