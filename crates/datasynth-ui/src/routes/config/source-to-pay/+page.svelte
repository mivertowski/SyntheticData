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
  <ConfigPageHeader title="Source-to-Pay" description="Configure spend analysis, sourcing, RFx, contracts, and supplier management" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Source-to-Pay Module" description="Enable end-to-end source-to-pay process generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.source_to_pay.enabled}
              label="Enable Source-to-Pay"
              description="Generate sourcing projects, RFx events, contracts, and supplier management data"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.source_to_pay.enabled}
        <FormSection title="Process Modules" description="Enable or disable individual S2C/S2P sub-processes">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.source_to_pay.spend_analysis}
                label="Spend Analysis"
                description="Generate spend classification and analysis records"
              />

              <Toggle
                bind:checked={$config.source_to_pay.sourcing_projects}
                label="Sourcing Projects"
                description="Generate strategic sourcing project records"
              />

              <Toggle
                bind:checked={$config.source_to_pay.qualification}
                label="Supplier Qualification"
                description="Generate supplier qualification and assessment records"
              />

              <Toggle
                bind:checked={$config.source_to_pay.rfx_events}
                label="RFx Events"
                description="Generate RFI, RFP, and RFQ events with supplier bids"
              />

              <Toggle
                bind:checked={$config.source_to_pay.contracts}
                label="Procurement Contracts"
                description="Generate procurement contracts and renewals"
              />

              <Toggle
                bind:checked={$config.source_to_pay.catalogs}
                label="Catalog Items"
                description="Generate catalog items from contracted suppliers"
              />

              <Toggle
                bind:checked={$config.source_to_pay.scorecards}
                label="Supplier Scorecards"
                description="Generate supplier performance scorecards"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Process Parameters" description="Configure sourcing cycle and qualification settings">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Avg Sourcing Cycle"
                htmlFor="sourcing-cycle"
                helpText="Average number of days for a complete sourcing cycle"
                error={getError('source_to_pay.avg_sourcing_cycle_days')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="sourcing-cycle"
                      bind:value={$config.source_to_pay.avg_sourcing_cycle_days}
                      min="1"
                      max="365"
                      step="1"
                    />
                    <span class="suffix">days</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Qualification Pass Rate"
                htmlFor="qual-pass-rate"
                helpText="Proportion of suppliers passing qualification (0-100%)"
                error={getError('source_to_pay.qualification_pass_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="qual-pass-rate"
                      bind:value={$config.source_to_pay.qualification_pass_rate}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span class="suffix">{($config.source_to_pay.qualification_pass_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Contract Renewal Rate"
                htmlFor="renewal-rate"
                helpText="Proportion of contracts that are renewed (0-100%)"
                error={getError('source_to_pay.contract_renewal_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input
                      type="number"
                      id="renewal-rate"
                      bind:value={$config.source_to_pay.contract_renewal_rate}
                      min="0"
                      max="1"
                      step="0.01"
                    />
                    <span class="suffix">{($config.source_to_pay.contract_renewal_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Sourcing Lifecycle</h4>
          <p>
            Generates complete source-to-pay cycles including spend analysis,
            strategic sourcing projects, supplier qualification, RFx events
            with bids and evaluations, and contract management.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates sourcing_projects.csv, supplier_qualifications.csv,
            rfx_events.csv, supplier_bids.csv, bid_evaluations.csv,
            procurement_contracts.csv, catalog_items.csv, and supplier_scorecards.csv.
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
