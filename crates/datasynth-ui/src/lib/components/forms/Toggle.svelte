<script lang="ts">
  /**
   * Toggle switch component for boolean values.
   */
  let {
    checked = $bindable(false),
    disabled = false,
    label = '',
    description = '',
    id = crypto.randomUUID(),
    onchange = undefined,
  }: {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    description?: string;
    id?: string;
    onchange?: (() => void) | undefined;
  } = $props();
</script>

<label class="toggle-container" class:disabled>
  <div class="toggle-wrapper">
    <input
      type="checkbox"
      {id}
      bind:checked
      {disabled}
      {onchange}
      class="toggle-input"
    />
    <span class="toggle-track">
      <span class="toggle-thumb"></span>
    </span>
  </div>
  {#if label || description}
    <div class="toggle-text">
      {#if label}
        <span class="toggle-label">{label}</span>
      {/if}
      {#if description}
        <span class="toggle-description">{description}</span>
      {/if}
    </div>
  {/if}
</label>

<style>
  .toggle-container {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    cursor: pointer;
  }

  .toggle-container.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-wrapper {
    position: relative;
    flex-shrink: 0;
  }

  .toggle-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-track {
    display: block;
    width: 40px;
    height: 22px;
    background-color: var(--color-border);
    border-radius: 11px;
    position: relative;
    transition: background-color var(--transition-fast);
  }

  .toggle-input:checked + .toggle-track {
    background-color: var(--color-accent);
  }

  .toggle-input:focus-visible + .toggle-track {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    background-color: white;
    border-radius: 50%;
    transition: transform var(--transition-fast);
    box-shadow: var(--shadow-sm);
  }

  .toggle-input:checked + .toggle-track .toggle-thumb {
    transform: translateX(18px);
  }

  .toggle-text {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding-top: 2px;
  }

  .toggle-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .toggle-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }
</style>
