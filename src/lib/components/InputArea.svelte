<script lang="ts">
  import type { Model } from "$lib/types/message";

  interface Props {
    onSend: (text: string) => void;
    onStop?: () => void;
    streaming?: boolean;
    model?: string;
    onModelChange?: (model: string) => void;
    availableModels?: Model[];
    modelsLoaded?: boolean;
    defaultModelId?: string | null;
    onSetDefault?: (modelId: string) => void;
    initialValue?: string;
    onInput?: (text: string) => void;
  }

  let {
    onSend,
    onStop,
    streaming = false,
    model = "gpt-4o",
    onModelChange,
    availableModels = [],
    modelsLoaded = false,
    defaultModelId = null,
    onSetDefault,
    initialValue = "",
    onInput: onInputCallback,
  }: Props = $props();

  let inputText = $state("");
  let textareaEl: HTMLTextAreaElement | undefined = $state();
  let initialized = false;
  let dropdownOpen = $state(false);
  let dropdownEl: HTMLDivElement | undefined = $state();

  // Sync initialValue prop on first mount
  $effect(() => {
    if (!initialized && initialValue) {
      inputText = initialValue;
      initialized = true;
    }
  });

  // Close dropdown on outside click
  function handleWindowClick(event: MouseEvent) {
    if (dropdownOpen && dropdownEl && !dropdownEl.contains(event.target as Node)) {
      dropdownOpen = false;
    }
  }

  function handleSend() {
    const trimmed = inputText.trim();
    if (!trimmed || streaming) return;
    onSend(trimmed);
    inputText = "";
    if (textareaEl) {
      textareaEl.style.height = "auto";
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  }

  function handleInput() {
    if (!textareaEl) return;
    textareaEl.style.height = "auto";
    textareaEl.style.height = Math.min(textareaEl.scrollHeight, 200) + "px";
    onInputCallback?.(inputText);
  }

  function handleModelClick(modelId: string, event: MouseEvent) {
    if (event.shiftKey && onSetDefault) {
      onSetDefault(modelId);
    }
    onModelChange?.(modelId);
    dropdownOpen = false;
  }

  function handleDropdownKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      dropdownOpen = false;
    }
  }

  /** Display-friendly model name — strips date suffixes and normalizes. */
  function displayName(m: Model): string {
    return m.name ?? m.id;
  }

  $effect(() => {
    if (dropdownOpen) {
      document.addEventListener("click", handleWindowClick);
    } else {
      document.removeEventListener("click", handleWindowClick);
    }
    return () => document.removeEventListener("click", handleWindowClick);
  });
</script>

