<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
</script>

<div class="page">
  <ConfigPageHeader title="Analytics & ML Settings" description="Configure graph export, anomaly injection, and data quality variations" />

  {#if $config}
    <div class="sections">
      <!-- Graph Export Configuration -->
      <FormSection
        title="Graph Export"
        description="Export data as graphs for machine learning and network analysis"
      >
        <div class="section-content">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Export Document Flow Graph</span>
              <span class="toggle-description">
                Export document chains (PO→GR→Invoice→Payment) as a directed graph
                for process mining and anomaly detection.
              </span>
            </div>
            <Toggle bind:checked={$config.document_flows.export_flow_graph} />
          </div>

          <div class="info-card">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4M12 8h.01" />
            </svg>
            <div class="info-content">
              <strong>Graph Export Formats</strong>
              <p>
                Transaction and entity graphs can be exported in multiple ML-ready formats:
              </p>
              <ul>
                <li><strong>PyTorch Geometric:</strong> .pt files with node_features, edge_index, masks</li>
                <li><strong>Neo4j:</strong> CSV files with Cypher import scripts for graph DB</li>
                <li><strong>DGL:</strong> Deep Graph Library format for GNN training</li>
              </ul>
            </div>
          </div>

          <div class="feature-grid">
            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="5" r="3" />
                    <circle cx="5" cy="19" r="3" />
                    <circle cx="19" cy="19" r="3" />
                    <line x1="12" y1="8" x2="5" y2="16" />
                    <line x1="12" y1="8" x2="19" y2="16" />
                  </svg>
                </span>
                <span class="feature-title">Transaction Network</span>
              </div>
              <p class="feature-description">
                Accounts and entities as nodes, transactions as edges.
                Useful for detecting circular transactions and related-party anomalies.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
                    <circle cx="8.5" cy="7" r="4" />
                    <polyline points="17,11 19,13 23,9" />
                  </svg>
                </span>
                <span class="feature-title">Approval Network</span>
              </div>
              <p class="feature-description">
                Users as nodes, approvals as edges. Train models to detect
                unusual approval patterns and potential collusion.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                    <polyline points="3.27,6.96 12,12.01 20.73,6.96" />
                    <line x1="12" y1="22.08" x2="12" y2="12" />
                  </svg>
                </span>
                <span class="feature-title">Entity Ownership</span>
              </div>
              <p class="feature-description">
                Legal entities with ownership edges for consolidation analysis
                and transfer pricing relationship detection.
              </p>
            </div>
          </div>
        </div>
      </FormSection>

      <!-- Anomaly Injection -->
      <FormSection
        title="Anomaly Injection"
        description="Configure synthetic anomalies for ML training data"
      >
        <div class="section-content">
          <div class="info-card highlight">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
            <div class="info-content">
              <strong>Fraud Detection Training Data</strong>
              <p>
                Injected anomalies are fully labeled for supervised ML training.
                Each anomaly includes type, severity, detection method, and ground truth labels.
              </p>
            </div>
          </div>

          <div class="anomaly-categories">
            <div class="category-card">
              <h4>Fraud Patterns (20+ types)</h4>
              <ul>
                <li>Fictitious transactions</li>
                <li>Revenue manipulation</li>
                <li>Expense capitalization</li>
                <li>Split transactions (threshold avoidance)</li>
                <li>Round-tripping</li>
                <li>Ghost employee payments</li>
                <li>Duplicate payments</li>
                <li>Kickback schemes</li>
              </ul>
            </div>

            <div class="category-card">
              <h4>Error Patterns</h4>
              <ul>
                <li>Duplicate entries</li>
                <li>Reversed amounts</li>
                <li>Wrong period postings</li>
                <li>Misclassifications</li>
                <li>Missing references</li>
                <li>Incorrect tax codes</li>
              </ul>
            </div>

            <div class="category-card">
              <h4>Process Issues</h4>
              <ul>
                <li>Late postings</li>
                <li>Skipped approvals</li>
                <li>Out-of-sequence documents</li>
                <li>Missing documentation</li>
                <li>Threshold manipulation</li>
              </ul>
            </div>

            <div class="category-card">
              <h4>Statistical Anomalies</h4>
              <ul>
                <li>Unusual amounts</li>
                <li>Benford violations</li>
                <li>Trend breaks</li>
                <li>Outlier values</li>
                <li>Dormant account activity</li>
              </ul>
            </div>
          </div>

          <p class="note">
            Configure fraud and controls settings in the
            <a href="/config/compliance">Fraud & Controls</a> section.
          </p>
        </div>
      </FormSection>

      <!-- Data Quality Variations -->
      <FormSection
        title="Data Quality Variations"
        description="Inject realistic data quality issues for cleaning/preparation testing"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="info-card">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4M12 8h.01" />
            </svg>
            <div class="info-content">
              <strong>Data Quality Injection</strong>
              <p>
                Generate realistic data quality issues to test data cleaning pipelines:
              </p>
              <ul>
                <li><strong>Missing Values:</strong> MCAR, MAR, MNAR, and systematic patterns</li>
                <li><strong>Format Variations:</strong> Date formats, amount formats, ID formats</li>
                <li><strong>Duplicates:</strong> Exact, near, and fuzzy duplicates</li>
                <li><strong>Typos:</strong> Keyboard-aware, OCR errors, homophones</li>
                <li><strong>Encoding Issues:</strong> Mojibake, BOM, character corruption</li>
              </ul>
            </div>
          </div>

          <div class="feature-grid">
            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                    <line x1="9" y1="9" x2="15" y2="15" />
                    <line x1="15" y1="9" x2="9" y2="15" />
                  </svg>
                </span>
                <span class="feature-title">Missing Value Patterns</span>
              </div>
              <p class="feature-description">
                MCAR (random), MAR (depends on other values), MNAR (depends on missing value),
                and systematic patterns (entire field groups).
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                  </svg>
                </span>
                <span class="feature-title">Typo Generation</span>
              </div>
              <p class="feature-description">
                Keyboard-aware typos (QWERTY layout), transpositions, insertions,
                deletions, and OCR-style errors (0/O, 1/l, 5/S).
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="3" width="7" height="7" />
                    <rect x="14" y="3" width="7" height="7" />
                    <rect x="14" y="14" width="7" height="7" />
                    <rect x="3" y="14" width="7" height="7" />
                  </svg>
                </span>
                <span class="feature-title">Duplicate Detection Training</span>
              </div>
              <p class="feature-description">
                Exact duplicates, near duplicates (minor variations), and fuzzy
                duplicates (different representations of same entity).
              </p>
            </div>
          </div>
        </div>
      </FormSection>

      <!-- ML Features -->
      <FormSection
        title="ML Feature Engineering"
        description="Pre-computed features for machine learning"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="feature-grid four-col">
            <div class="mini-feature">
              <h5>Temporal</h5>
              <ul>
                <li>weekday</li>
                <li>period</li>
                <li>is_month_end</li>
                <li>is_quarter_end</li>
                <li>is_year_end</li>
              </ul>
            </div>

            <div class="mini-feature">
              <h5>Amount</h5>
              <ul>
                <li>log(amount)</li>
                <li>benford_prob</li>
                <li>is_round</li>
                <li>amount_zscore</li>
              </ul>
            </div>

            <div class="mini-feature">
              <h5>Structural</h5>
              <ul>
                <li>line_count</li>
                <li>unique_accounts</li>
                <li>has_intercompany</li>
                <li>account_depth</li>
              </ul>
            </div>

            <div class="mini-feature">
              <h5>Categorical</h5>
              <ul>
                <li>business_process</li>
                <li>source_type</li>
                <li>document_type</li>
                <li>company_code</li>
              </ul>
            </div>
          </div>

          <p class="note">
            Features are automatically computed and included in graph exports for
            GNN training with train/validation/test splits.
          </p>
        </div>
      </FormSection>
    </div>
  {:else}
    <div class="loading">Loading configuration...</div>
  {/if}
</div>

<style>
  .page {
    max-width: 900px;
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .section-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .toggle-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .toggle-description {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .info-card {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .info-card.highlight {
    border: 1px solid var(--color-warning);
    background-color: color-mix(in srgb, var(--color-warning) 5%, var(--color-background));
  }

  .info-card.highlight svg {
    color: var(--color-warning);
  }

  .info-card > svg {
    width: 24px;
    height: 24px;
    color: var(--color-accent);
    flex-shrink: 0;
    margin-top: 2px;
  }

  .info-content {
    flex: 1;
  }

  .info-content strong {
    display: block;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .info-content p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0 0 var(--space-2);
    line-height: 1.5;
  }

  .info-content ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .info-content li {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    padding-left: var(--space-3);
    position: relative;
  }

  .info-content li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0.5em;
    width: 4px;
    height: 4px;
    background-color: var(--color-accent);
    border-radius: 50%;
  }

  .info-content li strong {
    display: inline;
    font-size: inherit;
    margin: 0;
    color: var(--color-text-primary);
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-4);
  }

  .feature-grid.four-col {
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  }

  .feature-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .feature-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .feature-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-surface);
    border-radius: var(--radius-md);
    color: var(--color-accent);
  }

  .feature-icon svg {
    width: 18px;
    height: 18px;
  }

  .feature-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .feature-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .anomaly-categories {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--space-4);
  }

  .category-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .category-card h4 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 var(--space-2);
  }

  .category-card ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .category-card li {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    padding-left: var(--space-2);
    position: relative;
  }

  .category-card li::before {
    content: '•';
    position: absolute;
    left: 0;
    color: var(--color-accent);
  }

  .mini-feature {
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .mini-feature h5 {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 var(--space-2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .mini-feature ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .mini-feature li {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: var(--color-text-secondary);
  }

  .note {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    margin: 0;
  }

  .note a {
    color: var(--color-accent);
    text-decoration: none;
  }

  .note a:hover {
    text-decoration: underline;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }
</style>
