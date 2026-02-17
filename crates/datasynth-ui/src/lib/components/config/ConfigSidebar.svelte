<script lang="ts">
  import { page } from '$app/stores';
  import { configStore } from '$lib/stores/config';

  // Collapsible section state
  let collapsedSections = $state<Record<string, boolean>>({
    'Enterprise': true,
    'Interconnectivity': true,
    'Standards': true,
    'Specialized': true,
  });

  // Scroll indicator state
  let navElement: HTMLElement | undefined = $state(undefined);
  let showScrollIndicator = $state(false);

  function checkScroll() {
    if (!navElement) return;
    const { scrollTop, scrollHeight, clientHeight } = navElement;
    showScrollIndicator = scrollHeight - scrollTop - clientHeight > 20;
  }

  $effect(() => {
    if (!navElement) return;
    checkScroll();
    const observer = new ResizeObserver(() => checkScroll());
    observer.observe(navElement);
    return () => observer.disconnect();
  });

  function toggleSection(section: string) {
    collapsedSections[section] = !collapsedSections[section];
  }

  // Navigation structure
  const navItems = [
    {
      section: 'Generate',
      items: [
        { href: '/', label: 'Dashboard', icon: 'dashboard' },
        { href: '/stream', label: 'Stream Viewer', icon: 'stream' },
      ],
    },
    {
      section: 'Core Config',
      items: [
        { href: '/config', label: 'Overview', icon: 'settings' },
        { href: '/config/global', label: 'Global Settings', icon: 'globe' },
        { href: '/config/companies', label: 'Companies', icon: 'building' },
        { href: '/config/chart-of-accounts', label: 'Chart of Accounts', icon: 'chart' },
        { href: '/config/transactions', label: 'Transactions', icon: 'transactions' },
        { href: '/config/output', label: 'Output', icon: 'export' },
      ],
    },
    {
      section: 'Master Data',
      items: [
        { href: '/config/master-data', label: 'Overview', icon: 'database' },
        { href: '/config/master-data/vendors', label: 'Vendors', icon: 'vendor' },
        { href: '/config/master-data/customers', label: 'Customers', icon: 'customer' },
        { href: '/config/master-data/materials', label: 'Materials', icon: 'material' },
        { href: '/config/master-data/assets', label: 'Fixed Assets', icon: 'asset' },
        { href: '/config/master-data/employees', label: 'Employees', icon: 'employee' },
      ],
    },
    {
      section: 'Document Flows',
      items: [
        { href: '/config/document-flows', label: 'Overview', icon: 'flow' },
        { href: '/config/document-flows/p2p', label: 'Procure to Pay', icon: 'p2p' },
        { href: '/config/document-flows/o2c', label: 'Order to Cash', icon: 'o2c' },
      ],
    },
    {
      section: 'Enterprise',
      items: [
        { href: '/config/source-to-pay', label: 'Source-to-Pay', icon: 's2p' },
        { href: '/config/financial-reporting', label: 'Financial Reporting', icon: 'dollar' },
        { href: '/config/hr', label: 'HR / Payroll', icon: 'hr' },
        { href: '/config/manufacturing', label: 'Manufacturing', icon: 'manufacturing' },
        { href: '/config/sales-quotes', label: 'Sales Quotes', icon: 'quote' },
        { href: '/config/tax', label: 'Tax Accounting', icon: 'tax' },
        { href: '/config/treasury', label: 'Treasury', icon: 'treasury' },
        { href: '/config/project-accounting', label: 'Project Accounting', icon: 'project' },
        { href: '/config/esg', label: 'ESG / Sustainability', icon: 'esg' },
      ],
    },
    {
      section: 'Interconnectivity',
      items: [
        { href: '/config/vendor-network', label: 'Vendor Network', icon: 'network' },
        { href: '/config/customer-segmentation', label: 'Customer Segments', icon: 'segments' },
        { href: '/config/relationship-strength', label: 'Relationships', icon: 'relationship' },
        { href: '/config/cross-process-links', label: 'Cross-Links', icon: 'crosslink' },
        { href: '/config/intercompany', label: 'Intercompany', icon: 'ic' },
      ],
    },
    {
      section: 'Quality & Anomaly',
      items: [
        { href: '/config/compliance', label: 'Fraud & Controls', icon: 'shield' },
        { href: '/config/data-quality', label: 'Data Quality', icon: 'quality' },
        { href: '/config/anomaly-injection', label: 'Anomaly Injection', icon: 'anomaly' },
        { href: '/config/analytics', label: 'Analytics', icon: 'analytics' },
      ],
    },
    {
      section: 'Standards',
      items: [
        { href: '/config/accounting-standards', label: 'Accounting Standards', icon: 'standards' },
        { href: '/config/graph-export', label: 'Graph Export', icon: 'graph' },
        { href: '/config/quality-gates', label: 'Quality Gates', icon: 'gate' },
      ],
    },
    {
      section: 'Specialized',
      items: [
        { href: '/config/banking', label: 'Banking / KYC', icon: 'bank' },
        { href: '/config/fingerprint', label: 'Fingerprinting', icon: 'fingerprint' },
        { href: '/config/temporal', label: 'Temporal Patterns', icon: 'clock' },
        { href: '/config/audit', label: 'Audit Generation', icon: 'audit' },
        { href: '/config/ocpm', label: 'Process Mining', icon: 'process' },
        { href: '/config/scenario', label: 'Scenario', icon: 'scenario' },
        { href: '/config/behavioral-drift', label: 'Behavioral Drift', icon: 'drift' },
        { href: '/config/market-drift', label: 'Market Drift', icon: 'market' },
        { href: '/config/organizational-events', label: 'Org Events', icon: 'org' },
      ],
    },
    {
      section: 'Presets',
      items: [
        { href: '/presets', label: 'Manage Presets', icon: 'preset' },
      ],
    },
  ];

  let { collapsed = false } = $props();

  function isActive(href: string, currentPath: string): boolean {
    if (href === '/') {
      return currentPath === '/';
    }
    if (href === '/config') {
      return currentPath === '/config';
    }
    return currentPath.startsWith(href);
  }

  // Check if any item in a section is active (for auto-expand)
  function isSectionActive(group: typeof navItems[0]): boolean {
    return group.items.some(item => isActive(item.href, $page.url.pathname));
  }

  // Get icon SVG
  function getIcon(name: string): string {
    const icons: Record<string, string> = {
      dashboard: 'M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z',
      stream: 'M4 6h16M4 12h16M4 18h16',
      settings: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z',
      globe: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z',
      building: 'M6 22V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v18M6 22h12M10 6h.01M14 6h.01M10 10h.01M14 10h.01M10 14h.01M14 14h.01M10 18h.01M14 18h.01',
      chart: 'M3 3v18h18M7 16l4-4 4 4 4-8',
      transactions: 'M18 20V10M12 20V4M6 20v-6',
      export: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12',
      database: 'M21 5c0 1.1-4 2-9 2s-9-.9-9-2m18 0c0-1.1-4-2-9-2s-9 .9-9 2m18 0v14c0 1.1-4 2-9 2s-9-.9-9-2V5m18 7c0 1.1-4 2-9 2s-9-.9-9-2',
      vendor: 'M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M12 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
      customer: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 7a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
      material: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96L12 12.01l8.73-5.05M12 22.08V12',
      asset: 'M2 20h20M6 16V8a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8M9 16v-6h6v6',
      employee: 'M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2M12 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z',
      flow: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5',
      p2p: 'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v0a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2zM9 12h6M9 16h6',
      o2c: 'M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1zM12 11v6M9 14l3 3 3-3',
      s2p: 'M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3zM10 7h4M17 10v4M14 17h-4M7 14v-4',
      dollar: 'M12 1v22M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6',
      hr: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 7a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM20 8v6M23 11h-6',
      manufacturing: 'M2 20h20M5 20V8l5 4V8l5 4V4l5 4v12',
      quote: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zM14 2v6h6M16 13H8M16 17H8M10 9H8',
      network: 'M5.5 4.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 0 0-5 0zM13.5 19.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 0 0-5 0zM13.5 4.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 0 0-5 0zM5.5 19.5a2.5 2.5 0 1 0 5 0 2.5 2.5 0 0 0-5 0zM8 7v10M16 7v10',
      segments: 'M22 12h-4l-3 9L9 3l-3 9H2',
      relationship: 'M8 12h8M12 8v8M3 12a9 9 0 1 0 18 0 9 9 0 0 0-18 0z',
      crosslink: 'M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7',
      ic: 'M18 8h2a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V10a2 2 0 0 1 2-2h2M12 2v12M8 6l4-4 4 4',
      shield: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10zM9 12l2 2 4-4',
      quality: 'M22 11.08V12a10 10 0 1 1-5.93-9.14M22 4L12 14.01l-3-3',
      anomaly: 'M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0zM12 9v4M12 17h.01',
      analytics: 'M21 21H4.6c-.6 0-1-.4-1-1V3M7 14l4-4 4 4 6-6',
      standards: 'M4 19.5A2.5 2.5 0 0 1 6.5 17H20M4 19.5A2.5 2.5 0 0 0 6.5 22H20V2H6.5A2.5 2.5 0 0 0 4 4.5v15z',
      graph: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5',
      gate: 'M12 22V8M5 12H2l10-10 10 10h-3M7 22h10',
      bank: 'M3 21h18M3 10h18M5 6l7-3 7 3M4 10v11M20 10v11M8 14v3M12 14v3M16 14v3',
      fingerprint: 'M12 11c0 3.517-1.009 6.799-2.753 9.571M12 11c0-1.657 1.343-3 3-3s3 1.343 3 3c0 2.99-.723 5.815-2.006 8.304M12 11c0-1.657-1.343-3-3-3s-3 1.343-3 3c0 3.517 1.009 6.799 2.753 9.571M12 4c1.657 0 3 1.343 3 3v4M12 4c-1.657 0-3 1.343-3 3v4M12 4V2',
      clock: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 6v6l4 2',
      audit: 'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2M9 14l2 2 4-4',
      process: 'M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3zM12 6l4 4',
      scenario: 'M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2zM14 2v6h6M12 12v6M9 15h6',
      drift: 'M22 12h-4l-3 9L9 3l-3 9H2',
      market: 'M3 3v18h18M7 16l4-8 4 4 4-8',
      org: 'M12 2a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM5 19a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM19 19a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM12 8v4M7 19l5-7M17 19l-5-7',
      preset: 'M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2zM17 21v-8H7v8M7 3v5h8',
      tax: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zM14 2v6h6M9 13h6M9 17h4M12 9h.01',
      treasury: 'M3 21h18M3 10h18M5 6l7-3 7 3M4 10v11M8 10v11M12 10v11M16 10v11M20 10v11',
      project: 'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2M9 14h.01M9 17h.01M13 14h2M13 17h2M9 11h6',
      esg: 'M12 22c4-2.5 7-5.5 7-10a7 7 0 0 0-14 0c0 4.5 3 7.5 7 10zM12 8v4M12 16h.01',
    };
    return icons[name] || 'M12 12m-10 0a10 10 0 1 0 20 0 10 10 0 1 0-20 0';
  }

  const isDirty = configStore.isDirty;
  const isValid = configStore.isValid;
