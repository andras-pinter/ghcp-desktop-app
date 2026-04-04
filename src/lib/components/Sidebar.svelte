<script lang="ts">
  interface Props {
    collapsed: boolean;
    onToggle: () => void;
  }

  let { collapsed, onToggle }: Props = $props();
</script>

<nav class="sidebar" aria-label="Conversation sidebar" aria-hidden={collapsed}>
  <div class="sidebar-header">
    <button class="new-chat-btn" aria-label="New chat"> + New Chat </button>
    <button class="collapse-btn" onclick={onToggle} aria-label="Toggle sidebar"> ◀ </button>
  </div>

  <div class="sidebar-search">
    <input type="search" placeholder="Search..." aria-label="Search conversations" />
  </div>

  <div class="sidebar-content">
    <section class="sidebar-section">
      <h3 class="section-title">★ Favourites</h3>
      <p class="empty-hint">No favourites yet</p>
    </section>

    <section class="sidebar-section">
      <h3 class="section-title">📁 Projects</h3>
      <p class="empty-hint">No projects yet</p>
    </section>

    <section class="sidebar-section">
      <h3 class="section-title">🤖 Agents</h3>
      <p class="empty-hint">Default agent active</p>
    </section>

    <section class="sidebar-section">
      <h3 class="section-title">📅 Today</h3>
      <p class="empty-hint">No conversations yet</p>
    </section>
  </div>

  <div class="sidebar-footer">
    <button class="sidebar-action" aria-label="Skills">⚡ Skills</button>
    <button class="sidebar-action" aria-label="Settings">⚙️ Settings</button>
  </div>
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .sidebar-header {
    padding: var(--spacing-md);
    flex-shrink: 0;
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .collapse-btn {
    padding: var(--spacing-xs);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .collapse-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .new-chat-btn {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-accent);
    color: var(--color-text-inverse);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
    transition: background var(--transition-fast);
  }

  .new-chat-btn:hover {
    background: var(--color-accent-hover);
  }

  .sidebar-search {
    padding: 0 var(--spacing-md) var(--spacing-sm);
    flex-shrink: 0;
  }

  .sidebar-search input {
    width: 100%;
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    outline: none;
  }

  .sidebar-search input:focus {
    border-color: var(--color-border-focus);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--spacing-md);
  }

  .sidebar-section {
    margin-bottom: var(--spacing-lg);
  }

  .section-title {
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--spacing-xs);
  }

  .empty-hint {
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .sidebar-footer {
    padding: var(--spacing-md);
    border-top: 1px solid var(--color-border-secondary);
    display: flex;
    gap: var(--spacing-sm);
    flex-shrink: 0;
  }

  .sidebar-action {
    flex: 1;
    padding: var(--spacing-xs) var(--spacing-sm);
    background: transparent;
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all var(--transition-fast);
  }

  .sidebar-action:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }
</style>
