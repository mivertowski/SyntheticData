<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import ConfigSidebar from '$lib/components/config/ConfigSidebar.svelte';
  import { configStore } from '$lib/stores/config';

  let { children } = $props();

  // Mobile menu state
  let mobileMenuOpen = $state(false);

  // Determine if we should show sidebar based on route
  let showSidebar = $derived(
    $page.url.pathname.startsWith('/config') ||
    $page.url.pathname.startsWith('/presets') ||
    $page.url.pathname === '/' ||
    $page.url.pathname === '/stream'
  );

  // Close mobile menu on route change
  $effect(() => {
    $page.url.pathname;
    mobileMenuOpen = false;
  });

  // Load config on mount
  onMount(() => {
    configStore.load();
  });

  // Quick save keyboard shortcut
  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault();
      configStore.save();
    }
  }

  function toggleMobileMenu() {
    mobileMenuOpen = !mobileMenuOpen;
  }

  function closeMobileMenu() {
    mobileMenuOpen = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-layout" class:with-sidebar={showSidebar}>
  {#if showSidebar}
    <!-- Mobile hamburger button -->
    <button
      class="mobile-menu-toggle"
      onclick={toggleMobileMenu}
      aria-label="Toggle navigation menu"
      aria-expanded={mobileMenuOpen}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {#if mobileMenuOpen}
          <path d="M6 6l12 12M6 18L18 6" />
        {:else}
          <path d="M4 6h16M4 12h16M4 18h16" />
        {/if}
      </svg>
    </button>

    <!-- Mobile overlay -->
    {#if mobileMenuOpen}
      <button
        class="mobile-overlay"
        onclick={closeMobileMenu}
        aria-label="Close menu"
      ></button>
    {/if}

    <!-- Sidebar with mobile class -->
    <div class="sidebar-container" class:mobile-open={mobileMenuOpen}>
      <ConfigSidebar />
    </div>
  {/if}

  <div class="main-area">
    <main class="main-content">
      {@render children()}
    </main>

    <footer class="app-footer">
      <span class="footer-text">Synthetic Data Generator v0.1.0</span>
      <span class="footer-sep">|</span>
      <span class="footer-shortcut">Ctrl+S to save</span>
    </footer>
  </div>
</div>

<style>
  .app-layout {
    display: flex;
    min-height: 100vh;
  }

  .sidebar-container {
    display: contents;
  }

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background-color: var(--color-background);
  }

  .main-content {
    flex: 1;
    padding: var(--space-6);
    overflow-y: auto;
  }

  .app-footer {
    border-top: 1px solid var(--color-border);
    padding: var(--space-3) var(--space-6);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background-color: var(--color-background);
  }

  .footer-text {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .footer-sep {
    color: var(--color-border);
  }

  .footer-shortcut {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
    background-color: var(--color-surface);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
  }

  /* Mobile menu toggle button */
  .mobile-menu-toggle {
    display: none;
    position: fixed;
    top: var(--space-3);
    left: var(--space-3);
    z-index: 1001;
    width: 44px;
    height: 44px;
    padding: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background-color: var(--color-background);
    color: var(--color-text-primary);
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .mobile-menu-toggle svg {
    width: 100%;
    height: 100%;
  }

  /* Mobile overlay */
  .mobile-overlay {
    display: none;
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 999;
    border: none;
    cursor: pointer;
  }

  /* Mobile responsive styles */
  @media (max-width: 768px) {
    .mobile-menu-toggle {
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .mobile-overlay {
      display: block;
    }

    .sidebar-container {
      display: block;
      position: fixed;
      top: 0;
      left: 0;
      z-index: 1000;
      transform: translateX(-100%);
      transition: transform 200ms ease;
    }

    .sidebar-container.mobile-open {
      transform: translateX(0);
    }

    .main-content {
      padding: var(--space-4);
      padding-top: calc(var(--space-4) + 56px); /* Account for hamburger button */
    }

    .app-footer {
      padding: var(--space-2) var(--space-4);
      flex-wrap: wrap;
      gap: var(--space-2);
    }

    .footer-text {
      font-size: 0.6875rem;
    }
  }

  /* Tablet responsive styles */
  @media (max-width: 1024px) and (min-width: 769px) {
    .main-content {
      padding: var(--space-4);
    }
  }
</style>
