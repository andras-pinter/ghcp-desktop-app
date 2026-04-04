<script lang="ts">
  import type { Model } from "$lib/types/message";
  import { getModels } from "$lib/utils/commands";
  import { onMount } from "svelte";

  interface Props {
    onSend: (text: string) => void;
    onStop?: () => void;
    streaming?: boolean;
    model?: string;
    onModelChange?: (model: string) => void;
  }

  let { onSend, onStop, streaming = false, model = "gpt-4o", onModelChange }: Props = $props();

  let inputText = $state("");
  let textareaEl: HTMLTextAreaElement | undefined = $state();
  let availableModels = $state<Model[]>([{ id: "gpt-4o", name: "GPT-4o" }]);

  onMount(async () => {
    try {
      const models = await getModels();
      if (models.length > 0) {
        availableModels = models;
        // If current model isn't in the list, switch to first available
        if (!models.some((m) => m.id === model)) {
          onModelChange?.(models[0].id);
        }
      }
    } catch {
      // Keep fallback
    }
  });

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
  }

  function handleModelSelect(event: Event) {
    const target = event.target as HTMLSelectElement;
    onModelChange?.(target.value);
  }
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
        <div class="model-selector">
          <select value={model} onchange={handleModelSelect} aria-label="Select model">
            {#each availableModels as m (m.id)}
              <option value={m.id}>{m.name ?? m.id}</option>
            {/each}
          </select>
        </div>
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

  .model-selector select {
    appearance: none;
    background: transparent;
    border: none;
    color: var(--color-text-tertiary);
    font-family: var(--font-sans);
    font-size: var(--font-size-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
    outline: none;
    letter-spacing: var(--letter-spacing-normal);
  }

  .model-selector select:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

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
