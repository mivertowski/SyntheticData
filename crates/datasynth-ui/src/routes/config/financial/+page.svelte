<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle, InputNumber } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
</script>

<div class="page">
  <ConfigPageHeader title="Financial Settings" description="Configure balance coherence, subledgers, and foreign exchange" />

  {#if $config}
    <div class="sections">
      <!-- Balance Configuration -->
      <FormSection
        title="Balance Configuration"
        description="Opening balance and coherence validation"
      >
        <div class="section-content">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate Opening Balances</span>
              <span class="toggle-description">
                Create coherent opening balance sheet at period start
                (Assets = Liabilities + Equity)
              </span>
            </div>
            <Toggle bind:checked={$config.balance.generate_opening_balances} />
          </div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Generate Trial Balances</span>
              <span class="toggle-description">
                Generate period-end trial balances for each fiscal period
              </span>
            </div>
            <Toggle bind:checked={$config.balance.generate_trial_balances} />
          </div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Validate Balance Equation</span>
              <span class="toggle-description">
                Validate that Assets = Liabilities + Equity throughout generation
              </span>
            </div>
            <Toggle bind:checked={$config.balance.validate_balance_equation} />
          </div>

          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">Reconcile Subledgers</span>
              <span class="toggle-description">
                Ensure subledger balances reconcile to GL control accounts
              </span>
            </div>
            <Toggle bind:checked={$config.balance.reconcile_subledgers} />
          </div>
        </div>
      </FormSection>

      <!-- Financial Ratios -->
      <FormSection
        title="Target Financial Ratios"
        description="Target metrics for coherent balance generation"
      >
        <div class="section-content">
          <div class="form-grid">
            <FormGroup
              label="Target Gross Margin"
              htmlFor="target-gross-margin"
              helpText="Target gross margin ratio (0.0 to 1.0)"
            >
              <InputNumber
                id="target-gross-margin"
                bind:value={$config.balance.target_gross_margin}
                min={0}
                max={1}
                step={0.01}
              />
            </FormGroup>

            <FormGroup
              label="Target DSO (Days)"
              htmlFor="target-dso"
              helpText="Days Sales Outstanding - average collection period"
            >
              <InputNumber
                id="target-dso"
                bind:value={$config.balance.target_dso_days}
                min={1}
                max={365}
                step={1}
              />
            </FormGroup>

            <FormGroup
              label="Target DPO (Days)"
              htmlFor="target-dpo"
              helpText="Days Payable Outstanding - average payment period"
            >
              <InputNumber
                id="target-dpo"
                bind:value={$config.balance.target_dpo_days}
                min={1}
                max={365}
                step={1}
              />
            </FormGroup>

            <FormGroup
              label="Target Current Ratio"
              htmlFor="target-current-ratio"
              helpText="Current Assets / Current Liabilities"
            >
              <InputNumber
                id="target-current-ratio"
                bind:value={$config.balance.target_current_ratio}
                min={0.1}
                max={10}
                step={0.1}
              />
            </FormGroup>

            <FormGroup
              label="Target Debt-to-Equity"
              htmlFor="target-debt-equity"
              helpText="Total Debt / Total Equity ratio"
            >
              <InputNumber
                id="target-debt-equity"
                bind:value={$config.balance.target_debt_to_equity}
                min={0}
                max={10}
                step={0.1}
              />
            </FormGroup>
          </div>
        </div>
      </FormSection>

      <!-- Subledger Configuration -->
      <FormSection
        title="Subledger Configuration"
        description="Configure accounts receivable, accounts payable, and inventory subledgers"
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
              <strong>Subledger Generation</strong>
              <p>
                Subledgers provide detailed transaction records that reconcile to GL control accounts:
              </p>
              <ul>
                <li><strong>AR Subledger:</strong> Customer invoices, receipts, credit memos, aging</li>
                <li><strong>AP Subledger:</strong> Vendor invoices, payments, debit memos, aging</li>
                <li><strong>FA Subledger:</strong> Asset register, depreciation schedule</li>
                <li><strong>Inventory:</strong> Stock positions, movements, valuation</li>
              </ul>
            </div>
          </div>

          <div class="feature-grid">
            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
                    <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
                  </svg>
                </span>
                <span class="feature-title">Automatic Reconciliation</span>
              </div>
              <p class="feature-description">
                Subledger balances automatically reconcile to GL control accounts.
                Discrepancies are flagged as potential audit issues.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                    <line x1="16" y1="2" x2="16" y2="6" />
                    <line x1="8" y1="2" x2="8" y2="6" />
                    <line x1="3" y1="10" x2="21" y2="10" />
                  </svg>
                </span>
                <span class="feature-title">Aging Analysis</span>
              </div>
              <p class="feature-description">
                AR and AP aging buckets (current, 30, 60, 90, 120+ days) are
                generated based on payment behavior profiles.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
                  </svg>
                </span>
                <span class="feature-title">Cash Application</span>
              </div>
              <p class="feature-description">
                Receipts and payments are matched to open items with realistic
                partial payment and cash discount behavior.
              </p>
            </div>
          </div>
        </div>
      </FormSection>

      <!-- FX Configuration -->
      <FormSection
        title="Foreign Exchange Configuration"
        description="Currency translation and exchange rate settings"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="info-card">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
              <path d="M2 12h20" />
            </svg>
            <div class="info-content">
              <strong>Multi-Currency Support</strong>
              <p>
                The generator supports multi-currency transactions with realistic FX behavior:
              </p>
              <ul>
                <li><strong>Daily Rates:</strong> Generated using Ornstein-Uhlenbeck process for mean reversion</li>
                <li><strong>Rate Types:</strong> Spot, closing (month-end), and average rates</li>
                <li><strong>Translation:</strong> Foreign subsidiary trial balances translated at period-end</li>
                <li><strong>CTA:</strong> Currency Translation Adjustment entries for consolidation</li>
              </ul>
            </div>
          </div>

          <div class="feature-grid">
            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="23,6 13.5,15.5 8.5,10.5 1,18" />
                    <polyline points="17,6 23,6 23,12" />
                  </svg>
                </span>
                <span class="feature-title">Rate Volatility</span>
              </div>
              <p class="feature-description">
                FX rates follow realistic volatility patterns with mean reversion
                around historical averages for major currency pairs.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                    <polyline points="7,10 12,15 17,10" />
                    <line x1="12" y1="15" x2="12" y2="3" />
                  </svg>
                </span>
                <span class="feature-title">Realized Gains/Losses</span>
              </div>
              <p class="feature-description">
                FX gains and losses are realized when foreign currency invoices
                are settled at rates different from booking.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 20V10M18 20V4M6 20v-4" />
                  </svg>
                </span>
                <span class="feature-title">Revaluation</span>
              </div>
              <p class="feature-description">
                Period-end revaluation of foreign currency balances generates
                unrealized gain/loss entries.
              </p>
            </div>
          </div>
        </div>
      </FormSection>

      <!-- Period Close Configuration -->
      <FormSection
        title="Period Close Configuration"
        description="Month-end and year-end closing activities"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="feature-grid">
            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                    <line x1="16" y1="2" x2="16" y2="6" />
                    <line x1="8" y1="2" x2="8" y2="6" />
                    <line x1="3" y1="10" x2="21" y2="10" />
                  </svg>
                </span>
                <span class="feature-title">Accruals</span>
              </div>
              <p class="feature-description">
                Month-end accrual entries for expenses incurred but not yet invoiced.
                Reversing entries posted at period start.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                  </svg>
                </span>
                <span class="feature-title">Depreciation</span>
              </div>
              <p class="feature-description">
                Monthly depreciation runs calculate and post depreciation
                expense for all active fixed assets.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8" />
                    <polyline points="16,6 12,2 8,6" />
                    <line x1="12" y1="2" x2="12" y2="15" />
                  </svg>
                </span>
                <span class="feature-title">Trial Balance</span>
              </div>
              <p class="feature-description">
                Period-end trial balance generated with all accounts showing
                debit and credit balances, ready for consolidation.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <span class="feature-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                    <polyline points="16,17 21,12 16,7" />
                    <line x1="21" y1="12" x2="9" y2="12" />
                  </svg>
                </span>
                <span class="feature-title">Year-End Close</span>
              </div>
              <p class="feature-description">
                Year-end closing entries transfer P&L to retained earnings
                and reset income/expense accounts.
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

  .form-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-4);
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-4);
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

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

</style>
