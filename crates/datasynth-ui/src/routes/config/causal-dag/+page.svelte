<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle, InputNumber } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const DAG_PRESETS = [
    { value: 'minimal', label: 'Minimal', description: '6 nodes -- core accounting relationships only' },
    { value: 'financial_process', label: 'Financial Process', description: '12 nodes -- includes document flows and period close' },
    { value: 'full', label: 'Full', description: '17 nodes -- complete causal graph with all business processes' },
  ];

  const INTERVENTION_TYPES = ['increase', 'decrease', 'set', 'multiply'];

  function ensureCausalConfig() {
    if (!$config) return;
    if (!$config.causal_dag) {
      $config.causal_dag = {
        enabled: false,
        preset: 'financial_process',
        interventions: [],
        constraints: {
          preserve_accounting_identity: true,
          preserve_document_chains: true,
          preserve_period_close: true,
          preserve_balance_coherence: true,
        },
      };
    }
  }

  function addIntervention() {
    ensureCausalConfig();
    if (!$config?.causal_dag) return;
    const interventions = $config.causal_dag.interventions || [];
    $config.causal_dag.interventions = [
      ...interventions,
      { type: 'increase', target_node: '', magnitude: 0.1, timing: 'immediate' },
    ];
  }

  function removeIntervention(index: number) {
    if (!$config?.causal_dag) return;
    $config.causal_dag.interventions = $config.causal_dag.interventions.filter(
      (_: any, i: number) => i !== index
    );
  }
</script>

<div class="page">
  <ConfigPageHeader
    title="Causal DAG & Scenarios"
    description="Configure causal directed acyclic graphs for counterfactual scenario generation"
  />

  {#if $config}
    <div class="page-content">
      <FormSection title="Causal DAG" description="Enable causal graph-based scenario generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              checked={$config.causal_dag?.enabled ?? false}
              label="Enable Causal DAG"
              description="Generate counterfactual scenarios using causal inference"
              onchange={() => {
                ensureCausalConfig();
                if ($config.causal_dag) $config.causal_dag.enabled = !$config.causal_dag.enabled;
              }}
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.causal_dag?.enabled}
        <FormSection title="DAG Preset" description="Select the complexity of the causal graph">
          {#snippet children()}
            <div class="preset-selector">
              {#each DAG_PRESETS as preset}
                <label class="preset-option" class:selected={$config.causal_dag?.preset === preset.value}>
                  <input
                    type="radio"
                    name="dag-preset"
                    value={preset.value}
                    checked={$config.causal_dag?.preset === preset.value}
                    onchange={() => {
                      if ($config.causal_dag) $config.causal_dag.preset = preset.value;
                    }}
                  />
                  <span class="preset-label">{preset.label}</span>
                  <span class="preset-desc">{preset.description}</span>
                </label>
              {/each}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Interventions" description="Define what-if interventions on causal graph nodes">
          {#snippet children()}
            <div class="form-stack">
              {#each ($config.causal_dag?.interventions ?? []) as intervention, i}
                <div class="intervention-row">
                  <select bind:value={intervention.type}>
                    {#each INTERVENTION_TYPES as t}
                      <option value={t}>{t}</option>
                    {/each}
                  </select>
                  <input
                    type="text"
                    bind:value={intervention.target_node}
                    placeholder="Target node"
                  />
                  <input
                    type="number"
                    bind:value={intervention.magnitude}
                    min="0"
                    max="10"
                    step="0.1"
                    placeholder="Magnitude"
                  />
                  <select bind:value={intervention.timing}>
                    <option value="immediate">Immediate</option>
                    <option value="gradual">Gradual</option>
                    <option value="delayed">Delayed</option>
                  </select>
                  <button class="btn-danger" onclick={() => removeIntervention(i)}>Remove</button>
                </div>
              {/each}
              <button class="btn-outline" onclick={addIntervention}>+ Add Intervention</button>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Constraints" description="ConfigMutator constraints to preserve data integrity during scenario generation">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.causal_dag.constraints.preserve_accounting_identity}
                label="Preserve Accounting Identity"
                description="Ensure Assets = Liabilities + Equity after mutations"
              />
              <Toggle
                bind:checked={$config.causal_dag.constraints.preserve_document_chains}
                label="Preserve Document Chains"
                description="Maintain PO -> GR -> Invoice -> Payment reference integrity"
              />
              <Toggle
                bind:checked={$config.causal_dag.constraints.preserve_period_close}
                label="Preserve Period Close"
                description="Keep fiscal period boundaries and closing entry sequences"
              />
              <Toggle
                bind:checked={$config.causal_dag.constraints.preserve_balance_coherence}
                label="Preserve Balance Coherence"
                description="Trial balance totals remain consistent across mutations"
              />
            </div>
          {/snippet}
        </FormSection>
      {/if}
    </div>
  {/if}
</div>

<style>
  .page { max-width: 960px; }
  .page-content { display: flex; flex-direction: column; gap: var(--space-5); }
  .form-stack { display: flex; flex-direction: column; gap: var(--space-4); }
  .preset-selector { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-3); }
  .preset-option { display: flex; flex-direction: column; padding: var(--space-3); border: 2px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); }
  .preset-option:hover { border-color: var(--color-accent); }
  .preset-option.selected { border-color: var(--color-accent); background-color: rgba(59, 130, 246, 0.05); }
  .preset-option input { display: none; }
  .preset-label { font-weight: 600; font-size: 0.875rem; color: var(--color-text-primary); margin-bottom: var(--space-1); }
  .preset-desc { font-size: 0.75rem; color: var(--color-text-secondary); }
  .intervention-row { display: grid; grid-template-columns: 120px 1fr 100px 120px auto; gap: var(--space-2); align-items: center; padding: var(--space-3); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .intervention-row select, .intervention-row input { font-size: 0.8125rem; padding: var(--space-2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); background-color: var(--color-surface); color: var(--color-text-primary); }
  .btn-danger { background-color: var(--color-danger); color: white; border: none; padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); cursor: pointer; font-size: 0.75rem; }
  .btn-outline { background: none; border: 1px solid var(--color-border); padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); cursor: pointer; font-size: 0.8125rem; color: var(--color-text-secondary); }
  .btn-outline:hover { background-color: var(--color-background); color: var(--color-text-primary); }
  @media (max-width: 768px) { .preset-selector { grid-template-columns: 1fr; } .intervention-row { grid-template-columns: 1fr; } }
</style>
