<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle, DistributionEditor } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  // Default customer distribution settings
  const defaultDistribution = {
    credit_rating_aaa: 0.10,
    credit_rating_aa: 0.20,
    credit_rating_a: 0.35,
    credit_rating_bbb: 0.25,
    credit_rating_bb: 0.10,
  };

  // Ensure distribution exists
  $effect(() => {
    if ($config?.master_data?.customers && Object.keys($config.master_data.customers.distribution).length === 0) {
      $config.master_data.customers.distribution = { ...defaultDistribution };
    }
  });

  // Credit rating labels
  const creditRatingLabels: Record<string, string> = {
    credit_rating_aaa: 'AAA (Excellent)',
    credit_rating_aa: 'AA (Very Good)',
    credit_rating_a: 'A (Good)',
    credit_rating_bbb: 'BBB (Adequate)',
    credit_rating_bb: 'BB (Speculative)',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Customer Configuration" description="Configure customer accounts and credit settings" />

  {#if $config}
    <div class="sections">
      <!-- Entity Count -->
      <FormSection
        title="Customer Count"
        description="Number of customer master records to generate"
      >
        <div class="section-content">
          <FormGroup
            label="Number of Customers"
            htmlFor="customer-count"
            helpText="Typical range: 100-1000 for medium-sized company"
          >
            <input
              id="customer-count"
              type="number"
              min="1"
              max="100000"
              step="1"
              bind:value={$config.master_data.customers.count}
            />
          </FormGroup>

          <div class="quick-presets">
            <span class="preset-label">Quick presets:</span>
            <button type="button" onclick={() => $config.master_data.customers.count = 100}>
              Small (100)
            </button>
            <button type="button" onclick={() => $config.master_data.customers.count = 500}>
              Medium (500)
            </button>
            <button type="button" onclick={() => $config.master_data.customers.count = 2000}>
              Large (2000)
            </button>
            <button type="button" onclick={() => $config.master_data.customers.count = 10000}>
              Retail (10K)
            </button>
          </div>
        </div>
      </FormSection>

      <!-- Credit Rating Distribution -->
      <FormSection
        title="Credit Rating Distribution"
        description="Distribution of credit ratings across customers"
      >
        <div class="section-content">
          <p class="section-intro">
            Credit ratings affect payment behavior, credit limits, and dunning processes.
            Higher-rated customers receive larger credit limits and better terms.
          </p>

          <DistributionEditor
            label="Credit Ratings"
            bind:distribution={$config.master_data.customers.distribution}
            labels={creditRatingLabels}
            helpText="Rating distribution affects AR aging and bad debt rates"
          />
        </div>
      </FormSection>

      <!-- Customer Characteristics -->
      <FormSection
        title="Customer Characteristics"
        description="Behavioral patterns and attributes"
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
              <strong>Payment Behavior Types</strong>
              <p>
                Customers exhibit different payment patterns that affect AR aging:
              </p>
              <ul>
                <li><strong>Early Payer:</strong> Often takes cash discounts, pays before due date</li>
                <li><strong>On-Time:</strong> Consistent payment on or near due date</li>
                <li><strong>Late Payer:</strong> Habitually pays past due, triggers dunning</li>
                <li><strong>Delinquent:</strong> Chronic non-payment, write-off candidate</li>
              </ul>
            </div>
          </div>

          <div class="characteristics-grid">
            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 1v22M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
                  </svg>
                </span>
                <span class="characteristic-title">Credit Limits</span>
              </div>
              <p class="characteristic-description">
                Credit limits are assigned based on rating, ranging from $10K for BB
                customers to $1M+ for AAA. Limits affect order hold behavior.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                  </svg>
                </span>
                <span class="characteristic-title">Intercompany Customers</span>
              </div>
              <p class="characteristic-description">
                A percentage of customers will be flagged as intercompany for IC revenue
                and AR transaction generation.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                  </svg>
                </span>
                <span class="characteristic-title">Dunning Levels</span>
              </div>
              <p class="characteristic-description">
                Late payers progress through dunning levels (reminder, warning, final notice)
                based on days past due and amount outstanding.
              </p>
            </div>
          </div>
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

  .section-intro {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .quick-presets {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .preset-label {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .quick-presets button {
    padding: var(--space-1) var(--space-3);
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .quick-presets button:hover {
    background-color: var(--color-surface);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .info-card {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
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

  .characteristics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-4);
  }

  .characteristic-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .characteristic-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .characteristic-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-surface);
    border-radius: var(--radius-md);
    color: var(--color-accent);
  }

  .characteristic-icon svg {
    width: 18px;
    height: 18px;
  }

  .characteristic-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .characteristic-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  input[type="number"] {
    width: 100%;
    max-width: 200px;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }

  input[type="number"]:focus {
    outline: none;
    border-color: var(--color-accent);
  }
</style>
