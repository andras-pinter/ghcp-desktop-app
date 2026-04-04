<script lang="ts">
  interface Props {
    onSend: (text: string) => void;
  }

  let { onSend }: Props = $props();

  let inputText = $state("");
  let textareaEl: HTMLTextAreaElement | undefined = $state();

  function handleSend() {
    const trimmed = inputText.trim();
    if (!trimmed) return;
    onSend(trimmed);
    inputText = "";
    // Reset textarea height
    if (textareaEl) {
      textareaEl.style.height = "auto";
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Enter to send (default), Shift+Enter for newline
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
</script>

<div class="input-area">
  <div class="input-wrapper">
    <textarea
      bind:this={textareaEl}
      bind:value={inputText}
      onkeydown={handleKeydown}
      oninput={handleInput}
      placeholder="Message Copilot..."
      rows="1"
      aria-label="Message input"
    ></textarea>
    <div class="input-toolbar">
      <div class="input-actions-left">
        <button class="toolbar-btn" aria-label="Attach file" title="Attach file">📎</button>
        <button class="toolbar-btn" aria-label="Web search" title="Web search">🌐</button>
      </div>
      <div class="input-actions-right">
        <button
          class="send-btn"
          onclick={handleSend}
          disabled={!inputText.trim()}
          aria-label="Send message"
        >
          Send ➤
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .input-area {
    padding: var(--spacing-md) var(--spacing-xl);
    border-top: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
  }

  .input-wrapper {
    max-width: 800px;
    margin: 0 auto;
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    background: var(--color-bg-input);
    overflow: hidden;
    transition: border-color var(--transition-fast);
  }

  .input-wrapper:focus-within {
    border-color: var(--color-border-focus);
  }

  textarea {
    width: 100%;
    padding: var(--spacing-md);
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

  .input-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-top: 1px solid var(--color-border-secondary);
  }

  .input-actions-left,
  .input-actions-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .toolbar-btn {
    padding: var(--spacing-xs) var(--spacing-sm);
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .toolbar-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .send-btn {
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-accent);
    color: var(--color-text-inverse);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .send-btn:hover:not(:disabled) {
    background: var(--color-accent-hover);
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
