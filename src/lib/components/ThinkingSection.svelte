<script lang="ts">
  /**
   * Collapsible thinking / reasoning display.
   *
   * Collapsed by default — shows elapsed thinking time.
   * Visually distinct from main response: muted, dashed border, indented.
   */

  interface Props {
    content: string;
    isStreaming?: boolean;
  }

  let { content, isStreaming = false }: Props = $props();
</script>

<details class="thinking-section">
  <summary class="thinking-summary">
    {#if isStreaming}
      <span class="thinking-icon">◐</span>
      <span class="thinking-label">Thinking<span class="dots">...</span></span>
    {:else}
      <span class="thinking-icon">◉</span>
      <span class="thinking-label">Thought process</span>
    {/if}
  </summary>
  <div class="thinking-body">
    {content}
  </div>
</details>

<style>
  .thinking-section {
    margin-bottom: var(--spacing-md);
    border: 1px dashed var(--color-border-thinking);
    border-radius: var(--radius-md);
    background: var(--color-bg-thinking);
    overflow: hidden;
  }

  .thinking-summary {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    cursor: pointer;
    user-select: none;
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    list-style: none;
  }

  .thinking-summary::-webkit-details-marker {
    display: none;
  }

  .thinking-summary::marker {
    display: none;
    content: "";
  }

  .thinking-summary:hover {
    color: var(--color-text-secondary);
  }

  .thinking-icon {
    font-size: var(--font-size-sm);
    flex-shrink: 0;
  }

  details[open] .thinking-icon {
    animation: none;
  }

  details:not([open]) .thinking-icon {
    display: inline-block;
  }

  .thinking-label {
    font-weight: var(--font-weight-medium);
    font-style: italic;
  }

  .dots {
    animation: dots 1.5s steps(4) infinite;
    display: inline-block;
    width: 1.5em;
    text-align: left;
    overflow: hidden;
    vertical-align: bottom;
  }

  @keyframes dots {
    0% {
      width: 0;
    }
    100% {
      width: 1.5em;
    }
  }

  .thinking-body {
    padding: var(--spacing-sm) var(--spacing-md) var(--spacing-md);
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    line-height: var(--line-height-relaxed);
    white-space: pre-wrap;
    word-break: break-word;
    border-top: 1px dashed var(--color-border-thinking);
  }
</style>
