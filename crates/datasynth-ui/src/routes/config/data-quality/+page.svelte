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

  function getDuplicateRateTotal(): number {
    if (!$config?.data_quality?.duplicates) return 0;
    return $config.data_quality.duplicates.exact_rate + $config.data_quality.duplicates.fuzzy_rate;
  }

  const MISSING_MECHANISMS = [
    { value: 'mcar', label: 'MCAR - Missing Completely At Random' },
    { value: 'mar', label: 'MAR - Missing At Random' },
    { value: 'mnar', label: 'MNAR - Missing Not At Random' },
    { value: 'systematic', label: 'Systematic - Entire field groups' },
  ];
</script>

<div class="page">
  <ConfigPageHeader title="Data Quality" description="Simulate real-world data quality issues like missing values, typos, and encoding problems" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Data Quality Settings" description="Enable realistic data quality issue simulation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.data_quality.enabled}
              label="Enable Data Quality Issues"
              description="Inject realistic data quality problems for testing data cleaning pipelines"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.data_quality.enabled}
        <FormSection title="Missing Values" description="Configure missing value patterns and rates">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.data_quality.missing_values.enabled}
                label="Enable Missing Values"
                description="Inject missing values with configurable patterns"
              />

              {#if $config.data_quality.missing_values.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Mechanism"
                    htmlFor="missing-mechanism"
                    helpText="Statistical mechanism for missing value generation"
                  >
                    {#snippet children()}
                      <select id="missing-mechanism" bind:value={$config.data_quality.missing_values.mechanism}>
                        {#each MISSING_MECHANISMS as mechanism}
                          <option value={mechanism.value}>{mechanism.label}</option>
                        {/each}
                      </select>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Overall Rate"
                    htmlFor="missing-rate"
                    helpText="Proportion of values to make missing (0-1)"
                    error={getError('data_quality.missing_values.overall_rate')}
                  >
                    {#snippet children()}
                      <div class="slider-with-value">
                        <input
                          type="range"
                          id="missing-rate"
                          bind:value={$config.data_quality.missing_values.overall_rate}
                          min="0"
                          max="1"
                          step="0.01"
                        />
                        <span>{($config.data_quality.missing_values.overall_rate * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Typos" description="Configure typographical error injection">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.data_quality.typos.enabled}
                label="Enable Typo Injection"
                description="Add realistic typographical errors to text fields"
              />

              {#if $config.data_quality.typos.enabled}
                <FormGroup
                  label="Typo Rate"
                  htmlFor="typo-rate"
                  helpText="Proportion of text values affected by typos (0-0.1)"
                  error={getError('data_quality.typos.rate')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="typo-rate"
                        bind:value={$config.data_quality.typos.rate}
                        min="0"
                        max="0.1"
                        step="0.005"
                      />
                      <span>{($config.data_quality.typos.rate * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <div class="form-grid">
                  <Toggle
                    bind:checked={$config.data_quality.typos.keyboard_aware}
                    label="Keyboard-Aware"
                    description="Generate typos based on QWERTY keyboard proximity"
                  />
                  <Toggle
                    bind:checked={$config.data_quality.typos.transposition}
                    label="Transposition"
                    description="Swap adjacent characters (e.g., 'teh' instead of 'the')"
                  />
                  <Toggle
                    bind:checked={$config.data_quality.typos.ocr_errors}
                    label="OCR Errors"
                    description="Simulate optical character recognition errors (0/O, 1/l, 5/S)"
                  />
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Format Variations" description="Configure field format inconsistencies">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.data_quality.format_variations.enabled}
                label="Enable Format Variations"
                description="Introduce inconsistent formatting across records"
              />

              {#if $config.data_quality.format_variations.enabled}
                <div class="form-grid">
                  <Toggle
                    bind:checked={$config.data_quality.format_variations.date_formats}
                    label="Date Formats"
                    description="Mix ISO, US (MM/DD), and EU (DD/MM) date formats"
                  />
                  <Toggle
                    bind:checked={$config.data_quality.format_variations.amount_formats}
                    label="Amount Formats"
                    description="Mix comma/period decimal separators and grouping"
                  />
                  <Toggle
                    bind:checked={$config.data_quality.format_variations.identifier_formats}
                    label="Identifier Formats"
                    description="Vary case, padding, and prefix styles for IDs"
                  />
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Duplicates" description="Configure duplicate record injection">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.data_quality.duplicates.enabled}
                label="Enable Duplicate Injection"
                description="Inject exact and fuzzy duplicate records"
              />

              {#if $config.data_quality.duplicates.enabled}
                <FormGroup
                  label="Overall Duplicate Rate"
                  htmlFor="dup-rate"
                  helpText="Proportion of records that are duplicates (0-0.1)"
                  error={getError('data_quality.duplicates.rate')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="dup-rate"
                        bind:value={$config.data_quality.duplicates.rate}
                        min="0"
                        max="0.1"
                        step="0.005"
                      />
                      <span>{($config.data_quality.duplicates.rate * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <div class="distribution-grid">
                  <div class="distribution-item">
                    <label>Exact Duplicate Rate</label>
                    <div class="slider-with-value">
                      <input
                        type="range"
                        bind:value={$config.data_quality.duplicates.exact_rate}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.data_quality.duplicates.exact_rate * 100).toFixed(0)}%</span>
                    </div>
                  </div>
                  <div class="distribution-item">
                    <label>Fuzzy Duplicate Rate</label>
                    <div class="slider-with-value">
                      <input
                        type="range"
                        bind:value={$config.data_quality.duplicates.fuzzy_rate}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span>{($config.data_quality.duplicates.fuzzy_rate * 100).toFixed(0)}%</span>
                    </div>
                  </div>
                </div>

                <div class="distribution-total" class:warning={Math.abs(getDuplicateRateTotal() - 1.0) > 0.01}>
                  Total: {(getDuplicateRateTotal() * 100).toFixed(0)}%
                  {#if Math.abs(getDuplicateRateTotal() - 1.0) > 0.01}
                    <span class="warning-text">(should sum to 100%)</span>
                  {/if}
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Encoding Issues" description="Configure character encoding problems">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.data_quality.encoding.enabled}
                label="Enable Encoding Issues"
                description="Inject character encoding problems for robustness testing"
              />

              {#if $config.data_quality.encoding.enabled}
                <div class="form-grid">
                  <Toggle
                    bind:checked={$config.data_quality.encoding.mojibake}
                    label="Mojibake"
                    description="Simulate encoding misinterpretation (e.g., UTF-8 as Latin-1)"
                  />
                  <Toggle
                    bind:checked={$config.data_quality.encoding.bom_issues}
                    label="BOM Issues"
                    description="Add byte order mark inconsistencies"
                  />
                  <Toggle
                    bind:checked={$config.data_quality.encoding.html_entities}
                    label="HTML Entities"
                    description="Replace characters with HTML entity references"
                  />
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Missing Values</h4>
          <p>Four statistical mechanisms: MCAR (completely random), MAR (dependent on observed data), MNAR (dependent on missing value itself), and Systematic (entire field groups).</p>
        </div>
        <div class="info-card">
          <h4>Typos & OCR Errors</h4>
          <p>Keyboard-aware substitutions based on QWERTY proximity, character transpositions, and OCR-like confusion patterns (0/O, 1/l, 5/S) for realistic text corruption.</p>
        </div>
        <div class="info-card">
          <h4>Format Variations</h4>
          <p>Inconsistent formatting across records: date formats (ISO/US/EU), amount formats (comma vs. period decimals), and identifier casing and padding variations.</p>
        </div>
        <div class="info-card">
          <h4>Duplicate Records</h4>
          <p>Configurable exact and fuzzy duplicate injection with controllable rates. Fuzzy duplicates have subtle field variations that challenge deduplication algorithms.</p>
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
  .warning-text { font-family: var(--font-sans); margin-left: var(--space-2); color: var(--color-warning, #eab308); }
  .info-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .event-list { display: flex; flex-direction: column; gap: var(--space-3); }
  .event-item { display: grid; grid-template-columns: 1fr 1fr 2fr 1fr 1fr auto; gap: var(--space-2); align-items: center; padding: var(--space-3); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .event-item input, .event-item select { font-size: 0.8125rem; }
  .btn-danger { background-color: var(--color-error, #ef4444); color: white; border: none; padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); cursor: pointer; font-size: 0.75rem; }
  .btn-outline { background: none; border: 1px solid var(--color-border); padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); cursor: pointer; font-size: 0.8125rem; color: var(--color-text-secondary); }
  .btn-outline:hover { background-color: var(--color-background); color: var(--color-text-primary); }
  select { width: 100%; padding: var(--space-2) var(--space-3); font-size: 0.875rem; color: var(--color-text-primary); background-color: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; }
  select:focus { outline: none; border-color: var(--color-accent); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid { grid-template-columns: 1fr; } .event-item { grid-template-columns: 1fr; } }
</style>
