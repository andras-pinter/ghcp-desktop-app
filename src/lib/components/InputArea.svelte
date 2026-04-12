<script lang="ts">
  import type { Model } from "$lib/types/message";
  import type { Agent } from "$lib/types/agent";
  import type { Skill } from "$lib/types/skill";
  import type { UrlPreview } from "$lib/types/web-research";
  import type { ChatFileData } from "$lib/types/project";
  import type { PopupItem, MessageOverrides, SlashCommand } from "$lib/types/commands";
  import { emptyOverrides, SLASH_COMMANDS } from "$lib/types/commands";
  import { parseCommand, type CommandParseResult } from "$lib/utils/command-parser";
  import CommandPopup from "./CommandPopup.svelte";
  import { fetchUrl, pickFileForChat } from "$lib/utils/commands";
  import { formatBytes } from "$lib/utils/format";
  import { getSettings } from "$lib/stores/settings.svelte";
  import { getNetwork } from "$lib/stores/network.svelte";
  import { tick } from "svelte";

  interface Props {
    onSend: (
      text: string,
      urls?: UrlPreview[],
      files?: ChatFileData[],
      overrides?: MessageOverrides,
    ) => void;
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
    defaultAgentId?: string | null;
    onAgentChange?: (agentId: string | null) => void;
    /** Available skills (for /skill autocomplete). */
    skills?: Skill[];
    /** Files injected externally (e.g. dropped on ChatView). InputArea absorbs them. */
    externalFiles?: ChatFileData[];
    onExternalFilesConsumed?: () => void;
    /** Background extraction status per filename, managed by ChatView. */
    extractionStatuses?: Record<string, "extracting" | "done" | "error">;
    /** Callback for action commands that the input area cannot handle itself. */
    onCommand?: (command: string, args?: string) => void;
    /** Whether an active conversation exists (hides conversation-only commands). */
    hasConversation?: boolean;
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
    defaultAgentId = null,
    onAgentChange,
    skills = [],
    externalFiles = [],
    onExternalFilesConsumed,
    extractionStatuses = {},
    onCommand,
    hasConversation = false,
  }: Props = $props();

  const settings = getSettings();
  const network = getNetwork();

  let inputText = $state("");
  let textareaEl: HTMLTextAreaElement | undefined = $state();
  let initialized = false;
  let dropdownOpen = $state(false);
  let dropdownEl: HTMLDivElement | undefined = $state();
  let agentDropdownOpen = $state(false);
  let agentDropdownEl: HTMLDivElement | undefined = $state();

  // ── Command popup state ──────────────────────────────────────────────
  let commandResult = $state<CommandParseResult>(null);
  let popupFocusIndex = $state(-1);
  let overrides: MessageOverrides = $state(emptyOverrides());

  /** Items to show in the popup — derived from the parse result. */
  let popupItems = $derived<PopupItem[]>(commandResult ? commandResult.items : []);
  /** Whether the popup should be visible. */
  let popupVisible = $derived(commandResult !== null && popupItems.length > 0);

  /** Local override for the send shortcut — does not persist to settings. */
  let localSendShortcut: "enter" | "cmd-enter" | null = $state(null);
  let activeSendShortcut = $derived(localSendShortcut ?? settings.sendShortcut ?? "enter");
  const isMac = navigator.userAgent.includes("Mac");
  const modLabel = $derived(isMac ? "⌘" : "Ctrl");

  // URL input state
  let urlInputVisible = $state(false);
  let urlInputText = $state("");
  let urlInputEl: HTMLInputElement | undefined = $state();
  let attachedUrls: UrlPreview[] = $state([]);

  // File attachment state
  let attachedFiles: ChatFileData[] = $state([]);
  let fileDropActive = $state(false);
  let fileError = $state("");

  // Absorb files injected externally (e.g. dropped on the ChatView area).
  // Replaces loading placeholders with real data when content arrives.
  $effect(() => {
    if (externalFiles && externalFiles.length > 0) {
      let updated = [...attachedFiles];
      for (const f of externalFiles) {
        const idx = updated.findIndex((a) => a.name === f.name);
        if (idx >= 0) {
          // Replace loading placeholder with real file data
          const existing = updated[idx];
          if (existing.loading && !f.loading) {
            updated[idx] = f;
          }
        } else {
          updated = [...updated, f];
        }
      }
      attachedFiles = updated;
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
    // Don't send while files are still being read from disk
    if (attachedFiles.some((f) => f.loading)) return;

    // Intercept slash commands that would otherwise be sent as text
    const cmdMatch = trimmed.match(/^\/(\S+)(?:\s+(.*))?$/);
    if (cmdMatch) {
      // Resolve aliases
      const rawName = cmdMatch[1] === "web" ? "fetch" : cmdMatch[1] === "?" ? "help" : cmdMatch[1];
      const cmdArg = cmdMatch[2]?.trim() || undefined;
      const cmd = SLASH_COMMANDS.find((c) => c.name === rawName);
      if (cmd) {
        inputText = "";
        commandResult = null;
        if (textareaEl) textareaEl.style.height = "auto";
        onCommand?.(cmd.name, cmdArg);
        return;
      }
    }

    const urls = attachedUrls.filter((u) => u.content !== null);
    const hasOverrides =
      overrides.agentId !== null || overrides.modelId !== null || overrides.skillIds.length > 0;
    onSend(
      trimmed,
      urls.length > 0 ? urls : undefined,
      attachedFiles.length > 0 ? attachedFiles : undefined,
      hasOverrides ? { ...overrides } : undefined,
    );
    inputText = "";
    attachedUrls = [];
    attachedFiles = [];
    urlInputVisible = false;
    urlInputText = "";
    overrides = emptyOverrides();
    commandResult = null;
    if (textareaEl) {
      textareaEl.style.height = "auto";
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // ── Command popup keyboard navigation ─────────────────────────────
    if (popupVisible) {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        popupFocusIndex = Math.min(popupFocusIndex + 1, popupItems.length - 1);
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        popupFocusIndex = Math.max(popupFocusIndex - 1, -1);
        return;
      }
      if (event.key === "Tab" || (event.key === "Enter" && !event.shiftKey)) {
        if (popupFocusIndex >= 0 && popupFocusIndex < popupItems.length) {
          event.preventDefault();
          selectPopupItem(popupItems[popupFocusIndex]);
          return;
        }
        // If no focused item, Tab does nothing; Enter falls through to normal send
        if (event.key === "Tab") {
          event.preventDefault();
          return;
        }
      }
      if (event.key === "Escape") {
        event.preventDefault();
        commandResult = null;
        popupFocusIndex = -1;
        return;
      }
    }

    // ── Normal keydown handling ────────────────────────────────────────
    const mod = event.metaKey || event.ctrlKey;
    if (activeSendShortcut === "cmd-enter") {
      if (event.key === "Enter" && mod) {
        event.preventDefault();
        handleSend();
        return;
      }
      if (event.key === "Enter" && !mod && handleListContinuation(event)) return;
    } else {
      if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        handleSend();
        return;
      }
      if (event.key === "Enter" && event.shiftKey && handleListContinuation(event)) return;
    }
  }

  /**
   * Auto-continue numbered and bullet lists when pressing the newline key.
   * Returns true if the event was handled (caller should return early).
   *
   * Behaviour:
   * - `1. text` + Enter → inserts `\n2. `
   * - `- text` or `* text` + Enter → inserts `\n- ` / `\n* `
   * - Empty list item (e.g. `3. ` with nothing after) + Enter → removes the marker
   */
  function handleListContinuation(event: KeyboardEvent): boolean {
    if (!textareaEl) return false;

    // Skip if text is selected — let the browser handle the replacement
    if (textareaEl.selectionStart !== textareaEl.selectionEnd) return false;

    const pos = textareaEl.selectionStart;
    const text = inputText;

    // Only trigger at end of line, not in the middle
    const lineEnd = text.indexOf("\n", pos);
    const actualLineEnd = lineEnd === -1 ? text.length : lineEnd;
    if (pos !== actualLineEnd) return false;

    // Find the start of the current line
    const lineStart = text.lastIndexOf("\n", pos - 1) + 1;
    const lineText = text.slice(lineStart, pos);

    // Try numbered list: `  1. content`
    const numMatch = lineText.match(/^(\s*)(\d+)\.\s(.*)$/);
    if (numMatch) {
      const [, indent, numStr, content] = numMatch;
      if (content.length === 0) {
        // Empty list item → remove the marker
        event.preventDefault();
        inputText = text.slice(0, lineStart) + text.slice(pos);
        tick().then(() => {
          if (textareaEl) {
            textareaEl.selectionStart = textareaEl.selectionEnd = lineStart;
            handleInput();
          }
        });
        return true;
      }
      // Continue with next number
      const next = parseInt(numStr, 10) + 1;
      const insertion = `\n${indent}${next}. `;
      event.preventDefault();
      inputText = text.slice(0, pos) + insertion + text.slice(pos);
      const newPos = pos + insertion.length;
      tick().then(() => {
        if (textareaEl) {
          textareaEl.selectionStart = textareaEl.selectionEnd = newPos;
          handleInput();
        }
      });
      return true;
    }

    // Try bullet list: `  - content` or `  * content`
    const bulletMatch = lineText.match(/^(\s*)([-*])\s(.*)$/);
    if (bulletMatch) {
      const [, indent, bullet, content] = bulletMatch;
      if (content.length === 0) {
        // Empty bullet → remove the marker
        event.preventDefault();
        inputText = text.slice(0, lineStart) + text.slice(pos);
        tick().then(() => {
          if (textareaEl) {
            textareaEl.selectionStart = textareaEl.selectionEnd = lineStart;
            handleInput();
          }
        });
        return true;
      }
      const insertion = `\n${indent}${bullet} `;
      event.preventDefault();
      inputText = text.slice(0, pos) + insertion + text.slice(pos);
      const newPos = pos + insertion.length;
      tick().then(() => {
        if (textareaEl) {
          textareaEl.selectionStart = textareaEl.selectionEnd = newPos;
          handleInput();
        }
      });
      return true;
    }

    return false;
  }

  function handleInput() {
    if (!textareaEl) return;
    textareaEl.style.height = "auto";
    textareaEl.style.height = Math.min(textareaEl.scrollHeight, 200) + "px";
    onInputCallback?.(inputText);

    // Run the command parser on every input to drive the popup
    const cursor = textareaEl.selectionStart ?? inputText.length;
    commandResult = parseCommand(
      inputText,
      cursor,
      agents,
      availableModels,
      skills,
      hasConversation,
    );
    popupFocusIndex = commandResult && commandResult.items.length > 0 ? 0 : -1;
  }

  function handleModelClick(modelId: string, event: MouseEvent) {
    if (event.shiftKey && onSetDefault) {
      onSetDefault(modelId);
    }
    onModelChange?.(modelId);
    dropdownOpen = false;
  }

  let focusedModelIndex = $state(-1);

  function handleDropdownKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      dropdownOpen = false;
      return;
    }
    const opts = availableModels;
    if (!opts.length) return;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      focusedModelIndex = Math.min(focusedModelIndex + 1, opts.length - 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      focusedModelIndex = Math.max(focusedModelIndex - 1, 0);
    } else if (event.key === "Home") {
      event.preventDefault();
      focusedModelIndex = 0;
    } else if (event.key === "End") {
      event.preventDefault();
      focusedModelIndex = opts.length - 1;
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (focusedModelIndex >= 0 && focusedModelIndex < opts.length) {
        onModelChange?.(opts[focusedModelIndex].id);
        dropdownOpen = false;
      }
    }
  }

  let focusedAgentIndex = $state(-1);

  function handleAgentDropdownKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      agentDropdownOpen = false;
      return;
    }
    if (!agents.length) return;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      focusedAgentIndex = Math.min(focusedAgentIndex + 1, agents.length - 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      focusedAgentIndex = Math.max(focusedAgentIndex - 1, 0);
    } else if (event.key === "Home") {
      event.preventDefault();
      focusedAgentIndex = 0;
    } else if (event.key === "End") {
      event.preventDefault();
      focusedAgentIndex = agents.length - 1;
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (focusedAgentIndex >= 0 && focusedAgentIndex < agents.length) {
        const agent = agents[focusedAgentIndex];
        onAgentChange?.(agent.id);
        agentDropdownOpen = false;
      }
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

  // ── Command popup selection & dispatch ────────────────────────────────

  /** Replace the trigger text in the textarea and re-position cursor. */
  function replaceCommandText(rangeStart: number, rangeEnd: number, replacement: string) {
    inputText = inputText.slice(0, rangeStart) + replacement + inputText.slice(rangeEnd);
    tick().then(() => {
      if (textareaEl) {
        const newPos = rangeStart + replacement.length;
        textareaEl.selectionStart = textareaEl.selectionEnd = newPos;
        handleInput();
      }
    });
  }

  /**
   * Enter sub-command mode: replace trigger text with `/{cmd} ` and
   * immediately compute the sub-command popup items.  This avoids relying
   * on the async tick → handleInput roundtrip which can race with
   * Svelte's reactive update cycle.
   */
  function enterSubCommandMode(cmd: SlashCommand, rangeStart: number, rangeEnd: number) {
    const replacement = `/${cmd.name} `;
    inputText = inputText.slice(0, rangeStart) + replacement + inputText.slice(rangeEnd);
    const newPos = rangeStart + replacement.length;

    // Compute sub-command result synchronously so the popup stays open
    commandResult = parseCommand(
      inputText,
      newPos,
      agents,
      availableModels,
      skills,
      hasConversation,
    );
    popupFocusIndex = commandResult && commandResult.items.length > 0 ? 0 : -1;

    tick().then(() => {
      if (textareaEl) {
        textareaEl.selectionStart = textareaEl.selectionEnd = newPos;
        textareaEl.style.height = "auto";
        textareaEl.style.height = Math.min(textareaEl.scrollHeight, 200) + "px";
      }
    });
  }

  /** Handle popup item selection (click or Enter/Tab). */
  function selectPopupItem(item: PopupItem) {
    if (!commandResult) return;
    const { rangeStart, rangeEnd } = commandResult;

    switch (item.kind) {
      case "command": {
        const cmd = item.command;
        // Enumerable sub-commands: enter sub-command mode synchronously
        if (cmd.argType === "model" || cmd.argType === "skill") {
          enterSubCommandMode(cmd, rangeStart, rangeEnd);
          return;
        }
        // Free-text/url/format args: insert "/{name} " and let user type
        if (cmd.argType === "text" || cmd.argType === "url" || cmd.argType === "format") {
          replaceCommandText(rangeStart, rangeEnd, `/${cmd.name} `);
          return;
        }
        // Execute action commands with no args immediately
        dispatchCommand(cmd.name, rangeStart, rangeEnd);
        break;
      }
      case "agent":
        overrides = { ...overrides, agentId: item.agent.id };
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        break;
      case "model":
        overrides = { ...overrides, modelId: item.model.id };
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        break;
      case "skill":
        if (!overrides.skillIds.includes(item.skill.id)) {
          overrides = { ...overrides, skillIds: [...overrides.skillIds, item.skill.id] };
        }
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        break;
    }
    popupFocusIndex = -1;
  }

  /** Dispatch a resolved slash command (no more popup). */
  function dispatchCommand(name: string, rangeStart: number, rangeEnd: number) {
    const textAfterCmd = inputText.slice(rangeEnd).trim();

    switch (name) {
      case "file":
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        handleAttachFile();
        break;
      case "help":
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        onCommand?.("help");
        break;
      case "delete":
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        onCommand?.("delete");
        break;
      case "favorite":
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        onCommand?.("favorite");
        break;
      case "title":
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        onCommand?.("title");
        break;
      case "export":
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        onCommand?.("export");
        break;
      case "fetch": {
        const cmdTextFull = inputText.slice(rangeStart);
        const spacePos = cmdTextFull.indexOf(" ");
        const url = spacePos >= 0 ? cmdTextFull.slice(spacePos + 1).trim() : "";
        if (url) {
          inputText =
            inputText.slice(0, rangeStart) + inputText.slice(rangeStart + cmdTextFull.length);
          commandResult = null;
          addUrl(url);
        }
        break;
      }
      default:
        // For any command we don't handle, delegate to parent
        replaceCommandText(rangeStart, rangeEnd, "");
        commandResult = null;
        onCommand?.(name, textAfterCmd || undefined);
    }
    tick().then(() => {
      if (textareaEl) {
        textareaEl.style.height = "auto";
        textareaEl.style.height = Math.min(textareaEl.scrollHeight, 200) + "px";
      }
    });
  }

  /** Remove a per-message override. */
  function clearOverrideAgent() {
    overrides = { ...overrides, agentId: null };
  }
  function clearOverrideModel() {
    overrides = { ...overrides, modelId: null };
  }
  function removeOverrideSkill(skillId: string) {
    overrides = { ...overrides, skillIds: overrides.skillIds.filter((id) => id !== skillId) };
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
    if (dropdownOpen || agentDropdownOpen) {
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
    {#if popupVisible}
      <CommandPopup items={popupItems} focusedIndex={popupFocusIndex} onSelect={selectPopupItem} />
    {/if}
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
          {@const exStatus = extractionStatuses[file.name]}
          <div
            class="file-pill"
            class:loading={file.loading}
            class:extracting={exStatus === "extracting"}
            class:extracted={exStatus === "done"}
            class:extract-error={exStatus === "error"}
          >
            {#if file.loading || exStatus === "extracting"}
              <span class="file-pill-spinner"></span>
            {:else if exStatus === "done"}
              <span class="file-pill-icon">✓</span>
            {:else if exStatus === "error"}
              <span class="file-pill-icon">⚠</span>
            {:else}
              <span class="file-pill-icon">📎</span>
            {/if}
            <span class="file-pill-name">{file.name}</span>
            {#if file.loading}
              <span class="file-pill-status">reading…</span>
            {:else if exStatus === "extracting"}
              <span class="file-pill-status">extracting…</span>
            {:else if file.size > 0}
              <span class="file-pill-size">{formatBytes(file.size)}</span>
            {/if}
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

    {#if !network.isOnline}
      <div class="offline-indicator" role="status">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
          <path
            d="M1 1l14 14M3.5 7.5a6 6 0 0 1 4-2.8M6 10.3a3.5 3.5 0 0 1 4 0M8 13a.5.5 0 1 1 0-1 .5.5 0 0 1 0 1z"
            stroke="currentColor"
            stroke-width="1.3"
            fill="none"
            stroke-linecap="round"
          />
        </svg>
        You're offline — sending is disabled
      </div>
    {:else}
      {#if overrides.agentId || overrides.modelId || overrides.skillIds.length > 0}
        <div class="override-badges" role="status" aria-label="Per-message overrides">
          {#if overrides.agentId}
            {@const agent = agents.find((a) => a.id === overrides.agentId)}
            <button
              class="override-badge override-badge-agent"
              onclick={clearOverrideAgent}
              aria-label="Remove agent override"
            >
              <span class="override-badge-icon">{agent?.avatar ?? "🤖"}</span>
              <span class="override-badge-label">{agent?.name ?? overrides.agentId}</span>
              <span class="override-badge-x" aria-hidden="true">×</span>
            </button>
          {/if}
          {#if overrides.modelId}
            {@const mdl = availableModels.find((m) => m.id === overrides.modelId)}
            <button
              class="override-badge override-badge-model"
              onclick={clearOverrideModel}
              aria-label="Remove model override"
            >
              <span class="override-badge-icon">📊</span>
              <span class="override-badge-label">{mdl?.name ?? overrides.modelId}</span>
              <span class="override-badge-x" aria-hidden="true">×</span>
            </button>
          {/if}
          {#each overrides.skillIds as sid (sid)}
            {@const sk = skills.find((s) => s.id === sid)}
            <button
              class="override-badge override-badge-skill"
              onclick={() => removeOverrideSkill(sid)}
              aria-label="Remove skill override"
            >
              <span class="override-badge-icon">⚡</span>
              <span class="override-badge-label">{sk?.name ?? sid}</span>
              <span class="override-badge-x" aria-hidden="true">×</span>
            </button>
          {/each}
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
    {/if}
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
              onclick={() => {
                agentDropdownOpen = !agentDropdownOpen;
                focusedAgentIndex = -1;
              }}
              aria-label="Select agent"
              aria-expanded={agentDropdownOpen}
              aria-haspopup="listbox"
              title="Select agent persona"
            >
              <span class="agent-trigger-avatar"
                >{agents.find((a) => a.id === (selectedAgentId ?? defaultAgentId))?.avatar ??
                  "🤖"}</span
              >
              <span class="agent-trigger-name"
                >{agents.find((a) => a.id === (selectedAgentId ?? defaultAgentId))?.name ??
                  "Default"}</span
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
                aria-activedescendant={focusedAgentIndex >= 0
                  ? `agent-opt-${agents[focusedAgentIndex]?.id}`
                  : undefined}
                onkeydown={handleAgentDropdownKeydown}
              >
                {#each agents as agent, i (agent.id)}
                  <button
                    id="agent-opt-{agent.id}"
                    class="agent-option"
                    class:selected={agent.id === selectedAgentId}
                    class:focused={i === focusedAgentIndex}
                    role="option"
                    aria-selected={agent.id === selectedAgentId}
                    onclick={() => {
                      onAgentChange?.(agent.id);
                      agentDropdownOpen = false;
                    }}
                  >
                    <span class="agent-option-avatar">{agent.avatar ?? "🤖"}</span>
                    <span class="agent-option-name">{agent.name}</span>
                    {#if agent.id === defaultAgentId}
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
              onclick={() => {
                dropdownOpen = !dropdownOpen;
                focusedModelIndex = -1;
              }}
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
                aria-activedescendant={focusedModelIndex >= 0
                  ? `model-opt-${availableModels[focusedModelIndex]?.id}`
                  : undefined}
                onkeydown={handleDropdownKeydown}
              >
                <div class="model-dropdown-hint" aria-hidden="true">
                  <span>⇧ click to set default</span>
                </div>
                {#each availableModels as m, i (m.id)}
                  <button
                    id="model-opt-{m.id}"
                    class="model-option"
                    class:selected={m.id === model}
                    class:is-default={m.id === defaultModelId}
                    class:focused={i === focusedModelIndex}
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
      <div class="send-group">
        <button
          class="send-mode-label"
          onclick={() => {
            localSendShortcut = activeSendShortcut === "enter" ? "cmd-enter" : "enter";
          }}
          title={activeSendShortcut === "enter"
            ? `Switch to ${modLabel}+Enter to send`
            : "Switch to Enter to send"}
          aria-label={activeSendShortcut === "enter"
            ? `Send with Enter. Click to switch to ${modLabel}+Enter`
            : `Send with ${modLabel}+Enter. Click to switch to Enter`}
        >
          {activeSendShortcut === "enter" ? "↵ send" : `${modLabel}↵ send`}
        </button>
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
            class:active={!!inputText.trim() && network.isOnline}
            onclick={handleSend}
            disabled={!inputText.trim() || !network.isOnline}
            aria-label={network.isOnline ? "Send message" : "Offline — sending disabled"}
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 2.5l-4.5 4.5h3V13h3V7h3L8 2.5z" />
            </svg>
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .input-area {
    width: 100%;
  }

  .input-box {
    position: relative;
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

  .model-option.focused {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
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

  /* ── Send group (toggle + send button) ───────────── */

  .send-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .send-mode-label {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.65rem;
    font-family: var(--font-body);
    color: var(--color-text-tertiary);
    white-space: nowrap;
    letter-spacing: 0.02em;
    padding: 2px 0;
    user-select: none;
    width: 4.8em;
    text-align: right;
    transition: color var(--transition-fast);
  }

  .send-mode-label:hover {
    color: var(--color-text-secondary);
  }

  .send-mode-label:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
    border-radius: var(--radius-sm);
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

  .agent-option.focused {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
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

  .file-pill.loading,
  .file-pill.extracting {
    opacity: 0.7;
  }

  .file-pill.extracted {
    border-color: var(--color-accent-copper);
  }

  .file-pill.extract-error {
    border-color: var(--color-error, #dc2626);
    opacity: 0.8;
  }

  .file-pill-spinner {
    width: 12px;
    height: 12px;
    border: 1.5px solid var(--color-border-secondary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    flex-shrink: 0;
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

  .file-pill-size,
  .file-pill-status {
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

  /* ── Offline indicator ─────────────────────────── */

  .offline-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-lg);
    color: var(--color-text-tertiary);
    font-size: var(--font-size-sm);
    user-select: none;
  }

  /* ── Override badges ─────────────────────────── */

  .override-badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-md) 0;
  }

  .override-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px var(--spacing-sm);
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border-primary);
    background: var(--color-bg-secondary);
    color: var(--color-text-secondary);
    font-size: var(--font-size-xs);
    font-family: var(--font-sans);
    cursor: pointer;
    transition:
      background 120ms ease,
      border-color 120ms ease;
  }

  .override-badge:hover {
    background: var(--color-bg-hover);
  }

  .override-badge-agent {
    border-color: var(--color-accent-copper);
  }

  .override-badge-model {
    border-color: var(--color-accent-lavender, var(--color-accent-copper));
  }

  .override-badge-skill {
    border-color: var(--color-accent-gold, var(--color-accent-copper));
  }

  .override-badge-icon {
    font-size: 12px;
    line-height: 1;
  }

  .override-badge-label {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .override-badge-x {
    margin-left: 2px;
    font-size: 13px;
    line-height: 1;
    opacity: 0.6;
  }

  .override-badge:hover .override-badge-x {
    opacity: 1;
  }
</style>
