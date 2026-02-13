<script lang="ts">
  import { configStore, DRIFT_TYPES, CALENDAR_REGIONS, HALF_DAY_POLICIES, MONTH_END_CONVENTIONS, PERIOD_END_MODELS, FISCAL_CALENDAR_TYPES, COMMON_TIMEZONES, POSTING_TYPES, DEFAULT_INTRADAY_SEGMENTS } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
  const validationErrors = configStore.validationErrors;

  // Track which main section is active
  let activeTab: 'patterns' | 'drift' = 'patterns';

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  function toggleRegion(region: string) {
    if (!$config) return;
    const regions = $config.temporal_patterns.calendars.regions;
    const idx = regions.indexOf(region);
    if (idx === -1) {
      $config.temporal_patterns.calendars.regions = [...regions, region];
    } else {
      $config.temporal_patterns.calendars.regions = regions.filter(r => r !== region);
    }
  }

  function addIntraDaySegment() {
    if (!$config) return;
    $config.temporal_patterns.intraday.segments = [
      ...$config.temporal_patterns.intraday.segments,
      { name: '', start: '09:00', end: '17:00', multiplier: 1.0, posting_type: 'both' }
    ];
  }

  function removeIntraDaySegment(index: number) {
    if (!$config) return;
    $config.temporal_patterns.intraday.segments = $config.temporal_patterns.intraday.segments.filter((_, i) => i !== index);
  }

  function useDefaultSegments() {
    if (!$config) return;
    $config.temporal_patterns.intraday.segments = [...DEFAULT_INTRADAY_SEGMENTS];
  }

  function addEntityTimezoneMapping() {
    if (!$config) return;
    $config.temporal_patterns.timezones.entity_mappings = [
      ...$config.temporal_patterns.timezones.entity_mappings,
      { pattern: '', timezone: 'America/New_York' }
    ];
  }

  function removeEntityTimezoneMapping(index: number) {
    if (!$config) return;
    $config.temporal_patterns.timezones.entity_mappings = $config.temporal_patterns.timezones.entity_mappings.filter((_, i) => i !== index);
  }
</script>

