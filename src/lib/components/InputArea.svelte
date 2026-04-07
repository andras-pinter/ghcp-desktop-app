<script lang="ts">
  import type { Model } from "$lib/types/message";
  import type { Agent } from "$lib/types/agent";
  import type { UrlPreview } from "$lib/types/web-research";
  import type { ChatFileData } from "$lib/types/project";
  import { fetchUrl, pickFileForChat } from "$lib/utils/commands";
  import { formatBytes } from "$lib/utils/format";

  interface Props {
    onSend: (text: string, urls?: UrlPreview[], files?: ChatFileData[]) => void;
    onStop?: () => void;
    streaming?: boolean;
    extractingFiles?: boolean;
    model?: string;
    onModelChange?: (model: string) => void;
    availableModels?: Model[];
    modelsLoaded?: boolean;
    defaultModelId?: string | null;
    onSetDefault?: (modelId: string) => void;
    initialValue?: string;
    onInput?: (text: string) => void;
    agents?: Agent[];
    agentsLoaded?: boolean;
    selectedAgentId?: string | null;
    onAgentChange?: (agentId: string | null) => void;
    /** Files injected externally (e.g. dropped on ChatView). InputArea absorbs them. */
    externalFiles?: ChatFileData[];
    onExternalFilesConsumed?: () => void;
  }

  let {
    onSend,
    onStop,
    streaming = false,
    extractingFiles = false,
    model = "gpt-4o",
    onModelChange,
    availableModels = [],
    modelsLoaded = false,
    defaultModelId = null,
    onSetDefault,
    initialValue = "",
    onInput: onInputCallback,
    agents = [],
    agentsLoaded = false,
    selectedAgentId = null,
    onAgentChange,
    externalFiles = [],
    onExternalFilesConsumed,
  }: Props = $props();

  let inputText = $state("");
  let textareaEl: HTMLTextAreaElement | undefined = $state();
  let initialized = false;
  let dropdownOpen = $state(false);
  let dropdownEl: HTMLDivElement | undefined = $state();
  let agentDropdownOpen = $state(false);
  let agentDropdownEl: HTMLDivElement | undefined = $state();

  // URL input state
  let urlInputVisible = $state(false);
  let urlInputText = $state("");
  let urlInputEl: HTMLInputElement | undefined = $state();
  let attachedUrls: UrlPreview[] = $state([]);

  // File attachment state
  let attachedFiles: ChatFileData[] = $state([]);
  let fileDropActive = $state(false);
  let fileError = $state("");

  // Absorb files injected externally (e.g. dropped on the ChatView area)
  $effect(() => {
    if (externalFiles && externalFiles.length > 0) {
      for (const f of externalFiles) {
        if (!attachedFiles.some((a) => a.name === f.name)) {
          attachedFiles = [...attachedFiles, f];
        }
      }
      onExternalFilesConsumed?.();
    }
  });

  // Sync initialValue prop on first mount
  $effect(() => {
    if (!initialized && initialValue) {
      inputText = initialValue;
      initialized = true;
    }
  });

  // Auto-focus URL input when it appears
  $effect(() => {
    if (urlInputVisible && urlInputEl) {
      urlInputEl.focus();
    }
  });

  // Close dropdown on outside click
  function handleWindowClick(event: MouseEvent) {
    if (dropdownOpen && dropdownEl && !dropdownEl.contains(event.target as Node)) {
      dropdownOpen = false;
    }
    if (agentDropdownOpen && agentDropdownEl && !agentDropdownEl.contains(event.target as Node)) {
      agentDropdownOpen = false;
    }
  }

  function handleSend() {
    const trimmed = inputText.trim();
    if (!trimmed || streaming) return;
    const urls = attachedUrls.filter((u) => u.content !== null);
    onSend(
      trimmed,
      urls.length > 0 ? urls : undefined,
      attachedFiles.length > 0 ? attachedFiles : undefined,
    );
    inputText = "";
    attachedUrls = [];
    attachedFiles = [];
    urlInputVisible = false;
    urlInputText = "";
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

  /** Extract domain from a URL string. */
  function extractDomain(url: string): string {
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch {
      return url;
    }
  }

  /** Toggle the URL input bar. */
  function toggleUrlInput() {
    urlInputVisible = !urlInputVisible;
    if (!urlInputVisible) {
      urlInputText = "";
    }
  }

  /** Add a URL to the attached list and fetch its content. */
  async function addUrl(raw: string) {
    const trimmed = raw.trim();
    if (!trimmed) return;

    // Auto-upgrade http:// to https:// for safety; add https:// if no scheme
    let url = trimmed;
    if (/^http:\/\//i.test(url)) {
      url = url.replace(/^http:\/\//i, "https://");
    } else if (!/^https:\/\//i.test(url)) {
      url = "https://" + url;
    }

    // Don't add duplicates
    if (attachedUrls.some((u) => u.url === url)) {
      urlInputText = "";
      return;
    }

    const preview: UrlPreview = {
      url,
      domain: extractDomain(url),
      content: null,
      loading: true,
      error: null,
    };

    attachedUrls = [...attachedUrls, preview];
    urlInputText = "";

    // Fetch in background
    try {
      const content = await fetchUrl(url);
      attachedUrls = attachedUrls.map((u) =>
        u.url === url ? { ...u, content, loading: false } : u,
      );
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      attachedUrls = attachedUrls.map((u) =>
        u.url === url ? { ...u, loading: false, error: msg } : u,
      );
    }
  }

  function handleUrlKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      addUrl(urlInputText);
    } else if (event.key === "Escape") {
      urlInputVisible = false;
      urlInputText = "";
      textareaEl?.focus();
    }
  }

  function removeUrl(url: string) {
    attachedUrls = attachedUrls.filter((u) => u.url !== url);
  }

  /** Open file dialog (Rust-side) and attach the selected file. */
  async function handleAttachFile() {
    fileError = "";
    try {
      const data = await pickFileForChat();
      if (!data) return; // user cancelled
      if (!attachedFiles.some((f) => f.name === data.name)) {
        attachedFiles = [...attachedFiles, data];
      }
    } catch (e) {
      const msg = String(e);
      if (msg.includes("too large")) {
        fileError = "File is too large. Maximum size is 50 MB.";
      } else {
        fileError = msg;
      }
    }
  }

  /** Remove an attached file. */
  function removeFile(name: string) {
    attachedFiles = attachedFiles.filter((f) => f.name !== name);
  }

  /** Handle drag-and-drop onto the input area. */
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    fileDropActive = true;
  }

  function handleDragLeave() {
    fileDropActive = false;
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    fileDropActive = false;
    fileError = "";
    const files = event.dataTransfer?.files;
    if (!files || files.length === 0) return;

    const MAX_FILE_SIZE = 50 * 1024 * 1024;
    for (const file of files) {
      if (attachedFiles.some((f) => f.name === file.name)) continue;
      if (file.size > MAX_FILE_SIZE) {
        fileError = `"${file.name}" is too large (${formatBytes(file.size)}). Maximum is 50 MB.`;
        continue;
      }
      const arrayBuf = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuf);
      let binary = "";
      for (let i = 0; i < bytes.length; i++) {
        binary += String.fromCharCode(bytes[i]);
      }
      const base64 = btoa(binary);
      attachedFiles = [
        ...attachedFiles,
        {
          name: file.name,
          contentType: file.type || "application/octet-stream",
          size: file.size,
          contentBase64: base64,
        },
      ];
    }
  }

  /** Handle paste — auto-detect URLs and add them. */
  function handlePaste(event: ClipboardEvent) {
    const text = event.clipboardData?.getData("text/plain")?.trim();
    if (text && /^https?:\/\/\S+$/i.test(text)) {
      event.preventDefault();
      addUrl(text);
    }
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

<div
  class="input-area"
  class:file-drop-active={fileDropActive}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="region"
  aria-label="Message input area"
>
  <div class="input-box">
    {#if extractingFiles}
      <div class="file-processing" role="status">
        <span class="extract-spinner"></span>
        <span>Processing files…</span>
      </div>
    {/if}
    {#if fileError}
      <div class="file-error" role="alert">
        <span>⚠️ {fileError}</span>
        <button class="file-error-dismiss" onclick={() => (fileError = "")} aria-label="Dismiss"
          >✕</button
        >
      </div>
    {/if}
    {#if attachedFiles.length > 0}
      <div class="file-pills">
        {#each attachedFiles as file (file.name)}
          <div class="file-pill">
            <span class="file-pill-icon">📎</span>
            <span class="file-pill-name">{file.name}</span>
            <span class="file-pill-size">{formatBytes(file.size)}</span>
            <button
              class="pill-remove"
              onclick={() => removeFile(file.name)}
              aria-label="Remove file">✕</button
            >
          </div>
        {/each}
      </div>
    {/if}
    {#if attachedUrls.length > 0}
      <div class="url-pills">
        {#each attachedUrls as urlPreview (urlPreview.url)}
          <div class="url-pill" class:error={!!urlPreview.error} class:loading={urlPreview.loading}>
            <svg
              class="url-pill-icon"
              width="12"
              height="12"
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
            <span class="url-pill-domain">{urlPreview.domain}</span>
            {#if urlPreview.loading}
              <span class="url-pill-spinner"></span>
            {:else if urlPreview.error}
              <span class="url-pill-error" title={urlPreview.error}>✗</span>
            {:else}
              <span class="url-pill-ok">✓</span>
            {/if}
            <button
              class="url-pill-remove"
              onclick={() => removeUrl(urlPreview.url)}
              aria-label="Remove {urlPreview.domain}"
            >
              ✕
            </button>
          </div>
        {/each}
      </div>
    {/if}

    {#if urlInputVisible}
      <div class="url-input-row">
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="url-input-icon"
        >
          <circle cx="12" cy="12" r="10" />
          <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
          <path d="M2 12h20" />
        </svg>
        <input
          bind:this={urlInputEl}
          bind:value={urlInputText}
          onkeydown={handleUrlKeydown}
          type="url"
          placeholder="Paste a URL and press Enter..."
          class="url-input"
          aria-label="URL input"
        />
      </div>
    {/if}

    <textarea
      bind:this={textareaEl}
      bind:value={inputText}
      onkeydown={handleKeydown}
      oninput={handleInput}
      onpaste={handlePaste}
      placeholder="Message Copilot..."
      rows="1"
      aria-label="Message input"
    ></textarea>
    <div class="input-actions">
      <div class="actions-left">
        <button
          class="action-btn"
          aria-label="Attach file"
          title="Attach file"
          onclick={handleAttachFile}
        >
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

        <button
          class="action-btn"
          class:active-toggle={urlInputVisible}
          onclick={toggleUrlInput}
          aria-label="Add URL"
          title="Fetch web page content"
        >
          <svg
            width="16"
            height="16"
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
        </button>

        {#if agentsLoaded && agents.length > 0}
          <div class="agent-picker" bind:this={agentDropdownEl}>
            <button
              class="agent-trigger"
              onclick={() => (agentDropdownOpen = !agentDropdownOpen)}
              aria-label="Select agent"
              aria-expanded={agentDropdownOpen}
              aria-haspopup="listbox"
              title="Select agent persona"
            >
              <span class="agent-trigger-avatar"
                >{agents.find((a) => a.id === selectedAgentId)?.avatar ?? "🤖"}</span
              >
              <span class="agent-trigger-name"
                >{agents.find((a) => a.id === selectedAgentId)?.name ?? "Default"}</span
              >
              <svg
                class="agent-trigger-chevron"
                class:open={agentDropdownOpen}
                width="10"
                height="10"
                viewBox="0 0 16 16"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M4.5 6l3.5 4 3.5-4H4.5z" />
              </svg>
            </button>

            {#if agentDropdownOpen}
              <div
                class="agent-dropdown"
                role="listbox"
                tabindex="-1"
                aria-label="Available agents"
              >
                {#each agents as agent (agent.id)}
                  <button
                    class="agent-option"
                    class:selected={agent.id === selectedAgentId ||
                      (selectedAgentId === null && agent.isDefault)}
                    role="option"
                    aria-selected={agent.id === selectedAgentId ||
                      (selectedAgentId === null && agent.isDefault)}
                    onclick={() => {
                      onAgentChange?.(agent.isDefault ? null : agent.id);
                      agentDropdownOpen = false;
                    }}
                  >
                    <span class="agent-option-avatar">{agent.avatar ?? "🤖"}</span>
                    <span class="agent-option-name">{agent.name}</span>
                    {#if agent.isDefault}
                      <span class="agent-option-badge">default</span>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

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
      {:else if extractingFiles}
        <button class="send-btn extracting" disabled aria-label="Processing files">
          <span class="extract-spinner"></span>
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

  /* ── Web button active toggle ─────────────────────── */

  .active-toggle {
    color: var(--color-accent-copper) !important;
    background: var(--color-accent-subtle) !important;
  }

  /* ── URL input row ────────────────────────────────── */

  .url-input-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-lg);
    border-bottom: 1px solid var(--color-border-secondary);
  }

  .url-input-icon {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .url-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    outline: none;
    padding: var(--spacing-xs) 0;
  }

  .url-input::placeholder {
    color: var(--color-text-tertiary);
  }

  /* ── URL preview pills ────────────────────────────── */

  .url-pills {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-lg) 0;
  }

  .url-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px 2px 8px;
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-full);
    background: var(--color-bg-secondary);
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    max-width: 240px;
    transition:
      border-color var(--transition-fast),
      background var(--transition-fast);
  }

  .url-pill.error {
    border-color: var(--color-error);
    background: var(--color-error-subtle);
  }

  .url-pill-icon {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .url-pill-domain {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .url-pill-spinner {
    width: 8px;
    height: 8px;
    border: 1.5px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
    flex-shrink: 0;
  }

  .url-pill-ok {
    color: var(--color-success);
    font-size: 10px;
    flex-shrink: 0;
  }

  .url-pill-error {
    color: var(--color-error);
    font-size: 10px;
    flex-shrink: 0;
  }

  .url-pill-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--color-text-tertiary);
    font-size: 10px;
    cursor: pointer;
    flex-shrink: 0;
    transition: all var(--transition-fast);
    padding: 0;
    line-height: 1;
  }

  .url-pill-remove:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  /* ── Agent Picker ── */

  .agent-picker {
    position: relative;
  }

  .agent-trigger {
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

  .agent-trigger:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-border-primary);
    color: var(--color-text-secondary);
  }

  .agent-trigger-avatar {
    font-size: 12px;
    line-height: 1;
  }

  .agent-trigger-name {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .agent-trigger-chevron {
    flex-shrink: 0;
    opacity: 0.5;
    transition: transform var(--transition-fast);
  }

  .agent-trigger-chevron.open {
    transform: rotate(180deg);
  }

  .agent-dropdown {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 0;
    min-width: 180px;
    max-width: 240px;
    max-height: 280px;
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

  .agent-option {
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

  .agent-option:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .agent-option.selected {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    font-weight: 560;
  }

  .agent-option-avatar {
    font-size: 12px;
    line-height: 1;
    flex-shrink: 0;
  }

  .agent-option-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .agent-option-badge {
    font-size: 9px;
    padding: 1px 5px;
    border-radius: var(--radius-full);
    background: var(--color-bg-hover);
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  /* ── File drop zone ── */

  :global(.input-area).file-drop-active {
    outline: 2px dashed var(--color-accent-copper);
    outline-offset: -2px;
    background: color-mix(in srgb, var(--color-accent-copper) 5%, transparent);
  }

  /* ── File error ── */

  .file-error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    font-size: 0.8rem;
    color: var(--color-error, #b91c1c);
    background: var(--color-error-bg, #fef2f2);
    border-radius: 6px;
    margin: 8px 12px 0;
  }

  .file-error-dismiss {
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    font-size: 0.85rem;
    padding: 0 0 0 8px;
    opacity: 0.7;
  }

  .file-error-dismiss:hover {
    opacity: 1;
  }

  /* ── File processing indicator ── */

  .file-processing {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    font-size: 0.8rem;
    color: var(--color-accent-copper);
    margin: 8px 12px 0;
  }

  .extract-spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 600ms linear infinite;
    flex-shrink: 0;
  }

  .send-btn.extracting {
    opacity: 0.6;
  }

  /* ── File pills ── */

  .file-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 12px 0;
  }

  .file-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--color-text-primary);
    max-width: 200px;
  }

  .file-pill-icon {
    font-size: 12px;
    flex-shrink: 0;
  }

  .file-pill-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-pill-size {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .pill-remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border: none;
    background: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    font-size: 10px;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    transition: all var(--transition-fast);
    padding: 0;
  }

  .pill-remove:hover {
    background: var(--color-bg-hover);
    color: var(--color-error);
  }
</style>
