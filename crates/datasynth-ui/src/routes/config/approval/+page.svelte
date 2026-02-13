<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle, InputNumber } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  function addThreshold() {
    if ($config) {
      const thresholds = $config.approval.thresholds;
      const lastAmount = thresholds.length > 0 ? thresholds[thresholds.length - 1].amount : 0;
      $config.approval.thresholds = [
        ...thresholds,
        { amount: lastAmount * 2 || 1000, level: thresholds.length + 1, roles: ['manager'] }
      ];
    }
  }

  function removeThreshold(index: number) {
    if ($config) {
      $config.approval.thresholds = $config.approval.thresholds.filter((_, i) => i !== index);
    }
  }

  function addRole(thresholdIndex: number, role: string) {
    if ($config && role) {
      const threshold = $config.approval.thresholds[thresholdIndex];
      if (!threshold.roles.includes(role)) {
        threshold.roles = [...threshold.roles, role];
      }
    }
  }

  function removeRole(thresholdIndex: number, roleIndex: number) {
    if ($config) {
      const threshold = $config.approval.thresholds[thresholdIndex];
      threshold.roles = threshold.roles.filter((_, i) => i !== roleIndex);
    }
  }

  const roleOptions = ['junior_accountant', 'senior_accountant', 'controller', 'manager', 'director', 'vp', 'executive'];
</script>

<div class="page">
  <ConfigPageHeader title="Approval Workflow" description="Configure approval thresholds and workflow behavior" />

  {#if $config}
    <div class="sections">
      <FormSection
        title="Workflow Settings"
        description="Enable and configure approval workflow generation"
      >
        <div class="section-content">
          <div class="toggle-row highlight">
            <div class="toggle-info">
              <span class="toggle-label">Enable Approval Workflow</span>
              <span class="toggle-description">
                Generate approval chains and workflow data for transactions above thresholds
              </span>
            </div>
            <Toggle bind:checked={$config.approval.enabled} />
          </div>

          <div class="form-grid">
            <FormGroup
              label="Auto-Approve Threshold"
              htmlFor="auto-approve"
              helpText="Transactions below this amount are auto-approved"
            >
              <InputNumber
                id="auto-approve"
                bind:value={$config.approval.auto_approve_threshold}
                min={0}
                max={1000000}
                step={100}
              />
            </FormGroup>

            <FormGroup
              label="Rejection Rate"
              htmlFor="rejection-rate"
              helpText="Percentage of approvals that are rejected (0-10%)"
            >
              <InputNumber
                id="rejection-rate"
                bind:value={$config.approval.rejection_rate}
                min={0}
                max={0.1}
                step={0.01}
              />
            </FormGroup>

            <FormGroup
              label="Revision Rate"
              htmlFor="revision-rate"
              helpText="Percentage requiring revision before approval (0-10%)"
            >
              <InputNumber
                id="revision-rate"
                bind:value={$config.approval.revision_rate}
                min={0}
                max={0.1}
                step={0.01}
              />
            </FormGroup>

            <FormGroup
              label="Avg Approval Delay (hours)"
              htmlFor="approval-delay"
              helpText="Average time for approval processing"
            >
              <InputNumber
                id="approval-delay"
                bind:value={$config.approval.average_approval_delay_hours}
                min={0}
                max={168}
                step={0.5}
              />
            </FormGroup>
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Approval Thresholds"
        description="Define approval levels based on transaction amount"
      >
        <div class="section-content">
          <div class="thresholds-list">
            {#each $config.approval.thresholds as threshold, i}
              <div class="threshold-card">
                <div class="threshold-header">
                  <span class="threshold-level">Level {threshold.level}</span>
                  <button class="btn-remove" onclick={() => removeThreshold(i)}>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </div>

                <div class="threshold-content">
                  <FormGroup label="Amount Threshold" htmlFor={`threshold-${i}`}>
                    <InputNumber
                      id={`threshold-${i}`}
                      bind:value={threshold.amount}
                      min={0}
                      step={1000}
                    />
                  </FormGroup>

                  <div class="roles-section">
                    <label>Required Approvers</label>
                    <div class="roles-list">
                      {#each threshold.roles as role, j}
                        <span class="role-tag">
                          {role.replace('_', ' ')}
                          <button onclick={() => removeRole(i, j)}>×</button>
                        </span>
                      {/each}
                    </div>
                    <select onchange={(e) => { addRole(i, e.currentTarget.value); e.currentTarget.value = ''; }}>
                      <option value="">Add role...</option>
                      {#each roleOptions.filter(r => !threshold.roles.includes(r)) as role}
                        <option value={role}>{role.replace('_', ' ')}</option>
                      {/each}
                    </select>
                  </div>
                </div>
              </div>
            {/each}
          </div>

          <button class="btn-add" onclick={addThreshold}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            Add Threshold
          </button>
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

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
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

  .toggle-row.highlight {
    border: 1px solid var(--color-border);
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

  .thresholds-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .threshold-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .threshold-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-3);
  }

  .threshold-level {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-accent);
  }

  .btn-remove {
    padding: var(--space-1);
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .btn-remove:hover {
    color: var(--color-error);
    background-color: rgba(239, 68, 68, 0.1);
  }

  .btn-remove svg {
    width: 16px;
    height: 16px;
  }

  .threshold-content {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: var(--space-4);
    align-items: start;
  }

  .roles-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .roles-section label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .roles-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .role-tag {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    font-size: 0.75rem;
    background-color: var(--color-surface);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
  }

  .role-tag button {
    padding: 0;
    margin-left: var(--space-1);
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
  }

  .role-tag button:hover {
    color: var(--color-error);
  }

  .btn-add {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    color: var(--color-accent);
    background: none;
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .btn-add:hover {
    border-color: var(--color-accent);
    background-color: rgba(99, 102, 241, 0.05);
  }

  .btn-add svg {
    width: 16px;
    height: 16px;
  }

  select {
    padding: var(--space-2) var(--space-3);
    font-size: 0.8125rem;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  select:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  @media (max-width: 640px) {
    .form-grid {
      grid-template-columns: 1fr;
    }

    .threshold-content {
      grid-template-columns: 1fr;
    }
  }
</style>