<div class="page">
  <ConfigPageHeader title="Temporal Configuration" description="Configure business day calculations, period-end dynamics, and temporal drift simulation" />

  <!-- Tab Navigation -->
  <div class="tab-nav">
    <button
      class="tab-btn"
      class:active={activeTab === 'patterns'}
      onclick={() => activeTab = 'patterns'}
    >
      Temporal Patterns
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'drift'}
      onclick={() => activeTab = 'drift'}
    >
      Temporal Drift (ML)
    </button>
  </div>

  {#if $config}
    <div class="page-content">
      <!-- ============================================================= -->
      <!-- TEMPORAL PATTERNS TAB -->
      <!-- ============================================================= -->
      {#if activeTab === 'patterns'}
        <FormSection title="Temporal Patterns" description="Enable realistic business temporal patterns">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.temporal_patterns.enabled}
                label="Enable Temporal Patterns"
                description="Activate business day calculations, period-end dynamics, and processing lags"
              />
            </div>
          {/snippet}
        </FormSection>

        {#if $config.temporal_patterns.enabled}
          <!-- Business Days Section -->
          <FormSection title="Business Day Calculations" description="Configure business day rules and settlement dates">
            {#snippet children()}
              <div class="form-stack">
                <Toggle
                  bind:checked={$config.temporal_patterns.business_days.enabled}
                  label="Enable Business Day Calculations"
                  description="Apply business day logic to dates and settlements"
                />

                {#if $config.temporal_patterns.business_days.enabled}
                  <div class="form-grid">
                    <FormGroup
                      label="Half-Day Policy"
                      htmlFor="half-day-policy"
                      helpText="How to treat half-day holidays"
                    >
                      {#snippet children()}
                        <select
                          id="half-day-policy"
                          bind:value={$config.temporal_patterns.business_days.half_day_policy}
                        >
                          {#each HALF_DAY_POLICIES as policy}
                            <option value={policy.value}>{policy.label}</option>
                          {/each}
                        </select>
                      {/snippet}
                    </FormGroup>

                    <FormGroup
                      label="Month-End Convention"
                      htmlFor="month-end-convention"
                      helpText="How to handle dates falling on non-business days"
                    >
                      {#snippet children()}
                        <select
                          id="month-end-convention"
                          bind:value={$config.temporal_patterns.business_days.month_end_convention}
                        >
                          {#each MONTH_END_CONVENTIONS as convention}
                            <option value={convention.value}>{convention.label}</option>
                          {/each}
                        </select>
                      {/snippet}
                    </FormGroup>
                  </div>

                  <h4 class="subsection-title">Settlement Rules</h4>
                  <div class="form-grid-3">
                    <FormGroup label="Equity (T+N)" htmlFor="equity-days" helpText="Stock settlement days">
                      {#snippet children()}
                        <input type="number" id="equity-days" bind:value={$config.temporal_patterns.business_days.settlement_rules.equity_days} min="0" max="10" />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="Gov't Bonds" htmlFor="govt-bonds-days" helpText="Government bond settlement">
                      {#snippet children()}
                        <input type="number" id="govt-bonds-days" bind:value={$config.temporal_patterns.business_days.settlement_rules.government_bonds_days} min="0" max="10" />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="FX Spot" htmlFor="fx-spot-days" helpText="Foreign exchange spot">
                      {#snippet children()}
                        <input type="number" id="fx-spot-days" bind:value={$config.temporal_patterns.business_days.settlement_rules.fx_spot_days} min="0" max="10" />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="Corp Bonds" htmlFor="corp-bonds-days" helpText="Corporate bond settlement">
                      {#snippet children()}
                        <input type="number" id="corp-bonds-days" bind:value={$config.temporal_patterns.business_days.settlement_rules.corporate_bonds_days} min="0" max="10" />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="Wire Cutoff" htmlFor="wire-cutoff" helpText="Same-day wire cutoff time">
                      {#snippet children()}
                        <input type="time" id="wire-cutoff" bind:value={$config.temporal_patterns.business_days.settlement_rules.wire_cutoff_time} />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="ACH Days" htmlFor="ach-days" helpText="ACH transfer settlement">
                      {#snippet children()}
                        <input type="number" id="ach-days" bind:value={$config.temporal_patterns.business_days.settlement_rules.ach_days} min="0" max="10" />
                      {/snippet}
                    </FormGroup>
                  </div>
                {/if}
              </div>
            {/snippet}
          </FormSection>

          <!-- Calendar Regions Section -->
          <FormSection title="Holiday Calendars" description="Select regions for holiday calendars">
            {#snippet children()}
              <div class="region-grid">
                {#each CALENDAR_REGIONS as region}
                  <label class="region-chip" class:selected={$config.temporal_patterns.calendars.regions.includes(region.value)}>
                    <input
                      type="checkbox"
                      checked={$config.temporal_patterns.calendars.regions.includes(region.value)}
                      onchange={() => toggleRegion(region.value)}
                    />
                    <span class="region-label">{region.label}</span>
                    <span class="region-code">{region.value}</span>
                  </label>
                {/each}
              </div>
            {/snippet}
          </FormSection>

          <!-- Period-End Dynamics Section -->
          <FormSection title="Period-End Dynamics" description="Configure month/quarter/year-end volume patterns">
            {#snippet children()}
              <div class="form-stack">
                <FormGroup
                  label="Period-End Model"
                  htmlFor="period-end-model"
                  helpText="How volume accelerates toward period end"
                >
                  {#snippet children()}
                    <select
                      id="period-end-model"
                      bind:value={$config.temporal_patterns.period_end.model}
                    >
                      <option value={null}>Disabled</option>
                      {#each PERIOD_END_MODELS as model}
                        <option value={model.value}>{model.label}</option>
                      {/each}
                    </select>
                  {/snippet}
                </FormGroup>

                {#if $config.temporal_patterns.period_end.model === 'exponential'}
                  <h4 class="subsection-title">Month-End Configuration</h4>
                  <div class="form-grid">
                    <FormGroup label="Start Day" htmlFor="me-start-day" helpText="Days before month-end to start (negative)">
                      {#snippet children()}
                        <input type="number" id="me-start-day" value={$config.temporal_patterns.period_end.month_end?.start_day ?? -10} onchange={(e) => {
                          if (!$config.temporal_patterns.period_end.month_end) {
                            $config.temporal_patterns.period_end.month_end = { inherit_from: null, additional_multiplier: null, start_day: null, base_multiplier: null, peak_multiplier: null, decay_rate: null, sustained_high_days: null };
                          }
                          $config.temporal_patterns.period_end.month_end.start_day = parseInt(e.currentTarget.value);
                        }} min="-30" max="0" />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="Base Multiplier" htmlFor="me-base-mult" helpText="Starting activity multiplier">
                      {#snippet children()}
                        <input type="number" id="me-base-mult" value={$config.temporal_patterns.period_end.month_end?.base_multiplier ?? 1.0} onchange={(e) => {
                          if (!$config.temporal_patterns.period_end.month_end) {
                            $config.temporal_patterns.period_end.month_end = { inherit_from: null, additional_multiplier: null, start_day: null, base_multiplier: null, peak_multiplier: null, decay_rate: null, sustained_high_days: null };
                          }
                          $config.temporal_patterns.period_end.month_end.base_multiplier = parseFloat(e.currentTarget.value);
                        }} min="0.1" max="5" step="0.1" />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="Peak Multiplier" htmlFor="me-peak-mult" helpText="Activity on last day">
                      {#snippet children()}
                        <input type="number" id="me-peak-mult" value={$config.temporal_patterns.period_end.month_end?.peak_multiplier ?? 3.5} onchange={(e) => {
                          if (!$config.temporal_patterns.period_end.month_end) {
                            $config.temporal_patterns.period_end.month_end = { inherit_from: null, additional_multiplier: null, start_day: null, base_multiplier: null, peak_multiplier: null, decay_rate: null, sustained_high_days: null };
                          }
                          $config.temporal_patterns.period_end.month_end.peak_multiplier = parseFloat(e.currentTarget.value);
                        }} min="1" max="10" step="0.1" />
                      {/snippet}
                    </FormGroup>
                    <FormGroup label="Decay Rate" htmlFor="me-decay-rate" helpText="Exponential decay rate (0.1-0.5)">
                      {#snippet children()}
                        <input type="number" id="me-decay-rate" value={$config.temporal_patterns.period_end.month_end?.decay_rate ?? 0.3} onchange={(e) => {
                          if (!$config.temporal_patterns.period_end.month_end) {
                            $config.temporal_patterns.period_end.month_end = { inherit_from: null, additional_multiplier: null, start_day: null, base_multiplier: null, peak_multiplier: null, decay_rate: null, sustained_high_days: null };
                          }
                          $config.temporal_patterns.period_end.month_end.decay_rate = parseFloat(e.currentTarget.value);
                        }} min="0.1" max="1.0" step="0.05" />
                      {/snippet}
                    </FormGroup>
                  </div>
                {/if}
              </div>
            {/snippet}
          </FormSection>

          <!-- Processing Lags Section -->
          <FormSection title="Processing Lags" description="Configure event-to-posting time delays">
            {#snippet children()}
              <div class="form-stack">
                <Toggle
                  bind:checked={$config.temporal_patterns.processing_lags.enabled}
                  label="Enable Processing Lags"
                  description="Add realistic delays between events and postings"
                />

                {#if $config.temporal_patterns.processing_lags.enabled}
                  <div class="info-box">
                    Lag distributions use log-normal parameters (mu, sigma). Typical values: mu=0.5-2.0, sigma=0.3-1.0.
                    Higher mu = longer average lag. Higher sigma = more variation.
                  </div>
                {/if}
              </div>
            {/snippet}
          </FormSection>

          <!-- Fiscal Calendar Section -->
          <FormSection title="Fiscal Calendar" description="Configure non-calendar fiscal year">
            {#snippet children()}
              <div class="form-stack">
                <Toggle
                  bind:checked={$config.temporal_patterns.fiscal_calendar.enabled}
                  label="Enable Custom Fiscal Calendar"
                  description="Use non-calendar year fiscal periods"
                />

                {#if $config.temporal_patterns.fiscal_calendar.enabled}
                  <div class="form-grid">
                    <FormGroup
                      label="Calendar Type"
                      htmlFor="fiscal-type"
                      helpText="Type of fiscal calendar"
                    >
                      {#snippet children()}
                        <select
                          id="fiscal-type"
                          bind:value={$config.temporal_patterns.fiscal_calendar.calendar_type}
                        >
                          {#each FISCAL_CALENDAR_TYPES as type}
                            <option value={type.value}>{type.label}</option>
                          {/each}
                        </select>
                      {/snippet}
                    </FormGroup>

                    {#if $config.temporal_patterns.fiscal_calendar.calendar_type === 'custom'}
                      <FormGroup label="Year Start Month" htmlFor="fiscal-month" helpText="Month fiscal year begins (1-12)">
                        {#snippet children()}
                          <input type="number" id="fiscal-month" bind:value={$config.temporal_patterns.fiscal_calendar.year_start_month} min="1" max="12" />
                        {/snippet}
                      </FormGroup>
                    {/if}
                  </div>
                {/if}
              </div>
            {/snippet}
          </FormSection>

          <!-- Intra-Day Patterns Section -->
          <FormSection title="Intra-Day Patterns" description="Configure time-of-day activity patterns">
            {#snippet children()}
              <div class="form-stack">
                <Toggle
                  bind:checked={$config.temporal_patterns.intraday.enabled}
                  label="Enable Intra-Day Patterns"
                  description="Apply different activity multipliers by time of day"
                />

                {#if $config.temporal_patterns.intraday.enabled}
                  <div class="segment-actions">
                    <button class="btn-secondary btn-sm" onclick={useDefaultSegments}>
                      Use Default Segments
                    </button>
                    <button class="btn-secondary btn-sm" onclick={addIntraDaySegment}>
                      Add Segment
                    </button>
                  </div>

                  {#if $config.temporal_patterns.intraday.segments.length > 0}
                    <div class="segments-list">
                      {#each $config.temporal_patterns.intraday.segments as segment, i}
                        <div class="segment-row">
                          <input type="text" placeholder="Name" bind:value={segment.name} class="segment-name" />
                          <input type="time" bind:value={segment.start} class="segment-time" />
                          <span class="segment-separator">to</span>
                          <input type="time" bind:value={segment.end} class="segment-time" />
                          <input type="number" bind:value={segment.multiplier} min="0" max="5" step="0.1" class="segment-multiplier" />
                          <select bind:value={segment.posting_type} class="segment-posting">
                            {#each POSTING_TYPES as type}
                              <option value={type.value}>{type.label}</option>
                            {/each}
                          </select>
                          <button class="btn-icon btn-danger" onclick={() => removeIntraDaySegment(i)} title="Remove">
                            ×
                          </button>
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <p class="empty-message">No segments configured. Click "Use Default Segments" to add common patterns.</p>
                  {/if}
                {/if}
              </div>
            {/snippet}
          </FormSection>

          <!-- Timezone Section -->
          <FormSection title="Timezone Handling" description="Configure multi-region timezone support">
            {#snippet children()}
              <div class="form-stack">
                <Toggle
                  bind:checked={$config.temporal_patterns.timezones.enabled}
                  label="Enable Timezone Handling"
                  description="Apply timezone conversions for multi-region entities"
                />

                {#if $config.temporal_patterns.timezones.enabled}
                  <div class="form-grid">
                    <FormGroup
                      label="Default Timezone"
                      htmlFor="default-tz"
                      helpText="Timezone for entities without specific mapping"
                    >
                      {#snippet children()}
                        <select id="default-tz" bind:value={$config.temporal_patterns.timezones.default_timezone}>
                          {#each COMMON_TIMEZONES as tz}
                            <option value={tz.value}>{tz.label} ({tz.offset})</option>
                          {/each}
                        </select>
                      {/snippet}
                    </FormGroup>

                    <FormGroup
                      label="Consolidation Timezone"
                      htmlFor="consolidation-tz"
                      helpText="Timezone for group reporting"
                    >
                      {#snippet children()}
                        <select id="consolidation-tz" bind:value={$config.temporal_patterns.timezones.consolidation_timezone}>
                          {#each COMMON_TIMEZONES as tz}
                            <option value={tz.value}>{tz.label} ({tz.offset})</option>
                          {/each}
                        </select>
                      {/snippet}
                    </FormGroup>
                  </div>

                  <h4 class="subsection-title">Entity Timezone Mappings</h4>
                  <button class="btn-secondary btn-sm" onclick={addEntityTimezoneMapping}>
                    Add Mapping
                  </button>

                  {#if $config.temporal_patterns.timezones.entity_mappings.length > 0}
                    <div class="mappings-list">
                      {#each $config.temporal_patterns.timezones.entity_mappings as mapping, i}
                        <div class="mapping-row">
                          <input type="text" placeholder="Pattern (e.g., EU_*)" bind:value={mapping.pattern} class="mapping-pattern" />
                          <span class="mapping-arrow">→</span>
                          <select bind:value={mapping.timezone} class="mapping-timezone">
                            {#each COMMON_TIMEZONES as tz}
                              <option value={tz.value}>{tz.label}</option>
                            {/each}
                          </select>
                          <button class="btn-icon btn-danger" onclick={() => removeEntityTimezoneMapping(i)} title="Remove">
                            ×
                          </button>
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <p class="empty-message">No entity mappings configured. Use patterns like "EU_*" or "*_APAC".</p>
                  {/if}
                {/if}
              </div>
            {/snippet}
          </FormSection>
        {/if}

      <!-- ============================================================= -->
      <!-- TEMPORAL DRIFT TAB (ML Training) -->
      <!-- ============================================================= -->
      {:else if activeTab === 'drift'}
        <FormSection title="Drift Simulation" description="Enable temporal drift to simulate realistic data evolution for ML training">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.temporal.enabled}
                label="Enable Temporal Drift"
                description="Generate data that shows realistic temporal evolution"
              />

              {#if $config.temporal.enabled}
                <FormGroup
                  label="Drift Type"
                  htmlFor="drift-type"
                  helpText="Select the type of drift pattern to simulate"
                >
                  {#snippet children()}
                    <div class="drift-type-selector">
                      {#each DRIFT_TYPES as type}
                        <label class="drift-type-option" class:selected={$config.temporal.drift_type === type.value}>
                          <input
                            type="radio"
                            name="drift-type"
                            value={type.value}
                            bind:group={$config.temporal.drift_type}
                          />
                          <span class="drift-type-label">{type.label}</span>
                          <span class="drift-type-desc">{type.description}</span>
                        </label>
                      {/each}
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Drift Start Period"
                  htmlFor="drift-start"
                  helpText="Period (month) when drift begins (0 = from start)"
                >
                  {#snippet children()}
                    <input
                      type="number"
                      id="drift-start"
                      bind:value={$config.temporal.drift_start_period}
                      min="0"
                      max="120"
                    />
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        {#if $config.temporal.enabled}
          <FormSection title="Amount Distribution Drift" description="How transaction amounts change over time">
            {#snippet children()}
              <div class="form-grid">
                <FormGroup
                  label="Mean Drift"
                  htmlFor="amount-mean-drift"
                  helpText="Amount mean shift per period (e.g., 0.02 = 2% increase per month)"
                  error={getError('temporal.amount_mean_drift')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="amount-mean-drift"
                        bind:value={$config.temporal.amount_mean_drift}
                        min="-0.1"
                        max="0.1"
                        step="0.005"
                      />
                      <span class="slider-value">{($config.temporal.amount_mean_drift * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Variance Drift"
                  htmlFor="amount-variance-drift"
                  helpText="Amount variance increase per period (simulates increasing volatility)"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="amount-variance-drift"
                        bind:value={$config.temporal.amount_variance_drift}
                        min="0"
                        max="0.1"
                        step="0.005"
                      />
                      <span class="slider-value">{($config.temporal.amount_variance_drift * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>
            {/snippet}
          </FormSection>

          <FormSection title="Anomaly & Concept Drift" description="How patterns and anomaly rates evolve">
            {#snippet children()}
              <div class="form-grid">
                <FormGroup
                  label="Anomaly Rate Drift"
                  htmlFor="anomaly-drift"
                  helpText="Increase in anomaly rate per period (simulates degrading controls)"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="anomaly-drift"
                        bind:value={$config.temporal.anomaly_rate_drift}
                        min="0"
                        max="0.01"
                        step="0.0005"
                      />
                      <span class="slider-value">{($config.temporal.anomaly_rate_drift * 100).toFixed(2)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Concept Drift Rate"
                  htmlFor="concept-drift"
                  helpText="Rate of feature distribution changes (0-1, higher = faster changes)"
                  error={getError('temporal.concept_drift_rate')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="concept-drift"
                        bind:value={$config.temporal.concept_drift_rate}
                        min="0"
                        max="0.1"
                        step="0.005"
                      />
                      <span class="slider-value">{($config.temporal.concept_drift_rate * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>
            {/snippet}
          </FormSection>

          <FormSection title="Sudden Drift Events" description="Configure occasional sudden shifts in distributions">
            {#snippet children()}
              <div class="form-grid">
                <FormGroup
                  label="Sudden Drift Probability"
                  htmlFor="sudden-prob"
                  helpText="Probability of a sudden shift occurring in any period"
                  error={getError('temporal.sudden_drift_probability')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="sudden-prob"
                        bind:value={$config.temporal.sudden_drift_probability}
                        min="0"
                        max="0.2"
                        step="0.01"
                      />
                      <span class="slider-value">{($config.temporal.sudden_drift_probability * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Sudden Drift Magnitude"
                  htmlFor="sudden-mag"
                  helpText="Magnitude multiplier when sudden drift occurs"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="sudden-mag"
                        bind:value={$config.temporal.sudden_drift_magnitude}
                        min="1"
                        max="5"
                        step="0.1"
                      />
                      <span class="slider-value">{$config.temporal.sudden_drift_magnitude.toFixed(1)}x</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>

              <Toggle
                bind:checked={$config.temporal.seasonal_drift}
                label="Enable Seasonal Drift"
                description="Add cyclic patterns that repeat annually"
              />
            {/snippet}
          </FormSection>
        {/if}
      {/if}

      <!-- Info Section -->
      <div class="info-section">
        <h2>About {activeTab === 'patterns' ? 'Temporal Patterns' : 'Temporal Drift'}</h2>
        <div class="info-grid">
          {#if activeTab === 'patterns'}
            <div class="info-card">
              <h3>Business Days</h3>
              <p>
                Calculate settlement dates using T+N rules, handle holidays across 11 regions,
                and apply proper month-end conventions for financial instruments.
              </p>
            </div>
            <div class="info-card">
              <h3>Period-End Dynamics</h3>
              <p>
                Model realistic volume acceleration toward period close with exponential curves
                instead of flat multipliers. Quarter and year-end can inherit from month-end.
              </p>
            </div>
            <div class="info-card">
              <h3>Processing Lags</h3>
              <p>
                Add realistic delays between business events and posting timestamps using
                log-normal distributions. Includes cross-day posting for late events.
              </p>
            </div>
            <div class="info-card">
              <h3>Multi-Region Timezones</h3>
              <p>
                Map entities to timezones by pattern matching (e.g., "EU_*" → Europe/London).
                All timestamps can be converted to consolidation timezone for reporting.
              </p>
            </div>
          {:else}
            <div class="info-card">
              <h3>Use Cases</h3>
              <p>
                Temporal drift simulation is useful for training drift detection models,
                testing temporal robustness, and simulating realistic data evolution
                like inflation or changing fraud patterns.
              </p>
            </div>
            <div class="info-card">
              <h3>Drift Types</h3>
              <p>
                <strong>Gradual:</strong> Continuous drift like inflation.<br/>
                <strong>Sudden:</strong> Point-in-time shifts like policy changes.<br/>
                <strong>Recurring:</strong> Cyclic patterns like seasonal variations.
              </p>
            </div>
          {/if}
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
  .page {
    max-width: 900px;
  }

  /* Tab Navigation */
  .tab-nav {
    display: flex;
    gap: var(--space-1);
    margin-bottom: var(--space-5);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: var(--space-2);
  }

  .tab-btn {
    padding: var(--space-2) var(--space-4);
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    transition: all var(--transition-fast);
  }

  .tab-btn:hover {
    color: var(--color-text-primary);
    background: var(--color-surface);
  }

  .tab-btn.active {
    color: var(--color-accent);
    background: var(--color-surface);
    border-bottom: 2px solid var(--color-accent);
    margin-bottom: -1px;
  }

  .page-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .form-stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .form-grid-3 {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
  }

  .subsection-title {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin-top: var(--space-4);
    margin-bottom: var(--space-2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Region Grid */
  .region-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--space-2);
  }

  .region-chip {
    display: flex;
    flex-direction: column;
    padding: var(--space-2) var(--space-3);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .region-chip:hover {
    border-color: var(--color-accent);
  }

  .region-chip.selected {
    border-color: var(--color-accent);
    background-color: rgba(59, 130, 246, 0.08);
  }

  .region-chip input {
    display: none;
  }

  .region-label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .region-code {
    font-size: 0.6875rem;
    color: var(--color-text-secondary);
    font-family: var(--font-mono);
  }

  /* Intra-Day Segments */
  .segment-actions {
    display: flex;
    gap: var(--space-2);
  }

  .segments-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .segment-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    background: var(--color-surface);
    border-radius: var(--radius-md);
  }

  .segment-name {
    width: 120px;
  }

  .segment-time {
    width: 90px;
  }

  .segment-separator {
    color: var(--color-text-secondary);
    font-size: 0.75rem;
  }

  .segment-multiplier {
    width: 70px;
  }

  .segment-posting {
    width: 100px;
  }

  /* Timezone Mappings */
  .mappings-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  .mapping-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    background: var(--color-surface);
    border-radius: var(--radius-md);
  }

  .mapping-pattern {
    width: 140px;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
  }

  .mapping-arrow {
    color: var(--color-text-secondary);
  }

  .mapping-timezone {
    flex: 1;
  }

  /* Buttons */
  .btn-sm {
    font-size: 0.75rem;
    padding: var(--space-1) var(--space-2);
  }

  .btn-icon {
    width: 24px;
    height: 24px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    border: none;
    cursor: pointer;
    font-size: 1rem;
  }

  .btn-danger {
    background: transparent;
    color: var(--color-error);
  }

  .btn-danger:hover {
    background: rgba(239, 68, 68, 0.1);
  }

  /* Info */
  .info-box {
    padding: var(--space-3);
    background: rgba(59, 130, 246, 0.08);
    border-left: 3px solid var(--color-accent);
    border-radius: 0 var(--radius-md) var(--radius-md) 0;
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .empty-message {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    font-style: italic;
    padding: var(--space-3);
    text-align: center;
  }

  /* Drift Type Selector (from original) */
  .drift-type-selector {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-2);
  }

  .drift-type-option {
    display: flex;
    flex-direction: column;
    padding: var(--space-3);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .drift-type-option:hover {
    border-color: var(--color-accent);
  }

  .drift-type-option.selected {
    border-color: var(--color-accent);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .drift-type-option input {
    display: none;
  }

  .drift-type-label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .drift-type-desc {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  /* Sliders */
  .slider-with-value {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .slider-with-value input[type="range"] {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: var(--color-background);
    appearance: none;
    cursor: pointer;
  }

  .slider-with-value input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .slider-value {
    min-width: 60px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--color-text-primary);
  }

  /* Info Section */
  .info-section {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
  }

  .info-section h2 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: var(--space-4);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .info-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .info-card h3 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .info-card p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid,
    .form-grid-3,
    .drift-type-selector,
    .info-grid,
    .region-grid {
      grid-template-columns: 1fr;
    }

    .segment-row,
    .mapping-row {
      flex-wrap: wrap;
    }
  }
</style>
