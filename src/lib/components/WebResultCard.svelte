<script lang="ts">
  import type { SearchResult } from "$lib/types/web-research";

  interface Props {
    result: SearchResult;
  }

  let { result }: Props = $props();

  /** Extract domain from URL for display. */
  function domain(url: string): string {
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch {
      return url;
    }
  }

  /** Only allow HTTPS URLs as href targets. */
  function safeHref(url: string): string {
    try {
      const parsed = new URL(url);
      return parsed.protocol === "https:" ? url : "#";
    } catch {
      return "#";
    }
  }
</script>

<a
  class="web-result-card"
  href={safeHref(result.url)}
  target="_blank"
  rel="noopener noreferrer"
  aria-label="Web result: {result.title}"
>
  <div class="card-header">
    <svg
      class="globe-icon"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="10" />
      <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
      <path d="M2 12h20" />
    </svg>
    <span class="card-domain">{domain(result.url)}</span>
  </div>
  <div class="card-title">{result.title}</div>
  {#if result.snippet}
    <div class="card-snippet">{result.snippet}</div>
  {/if}
</a>

<style>
  .web-result-card {
    display: block;
    padding: var(--spacing-sm) var(--spacing-md);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    background: var(--color-bg-primary);
    text-decoration: none;
    color: inherit;
    transition:
      border-color var(--transition-fast),
      background var(--transition-fast);
  }

  .web-result-card:hover {
    border-color: var(--color-accent-copper);
    background: var(--color-bg-hover);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-bottom: 2px;
  }

  .globe-icon {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .card-domain {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-title {
    font-size: var(--font-size-sm);
    font-weight: 560;
    color: var(--color-text-link);
    line-height: var(--line-height-normal);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .web-result-card:hover .card-title {
    text-decoration: underline;
  }

  .card-snippet {
    margin-top: 2px;
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    line-height: var(--line-height-relaxed);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
