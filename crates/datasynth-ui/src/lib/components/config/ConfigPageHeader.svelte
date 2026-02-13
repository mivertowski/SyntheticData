<script lang="ts">
  import { configStore } from '$lib/stores/config';

  let {
    title,
    description,
  }: {
    title: string;
    description: string;
  } = $props();

  const isDirty = configStore.isDirty;
  const saving = configStore.saving;

  async function handleSave() {
    await configStore.save();
  }
</script>

<header class="page-header">
  <div>
    <h1>{title}</h1>
    <p>{description}</p>
  </div>
  <div class="header-actions">
    {#if $isDirty}
      <button class="btn-secondary" onclick={() => configStore.reset()}>
        Discard
      </button>
    {/if}
    <button
      class="btn-primary"
      onclick={handleSave}
      disabled={$saving || !$isDirty}
    >
      {$saving ? 'Saving...' : 'Save Changes'}
    </button>
  </div>
</header>

<style>
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-6);
  }

  .page-header h1 {
    margin-bottom: var(--space-1);
  }

  .header-actions {
    display: flex;
    gap: var(--space-2);
  }

  @media (max-width: 768px) {
    .page-header {
      flex-direction: column;
      gap: var(--space-4);
    }
  }
</style>
