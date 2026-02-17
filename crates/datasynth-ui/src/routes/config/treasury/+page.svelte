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
  <ConfigPageHeader title="Treasury & Cash Management" description="Configure cash positioning, forecasting, hedging, and debt instruments" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Treasury Module" description="Enable treasury and cash management data generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.treasury.enabled}
              label="Enable Treasury"
              description="Generate cash positions, forecasts, hedging instruments, and debt management"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.treasury.enabled}
        <FormSection title="Cash Management" description="Configure cash positioning, forecasting, and pooling">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.treasury.cash_positioning.enabled}
                label="Cash Positioning"
                description="Generate daily/weekly cash position reports"
              />

              {#if $config.treasury.cash_positioning.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Positioning Frequency"
                    htmlFor="cash-freq"
                    helpText="How often cash positions are calculated"
                    error={getError('treasury.cash_positioning.frequency')}
                  >
                    {#snippet children()}
                      <select id="cash-freq" bind:value={$config.treasury.cash_positioning.frequency}>
                        <option value="daily">Daily</option>
                        <option value="weekly">Weekly</option>
                        <option value="monthly">Monthly</option>
                      </select>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Minimum Balance"
                    htmlFor="min-balance"
                    helpText="Minimum cash balance policy threshold"
                    error={getError('treasury.cash_positioning.minimum_balance_policy')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input
                          type="number"
                          id="min-balance"
                          bind:value={$config.treasury.cash_positioning.minimum_balance_policy}
                          min="0"
                          step="10000"
                        />
                        <span class="suffix">USD</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}

              <Toggle
                bind:checked={$config.treasury.cash_forecasting.enabled}
                label="Cash Forecasting"
                description="Generate rolling cash flow forecasts with confidence intervals"
              />

              {#if $config.treasury.cash_forecasting.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Forecast Horizon"
                    htmlFor="horizon-days"
                    helpText="Number of days to forecast ahead"
                    error={getError('treasury.cash_forecasting.horizon_days')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input
                          type="number"
                          id="horizon-days"
                          bind:value={$config.treasury.cash_forecasting.horizon_days}
                          min="1"
                          max="365"
                          step="1"
                        />
                        <span class="suffix">days</span>
                      </div>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Confidence Interval"
                    htmlFor="confidence"
                    helpText="Forecast confidence interval (e.g., 0.90 = 90%)"
                    error={getError('treasury.cash_forecasting.confidence_interval')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input
                          type="number"
                          id="confidence"
                          bind:value={$config.treasury.cash_forecasting.confidence_interval}
                          min="0"
                          max="1"
                          step="0.05"
                        />
                        <span class="suffix">{($config.treasury.cash_forecasting.confidence_interval * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}

              <Toggle
                bind:checked={$config.treasury.cash_pooling.enabled}
                label="Cash Pooling"
                description="Generate intercompany cash pooling and sweep arrangements"
              />

              {#if $config.treasury.cash_pooling.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Pool Type"
                    htmlFor="pool-type"
                    helpText="Cash pooling structure type"
                    error={getError('treasury.cash_pooling.pool_type')}
                  >
                    {#snippet children()}
                      <select id="pool-type" bind:value={$config.treasury.cash_pooling.pool_type}>
                        <option value="zero_balancing">Zero Balancing</option>
                        <option value="notional">Notional Pooling</option>
                        <option value="target_balancing">Target Balancing</option>
                      </select>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Sweep Time"
                    htmlFor="sweep-time"
                    helpText="Daily sweep execution time"
                    error={getError('treasury.cash_pooling.sweep_time')}
                  >
                    {#snippet children()}
                      <input
                        type="time"
                        id="sweep-time"
                        bind:value={$config.treasury.cash_pooling.sweep_time}
                      />
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Risk Management" description="Configure hedging and intercompany netting">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.treasury.hedging.enabled}
                label="Hedging"
                description="Generate FX forwards, interest rate swaps, and hedge documentation"
              />

              {#if $config.treasury.hedging.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Hedge Ratio"
                    htmlFor="hedge-ratio"
                    helpText="Target hedge ratio for exposed positions"
                    error={getError('treasury.hedging.hedge_ratio')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input
                          type="number"
                          id="hedge-ratio"
                          bind:value={$config.treasury.hedging.hedge_ratio}
                          min="0"
                          max="1"
                          step="0.05"
                        />
                        <span class="suffix">{($config.treasury.hedging.hedge_ratio * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Effectiveness Method"
                    htmlFor="effectiveness"
                    helpText="Hedge effectiveness testing methodology"
                    error={getError('treasury.hedging.effectiveness_method')}
                  >
                    {#snippet children()}
                      <select id="effectiveness" bind:value={$config.treasury.hedging.effectiveness_method}>
                        <option value="regression">Regression Analysis</option>
                        <option value="dollar_offset">Dollar Offset</option>
                        <option value="critical_terms">Critical Terms Match</option>
                      </select>
                    {/snippet}
                  </FormGroup>
                </div>

                <Toggle
                  bind:checked={$config.treasury.hedging.hedge_accounting}
                  label="Hedge Accounting"
                  description="Apply ASC 815 / IFRS 9 hedge accounting designations"
                />
              {/if}

              <Toggle
                bind:checked={$config.treasury.netting.enabled}
                label="Intercompany Netting"
                description="Generate multilateral netting arrangements"
              />

              {#if $config.treasury.netting.enabled}
                <FormGroup
                  label="Netting Cycle"
                  htmlFor="netting-cycle"
                  helpText="Frequency of netting settlements"
                  error={getError('treasury.netting.cycle')}
                >
                  {#snippet children()}
                    <select id="netting-cycle" bind:value={$config.treasury.netting.cycle}>
                      <option value="weekly">Weekly</option>
                      <option value="bi_weekly">Bi-Weekly</option>
                      <option value="monthly">Monthly</option>
                    </select>
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Debt & Guarantees" description="Configure debt instruments and bank guarantees">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.treasury.debt.enabled}
                label="Debt Instruments"
                description="Generate term loans, revolving credit, and covenant tracking"
              />

              <Toggle
                bind:checked={$config.treasury.bank_guarantees.enabled}
                label="Bank Guarantees"
                description="Generate bank guarantees and letters of credit"
              />

              {#if $config.treasury.bank_guarantees.enabled}
                <FormGroup
                  label="Guarantee Count"
                  htmlFor="guarantee-count"
                  helpText="Number of active bank guarantees to generate"
                  error={getError('treasury.bank_guarantees.count')}
                >
                  {#snippet children()}
                    <input
                      type="number"
                      id="guarantee-count"
                      bind:value={$config.treasury.bank_guarantees.count}
                      min="0"
                      step="1"
                    />
                  {/snippet}
                </FormGroup>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Anomaly Rate" description="Treasury-specific anomaly injection">
          {#snippet children()}
            <FormGroup
              label="Anomaly Rate"
              htmlFor="treasury-anomaly"
              helpText="Rate of anomalous treasury records (0-100%)"
              error={getError('treasury.anomaly_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="treasury-anomaly"
                    bind:value={$config.treasury.anomaly_rate}
                    min="0"
                    max="1"
                    step="0.005"
                  />
                  <span class="suffix">{($config.treasury.anomaly_rate * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Treasury Operations</h4>
          <p>
            Generates comprehensive treasury data including daily cash positions,
            rolling forecasts with AR/AP aging curves, zero-balance pooling,
            FX/IR hedging with effectiveness testing, and debt covenant monitoring.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates cash_positions.csv, cash_forecasts.csv, pool_sweeps.csv,
            hedge_instruments.csv, hedge_effectiveness.csv, debt_instruments.csv,
            covenant_tests.csv, netting_runs.csv, and bank_guarantees.csv.
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
