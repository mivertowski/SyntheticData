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
  <ConfigPageHeader title="Project Accounting" description="Configure project types, WBS, cost allocation, and earned value management" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Project Accounting Module" description="Enable project accounting data generation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.project_accounting.enabled}
              label="Enable Project Accounting"
              description="Generate projects, WBS, cost allocations, milestones, and EVM metrics"
            />

            {#if $config.project_accounting.enabled}
              <FormGroup
                label="Project Count"
                htmlFor="project-count"
                helpText="Number of projects to generate"
                error={getError('project_accounting.project_count')}
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="project-count"
                    bind:value={$config.project_accounting.project_count}
                    min="1"
                    step="1"
                  />
                {/snippet}
              </FormGroup>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      {#if $config.project_accounting.enabled}
        <FormSection title="Project Type Distribution" description="Distribution of project types (must sum to 100%)">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup label="Capital" htmlFor="pt-capital" helpText="Capital expenditure projects" error={getError('project_accounting.project_types')}>
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="pt-capital" bind:value={$config.project_accounting.project_types.capital} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.project_types.capital * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup label="Internal" htmlFor="pt-internal" helpText="Internal organizational projects" error={getError('project_accounting.project_types')}>
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="pt-internal" bind:value={$config.project_accounting.project_types.internal} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.project_types.internal * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup label="Customer" htmlFor="pt-customer" helpText="Customer-facing contract projects" error={getError('project_accounting.project_types')}>
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="pt-customer" bind:value={$config.project_accounting.project_types.customer} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.project_types.customer * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup label="R&D" htmlFor="pt-rnd" helpText="Research and development projects" error={getError('project_accounting.project_types')}>
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="pt-rnd" bind:value={$config.project_accounting.project_types.r_and_d} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.project_types.r_and_d * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup label="Maintenance" htmlFor="pt-maint" helpText="Maintenance and repair projects" error={getError('project_accounting.project_types')}>
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="pt-maint" bind:value={$config.project_accounting.project_types.maintenance} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.project_types.maintenance * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup label="Technology" htmlFor="pt-tech" helpText="IT and technology projects" error={getError('project_accounting.project_types')}>
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="pt-tech" bind:value={$config.project_accounting.project_types.technology} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.project_types.technology * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="WBS Structure" description="Work Breakdown Structure hierarchy settings">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Max Depth"
                htmlFor="wbs-depth"
                helpText="Maximum WBS hierarchy depth"
                error={getError('project_accounting.wbs.max_depth')}
              >
                {#snippet children()}
                  <input type="number" id="wbs-depth" bind:value={$config.project_accounting.wbs.max_depth} min="1" max="10" step="1" />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Min Elements / Level"
                htmlFor="wbs-min"
                helpText="Minimum WBS elements per hierarchy level"
                error={getError('project_accounting.wbs.min_elements_per_level')}
              >
                {#snippet children()}
                  <input type="number" id="wbs-min" bind:value={$config.project_accounting.wbs.min_elements_per_level} min="1" step="1" />
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Max Elements / Level"
                htmlFor="wbs-max"
                helpText="Maximum WBS elements per hierarchy level"
                error={getError('project_accounting.wbs.max_elements_per_level')}
              >
                {#snippet children()}
                  <input type="number" id="wbs-max" bind:value={$config.project_accounting.wbs.max_elements_per_level} min="1" step="1" />
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Cost Allocation Rates" description="What proportion of source documents get tagged to projects">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Time Entry Rate"
                htmlFor="ca-time"
                helpText="Proportion of time entries allocated to projects"
                error={getError('project_accounting.cost_allocation.time_entry_project_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="ca-time" bind:value={$config.project_accounting.cost_allocation.time_entry_project_rate} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.cost_allocation.time_entry_project_rate * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Expense Rate"
                htmlFor="ca-expense"
                helpText="Proportion of expenses allocated to projects"
                error={getError('project_accounting.cost_allocation.expense_project_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="ca-expense" bind:value={$config.project_accounting.cost_allocation.expense_project_rate} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.cost_allocation.expense_project_rate * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Purchase Order Rate"
                htmlFor="ca-po"
                helpText="Proportion of POs allocated to projects"
                error={getError('project_accounting.cost_allocation.purchase_order_project_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="ca-po" bind:value={$config.project_accounting.cost_allocation.purchase_order_project_rate} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.cost_allocation.purchase_order_project_rate * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Vendor Invoice Rate"
                htmlFor="ca-vi"
                helpText="Proportion of vendor invoices allocated to projects"
                error={getError('project_accounting.cost_allocation.vendor_invoice_project_rate')}
              >
                {#snippet children()}
                  <div class="input-with-suffix">
                    <input type="number" id="ca-vi" bind:value={$config.project_accounting.cost_allocation.vendor_invoice_project_rate} min="0" max="1" step="0.05" />
                    <span class="suffix">{($config.project_accounting.cost_allocation.vendor_invoice_project_rate * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Revenue Recognition" description="Project revenue recognition settings (ASC 606 / IFRS 15)">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.project_accounting.revenue_recognition.enabled}
                label="Enable Revenue Recognition"
                description="Generate project revenue recognition schedules"
              />

              {#if $config.project_accounting.revenue_recognition.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Recognition Method"
                    htmlFor="rev-method"
                    helpText="Revenue recognition method"
                    error={getError('project_accounting.revenue_recognition.method')}
                  >
                    {#snippet children()}
                      <select id="rev-method" bind:value={$config.project_accounting.revenue_recognition.method}>
                        <option value="percentage_of_completion">Percentage of Completion</option>
                        <option value="completed_contract">Completed Contract</option>
                        <option value="milestone">Milestone-Based</option>
                      </select>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Completion Measure"
                    htmlFor="completion-measure"
                    helpText="How completion percentage is measured"
                    error={getError('project_accounting.revenue_recognition.completion_measure')}
                  >
                    {#snippet children()}
                      <select id="completion-measure" bind:value={$config.project_accounting.revenue_recognition.completion_measure}>
                        <option value="cost_to_cost">Cost-to-Cost</option>
                        <option value="efforts_expended">Efforts Expended</option>
                        <option value="units_delivered">Units Delivered</option>
                      </select>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Avg Contract Value"
                    htmlFor="contract-value"
                    helpText="Average contract value for customer projects"
                    error={getError('project_accounting.revenue_recognition.avg_contract_value')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input type="number" id="contract-value" bind:value={$config.project_accounting.revenue_recognition.avg_contract_value} min="0" step="10000" />
                        <span class="suffix">USD</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Milestones & Change Orders" description="Configure project milestones and change order handling">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.project_accounting.milestones.enabled}
                label="Enable Milestones"
                description="Generate project milestones with payment triggers"
              />

              {#if $config.project_accounting.milestones.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Avg Milestones / Project"
                    htmlFor="avg-milestones"
                    helpText="Average number of milestones per project"
                    error={getError('project_accounting.milestones.avg_per_project')}
                  >
                    {#snippet children()}
                      <input type="number" id="avg-milestones" bind:value={$config.project_accounting.milestones.avg_per_project} min="1" step="1" />
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Payment Milestone Rate"
                    htmlFor="payment-milestone"
                    helpText="Proportion of milestones that trigger payments"
                    error={getError('project_accounting.milestones.payment_milestone_rate')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input type="number" id="payment-milestone" bind:value={$config.project_accounting.milestones.payment_milestone_rate} min="0" max="1" step="0.05" />
                        <span class="suffix">{($config.project_accounting.milestones.payment_milestone_rate * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}

              <Toggle
                bind:checked={$config.project_accounting.change_orders.enabled}
                label="Enable Change Orders"
                description="Generate project change orders with approval workflows"
              />

              {#if $config.project_accounting.change_orders.enabled}
                <div class="form-grid">
                  <FormGroup
                    label="Change Order Probability"
                    htmlFor="co-prob"
                    helpText="Probability a project has change orders"
                    error={getError('project_accounting.change_orders.probability')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input type="number" id="co-prob" bind:value={$config.project_accounting.change_orders.probability} min="0" max="1" step="0.05" />
                        <span class="suffix">{($config.project_accounting.change_orders.probability * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Max per Project"
                    htmlFor="co-max"
                    helpText="Maximum change orders per project"
                    error={getError('project_accounting.change_orders.max_per_project')}
                  >
                    {#snippet children()}
                      <input type="number" id="co-max" bind:value={$config.project_accounting.change_orders.max_per_project} min="0" step="1" />
                    {/snippet}
                  </FormGroup>

                  <FormGroup
                    label="Approval Rate"
                    htmlFor="co-approval"
                    helpText="Proportion of change orders that get approved"
                    error={getError('project_accounting.change_orders.approval_rate')}
                  >
                    {#snippet children()}
                      <div class="input-with-suffix">
                        <input type="number" id="co-approval" bind:value={$config.project_accounting.change_orders.approval_rate} min="0" max="1" step="0.05" />
                        <span class="suffix">{($config.project_accounting.change_orders.approval_rate * 100).toFixed(0)}%</span>
                      </div>
                    {/snippet}
                  </FormGroup>
                </div>
              {/if}
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Retainage & Earned Value" description="Configure retainage and EVM reporting">
          {#snippet children()}
            <div class="form-stack">
              <Toggle
                bind:checked={$config.project_accounting.retainage.enabled}
                label="Enable Retainage"
                description="Withhold a percentage of payments until project completion"
              />

              {#if $config.project_accounting.retainage.enabled}
                <FormGroup
                  label="Retainage Percentage"
                  htmlFor="retainage-pct"
                  helpText="Default retainage withholding percentage"
                  error={getError('project_accounting.retainage.default_percentage')}
                >
                  {#snippet children()}
                    <div class="input-with-suffix">
                      <input type="number" id="retainage-pct" bind:value={$config.project_accounting.retainage.default_percentage} min="0" max="1" step="0.01" />
                      <span class="suffix">{($config.project_accounting.retainage.default_percentage * 100).toFixed(0)}%</span>
                    </div>
                  {/snippet}
                </FormGroup>
              {/if}

              <Toggle
                bind:checked={$config.project_accounting.earned_value.enabled}
                label="Enable Earned Value Management"
                description="Generate CPI, SPI, EAC, and other EVM metrics"
              />

              {#if $config.project_accounting.earned_value.enabled}
                <FormGroup
                  label="EVM Frequency"
                  htmlFor="evm-freq"
                  helpText="How often earned value metrics are calculated"
                  error={getError('project_accounting.earned_value.frequency')}
                >
                  {#snippet children()}
                    <select id="evm-freq" bind:value={$config.project_accounting.earned_value.frequency}>
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

        <FormSection title="Anomaly Rate" description="Project-specific anomaly injection">
          {#snippet children()}
            <FormGroup
              label="Anomaly Rate"
              htmlFor="proj-anomaly"
              helpText="Rate of anomalous project records (0-100%)"
              error={getError('project_accounting.anomaly_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input type="number" id="proj-anomaly" bind:value={$config.project_accounting.anomaly_rate} min="0" max="1" step="0.005" />
                  <span class="suffix">{($config.project_accounting.anomaly_rate * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Project Lifecycle</h4>
          <p>
            Generates full project lifecycles with WBS hierarchies, cost allocations
            from time entries/expenses/POs, milestone-based billing, change orders
            with approval workflows, and retainage tracking.
          </p>
        </div>
        <div class="info-card">
          <h4>Output Files</h4>
          <p>
            Generates projects.csv, wbs_elements.csv, project_costs.csv,
            project_milestones.csv, change_orders.csv, retainage.csv,
            and earned_value_snapshots.csv.
          </p>
        </div>
        <div class="info-card">
          <h4>EVM Metrics</h4>
          <p>
            Earned Value Management produces CPI (Cost Performance Index),
            SPI (Schedule Performance Index), EAC (Estimate at Completion),
            ETC (Estimate to Complete), and TCPI (To-Complete Performance Index).
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
