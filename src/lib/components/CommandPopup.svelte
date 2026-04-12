<script lang="ts">
  /**
   * Floating autocomplete popup for slash commands and @-mentions.
   *
   * Renders a filtered list of items (commands, agents, models, skills) above the
   * input area. Keyboard navigation (↑↓, Tab/Enter, Escape) is handled by the
   * parent InputArea — this component only renders and reports clicks.
   */
  import type { PopupItem } from "$lib/types/commands";
  import { strings } from "$lib/strings/en";

  interface Props {
    /** Filtered items to display. */
    items: PopupItem[];
    /** Index of the currently highlighted item (-1 = none). */
    focusedIndex: number;
    /** Called when the user clicks an item. */
    onSelect: (item: PopupItem) => void;
  }

  const { items, focusedIndex, onSelect }: Props = $props();

  /** Scroll the focused item into view when arrow keys change selection. */
  $effect(() => {
    if (focusedIndex < 0) return;
    // Use tick-like delay to let Svelte render the .focused class first
    requestAnimationFrame(() => {
      const el = document.querySelector(".command-popup .popup-item.focused");
      el?.scrollIntoView({ block: "nearest" });
    });
  });

  /** Unique key for Svelte {#each} — must never collide across items. */
  function itemKey(item: PopupItem): string {
    switch (item.kind) {
      case "command":
        return `cmd:${item.command.name}`;
      case "agent":
        return `agent:${item.agent.id}`;
      case "model":
        return `model:${item.model.id}`;
      case "skill":
        return `skill:${item.skill.id}`;
    }
  }

  function itemLabel(item: PopupItem): string {
    switch (item.kind) {
      case "command":
        return `/${item.command.name}`;
      case "agent":
        return item.agent.name;
      case "model":
        return item.model.name ?? item.model.id;
      case "skill":
        return item.skill.name;
    }
  }

  function itemDescription(item: PopupItem): string {
    switch (item.kind) {
      case "command":
        return item.command.description;
      case "agent":
        return item.agent.systemPrompt?.slice(0, 60) ?? "";
      case "model":
        return item.model.id;
      case "skill":
        return item.skill.description?.slice(0, 60) ?? "";
    }
  }

  function itemIcon(item: PopupItem): string {
    switch (item.kind) {
      case "command":
        return item.command.icon;
      case "agent":
        return item.agent.avatar ?? "🤖";
      case "model":
        return "📊";
      case "skill":
        return "⚡";
    }
  }

  function sectionHeading(item: PopupItem): string | null {
    switch (item.kind) {
      case "command":
        return strings.commands.heading;
      case "agent":
        return strings.commands.agentsHeading;
      case "model":
        return strings.commands.modelsHeading;
      case "skill":
        return strings.commands.skillsHeading;
    }
  }

  /** Whether this item starts a new section (different kind from previous). */
  function isNewSection(index: number): boolean {
    if (index === 0) return true;
    return items[index].kind !== items[index - 1].kind;
  }
</script>

{#if items.length > 0}
  <div class="command-popup" role="listbox" aria-label="Command suggestions">
    {#each items as item, i (itemKey(item))}
      {#if isNewSection(i)}
        <div class="popup-section-heading" aria-hidden="true">
          {sectionHeading(item)}
        </div>
      {/if}
      <button
        class="popup-item"
        class:focused={i === focusedIndex}
        role="option"
        aria-selected={i === focusedIndex}
        onmousedown={(e) => {
          e.preventDefault();
          onSelect(item);
        }}
      >
        <span class="popup-item-icon" aria-hidden="true">{itemIcon(item)}</span>
        <span class="popup-item-content">
          <span class="popup-item-label">{itemLabel(item)}</span>
          {#if itemDescription(item)}
            <span class="popup-item-desc">{itemDescription(item)}</span>
          {/if}
        </span>
      </button>
    {/each}
  </div>
{:else}
  <div class="command-popup command-popup-empty" role="status">
    <span class="popup-empty-text">{strings.commands.noMatches}</span>
  </div>
{/if}

<style>
  .command-popup {
    position: absolute;
    bottom: calc(100% + 6px);
    left: var(--spacing-sm);
    right: var(--spacing-sm);
    max-height: 320px;
    overflow-y: auto;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    box-shadow:
      0 8px 24px rgba(0, 0, 0, 0.12),
      0 2px 8px rgba(0, 0, 0, 0.06);
    z-index: 100;
    padding: var(--spacing-xs);
    animation: popupFadeIn 120ms ease;
  }

  @keyframes popupFadeIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .command-popup-empty {
    padding: var(--spacing-md);
    text-align: center;
  }

  .popup-empty-text {
    color: var(--color-text-tertiary);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
  }

  .popup-section-heading {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: 11px;
    font-weight: 600;
    font-family: var(--font-sans);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    user-select: none;
  }

  .popup-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-sm);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-primary);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    cursor: pointer;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .popup-item:hover,
  .popup-item.focused {
    background: var(--color-bg-secondary);
  }

  .popup-item-icon {
    flex-shrink: 0;
    font-size: 14px;
    width: 20px;
    text-align: center;
  }

  .popup-item-content {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .popup-item-label {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .popup-item-desc {
    font-size: 11px;
    color: var(--color-text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
