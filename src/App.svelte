<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import ChatView from "$lib/components/ChatView.svelte";
  import AuthScreen from "$lib/components/AuthScreen.svelte";

  let sidebarCollapsed = $state(false);
  // TODO: Replace with actual auth state from store
  let isAuthenticated = $state(true);

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  function handleKeydown(event: KeyboardEvent) {
    // Cmd+Shift+S: toggle sidebar
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === "S") {
      event.preventDefault();
      toggleSidebar();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if !isAuthenticated}
  <AuthScreen />
{:else}
  <div class="app-layout">
    <aside class="sidebar-container" class:collapsed={sidebarCollapsed}>
      <Sidebar collapsed={sidebarCollapsed} onToggle={toggleSidebar} />
    </aside>
    <main class="main-container">
      <ChatView />
    </main>
  </div>
  <footer class="status-bar">
    <span class="status-indicator online">● Online</span>
    <span class="status-separator">│</span>
    <span class="status-db">DB: —</span>
    <span class="status-separator">│</span>
    <span class="status-version">v0.1.0</span>
  </footer>
{/if}

<style>
  .app-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
    height: calc(100vh - var(--statusbar-height));
  }

  .sidebar-container {
    width: var(--sidebar-width);
    min-width: var(--sidebar-min-width);
    max-width: var(--sidebar-max-width);
    flex-shrink: 0;
    background: var(--color-bg-sidebar);
    border-right: 1px solid var(--color-border-primary);
    transition:
      width var(--transition-normal),
      min-width var(--transition-normal);
    overflow: hidden;
  }

  .sidebar-container.collapsed {
    width: 0;
    min-width: 0;
    border-right: none;
  }

  .main-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .status-bar {
    height: var(--statusbar-height);
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 0 var(--spacing-md);
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    background: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border-primary);
    flex-shrink: 0;
  }

  .status-indicator.online {
    color: var(--color-success);
  }

  .status-separator {
    color: var(--color-border-primary);
  }
</style>
