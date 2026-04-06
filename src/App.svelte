<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import ChatView from "$lib/components/ChatView.svelte";
  import AuthScreen from "$lib/components/AuthScreen.svelte";
  import { initAuth, getAuth } from "$lib/stores/auth.svelte";
  import { initModels } from "$lib/stores/models.svelte";
  import { initConversations } from "$lib/stores/conversations.svelte";
  import { onMount } from "svelte";

  let sidebarCollapsed = $state(false);
  let dataLoaded = $state(false);
  const auth = getAuth();

  onMount(() => {
    initAuth();
  });

  // Load conversations & models whenever auth becomes true (startup or fresh login)
  $effect(() => {
    if (auth.authenticated && !dataLoaded) {
      dataLoaded = true;
      Promise.all([initConversations(), initModels()]);
    }
  });

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === "S") {
      event.preventDefault();
      toggleSidebar();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if auth.loading}
  <div class="loading-screen">
    <div class="loading-drag-region" data-tauri-drag-region></div>
    <div class="loading-spinner"></div>
  </div>
{:else if !auth.authenticated}
  <AuthScreen />
{:else}
  <div class="app-root">
    <!-- Custom title bar (overlays native on macOS) -->
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-left" data-tauri-drag-region>
        <button
          class="titlebar-btn"
          onclick={toggleSidebar}
          aria-label="Toggle sidebar"
          title="Toggle sidebar (⌘⇧S)"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <rect
              x="1"
              y="2"
              width="4"
              height="12"
              rx="1"
              stroke="currentColor"
              stroke-width="1.5"
              fill="none"
            />
            <rect
              x="6"
              y="2"
              width="9"
              height="12"
              rx="1"
              stroke="currentColor"
              stroke-width="1.5"
              fill="none"
            />
          </svg>
        </button>
      </div>
      <span class="titlebar-title" data-tauri-drag-region>Chuck</span>
      <div class="titlebar-right" data-tauri-drag-region></div>
    </header>

    <!-- Body: sidebar + chat -->
    <div class="app-body">
      <aside class="sidebar-container" class:collapsed={sidebarCollapsed}>
        <Sidebar collapsed={sidebarCollapsed} />
      </aside>
      <main class="main-container">
        <ChatView />
      </main>
    </div>
  </div>
{/if}

<style>
  .app-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    animation: fadeIn 400ms ease both;
  }

  /* ── Title bar ── */

  .titlebar {
    height: var(--titlebar-height);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    padding: 0 var(--spacing-md);
    padding-left: var(--titlebar-traffic-light-offset);
    background: var(--color-bg-sidebar);
    border-bottom: 1px solid var(--color-border-secondary);
    user-select: none;
    -webkit-user-select: none;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .titlebar-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .titlebar-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .titlebar-title {
    flex: 1;
    text-align: center;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
  }

  .titlebar-right {
    width: 28px;
  }

  /* ── Body ── */

  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar-container {
    width: var(--sidebar-width);
    flex-shrink: 0;
    background: var(--color-bg-sidebar);
    border-right: 1px solid var(--color-border-secondary);
    transition: width var(--transition-slow);
    overflow: hidden;
  }

  .sidebar-container.collapsed {
    width: var(--sidebar-collapsed-width);
  }

  .main-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
    background: var(--color-bg-primary);
  }

  /* ── Loading ── */

  .loading-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--color-bg-primary);
    position: relative;
  }

  .loading-drag-region {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: var(--titlebar-height);
    -webkit-app-region: drag;
  }

  .loading-spinner {
    width: 24px;
    height: 24px;
    border: 2.5px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