<div class="input-area">
  <div class="input-box">
    <textarea
      bind:this={textareaEl}
      bind:value={inputText}
      onkeydown={handleKeydown}
      oninput={handleInput}
      placeholder="Message Copilot..."
      rows="1"
      aria-label="Message input"
    ></textarea>
    <div class="input-actions">
      <div class="actions-left">
        <button class="action-btn" aria-label="Attach file" title="Attach file">
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path
              d="M14 8.5l-5.5 5.5a3.5 3.5 0 0 1-5-5L9 3.5a2.5 2.5 0 0 1 3.5 3.5L7 12.5a1.5 1.5 0 0 1-2-2L10.5 5"
            />
          </svg>
        </button>

        {#if modelsLoaded && availableModels.length > 1}
          <div class="model-picker" bind:this={dropdownEl}>
            <button
              class="model-trigger"
              onclick={() => (dropdownOpen = !dropdownOpen)}
              aria-label="Select model"
              aria-expanded={dropdownOpen}
              aria-haspopup="listbox"
              title="Click to switch model · Shift+click to set default"
            >
              {#if model === defaultModelId}
                <svg
                  class="model-star"
                  width="10"
                  height="10"
                  viewBox="0 0 16 16"
                  fill="currentColor"
                  aria-label="Default model"
                >
                  <path
                    d="M8 1.5l1.85 4.1L14.2 6l-3.3 3 .85 4.5L8 11.3 4.25 13.5l.85-4.5L1.8 6l4.35-.4z"
                  />
                </svg>
              {/if}
              <span class="model-trigger-name"
                >{displayName(availableModels.find((m) => m.id === model) ?? { id: model })}</span
              >
              <svg
                class="model-trigger-chevron"
                class:open={dropdownOpen}
                width="10"
                height="10"
                viewBox="0 0 16 16"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M4.5 6l3.5 4 3.5-4H4.5z" />
              </svg>
            </button>

            {#if dropdownOpen}
              <div
                class="model-dropdown"
                role="listbox"
                tabindex="-1"
                aria-label="Available models"
                onkeydown={handleDropdownKeydown}
              >
                <div class="model-dropdown-hint">
                  <span>⇧ click to set default</span>
                </div>
                {#each availableModels as m (m.id)}
                  <button
                    class="model-option"
                    class:selected={m.id === model}
                    class:is-default={m.id === defaultModelId}
                    role="option"
                    aria-selected={m.id === model}
                    onclick={(e) => handleModelClick(m.id, e)}
                  >
                    <span class="model-option-name">{displayName(m)}</span>
                    {#if m.id === defaultModelId}
                      <svg
                        class="model-option-star"
                        width="10"
                        height="10"
                        viewBox="0 0 16 16"
                        fill="currentColor"
                        aria-label="Default"
                      >
                        <path
                          d="M8 1.5l1.85 4.1L14.2 6l-3.3 3 .85 4.5L8 11.3 4.25 13.5l.85-4.5L1.8 6l4.35-.4z"
                        />
                      </svg>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {:else if modelsLoaded}
          <span class="model-label">{displayName(availableModels[0] ?? { id: model })}</span>
        {:else}
          <span class="model-label model-loading">
            <span class="model-spinner"></span>
          </span>
        {/if}
      </div>
      {#if streaming}
        <button class="stop-btn" onclick={() => onStop?.()} aria-label="Stop streaming">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <rect x="3" y="3" width="10" height="10" rx="1.5" />
          </svg>
        </button>
      {:else}
        <button
          class="send-btn"
          class:active={!!inputText.trim()}
          onclick={handleSend}
          disabled={!inputText.trim()}
          aria-label="Send message"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 2.5l-4.5 4.5h3V13h3V7h3L8 2.5z" />
          </svg>
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .input-area {
    width: 100%;
  }

  .input-box {
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-lg);
    background: var(--color-bg-input);
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-input);
    transition:
      border-color var(--transition-fast),
      box-shadow var(--transition-normal);
  }

  .input-box:focus-within {
    border-color: var(--color-border-focus);
    box-shadow: var(--shadow-input-focus);
  }

  textarea {
    width: 100%;
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xs);
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    font-family: var(--font-sans);
    font-size: var(--font-size-base);
    line-height: var(--line-height-normal);
    resize: none;
    outline: none;
    min-height: var(--input-min-height);
  }

  textarea::placeholder {
    color: var(--color-text-tertiary);
  }

  .input-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-xs) var(--spacing-sm) var(--spacing-sm);
  }

  .actions-left {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .action-btn {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .action-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  /* ── Model picker ────────────────────────────────── */

  .model-picker {
    position: relative;
  }

  .model-trigger {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-full);
    color: var(--color-text-tertiary);
    font-family: var(--font-sans);
    font-size: var(--font-size-xs);
    cursor: pointer;
    transition: all var(--transition-fast);
    letter-spacing: var(--letter-spacing-normal);
    white-space: nowrap;
  }

  .model-trigger:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-border-primary);
    color: var(--color-text-secondary);
  }

  .model-star {
    color: var(--color-accent-copper);
    flex-shrink: 0;
  }

  .model-trigger-name {
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .model-trigger-chevron {
    flex-shrink: 0;
    opacity: 0.5;
    transition: transform var(--transition-fast);
  }

  .model-trigger-chevron.open {
    transform: rotate(180deg);
  }

  .model-dropdown {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 0;
    min-width: 220px;
    max-width: 280px;
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
    animation: dropdownFadeIn 120ms ease;
  }

  @keyframes dropdownFadeIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .model-dropdown-hint {
    padding: 4px 8px 6px;
    font-family: var(--font-sans);
    font-size: 10px;
    color: var(--color-text-tertiary);
    opacity: 0.6;
    user-select: none;
    border-bottom: 1px solid var(--color-border-secondary);
    margin-bottom: var(--spacing-xs);
  }

  .model-option {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    font-family: var(--font-sans);
    font-size: var(--font-size-xs);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
    letter-spacing: var(--letter-spacing-normal);
    white-space: nowrap;
    overflow: hidden;
  }

  .model-option:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .model-option.selected {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    font-weight: 560;
  }

  .model-option-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .model-option-star {
    color: var(--color-accent-copper);
    flex-shrink: 0;
  }

  .model-label {
    color: var(--color-text-tertiary);
    font-family: var(--font-sans);
    font-size: var(--font-size-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    letter-spacing: var(--letter-spacing-normal);
  }

  .model-loading {
    display: flex;
    align-items: center;
  }

  .model-spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--color-border-primary);
    border-top-color: var(--color-text-tertiary);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* ── Send / Stop buttons ────────────────────────── */

  .send-btn {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-tertiary);
    border: none;
    border-radius: var(--radius-full);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: all var(--transition-normal);
  }

  .send-btn.active {
    background: var(--color-accent);
    color: var(--color-text-inverse);
  }

  .send-btn:disabled {
    cursor: default;
  }

  .send-btn:hover:not(:disabled) {
    transform: scale(1.05);
  }

  .stop-btn {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-full);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .stop-btn:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-accent-copper);
    color: var(--color-accent-copper);
  }
</style>
