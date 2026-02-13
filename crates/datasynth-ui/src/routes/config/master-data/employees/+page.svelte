<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, DistributionEditor } from '$lib/components/forms';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  // Default employee distribution settings
  const defaultDistribution = {
    finance: 0.25,
    operations: 0.30,
    sales: 0.20,
    procurement: 0.15,
    management: 0.10,
  };

  // Ensure distribution exists
  $effect(() => {
    if ($config?.master_data?.employees && Object.keys($config.master_data.employees.distribution).length === 0) {
      $config.master_data.employees.distribution = { ...defaultDistribution };
    }
  });

  // Department labels
  const departmentLabels: Record<string, string> = {
    finance: 'Finance & Accounting',
    operations: 'Operations',
    sales: 'Sales & Marketing',
    procurement: 'Procurement',
    management: 'Management',
  };
</script>

<div class="page">
  <ConfigPageHeader title="Employees Configuration" description="Configure user accounts and approval hierarchies" />

  {#if $config}
    <div class="sections">
      <!-- Entity Count -->
      <FormSection
        title="Employee Count"
        description="Number of employee/user records to generate"
      >
        <div class="section-content">
          <FormGroup
            label="Number of Employees"
            htmlFor="employee-count"
            helpText="Users who create, approve, and process transactions"
          >
            <input
              id="employee-count"
              type="number"
              min="1"
              max="10000"
              step="1"
              bind:value={$config.master_data.employees.count}
            />
          </FormGroup>

          <div class="quick-presets">
            <span class="preset-label">Quick presets:</span>
            <button type="button" onclick={() => $config.master_data.employees.count = 10}>
              Small (10)
            </button>
            <button type="button" onclick={() => $config.master_data.employees.count = 50}>
              Medium (50)
            </button>
            <button type="button" onclick={() => $config.master_data.employees.count = 200}>
              Large (200)
            </button>
            <button type="button" onclick={() => $config.master_data.employees.count = 1000}>
              Enterprise (1000)
            </button>
          </div>
        </div>
      </FormSection>

      <!-- Department Distribution -->
      <FormSection
        title="Department Distribution"
        description="Distribution of employees across departments"
      >
        <div class="section-content">
          <p class="section-intro">
            Employees are assigned to departments which determine their transaction
            types, approval authorities, and Segregation of Duties (SoD) profiles.
          </p>

          <DistributionEditor
            label="Departments"
            bind:distribution={$config.master_data.employees.distribution}
            labels={departmentLabels}
            helpText="Affects transaction creation patterns and SoD conflict detection"
          />
        </div>
      </FormSection>

      <!-- Employee Characteristics -->
      <FormSection
        title="Employee Characteristics"
        description="Approval hierarchies and access controls"
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
              <strong>Approval Hierarchy</strong>
              <p>
                Employees are organized in a manager hierarchy that controls approval workflows:
              </p>
              <ul>
                <li><strong>Level 1:</strong> Staff - Creates transactions, limited approval ($0-$1K)</li>
                <li><strong>Level 2:</strong> Supervisor - Approves routine transactions ($1K-$10K)</li>
                <li><strong>Level 3:</strong> Manager - Approves significant transactions ($10K-$100K)</li>
                <li><strong>Level 4:</strong> Director/VP - Approves major transactions ($100K+)</li>
              </ul>
            </div>
          </div>

          <div class="characteristics-grid">
            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                  </svg>
                </span>
                <span class="characteristic-title">Segregation of Duties</span>
              </div>
              <p class="characteristic-description">
                System roles are assigned to prevent SoD conflicts. When violations
                occur, they're flagged for fraud/control testing.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                  </svg>
                </span>
                <span class="characteristic-title">Transaction Codes</span>
              </div>
              <p class="characteristic-description">
                Employees are assigned transaction codes (T-codes) based on their
                role, controlling what operations they can perform.
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
                    <circle cx="8.5" cy="7" r="4" />
                    <line x1="20" y1="8" x2="20" y2="14" />
                    <line x1="23" y1="11" x2="17" y2="11" />
                  </svg>
                </span>
                <span class="characteristic-title">User IDs</span>
              </div>
              <p class="characteristic-description">
                Realistic user IDs are generated following corporate naming
                conventions (e.g., JSMITH, J.SMITH, john.smith).
              </p>
            </div>

            <div class="characteristic-card">
              <div class="characteristic-header">
                <span class="characteristic-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="3" />
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4" />
                  </svg>
                </span>
                <span class="characteristic-title">System Roles</span>
              </div>
              <p class="characteristic-description">
                Composite roles combine multiple authorizations for realistic
                access patterns (AP Clerk, GL Accountant, Controller, etc.).
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
