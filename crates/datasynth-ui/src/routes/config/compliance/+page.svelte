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

  function getTotalFraudWeight(): number {
    if (!$config?.fraud?.fraud_type_distribution) return 0;
    const d = $config.fraud.fraud_type_distribution;
    return (
      d.suspense_account_abuse +
      d.fictitious_transaction +
      d.revenue_manipulation +
      d.expense_capitalization +
      d.split_transaction +
      d.timing_anomaly +
      d.unauthorized_access +
      d.duplicate_payment
    );
  }
</script>

<div class="page">
  <ConfigPageHeader title="Fraud & Controls" description="Configure anomaly injection and SOX control simulation" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Fraud Simulation" description="Inject realistic fraud patterns for ML training">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.fraud.enabled}
              label="Enable Fraud Simulation"
              description="Inject labeled fraud patterns into generated transactions"
            />

            <div class="form-grid">
              <FormGroup
                label="Fraud Rate"
                htmlFor="fraud-rate"
                helpText="Percentage of transactions that are fraudulent (0-10%)"
                error={getError('fraud.fraud_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="fraud-rate"
                      bind:value={$config.fraud.fraud_rate}
                      min="0"
                      max="0.1"
                      step="0.001"
                      disabled={!$config.fraud.enabled}
                    />
                    <span class="suffix">{($config.fraud.fraud_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <Toggle
              bind:checked={$config.fraud.clustering_enabled}
              label="Enable Fraud Clustering"
              description="Group fraud transactions together (realistic batch patterns)"
              disabled={!$config.fraud.enabled}
            />

            {#if $config.fraud.clustering_enabled}
              <FormGroup
                label="Clustering Factor"
                htmlFor="clustering-factor"
                helpText="How tightly fraud is clustered (higher = more concentrated)"
                error={getError('fraud.clustering_factor')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="clustering-factor"
                    bind:value={$config.fraud.clustering_factor}
                    min="1"
                    max="10"
                    step="0.5"
                    disabled={!$config.fraud.enabled}
                  />
                {/snippet}
              </FormGroup>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Fraud Type Distribution" description="Relative weights for each fraud category">
        {#snippet children()}
          <div class="distribution-grid">
            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Suspense Account Abuse</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.suspense_account_abuse * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.suspense_account_abuse}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>

            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Fictitious Transaction</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.fictitious_transaction * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.fictitious_transaction}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>

            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Revenue Manipulation</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.revenue_manipulation * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.revenue_manipulation}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>

            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Expense Capitalization</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.expense_capitalization * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.expense_capitalization}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>

            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Split Transaction</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.split_transaction * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.split_transaction}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>

            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Timing Anomaly</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.timing_anomaly * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.timing_anomaly}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>

            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Unauthorized Access</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.unauthorized_access * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.unauthorized_access}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>

            <div class="distribution-item">
              <div class="dist-header">
                <span class="dist-label">Duplicate Payment</span>
                <span class="dist-value">{($config.fraud.fraud_type_distribution.duplicate_payment * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                bind:value={$config.fraud.fraud_type_distribution.duplicate_payment}
                min="0"
                max="1"
                step="0.05"
                disabled={!$config.fraud.enabled}
              />
            </div>
          </div>

          <div class="distribution-total" class:warning={Math.abs(getTotalFraudWeight() - 1.0) > 0.01}>
            Total: {(getTotalFraudWeight() * 100).toFixed(0)}%
            {#if Math.abs(getTotalFraudWeight() - 1.0) > 0.01}
              <span class="warning-text">(should sum to 100%)</span>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Internal Controls (SOX)" description="Simulate SOX 404 compliance controls">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.internal_controls.enabled}
              label="Enable Internal Controls"
              description="Generate control mappings and SoD rules"
            />

            <div class="form-grid">
              <FormGroup
                label="Exception Rate"
                htmlFor="exception-rate"
                helpText="Rate of control exceptions (0-10%)"
                error={getError('internal_controls.exception_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="exception-rate"
                      bind:value={$config.internal_controls.exception_rate}
                      min="0"
                      max="0.1"
                      step="0.005"
                      disabled={!$config.internal_controls.enabled}
                    />
                    <span class="suffix">{($config.internal_controls.exception_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="SoD Violation Rate"
                htmlFor="sod-rate"
                helpText="Rate of Segregation of Duties violations"
                error={getError('internal_controls.sod_violation_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="sod-rate"
                      bind:value={$config.internal_controls.sod_violation_rate}
                      min="0"
                      max="0.1"
                      step="0.005"
                      disabled={!$config.internal_controls.enabled}
                    />
                    <span class="suffix">{($config.internal_controls.sod_violation_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="SOX Materiality Threshold"
                htmlFor="sox-threshold"
                helpText="Amount above which transactions are SOX-relevant"
                error={getError('internal_controls.sox_materiality_threshold')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="sox-threshold"
                      bind:value={$config.internal_controls.sox_materiality_threshold}
                      min="0"
                      step="1000"
                      disabled={!$config.internal_controls.enabled}
                    />
                    <span class="suffix">USD</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <Toggle
              bind:checked={$config.internal_controls.export_control_master_data}
              label="Export Control Master Data"
              description="Generate separate files for controls, mappings, and SoD rules"
              disabled={!$config.internal_controls.enabled}
            />
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="SOX Compliance" description="Sarbanes-Oxley Act compliance assessment generation">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.audit_standards.sox.enabled}
                label="Enable SOX Assessment"
                description="Generate SOX 302 and 404 compliance assessments"
              />
              {#if $config.audit_standards?.sox?.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="SOX 302 Certification"
                    htmlFor="sox-302"
                    helpText="Generate CEO/CFO certification assessments"
                  >
                    {#snippet children()}
                      <Toggle
                        bind:checked={$config.audit_standards.sox.section_302}
                        label="Section 302"
                        description="Disclosure controls and procedures certification"
                      />
                    {/snippet}
                  </FormGroup>
                  <FormGroup
                    label="SOX 404 Assessment"
                    htmlFor="sox-404"
                    helpText="Generate internal control over financial reporting assessments"
                  >
                    {#snippet children()}
                      <Toggle
                        bind:checked={$config.audit_standards.sox.section_404}
                        label="Section 404"
                        description="Internal control effectiveness assessment"
                      />
                    {/snippet}
                  </FormGroup>
                </div>
                <FormGroup
                  label="Materiality Threshold"
                  htmlFor="sox-materiality"
                  helpText="Threshold for material weakness determination"
                >
                  {#snippet children()}
                    <input
                      type="number"
                      id="sox-materiality"
                      bind:value={$config.audit_standards.sox.materiality_threshold}
                      min="0"
                      step="1000"
                    />
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>

      <div class="info-section">
        <h2>About Fraud & Controls</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>Fraud Patterns</h3>
            <p>
              Fraud simulation injects labeled anomalies for training ML models.
              Each fraud transaction includes metadata identifying the fraud type
              and technique used.
            </p>
          </div>
          <div class="info-card">
            <h3>Internal Controls</h3>
            <p>
              Control simulation generates SOX 404-style controls with mappings
              to accounts, processes, and approval thresholds. Includes SoD
              (Segregation of Duties) conflict detection.
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
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
    min-width: 50px;
    text-align: right;
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

  .distribution-item input[type="range"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
    .form-grid {
      grid-template-columns: 1fr;
    }

    .distribution-grid {
      grid-template-columns: 1fr;
    }

    .info-grid {
      grid-template-columns: 1fr;
    }

  }
</style>
