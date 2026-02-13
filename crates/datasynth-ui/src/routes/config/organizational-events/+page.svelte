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

  const EVENT_TYPES = [
    { value: 'acquisition', label: 'Acquisition' },
    { value: 'divestiture', label: 'Divestiture' },
    { value: 'restructuring', label: 'Restructuring' },
    { value: 'merger', label: 'Merger' },
    { value: 'ipo', label: 'IPO' },
    { value: 'policy_change', label: 'Policy Change' },
  ];

  function addEvent() {
    if (!$config) return;
    $config.organizational_events.events = [
      ...$config.organizational_events.events,
      {
        event_type: 'restructuring',
        date: '',
        description: '',
        volume_multiplier: 1.0,
        amount_multiplier: 1.0,
      },
    ];
  }

  function removeEvent(index: number) {
    if (!$config) return;
    $config.organizational_events.events = $config.organizational_events.events.filter(
      (_: unknown, i: number) => i !== index
    );
  }
</script>

<div class="page">
  <ConfigPageHeader title="Organizational Events" description="Define discrete organizational events that impact generated data" />

  {#if $config}
    <div class="page-content">
      <FormSection title="Organizational Events Settings" description="Enable discrete event simulation">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.organizational_events.enabled}
              label="Enable Organizational Events"
              description="Define discrete events like acquisitions, mergers, and restructurings that affect data generation"
            />
          </div>
        {/snippet}
      </FormSection>

      {#if $config.organizational_events.enabled}
        <FormSection title="Event List" description="Define organizational events and their impact on data generation">
          {#snippet children()}
            <div class="form-stack">
              {#if $config.organizational_events.events.length > 0}
                <div class="event-list">
                  {#each $config.organizational_events.events as event, i}
                    <div class="event-item">
                      <select bind:value={event.event_type}>
                        {#each EVENT_TYPES as type}
                          <option value={type.value}>{type.label}</option>
                        {/each}
                      </select>

                      <input
                        type="date"
                        bind:value={event.date}
                        placeholder="Date"
                      />

                      <input
                        type="text"
                        bind:value={event.description}
                        placeholder="Description of event..."
                      />

                      <input
                        type="number"
                        bind:value={event.volume_multiplier}
                        min="0"
                        max="10"
                        step="0.1"
                        placeholder="Vol"
                        title="Volume Multiplier"
                      />

                      <input
                        type="number"
                        bind:value={event.amount_multiplier}
                        min="0"
                        max="10"
                        step="0.1"
                        placeholder="Amt"
                        title="Amount Multiplier"
                      />

                      <button
                        class="btn-danger"
                        onclick={() => removeEvent(i)}
                        title="Remove event"
                      >
                        Remove
                      </button>
                    </div>
                  {/each}
                </div>

                <div class="event-legend">
                  <span class="legend-item">Type</span>
                  <span class="legend-item">Date</span>
                  <span class="legend-item">Description</span>
                  <span class="legend-item">Vol. Mult.</span>
                  <span class="legend-item">Amt. Mult.</span>
                  <span class="legend-item"></span>
                </div>
              {:else}
                <p class="empty-message">No events configured. Click "Add Event" to define organizational events.</p>
              {/if}

              <button class="btn-outline" onclick={addEvent}>
                + Add Event
              </button>
            </div>
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-cards">
        <div class="info-card">
          <h4>Acquisition</h4>
          <p>Models the impact of acquiring another company, typically increasing transaction volume, adding new entities, and creating integration-related entries.</p>
        </div>
        <div class="info-card">
          <h4>Divestiture</h4>
          <p>Simulates selling off a business unit, reducing transaction volumes and generating disposal-related accounting entries and asset write-offs.</p>
        </div>
        <div class="info-card">
          <h4>Restructuring</h4>
          <p>Models organizational restructuring events with associated one-time charges, employee-related costs, and temporary volume changes.</p>
        </div>
        <div class="info-card">
          <h4>Merger / IPO / Policy Change</h4>
          <p>Mergers combine entity data, IPOs add compliance requirements and reporting changes, and policy changes shift approval thresholds and process flows.</p>
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
  .suffix { font-size: 0.8125rem; color: var(--color-text-muted); font-family: var(--font-mono); }
  .distribution-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-4); }
  .distribution-item { display: flex; flex-direction: column; gap: var(--space-1); }
  .distribution-item label { font-size: 0.8125rem; font-weight: 500; color: var(--color-text-secondary); }
  .slider-with-value { display: flex; align-items: center; gap: var(--space-2); }
  .slider-with-value input[type='range'] { flex: 1; }
  .slider-with-value span { font-size: 0.8125rem; font-family: var(--font-mono); min-width: 44px; text-align: right; }
  .distribution-total { padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); font-size: 0.8125rem; background-color: var(--color-background); }
  .distribution-total.warning { background-color: rgba(234, 179, 8, 0.1); border: 1px solid #eab308; }
  .info-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: var(--space-4); margin-top: var(--space-4); }
  .info-card { padding: var(--space-4); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .info-card h4 { font-size: 0.875rem; font-weight: 600; color: var(--color-text-primary); margin-bottom: var(--space-2); }
  .info-card p { font-size: 0.8125rem; color: var(--color-text-secondary); margin: 0; }
  .event-list { display: flex; flex-direction: column; gap: var(--space-3); }
  .event-item { display: grid; grid-template-columns: 1fr 1fr 2fr 1fr 1fr auto; gap: var(--space-2); align-items: center; padding: var(--space-3); background-color: var(--color-background); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .event-item input, .event-item select { font-size: 0.8125rem; }
  .event-item select { padding: var(--space-1) var(--space-2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); background-color: var(--color-surface); }
  .event-item input[type='text'] { padding: var(--space-1) var(--space-2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); }
  .event-item input[type='date'] { padding: var(--space-1) var(--space-2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); }
  .event-item input[type='number'] { padding: var(--space-1) var(--space-2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); width: 100%; }
  .event-legend { display: grid; grid-template-columns: 1fr 1fr 2fr 1fr 1fr auto; gap: var(--space-2); padding: 0 var(--space-3); }
  .legend-item { font-size: 0.6875rem; font-weight: 500; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
  .empty-message { font-size: 0.8125rem; color: var(--color-text-secondary); font-style: italic; padding: var(--space-4); text-align: center; background-color: var(--color-background); border-radius: var(--radius-md); border: 1px dashed var(--color-border); }
  .btn-danger { background-color: var(--color-error, #ef4444); color: white; border: none; padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); cursor: pointer; font-size: 0.75rem; }
  .btn-danger:hover { opacity: 0.9; }
  .btn-outline { background: none; border: 1px solid var(--color-border); padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); cursor: pointer; font-size: 0.8125rem; color: var(--color-text-secondary); }
  .btn-outline:hover { background-color: var(--color-background); color: var(--color-text-primary); }
  .loading { display: flex; align-items: center; justify-content: center; padding: var(--space-10); color: var(--color-text-secondary); }
  @media (max-width: 768px) { .form-grid, .distribution-grid { grid-template-columns: 1fr; } .event-item { grid-template-columns: 1fr; } .event-legend { display: none; } }
</style>
