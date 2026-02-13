<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormGroup, FormSection, InputNumber } from '$lib/components/forms';
  import DistributionEditor from '$lib/components/forms/DistributionEditor.svelte';
  import ConfigPageHeader from '$lib/components/config/ConfigPageHeader.svelte';

  const config = configStore.config;

  const personaLabels: Record<string, string> = {
    junior_accountant: 'Junior Accountant',
    senior_accountant: 'Senior Accountant',
    controller: 'Controller',
    manager: 'Manager',
    automated_system: 'Automated System',
  };

  const personaDescriptions: Record<string, string> = {
    junior_accountant: 'Entry-level staff handling routine transactions',
    senior_accountant: 'Experienced staff with elevated permissions',
    controller: 'Finance leadership with approval authority',
    manager: 'Department managers with budget oversight',
    automated_system: 'System-generated transactions (interfaces, batches)',
  };
</script>

<div class="page">
  <ConfigPageHeader title="User Personas" description="Configure user types and their transaction distributions" />

  {#if $config}
    <div class="sections">
      <FormSection
        title="Transaction Source Distribution"
        description="Percentage of transactions created by each user type"
      >
        <div class="section-content">
          <div class="info-banner">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4M12 8h.01" />
            </svg>
            <span>This controls who creates transactions, affecting audit trails and approval workflows</span>
          </div>

          <DistributionEditor
            bind:distribution={$config.user_personas.persona_distribution}
            labels={personaLabels}
            descriptions={personaDescriptions}
          />
        </div>
      </FormSection>

      <FormSection
        title="User Counts"
        description="Number of users generated for each persona type"
      >
        <div class="section-content">
          <div class="user-count-grid">
            <div class="user-count-card">
              <div class="user-icon junior">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                  <circle cx="12" cy="7" r="4" />
                </svg>
              </div>
              <div class="user-info">
                <span class="user-label">Junior Accountants</span>
                <span class="user-desc">Entry-level staff</span>
              </div>
              <InputNumber
                bind:value={$config.user_personas.users_per_persona.junior_accountant}
                min={0}
                max={100}
                step={1}
              />
            </div>

            <div class="user-count-card">
              <div class="user-icon senior">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                  <circle cx="12" cy="7" r="4" />
                </svg>
              </div>
              <div class="user-info">
                <span class="user-label">Senior Accountants</span>
                <span class="user-desc">Experienced staff</span>
              </div>
              <InputNumber
                bind:value={$config.user_personas.users_per_persona.senior_accountant}
                min={0}
                max={50}
                step={1}
              />
            </div>

            <div class="user-count-card">
              <div class="user-icon controller">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                  <circle cx="12" cy="7" r="4" />
                </svg>
              </div>
              <div class="user-info">
                <span class="user-label">Controllers</span>
                <span class="user-desc">Finance leadership</span>
              </div>
              <InputNumber
                bind:value={$config.user_personas.users_per_persona.controller}
                min={0}
                max={10}
                step={1}
              />
            </div>

            <div class="user-count-card">
              <div class="user-icon manager">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                  <circle cx="12" cy="7" r="4" />
                </svg>
              </div>
              <div class="user-info">
                <span class="user-label">Managers</span>
                <span class="user-desc">Department heads</span>
              </div>
              <InputNumber
                bind:value={$config.user_personas.users_per_persona.manager}
                min={0}
                max={20}
                step={1}
              />
            </div>

            <div class="user-count-card">
              <div class="user-icon system">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                  <line x1="8" y1="21" x2="16" y2="21" />
                  <line x1="12" y1="17" x2="12" y2="21" />
                </svg>
              </div>
              <div class="user-info">
                <span class="user-label">Automated Systems</span>
                <span class="user-desc">Interface users</span>
              </div>
              <InputNumber
                bind:value={$config.user_personas.users_per_persona.automated_system}
                min={0}
                max={100}
                step={1}
              />
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

  .info-banner {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3);
    background-color: rgba(99, 102, 241, 0.1);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .info-banner svg {
    width: 16px;
    height: 16px;
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .user-count-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .user-count-card {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .user-icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    flex-shrink: 0;
  }

  .user-icon svg {
    width: 20px;
    height: 20px;
  }

  .user-icon.junior { background-color: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .user-icon.senior { background-color: rgba(59, 130, 246, 0.15); color: #3b82f6; }
  .user-icon.controller { background-color: rgba(168, 85, 247, 0.15); color: #a855f7; }
  .user-icon.manager { background-color: rgba(249, 115, 22, 0.15); color: #f97316; }
  .user-icon.system { background-color: rgba(107, 114, 128, 0.15); color: #6b7280; }

  .user-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .user-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .user-desc {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .user-count-card :global(input) {
    width: 80px;
    text-align: center;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--color-text-muted);
  }
</style>
