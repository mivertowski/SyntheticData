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
</script>

<div class="page">
  <ConfigPageHeader title="ESG / Sustainability" description="Configure environmental, social, and governance metrics generation" />

  {#if $config}
    <div class="page-content">
      <FormSection title="ESG Module" description="Enable ESG and sustainability data generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.esg.enabled}
              label="Enable ESG / Sustainability"
              description="Generate emissions, diversity, governance, and supply chain ESG data"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.esg.enabled}
        <FormSection title="Environmental" description="Emissions, energy, water, and waste metrics">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.esg.environmental.enabled}
                label="Enable Environmental Metrics"
                description="Generate GHG emissions, energy consumption, water usage, and waste data"
              />

              {#if $config.esg.environmental.enabled}
                <div class="form-grid">
                  <div class="form-stack">
                    <Toggle
                      bind:checked={$config.esg.environmental.scope1_enabled}
                      label="Scope 1 Emissions"
                      description="Direct GHG emissions from owned/controlled sources"
                    />
                    <Toggle
                      bind:checked={$config.esg.environmental.scope2_enabled}
                      label="Scope 2 Emissions"
                      description="Indirect emissions from purchased energy"
                    />
                    <Toggle
                      bind:checked={$config.esg.environmental.scope3_enabled}
                      label="Scope 3 Emissions"
                      description="Value chain emissions (upstream & downstream)"
                    />
                  </div>
                  <div class="form-stack">
                    <Toggle
                      bind:checked={$config.esg.environmental.energy.enabled}
                      label="Energy Tracking"
                      description="Track energy consumption by facility"
                    />
                    <Toggle
                      bind:checked={$config.esg.environmental.water.enabled}
                      label="Water Usage"
                      description="Track water withdrawal and recycling"
                    />
                    <Toggle
                      bind:checked={$config.esg.environmental.waste.enabled}
                      label="Waste Management"
                      description="Track waste generation and diversion"
                    />
                  </div>
                </div>

                {#if $config.esg.environmental.energy.enabled}
                  <div class="form-grid">
                    <FormGroup
                      label="Facility Count"
                      htmlFor="env-facilities"
                      helpText="Number of facilities to track energy for"
                      error={getError('esg.environmental.energy.facility_count')}
                    >
                      {#snippet children()}
                        <input type="number" id="env-facilities" bind:value={$config.esg.environmental.energy.facility_count} min="1" step="1" />
                      {/snippet}
                    </FormGroup>

                    <FormGroup
                      label="Renewable Target"
                      htmlFor="renewable-target"
                      helpText="Target percentage of energy from renewable sources"
                      error={getError('esg.environmental.energy.renewable_target')}
                    >
                      {#snippet children()}
                        <div class="input-with-suffix">
                          <input type="number" id="renewable-target" bind:value={$config.esg.environmental.energy.renewable_target} min="0" max="1" step="0.05" />
                          <span class="suffix">{($config.esg.environmental.energy.renewable_target * 100).toFixed(0)}%</span>
                        </div>
                      {/snippet}
                    </FormGroup>
                  </div>
                {/if}

                {#if $config.esg.environmental.waste.enabled}
                  <FormGroup
                    label="Waste Diversion Target"
                    htmlFor="diversion-target"
                    helpText="Target waste diversion rate from landfill"
                    error={getError('esg.environmental.waste.diversion_target')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input type="number" id="diversion-target" bind:value={$config.esg.environmental.waste.diversion_target} min="0" max="1" step="0.05" />
                        <span class="suffix">{($config.esg.environmental.waste.diversion_target * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                {/if}
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Social" description="Diversity, pay equity, and workplace safety metrics">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.esg.social.enabled}
                label="Enable Social Metrics"
                description="Generate diversity, pay equity, and safety performance data"
              />

              {#if $config.esg.social.enabled}
                <Toggle
                  bind:checked={$config.esg.social.diversity.enabled}
                  label="Diversity Metrics"
                  description="Generate workforce diversity breakdown by gender, ethnicity, and age group"
                />

                <Toggle
                  bind:checked={$config.esg.social.pay_equity.enabled}
                  label="Pay Equity Analysis"
                  description="Generate pay equity analysis with gap detection"
                />

                {#if $config.esg.social.pay_equity.enabled}
                  <FormGroup
                    label="Pay Gap Threshold"
                    htmlFor="pay-gap"
                    helpText="Acceptable pay equity gap threshold"
                    error={getError('esg.social.pay_equity.gap_threshold')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input type="number" id="pay-gap" bind:value={$config.esg.social.pay_equity.gap_threshold} min="0" max="1" step="0.01" />
                        <span class="suffix">{($config.esg.social.pay_equity.gap_threshold * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                {/if}

                <Toggle
                  bind:checked={$config.esg.social.safety.enabled}
                  label="Workplace Safety"
                  description="Generate safety incident records and TRIR metrics"
                />

                {#if $config.esg.social.safety.enabled}
                  <div class="form-grid">
                    <FormGroup
                      label="Target TRIR"
                      htmlFor="target-trir"
                      helpText="Target Total Recordable Incident Rate"
                      error={getError('esg.social.safety.target_trir')}
                    >
                      {#snippet children()}
                        <input type="number" id="target-trir" bind:value={$config.esg.social.safety.target_trir} min="0" step="0.1" />
                      {/snippet}
                    </FormGroup>

                    <FormGroup
                      label="Incident Count"
                      htmlFor="incident-count"
                      helpText="Number of safety incidents to generate"
                      error={getError('esg.social.safety.incident_count')}
                    >
                      {#snippet children()}
                        <input type="number" id="incident-count" bind:value={$config.esg.social.safety.incident_count} min="0" step="1" />
                      {/snippet}
                    </FormGroup>
                  </div>
                {/if}
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Governance" description="Board composition and independence metrics">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.esg.governance.enabled}
                label="Enable Governance Metrics"
                description="Generate board composition, independence, and committee data"
              />

              {#if $config.esg.governance.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Board Size"
                    htmlFor="board-size"
                    helpText="Number of board members"
                    error={getError('esg.governance.board_size')}
                  >
                    {#snippet children()}
                      <input type="number" id="board-size" bind:value={$config.esg.governance.board_size} min="1" step="1" />
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Independence Target"
                    htmlFor="independence-target"
                    helpText="Target proportion of independent directors"
                    error={getError('esg.governance.independence_target')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input type="number" id="independence-target" bind:value={$config.esg.governance.independence_target} min="0" max="1" step="0.01" />
                        <span class="suffix">{($config.esg.governance.independence_target * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Supply Chain ESG" description="Supplier ESG assessment and scoring">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.esg.supply_chain_esg.enabled}
                label="Enable Supply Chain ESG"
                description="Generate supplier ESG risk assessments and scores"
              />

              {#if $config.esg.supply_chain_esg.enabled}
                <FormGroup
                  label="Assessment Coverage"
                  htmlFor="assessment-coverage"
                  helpText="Proportion of suppliers with ESG assessments"
                  error={getError('esg.supply_chain_esg.assessment_coverage')}
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input type="number" id="assessment-coverage" bind:value={$config.esg.supply_chain_esg.assessment_coverage} min="0" max="1" step="0.05" />
                      <span class="suffix">{($config.esg.supply_chain_esg.assessment_coverage * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Reporting & Scenarios" description="ESG reporting frameworks and climate scenario analysis">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.esg.reporting.enabled}
                label="ESG Reporting"
                description="Generate ESG disclosures aligned with GRI/ESRS frameworks"
              />

              {#if $config.esg.reporting.enabled}
                <Toggle
                  bind:checked={$config.esg.reporting.materiality_assessment}
                  label="Materiality Assessment"
                  description="Generate double materiality assessments (impact + financial)"
                />
              {/if}

              <Toggle
                bind:checked={$config.esg.climate_scenarios.enabled}
                label="Climate Scenario Analysis"
                description="Generate TCFD-aligned climate scenario projections (Net Zero 2050, Stated Policies, Current Trajectory)"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Anomaly Rate" description="ESG-specific anomaly injection">
          {#snippet children()}
            <FormGroup
              label="Anomaly Rate"
              htmlFor="esg-anomaly"
              helpText="Rate of anomalous ESG records (0-100%)"
              error={getError('esg.anomaly_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input type="number" id="esg-anomaly" bind:value={$config.esg.anomaly_rate} min="0" max="1" step="0.005" />
                  <span class="suffix">{($config.esg.anomaly_rate * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>ESG / Sustainability</h4>
          <p>
            Generates comprehensive ESG data across environmental (GHG Scope 1-3,
            energy, water, waste), social (diversity, pay equity, safety TRIR),
            and governance (board composition, independence) dimensions.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates ghg_emissions.csv, energy_consumption.csv, water_usage.csv,
            waste_records.csv, diversity_metrics.csv, pay_equity_analysis.csv,
            safety_incidents.csv, board_composition.csv, supplier_esg_scores.csv,
            and esg_disclosures.csv.
          </p>
        </div>
        <div class="info-card">
          <h4>Frameworks</h4>
          <p>
            Supports GRI Standards, ESRS (EU), TCFD climate disclosures, and
            double materiality assessments. Supply chain ESG scoring covers
            environmental risk, labor practices, and governance factors.
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
  .form-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-4); }
  .form-stack { display: flex; flex-direction: column; gap: var(--space-4); }
  .input-with-suffix { display: flex; align-items: center; gap: var(--space-2); }
  .input-with-suffix input { flex: 1; }
  .suffix { font-size: 0.8125rem; color: var(--color-text-muted); font-family: var(--font-mono); }
  .info-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid { grid-template-columns: 1fr; } }
</style>
