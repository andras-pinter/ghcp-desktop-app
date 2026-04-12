<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import ChatView from "$lib/components/ChatView.svelte";
  import AuthScreen from "$lib/components/AuthScreen.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import McpSettings from "$lib/components/McpSettings.svelte";
  import SkillsPanel from "$lib/components/SkillsPanel.svelte";
  import AgentsPanel from "$lib/components/AgentsPanel.svelte";
  import ProjectView from "$lib/components/ProjectView.svelte";
  import SourcesPanel from "$lib/components/SourcesPanel.svelte";
  import UpdateBanner from "$lib/components/UpdateBanner.svelte";
  import { initAuth, getAuth } from "$lib/stores/auth.svelte";
  import { initModels, getModelStore } from "$lib/stores/models.svelte";
  import {
    initConversations,
    newConversation,
    initStreamingListeners,
  } from "$lib/stores/conversations.svelte";
  import { initMcp, loadRegistry as prefetchMcpRegistry } from "$lib/stores/mcp.svelte";
  import { initAgents, selectAgent, prefetchAgentRegistry } from "$lib/stores/agents.svelte";
  import { initSkills, prefetchRegistry as prefetchSkillRegistry } from "$lib/stores/skills.svelte";
  import { initProjects } from "$lib/stores/projects.svelte";
  import {
    initSources,
    syncAllEnabled,
    updateScanProgress,
    handleSyncComplete,
  } from "$lib/stores/sources.svelte";
  import { initSettings, getSettings } from "$lib/stores/settings.svelte";
  import { initNetwork } from "$lib/stores/network.svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
  import { onMount } from "svelte";

  type AppView =
    | "chat"
    | "settings"
    | "mcp-settings"
    | "skills"
    | "agents"
    | "projects"
    | "sources";

  let sidebarCollapsed = $state(false);
  let dataLoaded = $state(false);
  let currentView = $state<AppView>("chat");
  const auth = getAuth();
  const modelStore = getModelStore();
  const settingsRef = getSettings();
  let unlistenTray: UnlistenFn | undefined;
  let registeredHotkey: string | undefined;

  /** Get the best default model for a new conversation. */
  function defaultModel(): string {
    if (modelStore.defaultModelId) return modelStore.defaultModelId;
    if (modelStore.models.length > 0) return modelStore.models[0].id;
    return "gpt-4o";
  }

  /** Toggle window visibility for global hotkey. */
  async function toggleWindowVisibility(): Promise<void> {
    const win = getCurrentWindow();
    const visible = await win.isVisible();
    if (visible) {
      const focused = await win.isFocused();
      if (focused) {
        await win.hide();
      } else {
        await win.setFocus();
      }
    } else {
      await win.show();
      await win.setFocus();
    }
  }

  /** Register global hotkey. Unregisters previous one if different. */
  async function registerGlobalHotkey(shortcut: string): Promise<void> {
    try {
      if (registeredHotkey && registeredHotkey !== shortcut) {
        await unregister(registeredHotkey);
      }
      if (registeredHotkey === shortcut) return;
      await register(shortcut, (event) => {
        if (event.state === "Pressed") {
          toggleWindowVisibility();
        }
      });
      registeredHotkey = shortcut;
    } catch (e) {
      console.warn("Failed to register global hotkey:", e);
    }
  }

  onMount(() => {
    initAuth();
    initSettings().then(() => {
      // Register global hotkey after settings are loaded
      if (settingsRef.globalHotkey) {
        registerGlobalHotkey(settingsRef.globalHotkey);
      }
    });
    initNetwork();

    // Listen for tray "New Chat" menu item
    let trayCleanup: Promise<UnlistenFn> | undefined;
    trayCleanup = listen("tray-new-chat", () => {
      if (auth.authenticated) {
        newConversation(defaultModel());
        currentView = "chat";
      }
    });
    trayCleanup.then((fn) => (unlistenTray = fn));

    // Listen for git source progress + completion (global, not panel-scoped)
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenComplete: UnlistenFn | undefined;
    listen<{ total: number; fetched: number; phase: string; sourceId?: string }>(
      "git-import-progress",
      (event) => {
        updateScanProgress(
          event.payload.total,
          event.payload.fetched,
          event.payload.phase,
          event.payload.sourceId,
        );
      },
    ).then((fn) => (unlistenProgress = fn));
    listen<{ sourceId: string }>("git-source-sync-complete", (event) => {
      handleSyncComplete(event.payload.sourceId);
    }).then((fn) => (unlistenComplete = fn));

    // Prevent Tauri webview from navigating when files are dragged/dropped
    // outside the designated drop zone. Only preventDefault — do NOT
    // stopPropagation, so InputArea's own handlers still fire.
    const preventDragNav = (e: DragEvent) => e.preventDefault();
    document.addEventListener("dragover", preventDragNav);
    document.addEventListener("drop", preventDragNav);
    return () => {
      document.removeEventListener("dragover", preventDragNav);
      document.removeEventListener("drop", preventDragNav);
      unlistenTray?.();
      unlistenProgress?.();
      unlistenComplete?.();
      if (registeredHotkey) {
        unregister(registeredHotkey);
      }
    };
  });

  // Load conversations & models whenever auth becomes true (startup or fresh login)
  $effect(() => {
    if (auth.authenticated && !dataLoaded) {
      dataLoaded = true;
      Promise.all([
        initConversations(),
        initStreamingListeners(),
        initModels(),
        initMcp(),
        initAgents(),
        initSkills(),
        initProjects(),
        initSources(),
      ]).then(() => {
        // Select the user's default agent for the initial welcome screen
        if (settingsRef.defaultAgentId) {
          selectAgent(settingsRef.defaultAgentId);
        }
        // Prefetch registries in the background after core data is loaded
        prefetchMcpRegistry();
        prefetchSkillRegistry();
        prefetchAgentRegistry();
        // Auto-sync enabled git sources (silent, no UI disruption)
        syncAllEnabled();
      });
    } else if (!auth.authenticated) {
      dataLoaded = false;
    }
  });

  // Re-register global hotkey when setting changes
  $effect(() => {
    const hotkey = settingsRef.globalHotkey;
    if (hotkey && settingsRef.loaded && hotkey !== registeredHotkey) {
      registerGlobalHotkey(hotkey);
    }
  });

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  function navigateTo(view: string) {
    currentView = view as AppView;
  }

  function navigateBack() {
    currentView = "chat";
  }

  function handleKeydown(event: KeyboardEvent) {
    const mod = event.metaKey || event.ctrlKey;

    // Cmd/Ctrl+Shift+S → toggle sidebar
    if (mod && event.shiftKey && event.key === "S") {
      event.preventDefault();
      toggleSidebar();
      return;
    }

    // Cmd/Ctrl+N → new chat
    if (mod && event.key === "n" && !event.shiftKey) {
      event.preventDefault();
      if (auth.authenticated) {
        newConversation(defaultModel());
        currentView = "chat";
      }
      return;
    }

    // Cmd/Ctrl+, → open settings
    if (mod && event.key === ",") {
      event.preventDefault();
      if (auth.authenticated) {
        currentView = currentView === "settings" ? "chat" : "settings";
      }
      return;
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

    <UpdateBanner />

    <!-- Body: sidebar + chat -->
    <div class="app-body">
      <aside class="sidebar-container" class:collapsed={sidebarCollapsed}>
        <Sidebar collapsed={sidebarCollapsed} onNavigate={navigateTo} />
      </aside>
      <main class="main-container">
        {#if currentView === "settings"}
          <SettingsPanel onBack={navigateBack} />
        {:else if currentView === "mcp-settings"}
          <McpSettings onBack={navigateBack} />
        {:else if currentView === "skills"}
          <SkillsPanel onBack={navigateBack} />
        {:else if currentView === "agents"}
          <AgentsPanel onBack={navigateBack} />
        {:else if currentView === "projects"}
          <ProjectView
            onBack={navigateBack}
            onOpenChat={() => {
              currentView = "chat";
            }}
          />
        {:else if currentView === "sources"}
          <SourcesPanel onBack={navigateBack} />
        {:else}
          <ChatView />
        {/if}
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
