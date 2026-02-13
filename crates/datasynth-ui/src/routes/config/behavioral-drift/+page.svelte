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
  <ConfigPageHeader title="Behavioral Drift" description="Configure entity behavior evolution over time" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Behavioral Drift Settings" description="Enable entity behavior evolution simulation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.behavioral_drift.enabled}
              label="Enable Behavioral Drift"
              description="Simulate gradual changes in entity behavior patterns over the generation period"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.behavioral_drift.enabled}
        <FormSection title="Entity Drift Rates" description="Configure how much each entity type's behavior changes over time">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Vendor Behavior Drift"
                htmlFor="vendor-drift"
                helpText="Rate of vendor behavior change per period (0-0.2)"
                error={getError('behavioral_drift.vendor_behavior_drift')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="vendor-drift"
                      bind:value={$config.behavioral_drift.vendor_behavior_drift}
                      min="0"
                      max="0.2"
                      step="0.005"
                    />
                    <span>{($config.behavioral_drift.vendor_behavior_drift * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Customer Behavior Drift"
                htmlFor="customer-drift"
                helpText="Rate of customer behavior change per period (0-0.2)"
                error={getError('behavioral_drift.customer_behavior_drift')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="customer-drift"
                      bind:value={$config.behavioral_drift.customer_behavior_drift}
                      min="0"
                      max="0.2"
                      step="0.005"
                    />
                    <span>{($config.behavioral_drift.customer_behavior_drift * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Employee Behavior Drift"
                htmlFor="employee-drift"
                helpText="Rate of employee behavior change per period (0-0.2)"
                error={getError('behavioral_drift.employee_behavior_drift')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="employee-drift"
                      bind:value={$config.behavioral_drift.employee_behavior_drift}
                      min="0"
                      max="0.2"
                      step="0.005"
                    />
                    <span>{($config.behavioral_drift.employee_behavior_drift * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Drift Dynamics" description="Control the speed of behavioral changes">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Drift Velocity"
                htmlFor="drift-velocity"
                helpText="Overall speed multiplier for behavior changes (0-2, where 1.0 is normal pace)"
                error={getError('behavioral_drift.drift_velocity')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="drift-velocity"
                      bind:value={$config.behavioral_drift.drift_velocity}
                      min="0"
                      max="2"
                      step="0.1"
                    />
                    <span>{$config.behavioral_drift.drift_velocity.toFixed(1)}x</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Vendor Drift</h4>
          <p>Simulates changes in vendor behavior such as shifting payment terms, evolving pricing patterns, changes in delivery reliability, and supplier consolidation trends.</p>
        </div>
        <div class="info-card">
          <h4>Customer Drift</h4>
          <p>Models customer behavior evolution including changing purchase frequencies, order size trends, payment behavior shifts, and lifecycle stage transitions.</p>
        </div>
        <div class="info-card">
          <h4>Employee Drift</h4>
          <p>Captures employee behavioral changes such as evolving approval patterns, shifting work hours, changing expense patterns, and role-related behavioral adjustments.</p>
        </div>
        <div class="info-card">
          <h4>Drift Velocity</h4>
          <p>Controls the overall pace of behavioral changes. Lower values create gradual, realistic drift. Higher values accelerate changes for shorter training datasets.</p>
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
  .event-list { display: flex; flex-direction: column; gap: var(--space-3); }
  .event-item { display: grid; grid-template-columns: 1fr 1fr 2fr 1fr 1fr auto; gap: var(--space-2); align-items: center; padding: var(--space-3); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .event-item input, .event-item select { font-size: 0.8125rem; }
  .btn-danger { background-color: var(--color-error, #ef4444); color: white; border: none; padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); cursor: pointer; font-size: 0.75rem; }
  .btn-outline { background: none; border: 1px solid var(--color-border); padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); cursor: pointer; font-size: 0.8125rem; color: var(--color-text-secondary); }
  .btn-outline:hover { background-color: var(--color-background); color: var(--color-text-primary); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid { grid-template-columns: 1fr; } .event-item { grid-template-columns: 1fr; } }
</style>
