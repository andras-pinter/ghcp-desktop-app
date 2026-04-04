<script lang="ts">
  interface Props {
    collapsed: boolean;
    onToggle: () => void;
  }

  let { collapsed, onToggle }: Props = $props();
</script>

<nav class="sidebar" aria-label="Conversation sidebar" aria-hidden={collapsed}>
  <div class="sidebar-header">
    <button
      class="sidebar-icon-btn"
      onclick={onToggle}
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
    <button class="sidebar-icon-btn new-chat" aria-label="New chat" title="New chat">
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path d="M13 3L3 3M13 8L3 8M13 13L8 13" />
        <circle cx="11" cy="12" r="3" />
        <path d="M11 10.5v3M9.5 12h3" />
      </svg>
    </button>
  </div>

  <div class="sidebar-search">
    <svg
      class="search-icon"
      width="13"
      height="13"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <circle cx="7" cy="7" r="5" />
      <path d="M11 11l3.5 3.5" />
    </svg>
    <input type="search" placeholder="Search" aria-label="Search conversations" />
  </div>

  <div class="sidebar-content">
    <section class="sidebar-section">
      <h3 class="section-label">Today</h3>
      <p class="section-empty">No conversations yet</p>
    </section>
  </div>

  <div class="sidebar-footer">
    <button class="footer-btn" aria-label="Settings">
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
      <span>Settings</span>
    </button>
  </div>
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    animation: fadeIn 300ms ease both;
  }

  .sidebar-header {
    padding: var(--spacing-md);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sidebar-icon-btn {
    width: 30px;
    height: 30px;
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

  .sidebar-icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    transform: translateX(1px);
  }

  .sidebar-search {
    padding: 0 var(--spacing-md) var(--spacing-sm);
    flex-shrink: 0;
    position: relative;
  }

  .search-icon {
    position: absolute;
    left: 20px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--color-text-tertiary);
    pointer-events: none;
    margin-top: -4px;
  }

  .sidebar-search input {
    width: 100%;
    padding: 7px var(--spacing-sm) 7px 30px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    outline: none;
    transition:
      border-color var(--transition-fast),
      box-shadow var(--transition-fast);
  }

  .sidebar-search input:focus {
    border-color: var(--color-border-focus);
    box-shadow: var(--shadow-input-focus);
  }

  .sidebar-search input::placeholder {
    color: var(--color-text-tertiary);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--spacing-sm);
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

  .sidebar-footer {
    padding: var(--spacing-sm) var(--spacing-md);
    border-top: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
  }

  .footer-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    padding: 7px var(--spacing-sm);
    background: transparent;
    color: var(--color-text-secondary);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    transition: all var(--transition-fast);
  }

  .footer-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    transform: translateX(2px);
  }
</style>
