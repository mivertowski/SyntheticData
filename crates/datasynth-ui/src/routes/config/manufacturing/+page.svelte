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
  <ConfigPageHeader title="Manufacturing" description="Configure production orders, quality inspections, and cycle counts" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Manufacturing Module" description="Enable manufacturing process data generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.manufacturing.enabled}
              label="Enable Manufacturing"
              description="Generate production orders, quality inspections, and inventory cycle counts"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.manufacturing.enabled}
        <FormSection title="Process Modules" description="Enable or disable individual manufacturing sub-processes">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.manufacturing.production_orders}
                label="Production Orders"
                description="Generate production orders with planned and actual quantities"
              />

              <Toggle
                bind:checked={$config.manufacturing.wip_costing}
                label="WIP Costing"
                description="Generate work-in-progress cost tracking records"
              />

              <Toggle
                bind:checked={$config.manufacturing.routing}
                label="Routing Operations"
                description="Generate routing steps with setup times and run times"
              />

              <Toggle
                bind:checked={$config.manufacturing.quality_inspections}
                label="Quality Inspections"
                description="Generate quality inspection lots with inspection characteristics"
              />

              <Toggle
                bind:checked={$config.manufacturing.cycle_counts}
                label="Cycle Counts"
                description="Generate inventory cycle count records with variances"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Production Parameters" description="Configure production rates and lead times">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Scrap Rate"
                htmlFor="scrap-rate"
                helpText="Proportion of production output that is scrapped (0-100%)"
                error={getError('manufacturing.scrap_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="scrap-rate"
                      bind:value={$config.manufacturing.scrap_rate}
                      min="0"
                      max="1"
                      step="0.005"
                    />
                    <span class="suffix">{($config.manufacturing.scrap_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Rework Rate"
                htmlFor="rework-rate"
                helpText="Proportion of production output requiring rework (0-100%)"
                error={getError('manufacturing.rework_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="rework-rate"
                      bind:value={$config.manufacturing.rework_rate}
                      min="0"
                      max="1"
                      step="0.005"
                    />
                    <span class="suffix">{($config.manufacturing.rework_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Avg Lead Time"
                htmlFor="lead-time"
                helpText="Average production lead time in days"
                error={getError('manufacturing.avg_lead_time_days')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="lead-time"
                      bind:value={$config.manufacturing.avg_lead_time_days}
                      min="1"
                      max="365"
                      step="1"
                    />
                    <span class="suffix">days</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Production Process</h4>
          <p>
            Generates complete production order lifecycles including routing
            operations, material consumption, WIP costing, and goods receipt.
            Supports scrap and rework scenarios.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates production_orders.csv, routing_operations.csv,
            quality_inspection_lots.csv, and cycle_count_records.csv.
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
