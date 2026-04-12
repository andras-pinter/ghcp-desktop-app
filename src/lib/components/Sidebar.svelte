<script lang="ts">
  import {
    getConversationStore,
    clearActiveConversation,
    switchConversation,
    removeConversation,
    toggleFavourite,
    renameConversation,
    isStreaming,
    hasUnread,
  } from "$lib/stores/conversations.svelte";
  import { getSourceStore } from "$lib/stores/sources.svelte";
  import { formatDateGroup, truncate } from "$lib/utils/format";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  interface Props {
    collapsed: boolean;
    onNewChat?: () => void;
    onNavigate?: (view: string) => void;
  }

  let { collapsed, onNewChat, onNavigate }: Props = $props();

  const store = getConversationStore();
  const sourceStore = getSourceStore();

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
  let pendingDeleteId: string | null = $state(null);

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

  function handleContextMenuKeydown(event: KeyboardEvent) {
    const menu = event.currentTarget as HTMLElement;
    const items = Array.from(menu.querySelectorAll<HTMLElement>('[role="menuitem"]'));
    const active = document.activeElement as HTMLElement;
    const idx = items.indexOf(active);

    if (event.key === "ArrowDown") {
      event.preventDefault();
      const next = idx < items.length - 1 ? idx + 1 : 0;
      items[next]?.focus();
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      const prev = idx > 0 ? idx - 1 : items.length - 1;
      items[prev]?.focus();
    } else if (event.key === "Escape") {
      event.preventDefault();
      contextMenuConv = null;
    } else if (event.key === "Home") {
      event.preventDefault();
      items[0]?.focus();
    } else if (event.key === "End") {
      event.preventDefault();
      items[items.length - 1]?.focus();
    }
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

  function handleDelete(id: string) {
    contextMenuConv = null;
    pendingDeleteId = id;
  }

  async function confirmDelete() {
    if (!pendingDeleteId) return;
    const id = pendingDeleteId;
    pendingDeleteId = null;
    await removeConversation(id);
  }

  function cancelDelete() {
    pendingDeleteId = null;
  }

  async function handleToggleFavourite(id: string) {
    contextMenuConv = null;
    await toggleFavourite(id);
  }

  /** Keyboard handler for span[role="button"] elements (Enter/Space activates). */
  function actionKeydown(callback: () => void) {
    return (e: KeyboardEvent) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        e.stopPropagation();
        callback();
      }
    };
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
        aria-label="Sources"
        title="Git Sources"
        onclick={() => onNavigate?.("sources")}
      >
        {#if sourceStore.anySyncing}
          <span class="spinner spinner--sm"></span>
        {:else}
          <svg
            width="15"
            height="15"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <line x1="8" y1="1" x2="8" y2="11" />
            <circle cx="8" cy="12" r="2" />
            <line x1="8" y1="5" x2="13" y2="3" />
            <circle cx="13" cy="3" r="1.5" />
          </svg>
        {/if}
        <span class="nav-label">Sources</span>
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

<ConfirmDialog
  open={pendingDeleteId !== null}
  title="Delete this conversation?"
  detail="This cannot be undone."
  onconfirm={confirmDelete}
  oncancel={cancelDelete}
/>

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
        onmousedown={(e) => {
          if (e.button === 2) e.preventDefault();
        }}
        onselectstart={(e) => e.preventDefault()}
        title={conv.title ?? "Untitled"}
        aria-label="Open conversation: {conv.title ?? 'Untitled'}"
      >
        {#if isStreaming(conv.id)}
          <span class="conv-status conv-status--streaming" aria-label="Streaming"></span>
        {:else if hasUnread(conv.id)}
          <span class="conv-status conv-status--unread" aria-label="Unread"></span>
        {/if}
        <span class="conv-title" class:conv-title--unread={hasUnread(conv.id)}
          >{truncate(conv.title ?? "Untitled", 32)}</span
        >
        <span class="conv-actions">
          <span
            class="conv-action-btn"
            class:starred={conv.isFavourite}
            role="button"
            tabindex="0"
            onclick={(e) => {
              e.stopPropagation();
              handleToggleFavourite(conv.id);
            }}
            onkeydown={actionKeydown(() => handleToggleFavourite(conv.id))}
            title={conv.isFavourite ? "Remove from favourites" : "Add to favourites"}
            aria-label={conv.isFavourite ? "Remove from favourites" : "Add to favourites"}
          >
            {#if conv.isFavourite}
              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"
                ><path
                  d="M8 1.23l2.18 4.41 4.87.71-3.52 3.43.83 4.85L8 12.17l-4.36 2.46.83-4.85L1 6.35l4.87-.71L8 1.23z"
                /></svg
              >
            {:else}
              <svg
                width="12"
                height="12"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="1.2"
                aria-hidden="true"
                ><path
                  d="M8 1.23l2.18 4.41 4.87.71-3.52 3.43.83 4.85L8 12.17l-4.36 2.46.83-4.85L1 6.35l4.87-.71L8 1.23z"
                /></svg
              >
            {/if}
          </span>
          <span
            class="conv-action-btn delete-btn"
            role="button"
            tabindex="0"
            onclick={(e) => {
              e.stopPropagation();
              handleDelete(conv.id);
            }}
            onkeydown={actionKeydown(() => handleDelete(conv.id))}
            title="Delete conversation"
            aria-label="Delete conversation"
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
              aria-hidden="true"
              ><path
                d="M3 4h10M6 4V3a1 1 0 011-1h2a1 1 0 011 1v1m2 0v9a1 1 0 01-1 1H5a1 1 0 01-1-1V4h8z"
              /></svg
            >
          </span>
        </span>
      </button>
    {/if}

    {#if contextMenuConv === conv.id}
      <div class="context-menu" role="menu" tabindex="-1" onkeydown={handleContextMenuKeydown}>
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
    user-select: none;
    -webkit-user-select: none;
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
    flex: 1;
    min-width: 0;
  }

  .conv-title--unread {
    font-weight: 650;
  }

  /* ── Inline action icons (star + trash) ── */

  .conv-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .conv-item:hover .conv-actions,
  .conv-item:focus-within .conv-actions,
  .conv-item.active .conv-actions {
    opacity: 1;
  }

  /* Always show star if favourited */
  .conv-actions:has(.starred) {
    opacity: 1;
  }

  .conv-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    padding: 0;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .conv-action-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .conv-action-btn:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .conv-action-btn.starred {
    color: var(--color-accent-copper);
  }

  .conv-action-btn.starred:hover {
    color: var(--color-accent-copper);
  }

  .delete-btn:hover {
    color: var(--color-error);
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

  /* ── Conversation status dots ── */

  .conv-status {
    flex-shrink: 0;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    margin-right: 6px;
  }

  .conv-status--streaming {
    background: var(--color-status-streaming, #22c55e);
    animation: status-pulse 1.2s ease-in-out infinite;
  }

  .conv-status--unread {
    background: var(--color-status-unread, #3b82f6);
  }

  @keyframes status-pulse {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.4;
      transform: scale(0.75);
    }
  }
</style>
