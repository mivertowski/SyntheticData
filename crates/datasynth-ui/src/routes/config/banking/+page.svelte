<script lang="ts">
  import { configStore, RISK_APPETITES, RETAIL_PERSONAS, BUSINESS_PERSONAS } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;
  const validationErrors = configStore.validationErrors;

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  function getRetailPersonaSum(): number {
    if (!$config?.banking?.population?.retail_persona_weights) return 0;
    return Object.values($config.banking.population.retail_persona_weights).reduce((a, b) => a + b, 0);
  }

  function getBusinessPersonaSum(): number {
    if (!$config?.banking?.population?.business_persona_weights) return 0;
    return Object.values($config.banking.population.business_persona_weights).reduce((a, b) => a + b, 0);
  }

  function getTypologySum(): number {
    if (!$config?.banking?.typologies) return 0;
    const t = $config.banking.typologies;
    return t.structuring_rate + t.funnel_rate + t.layering_rate + t.mule_rate + t.fraud_rate;
  }

  function getSophisticationSum(): number {
    if (!$config?.banking?.typologies?.sophistication) return 0;
    const s = $config.banking.typologies.sophistication;
    return s.basic + s.standard + s.professional + s.advanced;
  }
</script>

<div class="page">
  <ConfigPageHeader title="Banking / KYC / AML" description="Configure banking transaction generation for compliance testing and fraud detection" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Banking Generation" description="Enable KYC/AML banking transaction generation">
        {#snippet children()}
          <Toggle
            bind:checked={$config.banking.enabled}
            label="Enable Banking Module"
            description="Generate banking customers, accounts, and transactions with KYC profiles"
          />
        {/snippet}
      </FormSection>

      {#if $config.banking.enabled}
        <FormSection title="Population" description="Customer population settings">
          {#snippet children()}
            <div class="form-stack">
              <div class="form-grid">
                <FormGroup
                  label="Retail Customers"
                  htmlFor="retail-customers"
                  helpText="Number of retail (individual) customers"
                  error={getError('banking.population')}
                >
                  {#snippet children()}
                    <input
                      type="number"
                      id="retail-customers"
                      bind:value={$config.banking.population.retail_customers}
                      min="0"
                      max="1000000"
                    />
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Business Customers"
                  htmlFor="business-customers"
                  helpText="Number of business customers"
                >
                  {#snippet children()}
                    <input
                      type="number"
                      id="business-customers"
                      bind:value={$config.banking.population.business_customers}
                      min="0"
                      max="100000"
                    />
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Trusts"
                  htmlFor="trusts"
                  helpText="Number of trust entities"
                >
                  {#snippet children()}
                    <input
                      type="number"
                      id="trusts"
                      bind:value={$config.banking.population.trusts}
                      min="0"
                      max="10000"
                    />
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Household Rate"
                  htmlFor="household-rate"
                  helpText="Proportion of retail customers in households"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="household-rate"
                        bind:value={$config.banking.population.household_rate}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span class="slider-value">{($config.banking.population.household_rate * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Retail Persona Distribution" description="Distribution of retail customer types">
          {#snippet children()}
            <div class="distribution-grid">
              {#each RETAIL_PERSONAS as persona}
                <div class="distribution-item">
                  <div class="dist-header">
                    <span class="dist-label">{persona.label}</span>
                    <span class="dist-value">{(($config.banking.population.retail_persona_weights[persona.value] || 0) * 100).toFixed(0)}%</span>
                  </div>
                  <input
                    type="range"
                    bind:value={$config.banking.population.retail_persona_weights[persona.value]}
                    min="0"
                    max="1"
                    step="0.05"
                  />
                </div>
              {/each}
            </div>
            <div class="distribution-total" class:warning={Math.abs(getRetailPersonaSum() - 1.0) > 0.01}>
              Total: {(getRetailPersonaSum() * 100).toFixed(0)}%
              {#if Math.abs(getRetailPersonaSum() - 1.0) > 0.01}
                <span class="warning-text">(should sum to 100%)</span>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Products" description="Product and channel settings">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Cash Intensity"
                htmlFor="cash-intensity"
                helpText="Proportion of transactions in cash"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="cash-intensity"
                      bind:value={$config.banking.products.cash_intensity}
                      min="0"
                      max="0.5"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.banking.products.cash_intensity * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Cross-Border Rate"
                htmlFor="cross-border"
                helpText="Rate of cross-border transactions"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="cross-border"
                      bind:value={$config.banking.products.cross_border_rate}
                      min="0"
                      max="0.3"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.banking.products.cross_border_rate * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Card vs Transfer"
                htmlFor="card-transfer"
                helpText="Proportion of card payments vs transfers"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="card-transfer"
                      bind:value={$config.banking.products.card_vs_transfer}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span class="slider-value">{($config.banking.products.card_vs_transfer * 100).toFixed(0)}% Card</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Debit Card Rate"
                htmlFor="debit-card"
                helpText="Proportion of customers with debit cards"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="debit-card"
                      bind:value={$config.banking.products.debit_card_rate}
                      min="0"
                      max="1"
                      step="0.05"
                    />
                    <span class="slider-value">{($config.banking.products.debit_card_rate * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Compliance" description="KYC and compliance settings">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Risk Appetite"
                htmlFor="risk-appetite"
                helpText="Institution's risk tolerance level"
              >
                {#snippet children()}
                  <div class="risk-appetite-selector">
                    {#each RISK_APPETITES as appetite}
                      <label class="risk-option" class:selected={$config.banking.compliance.risk_appetite === appetite.value}>
                        <input
                          type="radio"
                          name="risk-appetite"
                          value={appetite.value}
                          bind:group={$config.banking.compliance.risk_appetite}
                        />
                        <span class="risk-label">{appetite.label}</span>
                        <span class="risk-desc">{appetite.description}</span>
                      </label>
                    {/each}
                  </div>
                {/snippet}
              </FormGroup>

              <div class="form-grid">
                <FormGroup
                  label="KYC Completeness"
                  htmlFor="kyc-completeness"
                  helpText="Rate of complete KYC profiles"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="kyc-completeness"
                        bind:value={$config.banking.compliance.kyc_completeness}
                        min="0.5"
                        max="1"
                        step="0.01"
                      />
                      <span class="slider-value">{($config.banking.compliance.kyc_completeness * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="High-Risk Tolerance"
                  htmlFor="high-risk"
                  helpText="Proportion of high-risk customers accepted"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="high-risk"
                        bind:value={$config.banking.compliance.high_risk_tolerance}
                        min="0"
                        max="0.2"
                        step="0.01"
                      />
                      <span class="slider-value">{($config.banking.compliance.high_risk_tolerance * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="PEP Rate"
                  htmlFor="pep-rate"
                  helpText="Proportion of Politically Exposed Persons"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="pep-rate"
                        bind:value={$config.banking.compliance.pep_rate}
                        min="0"
                        max="0.05"
                        step="0.001"
                      />
                      <span class="slider-value">{($config.banking.compliance.pep_rate * 100).toFixed(1)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="EDD Threshold"
                  htmlFor="edd-threshold"
                  helpText="Amount triggering enhanced due diligence"
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input
                        type="number"
                        id="edd-threshold"
                        bind:value={$config.banking.compliance.edd_threshold}
                        min="0"
                        step="1000"
                      />
                      <span class="suffix">USD</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="AML Typologies" description="Money laundering pattern settings">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="Overall Suspicious Rate"
                htmlFor="suspicious-rate"
                helpText="Total rate of suspicious activity in dataset"
                error={getError('banking.typologies')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="suspicious-rate"
                      bind:value={$config.banking.typologies.suspicious_rate}
                      min="0"
                      max="0.1"
                      step="0.005"
                    />
                    <span class="slider-value">{($config.banking.typologies.suspicious_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <div class="typology-grid">
                <FormGroup
                  label="Structuring"
                  htmlFor="structuring"
                  helpText="Below-threshold deposits"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="structuring"
                        bind:value={$config.banking.typologies.structuring_rate}
                        min="0"
                        max="0.02"
                        step="0.001"
                      />
                      <span class="slider-value">{($config.banking.typologies.structuring_rate * 100).toFixed(2)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Funnel Accounts"
                  htmlFor="funnel"
                  helpText="Aggregation accounts"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="funnel"
                        bind:value={$config.banking.typologies.funnel_rate}
                        min="0"
                        max="0.02"
                        step="0.001"
                      />
                      <span class="slider-value">{($config.banking.typologies.funnel_rate * 100).toFixed(2)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Layering"
                  htmlFor="layering"
                  helpText="Complex transaction chains"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="layering"
                        bind:value={$config.banking.typologies.layering_rate}
                        min="0"
                        max="0.02"
                        step="0.001"
                      />
                      <span class="slider-value">{($config.banking.typologies.layering_rate * 100).toFixed(2)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Money Mules"
                  htmlFor="mule"
                  helpText="Mule account patterns"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="mule"
                        bind:value={$config.banking.typologies.mule_rate}
                        min="0"
                        max="0.02"
                        step="0.001"
                      />
                      <span class="slider-value">{($config.banking.typologies.mule_rate * 100).toFixed(2)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Fraud"
                  htmlFor="fraud"
                  helpText="ATO, synthetic identity"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="fraud"
                        bind:value={$config.banking.typologies.fraud_rate}
                        min="0"
                        max="0.02"
                        step="0.001"
                      />
                      <span class="slider-value">{($config.banking.typologies.fraud_rate * 100).toFixed(2)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Detectability"
                  htmlFor="detectability"
                  helpText="Base detectability of patterns"
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="detectability"
                        bind:value={$config.banking.typologies.detectability}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span class="slider-value">{($config.banking.typologies.detectability * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>

              <div class="distribution-total" class:warning={getTypologySum() > $config.banking.typologies.suspicious_rate + 0.001}>
                Typology Sum: {(getTypologySum() * 100).toFixed(2)}% / Suspicious Rate: {($config.banking.typologies.suspicious_rate * 100).toFixed(1)}%
                {#if getTypologySum() > $config.banking.typologies.suspicious_rate + 0.001}
                  <span class="warning-text">(typology sum exceeds suspicious rate)</span>
                {/if}
              </div>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Adversarial Spoofing" description="Generate evasion-aware transactions">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.banking.spoofing.enabled}
                label="Enable Spoofing Mode"
                description="Generate adversarial transactions that attempt to evade detection"
              />

              {#if $config.banking.spoofing.enabled}
                <FormGroup
                  label="Spoofing Intensity"
                  htmlFor="spoof-intensity"
                  helpText="How aggressively to spoof detection features"
                  error={getError('banking.spoofing.intensity')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="spoof-intensity"
                        bind:value={$config.banking.spoofing.intensity}
                        min="0"
                        max="1"
                        step="0.05"
                      />
                      <span class="slider-value">{($config.banking.spoofing.intensity * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>

                <div class="toggle-grid">
                  <Toggle
                    bind:checked={$config.banking.spoofing.spoof_timing}
                    label="Spoof Timing"
                    description="Vary transaction timing patterns"
                  />

                  <Toggle
                    bind:checked={$config.banking.spoofing.spoof_amounts}
                    label="Spoof Amounts"
                    description="Vary transaction amounts"
                  />

                  <Toggle
                    bind:checked={$config.banking.spoofing.spoof_merchants}
                    label="Spoof Merchants"
                    description="Vary merchant selection"
                  />

                  <Toggle
                    bind:checked={$config.banking.spoofing.spoof_geography}
                    label="Spoof Geography"
                    description="Vary geographic patterns"
                  />

                  <Toggle
                    bind:checked={$config.banking.spoofing.add_delays}
                    label="Add Delays"
                    description="Add delays to reduce velocity detection"
                  />
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Output" description="Configure banking output files">
          {#snippet children()}
            <div class="toggle-grid">
              <Toggle
                bind:checked={$config.banking.output.include_customers}
                label="Customer Master Data"
                description="banking_customers.csv"
              />

              <Toggle
                bind:checked={$config.banking.output.include_accounts}
                label="Account Master Data"
                description="bank_accounts.csv"
              />

              <Toggle
                bind:checked={$config.banking.output.include_transactions}
                label="Transactions"
                description="bank_transactions.csv"
              />

              <Toggle
                bind:checked={$config.banking.output.include_counterparties}
                label="Counterparties"
                description="counterparties.csv"
              />

              <Toggle
                bind:checked={$config.banking.output.include_transaction_labels}
                label="Transaction Labels"
                description="ML training labels per transaction"
              />

              <Toggle
                bind:checked={$config.banking.output.include_entity_labels}
                label="Entity Labels"
                description="ML training labels per entity"
              />

              <Toggle
                bind:checked={$config.banking.output.include_case_narratives}
                label="Case Narratives"
                description="Investigation narratives"
              />

              <Toggle
                bind:checked={$config.banking.output.include_graph}
                label="Graph Data"
                description="Network graph export"
              />
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="AML Typologies" description="Configure which AML typology patterns to inject">
          {#snippet children()}
            <div class="form-stack">
              <div class="aml-typology-grid">
                {#each ['structuring', 'layering', 'funnel', 'mule', 'round_tripping', 'fraud', 'spoofing'] as typology}
                  <label class="typology-option">
                    <input
                      type="checkbox"
                      checked={$config.banking?.aml_typologies?.includes(typology) ?? false}
                      onchange={() => {
                        if (!$config.banking) return;
                        if (!$config.banking.aml_typologies) $config.banking.aml_typologies = [];
                        const idx = $config.banking.aml_typologies.indexOf(typology);
                        if (idx >= 0) { $config.banking.aml_typologies.splice(idx, 1); $config.banking.aml_typologies = [...$config.banking.aml_typologies]; }
                        else { $config.banking.aml_typologies = [...$config.banking.aml_typologies, typology]; }
                      }}
                    />
                    <span>{typology.replace(/_/g, ' ')}</span>
                  </label>
                {/each}
              </div>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="KYC Profile Depth" description="Configure the level of KYC profile detail">
          {#snippet children()}
            <div class="form-stack">
              <FormGroup
                label="KYC Depth"
                htmlFor="kyc-depth"
                helpText="Level of KYC profile detail (basic, standard, enhanced)"
              >
                {#snippet children()}
                  <select id="kyc-depth" bind:value={$config.banking.kyc_depth}>
                    <option value="basic">Basic</option>
                    <option value="standard">Standard</option>
                    <option value="enhanced">Enhanced</option>
                  </select>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-section">
        <h2>About Banking Module</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>KYC Profiles</h3>
            <p>
              Each customer includes a KYC profile with expected activity envelope,
              declared turnover, source of funds, and beneficial ownership structure.
            </p>
          </div>
          <div class="info-card">
            <h3>AML Typologies</h3>
            <p>
              Generates realistic AML patterns including structuring, funnel accounts,
              layering chains, money mule networks, and various fraud schemes.
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

  .typology-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-4);
  }

  .toggle-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-3);
  }

  .distribution-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
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
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .dist-value {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    font-weight: 500;
  }

  .distribution-item input[type="range"],
  .slider-with-value input[type="range"] {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--color-background);
    appearance: none;
    cursor: pointer;
  }

  .distribution-item input[type="range"]::-webkit-slider-thumb,
  .slider-with-value input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .distribution-total {
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
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

  .risk-appetite-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-2);
  }

  .risk-option {
    display: flex;
    flex-direction: column;
    padding: var(--space-3);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: center;
  }

  .risk-option:hover {
    border-color: var(--color-accent);
  }

  .risk-option.selected {
    border-color: var(--color-accent);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .risk-option input {
    display: none;
  }

  .risk-label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }

  .risk-desc {
    font-size: 0.6875rem;
    color: var(--color-text-secondary);
    margin-top: var(--space-1);
  }

  .slider-with-value {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .slider-with-value input[type="range"] {
    flex: 1;
  }

  .slider-value {
    min-width: 60px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--color-text-primary);
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
    color: var(--color-text-secondary);
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

  .aml-typology-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: var(--space-2); }
  .typology-option { display: flex; align-items: center; gap: var(--space-2); padding: var(--space-2); background-color: var(--color-background); border-radius: var(--radius-sm); cursor: pointer; font-size: 0.8125rem; text-transform: capitalize; }

  @media (max-width: 768px) {
    .form-grid,
    .typology-grid,
    .distribution-grid,
    .toggle-grid,
    .risk-appetite-selector,
    .info-grid,
    .aml-typology-grid {
      grid-template-columns: 1fr;
    }

  }
</style>