</script>

<aside class="sidebar" class:collapsed>
  <div class="sidebar-header">
    {#if !collapsed}
      <span class="logo-text">SYNTH</span>
      <span class="logo-subtext">Configuration</span>
    {:else}
      <span class="logo-text-short">S</span>
    {/if}
  </div>

  <nav class="sidebar-nav" bind:this={navElement} onscroll={checkScroll}>
    {#each navItems as group}
      {@const isCollapsible = group.section in collapsedSections}
      {@const isGroupCollapsed = collapsedSections[group.section] && !isSectionActive(group)}
      <div class="nav-group">
        {#if !collapsed}
          <button
            type="button"
            class="nav-group-label"
            class:collapsible={isCollapsible}
            onclick={() => isCollapsible && toggleSection(group.section)}
          >
            <span>{group.section}</span>
            {#if isCollapsible}
              <svg
                class="collapse-chevron"
                class:collapsed={isGroupCollapsed}
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="M6 9l6 6 6-6" />
              </svg>
            {/if}
          </button>
        {/if}
        {#if !isGroupCollapsed || collapsed}
          {#each group.items as item}
            <a
              href={item.href}
              class="nav-item"
              class:active={isActive(item.href, $page.url.pathname)}
              title={collapsed ? item.label : ''}
            >
              <svg
                class="nav-icon"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d={getIcon(item.icon)} />
              </svg>
              {#if !collapsed}
                <span class="nav-label">{item.label}</span>
              {/if}
            </a>
          {/each}
        {/if}
      </div>
    {/each}
  </nav>

  {#if showScrollIndicator}
    <div class="scroll-indicator">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M6 9l6 6 6-6" />
      </svg>
    </div>
  {/if}

  <div class="sidebar-footer">
    {#if $isDirty}
      <div class="status-indicator" class:warning={!$isValid}>
        {#if !collapsed}
          <span class="status-dot" class:warning={!$isValid} class:active={$isValid}></span>
          <span>Unsaved changes</span>
        {:else}
          <span class="status-dot" class:warning={!$isValid} class:active={$isValid}></span>
        {/if}
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    min-width: 240px;
    height: 100vh;
    background-color: var(--color-surface);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    position: sticky;
    top: 0;
    overflow: hidden;
    transition: width 200ms ease, min-width 200ms ease;
  }

  .sidebar.collapsed {
    width: 60px;
    min-width: 60px;
  }

  .sidebar-header {
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-1);
  }

  .collapsed .sidebar-header {
    align-items: center;
  }

  .logo-text {
    font-size: 1rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--color-text-primary);
  }

  .logo-text-short {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-accent);
  }

  .logo-subtext {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--space-3);
    overflow-y: auto;
  }

  .nav-group {
    margin-bottom: var(--space-3);
  }

  .nav-group-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    font-size: 0.625rem;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    padding: var(--space-2) var(--space-2);
    margin-bottom: var(--space-1);
    background: none;
    border: none;
    cursor: default;
    border-radius: var(--radius-sm);
  }

  .nav-group-label.collapsible {
    cursor: pointer;
  }

  .nav-group-label.collapsible:hover {
    color: var(--color-text-secondary);
    background-color: var(--color-background);
  }

  .collapse-chevron {
    width: 12px;
    height: 12px;
    flex-shrink: 0;
    transition: transform 150ms ease;
  }

  .collapse-chevron.collapsed {
    transform: rotate(-90deg);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    text-decoration: none;
    font-size: 0.8125rem;
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .collapsed .nav-item {
    justify-content: center;
    padding: var(--space-2);
  }

  .nav-item:hover {
    background-color: var(--color-background);
    color: var(--color-text-primary);
  }

  .nav-item.active {
    background-color: var(--color-accent);
    color: white;
  }

  .nav-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .nav-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sidebar-footer {
    padding: var(--space-3);
    border-top: 1px solid var(--color-border);
    min-height: 48px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .collapsed .status-indicator {
    justify-content: center;
  }

  .status-indicator.warning {
    color: var(--color-warning);
  }

  .scroll-indicator {
    display: flex;
    justify-content: center;
    padding: var(--space-1) 0;
    background: linear-gradient(transparent, var(--color-surface) 40%);
    margin-top: -24px;
    position: relative;
    z-index: 1;
    pointer-events: none;
  }

  .scroll-indicator svg {
    width: 16px;
    height: 16px;
    color: var(--color-text-muted);
    animation: bounce 2s ease infinite;
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(3px); }
  }
</style>
