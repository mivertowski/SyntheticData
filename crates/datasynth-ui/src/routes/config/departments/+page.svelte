<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, Toggle, InputNumber } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';
  import type { CustomDepartmentConfig } from '$lib/stores/config';

  const config = configStore.config;

  function addDepartment() {
    if ($config) {
      const newDept: CustomDepartmentConfig = {
        code: `DEPT${$config.departments.custom_departments.length + 1}`,
        name: 'New Department',
        cost_center: null,
        primary_processes: [],
        parent_code: null,
      };
      $config.departments.custom_departments = [...$config.departments.custom_departments, newDept];
    }
  }

  function removeDepartment(index: number) {
    if ($config) {
      $config.departments.custom_departments = $config.departments.custom_departments.filter((_, i) => i !== index);
    }
  }

  function toggleProcess(deptIndex: number, process: string) {
    if ($config) {
      const dept = $config.departments.custom_departments[deptIndex];
      if (dept.primary_processes.includes(process)) {
        dept.primary_processes = dept.primary_processes.filter(p => p !== process);
      } else {
        dept.primary_processes = [...dept.primary_processes, process];
      }
    }
  }

  const processOptions = ['O2C', 'P2P', 'R2R', 'H2R', 'A2R'];
</script>

<div class="page">
  <ConfigPageHeader title="Departments" description="Configure organizational department structure" />

  {#if $config}
    <div class="sections">
      <FormSection
        title="Department Settings"
        description="Enable and configure department generation"
      >
        <div class="section-content">
          <div class="toggle-row highlight">
            <div class="toggle-info">
              <span class="toggle-label">Enable Departments</span>
              <span class="toggle-description">
                Generate department assignments for transactions and users
              </span>
            </div>
            <Toggle bind:checked={$config.departments.enabled} />
          </div>

          <FormGroup
            label="Headcount Multiplier"
            htmlFor="headcount-mult"
            helpText="Scale factor for department headcounts (1.0 = default)"
          >
            <InputNumber
              id="headcount-mult"
              bind:value={$config.departments.headcount_multiplier}
              min={0.1}
              max={10}
              step={0.1}
            />
          </FormGroup>
        </div>
      </FormSection>

      <FormSection
        title="Default Departments"
        description="Standard departments generated when enabled"
        collapsible
        collapsed
      >
        <div class="section-content">
          <div class="default-depts">
            <div class="dept-badge">
              <span class="dept-code">FIN</span>
              <span class="dept-name">Finance & Accounting</span>
            </div>
            <div class="dept-badge">
              <span class="dept-code">PROC</span>
              <span class="dept-name">Procurement</span>
            </div>
            <div class="dept-badge">
              <span class="dept-code">SALES</span>
              <span class="dept-name">Sales</span>
            </div>
            <div class="dept-badge">
              <span class="dept-code">WH</span>
              <span class="dept-name">Warehouse & Logistics</span>
            </div>
            <div class="dept-badge">
              <span class="dept-code">IT</span>
              <span class="dept-name">Information Technology</span>
            </div>
            <div class="dept-badge">
              <span class="dept-code">HR</span>
              <span class="dept-name">Human Resources</span>
            </div>
            <div class="dept-badge">
              <span class="dept-code">OPS</span>
              <span class="dept-name">Operations</span>
            </div>
            <div class="dept-badge">
              <span class="dept-code">EXEC</span>
              <span class="dept-name">Executive</span>
            </div>
          </div>
        </div>
      </FormSection>

      <FormSection
        title="Custom Departments"
        description="Add custom departments to the organization"
      >
        <div class="section-content">
          {#if $config.departments.custom_departments.length > 0}
            <div class="custom-depts">
              {#each $config.departments.custom_departments as dept, i}
                <div class="custom-dept-card">
                  <div class="dept-header">
                    <button class="btn-remove" onclick={() => removeDepartment(i)}>
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                      </svg>
                    </button>
                  </div>

                  <div class="dept-fields">
                    <FormGroup label="Code" htmlFor={`dept-code-${i}`}>
                      <input
                        id={`dept-code-${i}`}
                        type="text"
                        bind:value={dept.code}
                        placeholder="DEPT1"
                      />
                    </FormGroup>

                    <FormGroup label="Name" htmlFor={`dept-name-${i}`}>
                      <input
                        id={`dept-name-${i}`}
                        type="text"
                        bind:value={dept.name}
                        placeholder="Department Name"
                      />
                    </FormGroup>

                    <FormGroup label="Cost Center" htmlFor={`dept-cc-${i}`}>
                      <input
                        id={`dept-cc-${i}`}
                        type="text"
                        bind:value={dept.cost_center}
                        placeholder="Optional"
                      />
                    </FormGroup>

                    <FormGroup label="Parent" htmlFor={`dept-parent-${i}`}>
                      <input
                        id={`dept-parent-${i}`}
                        type="text"
                        bind:value={dept.parent_code}
                        placeholder="Optional"
                      />
                    </FormGroup>
                  </div>

                  <div class="process-selection">
                    <label>Primary Processes</label>
                    <div class="process-buttons">
                      {#each processOptions as process}
                        <button
                          class="process-btn"
                          class:active={dept.primary_processes.includes(process)}
                          onclick={() => toggleProcess(i, process)}
                        >
                          {process}
                        </button>
                      {/each}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="empty-state">
              <p>No custom departments defined</p>
            </div>
          {/if}

          <button class="btn-add" onclick={addDepartment}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            Add Custom Department
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

  .default-depts {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .dept-badge {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .dept-code {
    font-size: 0.75rem;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--color-accent);
    background-color: var(--color-surface);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .dept-name {
    font-size: 0.8125rem;
    color: var(--color-text-primary);
  }

  .custom-depts {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .custom-dept-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .dept-header {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--space-3);
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

  .dept-fields {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .process-selection {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .process-selection label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .process-buttons {
    display: flex;
    gap: var(--space-2);
  }

  .process-btn {
    padding: var(--space-1) var(--space-2);
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .process-btn:hover {
    border-color: var(--color-accent);
  }

  .process-btn.active {
    color: var(--color-accent);
    background-color: rgba(99, 102, 241, 0.1);
    border-color: var(--color-accent);
  }

  .empty-state {
    padding: var(--space-4);
    text-align: center;
    color: var(--color-text-muted);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .empty-state p {
    margin: 0;
    font-size: 0.875rem;
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

  input[type="text"] {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font-size: 0.8125rem;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  input[type="text"]:focus {
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

  @media (max-width: 768px) {
    .dept-fields {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 480px) {
    .dept-fields {
      grid-template-columns: 1fr;
    }
  }
</style>
