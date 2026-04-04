<script lang="ts">
  interface Props {
    collapsed: boolean;
  }

  let { collapsed }: Props = $props();
</script>

<nav class="sidebar" class:collapsed aria-label="Conversation sidebar">
  <!-- Fixed-width inner wrapper prevents reflow during width transition -->
  <div class="sidebar-inner">
    <!-- Actions -->
    <div class="sidebar-actions">
      <button class="nav-btn" aria-label="New chat" title="New chat">
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
      <section class="sidebar-section">
        <h3 class="section-label">Today</h3>
        <p class="section-empty">No conversations yet</p>
      </section>
    </div>

    <!-- Footer -->
    <div class="sidebar-footer">
      <button class="nav-btn" aria-label="Settings" title="Settings">
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

  /* ── Footer ── */

  .sidebar-footer {
    padding: var(--spacing-sm);
    border-top: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
  }
</style>
