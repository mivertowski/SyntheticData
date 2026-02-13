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

  function getTotalEngagementWeight(): number {
    if (!$config?.audit?.engagement_types) return 0;
    const e = $config.audit.engagement_types;
    return e.financial_statement + e.sox_icfr + e.integrated + e.review + e.agreed_upon_procedures;
  }

  function getTotalSamplingWeight(): number {
    if (!$config?.audit?.workpapers?.sampling) return 0;
    const s = $config.audit.workpapers.sampling;
    return s.statistical_rate + s.judgmental_rate + s.haphazard_rate + s.complete_examination_rate;
  }
</script>

<div class="page">
  <ConfigPageHeader title="Audit Generation" description="Generate audit engagements, workpapers, and evidence per ISA standards" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Audit Generation" description="Enable audit data generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.audit.enabled}
              label="Enable Audit Generation"
              description="Generate audit engagements, workpapers, evidence, and findings"
            />

            {#if $config.audit.enabled}
              <Toggle
                bind:checked={$config.audit.generate_workpapers}
                label="Generate Workpapers"
                description="Generate detailed workpaper documents per ISA 230"
              />
            {/if}
          </div>
        {/snippet}
      </FormSection>

      {#if $config.audit.enabled}
        <FormSection title="Engagement Types" description="Distribution of engagement types">
          {#snippet children()}
            <div class="distribution-grid">
              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">Financial Statement Audit</span>
                  <span class="dist-value">{($config.audit.engagement_types.financial_statement * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.engagement_types.financial_statement}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>

              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">SOX/ICFR Audit</span>
                  <span class="dist-value">{($config.audit.engagement_types.sox_icfr * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.engagement_types.sox_icfr}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>

              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">Integrated Audit</span>
                  <span class="dist-value">{($config.audit.engagement_types.integrated * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.engagement_types.integrated}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>

              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">Review Engagement</span>
                  <span class="dist-value">{($config.audit.engagement_types.review * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.engagement_types.review}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>

              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">Agreed-Upon Procedures</span>
                  <span class="dist-value">{($config.audit.engagement_types.agreed_upon_procedures * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.engagement_types.agreed_upon_procedures}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>
            </div>

            <div class="distribution-total" class:warning={Math.abs(getTotalEngagementWeight() - 1.0) > 0.01}>
              Total: {(getTotalEngagementWeight() * 100).toFixed(0)}%
              {#if Math.abs(getTotalEngagementWeight() - 1.0) > 0.01}
                <span class="warning-text">(should sum to 100%)</span>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Workpaper Settings" description="Configure workpaper generation">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Average Workpapers per Phase"
                htmlFor="avg-wp"
                helpText="Average number of workpapers generated per engagement phase"
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="avg-wp"
                    bind:value={$config.audit.workpapers.average_per_phase}
                    min="1"
                    max="20"
                  />
                {/snippet}
              </FormGroup>

              <div class="toggle-grid">
                <Toggle
                  bind:checked={$config.audit.workpapers.include_isa_references}
                  label="Include ISA References"
                  description="Include ISA standard references in workpapers"
                />

                <Toggle
                  bind:checked={$config.audit.workpapers.include_sample_details}
                  label="Include Sample Details"
                  description="Include detailed sample information"
                />

                <Toggle
                  bind:checked={$config.audit.workpapers.include_cross_references}
                  label="Include Cross References"
                  description="Include cross-references between workpapers"
                />
              </div>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Sampling Methods" description="Distribution of sampling methods used">
          {#snippet children()}
            <div class="distribution-grid">
              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">Statistical Sampling</span>
                  <span class="dist-value">{($config.audit.workpapers.sampling.statistical_rate * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.workpapers.sampling.statistical_rate}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>

              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">Judgmental Sampling</span>
                  <span class="dist-value">{($config.audit.workpapers.sampling.judgmental_rate * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.workpapers.sampling.judgmental_rate}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>

              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">Haphazard Sampling</span>
                  <span class="dist-value">{($config.audit.workpapers.sampling.haphazard_rate * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.workpapers.sampling.haphazard_rate}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>

              <div class="distribution-item">
                <div class="dist-header">
                  <span class="dist-label">100% Examination</span>
                  <span class="dist-value">{($config.audit.workpapers.sampling.complete_examination_rate * 100).toFixed(0)}%</span>
                </div>
                <input
                  type="range"
                  bind:value={$config.audit.workpapers.sampling.complete_examination_rate}
                  min="0"
                  max="1"
                  step="0.05"
                />
              </div>
            </div>

            <div class="distribution-total" class:warning={Math.abs(getTotalSamplingWeight() - 1.0) > 0.01}>
              Total: {(getTotalSamplingWeight() * 100).toFixed(0)}%
              {#if Math.abs(getTotalSamplingWeight() - 1.0) > 0.01}
                <span class="warning-text">(should sum to 100%)</span>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Team Configuration" description="Configure audit team settings">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Min Team Size"
                htmlFor="min-team"
                helpText="Minimum number of team members"
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="min-team"
                    bind:value={$config.audit.team.min_team_size}
                    min="1"
                    max="20"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Max Team Size"
                htmlFor="max-team"
                helpText="Maximum number of team members"
                error={getError('audit.team.max_team_size')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="max-team"
                    bind:value={$config.audit.team.max_team_size}
                    min="1"
                    max="20"
                  />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Specialist Probability"
                htmlFor="specialist"
                helpText="Probability of having a specialist on the team"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="specialist"
                      bind:value={$config.audit.team.specialist_probability}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span class="slider-value">{($config.audit.team.specialist_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Review Workflow" description="Configure review and sign-off process">
          {#snippet children()}
            <div class="form-stack">
              <div class="form-grid">
                <FormGroup
                  label="Average Review Delay"
                  htmlFor="review-delay"
                  helpText="Average days between preparer completion and first review"
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input
                        type="number"
                        id="review-delay"
                        bind:value={$config.audit.review.average_review_delay_days}
                        min="0"
                        max="30"
                      />
                      <span class="suffix">days</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Rework Probability"
                  htmlFor="rework"
                  helpText="Probability of review notes requiring rework"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="rework"
                        bind:value={$config.audit.review.rework_probability}
                        min="0"
                        max="0.5"
                        step="0.05"
                      />
                      <span class="slider-value">{($config.audit.review.rework_probability * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>

              <Toggle
                bind:checked={$config.audit.review.require_partner_signoff}
                label="Require Partner Sign-off"
                description="Require partner sign-off for all workpapers"
              />
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-section">
        <h2>About Audit Generation</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>ISA Compliance</h3>
            <p>
              Generated audit data follows International Standards on Auditing (ISA),
              including workpaper documentation (ISA 230), evidence (ISA 500),
              and risk assessment (ISA 315).
            </p>
          </div>
          <div class="info-card">
            <h3>Output Files</h3>
            <p>
              Generates audit_engagements.csv, audit_workpapers.csv, audit_evidence.csv,
              audit_risks.csv, audit_findings.csv, and audit_judgments.csv.
            </p>
          </div>
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

  .toggle-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-3);
  }

  .distribution-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .distribution-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .dist-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .dist-label {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .dist-value {
    font-size: 0.8125rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    font-weight: 500;
  }

  .distribution-item input[type="range"] {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--color-background);
    appearance: none;
    cursor: pointer;
  }

  .distribution-item input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .distribution-total {
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    font-size: 0.875rem;
    font-family: var(--font-mono);
    text-align: center;
    color: var(--color-text-secondary);
  }

  .distribution-total.warning {
    background-color: rgba(255, 193, 7, 0.1);
    color: var(--color-warning);
  }

  .warning-text {
    font-family: var(--font-sans);
    margin-left: var(--space-2);
  }

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
    min-width: 50px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--color-text-primary);
  }

  .input-with-suffix {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .input-with-suffix input {
    flex: 1;
  }

  .input-with-suffix .suffix {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

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
    .toggle-grid,
    .distribution-grid,
    .info-grid {
      grid-template-columns: 1fr;
    }

  }
</style>
