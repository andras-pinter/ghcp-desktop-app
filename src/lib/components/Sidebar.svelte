<script lang="ts">
  import {
    getConversationStore,
    clearActiveConversation,
    switchConversation,
    removeConversation,
    toggleFavourite,
    renameConversation,
  } from "$lib/stores/conversations.svelte";
  import { formatDateGroup, truncate } from "$lib/utils/format";

  interface Props {
    collapsed: boolean;
    onNewChat?: () => void;
    onNavigate?: (view: string) => void;
  }

  let { collapsed, onNewChat, onNavigate }: Props = $props();

  const store = getConversationStore();

  // Group conversations: favourites first, then by date
  let favourites = $derived(store.conversations.filter((c) => c.isFavourite));
  let nonFavourites = $derived(store.conversations.filter((c) => !c.isFavourite));

  // Group non-favourites by date label
  let dateGroups = $derived.by(() => {
    const groups: Record<string, typeof nonFavourites> = {};
    for (const conv of nonFavourites) {
      const label = formatDateGroup(conv.updatedAt);
      if (!groups[label]) groups[label] = [];
      groups[label].push(conv);
    }
    return groups;
  });

  let contextMenuConv: string | null = $state(null);
  let renamingConv: string | null = $state(null);
  let renameText = $state("");

  function handleNewChat() {
    clearActiveConversation();
    onNavigate?.("chat");
    onNewChat?.();
  }

  async function handleClick(id: string) {
    await switchConversation(id);
    onNavigate?.("chat");
  }

  function handleContextMenu(event: MouseEvent, id: string) {
    event.preventDefault();
    contextMenuConv = contextMenuConv === id ? null : id;
  }

  function startRename(id: string, currentTitle: string | null) {
    renamingConv = id;
    renameText = currentTitle ?? "";
    contextMenuConv = null;
  }

  async function commitRename() {
    if (renamingConv && renameText.trim()) {
      await renameConversation(renamingConv, renameText.trim());
    }
    renamingConv = null;
    renameText = "";
  }

  function handleRenameKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      commitRename();
    } else if (event.key === "Escape") {
      renamingConv = null;
    }
  }

  async function handleDelete(id: string) {
    contextMenuConv = null;
    await removeConversation(id);
  }

  async function handleToggleFavourite(id: string) {
    contextMenuConv = null;
    await toggleFavourite(id);
  }

  function handleWindowClick() {
    if (contextMenuConv) {
      contextMenuConv = null;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<nav class="sidebar" class:collapsed aria-label="Conversation sidebar">
  <div class="sidebar-inner">
    <!-- Actions -->
    <div class="sidebar-actions">
      <button class="nav-btn" aria-label="New chat" title="New chat" onclick={handleNewChat}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path
            d="M12.5 2.5l1 1L6 11l-2.5.5L4 9l7.5-7.5z"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linejoin="round"
          />
          <path d="M3 13.5h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span class="nav-label">New Chat</span>
      </button>
      <button class="nav-btn" aria-label="Search" title="Search">
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <circle cx="7" cy="7" r="5" />
          <path d="M11 11l3.5 3.5" />
        </svg>
        <span class="nav-label">Search</span>
      </button>
    </div>

    <!-- Conversation list -->
    <div class="sidebar-content">
      {#if favourites.length > 0}
        <section class="sidebar-section">
          <h3 class="section-label">★ Favourites</h3>
          {#each favourites as conv (conv.id)}
            {@render convItem(conv)}
          {/each}
        </section>
      {/if}

      {#each Object.entries(dateGroups) as [label, convos] (label)}
        <section class="sidebar-section">
          <h3 class="section-label">{label}</h3>
          {#each convos as conv (conv.id)}
            {@render convItem(conv)}
          {/each}
        </section>
      {/each}

      {#if !store.hasConversations}
        <section class="sidebar-section">
          <h3 class="section-label">Today</h3>
          <p class="section-empty">No conversations yet</p>
        </section>
      {/if}
    </div>

    <!-- Footer -->
    <div class="sidebar-footer">
      <button
        class="nav-btn"
        aria-label="Projects"
        title="Projects"
        onclick={() => onNavigate?.("projects")}
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <path d="M2 3.5h4.5l1.5 1.5H14v8H2z" />
        </svg>
        <span class="nav-label">Projects</span>
      </button>
      <button
        class="nav-btn"
        aria-label="Agents"
        title="Agents"
        onclick={() => onNavigate?.("agents")}
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <circle cx="8" cy="5" r="3" />
          <path d="M2.5 14c0-3 2.5-5 5.5-5s5.5 2 5.5 5" />
        </svg>
        <span class="nav-label">Agents</span>
      </button>
      <button
        class="nav-btn"
        aria-label="Skills"
        title="Skills"
        onclick={() => onNavigate?.("skills")}
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <path d="M8 1l1.5 3.5L13 5l-2.5 2.5.5 3.5L8 9.5 4.5 11l.5-3.5L2.5 5l3.5-.5z" />
        </svg>
        <span class="nav-label">Skills</span>
      </button>
      <button
        class="nav-btn"
        aria-label="MCP Servers"
        title="MCP Servers"
        onclick={() => onNavigate?.("mcp-settings")}
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <rect x="2" y="2" width="12" height="4" rx="1" />
          <rect x="2" y="10" width="12" height="4" rx="1" />
          <circle cx="5" cy="4" r="0.75" fill="currentColor" stroke="none" />
          <circle cx="5" cy="12" r="0.75" fill="currentColor" stroke="none" />
        </svg>
        <span class="nav-label">MCP</span>
      </button>
      <button
        class="nav-btn"
        aria-label="Settings"
        title="Settings"
        onclick={() => onNavigate?.("settings")}
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <circle cx="8" cy="8" r="2" />
          <path
            d="M8 1v2M8 13v2M1 8h2M13 8h2M2.9 2.9l1.4 1.4M11.7 11.7l1.4 1.4M13.1 2.9l-1.4 1.4M4.3 11.7l-1.4 1.4"
          />
        </svg>
        <span class="nav-label">Settings</span>
      </button>
    </div>
  </div>
</nav>

{#snippet convItem(conv: (typeof favourites)[0])}
  <div class="conv-item-wrapper" role="listitem">
    {#if renamingConv === conv.id}
      <div class="conv-rename">
        <input
          type="text"
          bind:value={renameText}
          onkeydown={handleRenameKeydown}
          onblur={commitRename}
          class="rename-input"
          aria-label="Rename conversation"
        />
      </div>
    {:else}
      <button
        class="conv-item"
        class:active={store.activeConversationId === conv.id}
        onclick={() => handleClick(conv.id)}
        oncontextmenu={(e) => handleContextMenu(e, conv.id)}
        title={conv.title ?? "Untitled"}
        aria-label="Open conversation: {conv.title ?? 'Untitled'}"
      >
        <span class="conv-title">{truncate(conv.title ?? "Untitled", 32)}</span>
      </button>
    {/if}

    {#if contextMenuConv === conv.id}
      <div class="context-menu" role="menu">
        <button
          class="context-item"
          role="menuitem"
          onclick={() => startRename(conv.id, conv.title)}
        >
          Rename
        </button>
        <button class="context-item" role="menuitem" onclick={() => handleToggleFavourite(conv.id)}>
          {conv.isFavourite ? "Unfavourite" : "Favourite"}
        </button>
        <button class="context-item danger" role="menuitem" onclick={() => handleDelete(conv.id)}>
          Delete
        </button>
      </div>
    {/if}
  </div>
{/snippet}

<style>
  .sidebar {
    height: 100%;
    overflow: hidden;
  }

  /*
   * Inner wrapper is always full sidebar width.
   * The parent container clips it during the width transition,
   * so nothing inside reflows — no jank.
   */
  .sidebar-inner {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
  }

  /* ── Actions ── */

  .sidebar-actions {
    padding: var(--spacing-sm);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* ── Shared nav button ── */

  .nav-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    height: 34px;
    padding: 0 var(--spacing-sm);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    transition: all var(--transition-fast);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .nav-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .nav-btn svg {
    flex-shrink: 0;
  }

  /* Labels fade out before the container finishes shrinking */
  .nav-label {
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 1;
    transition: opacity 180ms ease 80ms;
  }

  .collapsed .nav-label {
    opacity: 0;
    transition: opacity 80ms ease;
  }

  /* ── Content ── */

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--spacing-sm);
    opacity: 1;
    transition: opacity 180ms ease 80ms;
  }

  .collapsed .sidebar-content {
    opacity: 0;
    transition: opacity 80ms ease;
    pointer-events: none;
  }

  .sidebar-section {
    padding: var(--spacing-xs) 0;
  }

  .section-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-tertiary);
    padding: var(--spacing-xs) var(--spacing-sm);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .section-empty {
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    padding: var(--spacing-xs) var(--spacing-sm);
    font-style: italic;
  }

  /* ── Conversation items ── */

  .conv-item-wrapper {
    position: relative;
  }

  .conv-item {
    display: flex;
    align-items: center;
    width: 100%;
    height: 34px;
    padding: 0 var(--spacing-sm);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    transition: all var(--transition-fast);
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
  }

  .conv-item:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .conv-item.active {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    font-weight: var(--font-weight-medium);
  }

  .conv-title {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .conv-rename {
    padding: 2px var(--spacing-xs);
  }

  .rename-input {
    width: 100%;
    height: 30px;
    padding: 0 var(--spacing-xs);
    border: 1px solid var(--color-border-focus);
    border-radius: var(--radius-sm);
    background: var(--color-bg-input);
    color: var(--color-text-primary);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    outline: none;
    box-shadow: var(--shadow-input-focus);
  }

  /* ── Context menu ── */

  .context-menu {
    position: absolute;
    right: var(--spacing-xs);
    top: 100%;
    z-index: 100;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: var(--spacing-xs);
    min-width: 140px;
    animation: scaleIn 120ms ease;
  }

  .context-item {
    display: block;
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    text-align: left;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .context-item:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .context-item.danger:hover {
    background: #fef2f2;
    color: #dc2626;
  }

  :global([data-theme="dark"]) .context-item.danger:hover {
    background: #2d1b1b;
    color: #f87171;
  }

  /* ── Footer ── */

  .sidebar-footer {
    padding: var(--spacing-sm);
    border-top: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
  }
</style>
