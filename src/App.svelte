<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import ChatView from "$lib/components/ChatView.svelte";
  import AuthScreen from "$lib/components/AuthScreen.svelte";

  let sidebarCollapsed = $state(false);
  let isAuthenticated = $state(true);

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
{/if}

<style>
  .app-layout {
    display: flex;
    height: 100vh;
    overflow: hidden;
    animation: fadeIn 400ms ease both;
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
</style>
