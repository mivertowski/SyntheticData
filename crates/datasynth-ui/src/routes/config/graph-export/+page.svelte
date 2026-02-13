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

  function getSplitTotal(): number {
    if (!$config?.graph_export?.split) return 0;
    return (
      $config.graph_export.split.train +
      $config.graph_export.split.val +
      $config.graph_export.split.test
    );
  }
</script>

<div class="page">
  <ConfigPageHeader title="Graph Export" description="Configure graph format exports for GNN training and analysis" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Graph Export Settings" description="Enable and configure graph-based data exports">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.graph_export.enabled}
              label="Enable Graph Export"
              description="Export data as graph structures for machine learning and network analysis"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.graph_export.enabled}
        <FormSection title="Export Formats" description="Select which graph formats to generate">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.graph_export.pytorch_geometric}
                label="PyTorch Geometric"
                description="Export .pt files with node_features, edge_index, and masks for PyG training"
              />
              <Toggle
                bind:checked={$config.graph_export.neo4j}
                label="Neo4j"
                description="Export CSV files with Cypher import scripts for Neo4j graph database"
              />
              <Toggle
                bind:checked={$config.graph_export.dgl}
                label="DGL (Deep Graph Library)"
                description="Export in DGL format for heterogeneous graph neural network training"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Graph Types" description="Select which graph structures to build and export">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.graph_export.transaction_graph}
                label="Transaction Graph"
                description="Accounts and entities as nodes, transactions as edges for anomaly detection"
              />
              <Toggle
                bind:checked={$config.graph_export.approval_graph}
                label="Approval Graph"
                description="Users as nodes, approvals as edges for collusion and pattern detection"
              />
              <Toggle
                bind:checked={$config.graph_export.entity_graph}
                label="Entity Graph"
                description="Legal entities with ownership edges for consolidation and transfer pricing analysis"
              />
              <Toggle
                bind:checked={$config.graph_export.hypergraph}
                label="Hypergraph"
                description="Multi-way relationships connecting documents, accounts, and entities simultaneously"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Train/Val/Test Split" description="Configure dataset splitting ratios for ML training">
          {#snippet children()}
            <div class="form-stack">
              <div class="distribution-grid">
                <div class="distribution-item">
                  <label>Train Split</label>
                  <div class="slider-with-value">
                    <input
                      type="range"
                      bind:value={$config.graph_export.split.train}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.graph_export.split.train * 100).toFixed(0)}%</span>
                  </div>
                </div>

                <div class="distribution-item">
                  <label>Validation Split</label>
                  <div class="slider-with-value">
                    <input
                      type="range"
                      bind:value={$config.graph_export.split.val}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.graph_export.split.val * 100).toFixed(0)}%</span>
                  </div>
                </div>

                <div class="distribution-item">
                  <label>Test Split</label>
                  <div class="slider-with-value">
                    <input
                      type="range"
                      bind:value={$config.graph_export.split.test}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span>{($config.graph_export.split.test * 100).toFixed(0)}%</span>
                  </div>
                </div>
              </div>

              <div class="distribution-total" class:warning={Math.abs(getSplitTotal() - 1.0) > 0.01}>
                Total: {(getSplitTotal() * 100).toFixed(0)}%
                {#if Math.abs(getSplitTotal() - 1.0) > 0.01}
                  <span class="warning-text">(should sum to 100%)</span>
                {/if}
              </div>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>PyTorch Geometric</h4>
          <p>Generates .pt files containing node features, edge indices, and train/val/test masks ready for GNN training with PyG.</p>
        </div>
        <div class="info-card">
          <h4>Neo4j</h4>
          <p>Exports CSV node and relationship files with Cypher import scripts for loading into Neo4j graph database.</p>
        </div>
        <div class="info-card">
          <h4>DGL</h4>
          <p>Deep Graph Library format supporting heterogeneous graphs with multiple node and edge types.</p>
        </div>
        <div class="info-card">
          <h4>Hypergraph</h4>
          <p>Multi-way relationships that connect multiple entities in a single hyperedge, capturing complex process interactions.</p>
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
  .warning-text { font-family: var(--font-sans); margin-left: var(--space-2); color: var(--color-warning, #eab308); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid { grid-template-columns: 1fr; } .event-item { grid-template-columns: 1fr; } }
</style>
