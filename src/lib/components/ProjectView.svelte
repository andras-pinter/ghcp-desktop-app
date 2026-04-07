<script lang="ts">
  import {
    getProjectStore,
    newProject,
    selectProject,
    renameProject,
    updateProjectInstructions,
    removeProject,
    uploadProjectFile,
    deleteProjectFile,
  } from "$lib/stores/projects.svelte";
  import { switchConversation, getConversationStore } from "$lib/stores/conversations.svelte";
  import { updateConversation } from "$lib/utils/commands";
  import { formatBytes, truncate } from "$lib/utils/format";
  import type { FileUpload } from "$lib/types/project";

  interface Props {
    onBack: () => void;
    onOpenChat: () => void;
  }

  let { onBack, onOpenChat }: Props = $props();

  const store = getProjectStore();
  const convStore = getConversationStore();

  // ── View state ──────────────────────────────────────────────

  type ViewState = { kind: "list" } | { kind: "detail"; projectId: string } | { kind: "form" };

  let view = $state<ViewState>({ kind: "list" });

  // ── Form state (create new project) ─────────────────────────

  let formName = $state("");
  let formInstructions = $state("");
  let formSaving = $state(false);
  let formError = $state<string | null>(null);

  // ── Detail edit state ───────────────────────────────────────

  let editingName = $state(false);
  let editNameText = $state("");
  let editingInstructions = $state(false);
  let editInstructionsText = $state("");
  let confirmingDelete = $state(false);
  let uploading = $state(false);
  let uploadError = $state<string | null>(null);
  let assigningConversation = $state(false);

  // ── Helpers ─────────────────────────────────────────────────

  function resetForm() {
    formName = "";
    formInstructions = "";
    formSaving = false;
    formError = null;
  }

  function openDetail(id: string) {
    selectProject(id);
    editingName = false;
    editingInstructions = false;
    confirmingDelete = false;
    uploadError = null;
    view = { kind: "detail", projectId: id };
  }

  // ── Handlers ────────────────────────────────────────────────

  async function handleCreateProject() {
    const name = formName.trim();
    if (!name) return;
    formSaving = true;
    formError = null;
    try {
      const proj = await newProject(name, formInstructions.trim() || null);
      resetForm();
      openDetail(proj.id);
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      formSaving = false;
    }
  }

  async function handleRename() {
    const name = editNameText.trim();
    if (!name || !store.activeProjectId) return;
    await renameProject(store.activeProjectId, name);
    editingName = false;
  }

  async function handleSaveInstructions() {
    if (!store.activeProjectId) return;
    await updateProjectInstructions(store.activeProjectId, editInstructionsText.trim() || null);
    editingInstructions = false;
  }

  async function handleDelete() {
    if (!store.activeProjectId) return;
    await removeProject(store.activeProjectId);
    view = { kind: "list" };
  }

  let fileInputEl: HTMLInputElement | undefined = $state();

  /** Read a browser File object into a FileUpload struct (base64-encoded). */
  function readBrowserFile(file: File): Promise<FileUpload> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const arrayBuffer = reader.result as ArrayBuffer;
        const bytes = new Uint8Array(arrayBuffer);
        let binary = "";
        for (let i = 0; i < bytes.length; i++) {
          binary += String.fromCharCode(bytes[i]);
        }
        resolve({
          name: file.name,
          contentType: file.type || "application/octet-stream",
          contentBase64: btoa(binary),
        });
      };
      reader.onerror = () => reject(new Error("Failed to read file"));
      reader.readAsArrayBuffer(file);
    });
  }

  function handleFileUpload() {
    fileInputEl?.click();
  }

  async function handleFilesSelected(event: Event) {
    const input = event.target as HTMLInputElement;
    const files = input.files;
    if (!files || files.length === 0) return;

    uploadError = null;
    uploading = true;
    try {
      for (let i = 0; i < files.length; i++) {
        const fileData = await readBrowserFile(files[i]);
        await uploadProjectFile(fileData);
      }
    } catch (e) {
      uploadError = e instanceof Error ? e.message : String(e);
    } finally {
      uploading = false;
      // Reset input so the same file can be re-selected
      input.value = "";
    }
  }

  async function handleRemoveFile(fileId: string) {
    await deleteProjectFile(fileId);
  }

  async function handleAssignConversation(convId: string) {
    if (!store.activeProjectId) return;
    await updateConversation(convId, undefined, undefined, undefined, store.activeProjectId);
    await selectProject(store.activeProjectId);
    assigningConversation = false;
  }

  async function handleUnassignConversation(convId: string) {
    if (!store.activeProjectId) return;
    await updateConversation(convId, undefined, undefined, undefined, null);
    await selectProject(store.activeProjectId);
  }

  function handleOpenConversation(convId: string) {
    switchConversation(convId);
    onOpenChat();
  }

  // ── Utilities ───────────────────────────────────────────────

  /** File icon based on content type. */
  function fileIcon(contentType: string): string {
    if (contentType.startsWith("image/")) return "🖼️";
    if (contentType.startsWith("text/")) return "📄";
    if (contentType.includes("pdf")) return "📕";
    if (contentType.includes("json")) return "📋";
    return "📎";
  }

  /** Conversations not assigned to any project (available for assignment). */
  let unassignedConversations = $derived(convStore.conversations.filter((c) => !c.projectId));
</script>

<div class="projects-panel">
  <!-- Header -->
  <div class="panel-header">
    <button class="back-btn" onclick={onBack} aria-label="Back">
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path d="M10 3L5 8l5 5" />
      </svg>
    </button>
    <h2 class="panel-title">
      {#if view.kind === "form"}
        New Project
      {:else if view.kind === "detail"}
        {store.activeProject?.name ?? "Project"}
      {:else}
        Projects
      {/if}
    </h2>
  </div>

  <div class="panel-body">
    <!-- ═══════════════════ LIST VIEW ═══════════════════ -->
    {#if view.kind === "list"}
      <div class="project-list" role="list">
        {#if store.projects.length === 0 && !store.loading}
          <div class="empty-state">
            <div class="empty-icon">📁</div>
            <p class="empty-title">No projects yet</p>
            <p class="empty-desc">
              Projects let you group conversations and attach reference files with custom
              instructions.
            </p>
            <button
              class="btn btn-primary"
              onclick={() => {
                resetForm();
                view = { kind: "form" };
              }}
            >
              Create your first project
            </button>
          </div>
        {:else}
          {#each store.projects as project (project.id)}
            <button class="project-card" onclick={() => openDetail(project.id)}>
              <div class="project-card-icon">📁</div>
              <div class="project-card-body">
                <span class="project-card-name">{project.name}</span>
                <span class="project-card-meta">
                  {project.fileCount} file{project.fileCount !== 1 ? "s" : ""}
                  {#if project.instructions}
                    · has instructions
                  {/if}
                </span>
              </div>
              <svg
                class="project-card-chevron"
                width="14"
                height="14"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
              >
                <path d="M6 3l5 5-5 5" />
              </svg>
            </button>
          {/each}
        {/if}
      </div>

      {#if store.projects.length > 0}
        <div class="list-footer">
          <button
            class="btn btn-secondary"
            onclick={() => {
              resetForm();
              view = { kind: "form" };
            }}
          >
            + New Project
          </button>
        </div>
      {/if}

      <!-- ═══════════════════ CREATE FORM ═══════════════════ -->
    {:else if view.kind === "form"}
      <div class="form-section">
        <label class="form-label" for="project-name">Name</label>
        <input
          id="project-name"
          type="text"
          class="form-input"
          placeholder="e.g., Blog Engine, API Redesign"
          bind:value={formName}
          onkeydown={(e) => {
            if (e.key === "Enter") handleCreateProject();
          }}
        />
      </div>

      <div class="form-section">
        <label class="form-label" for="project-instructions">
          Custom Instructions
          <span class="form-hint">Optional — added to every chat in this project</span>
        </label>
        <textarea
          id="project-instructions"
          class="form-textarea"
          placeholder="e.g., Use TypeScript strict mode. Follow the project's existing patterns..."
          rows="5"
          bind:value={formInstructions}
        ></textarea>
      </div>

      {#if formError}
        <div class="form-error">{formError}</div>
      {/if}

      <div class="form-actions">
        <button
          class="btn btn-ghost"
          onclick={() => {
            resetForm();
            view = { kind: "list" };
          }}
        >
          Cancel
        </button>
        <button
          class="btn btn-primary"
          disabled={!formName.trim() || formSaving}
          onclick={handleCreateProject}
        >
          {formSaving ? "Creating…" : "Create Project"}
        </button>
      </div>

      <!-- ═══════════════════ DETAIL VIEW ═══════════════════ -->
    {:else if view.kind === "detail" && store.activeProject}
      <!-- Name -->
      <div class="detail-section">
        <div class="detail-section-header">
          <h3 class="detail-section-title">Name</h3>
          {#if !editingName}
            <button
              class="btn-inline"
              onclick={() => {
                editNameText = store.activeProject?.name ?? "";
                editingName = true;
              }}
            >
              Edit
            </button>
          {/if}
        </div>
        {#if editingName}
          <div class="inline-edit">
            <input
              type="text"
              class="form-input"
              bind:value={editNameText}
              onkeydown={(e) => {
                if (e.key === "Enter") handleRename();
                if (e.key === "Escape") editingName = false;
              }}
            />
            <button class="btn btn-sm" onclick={handleRename}>Save</button>
            <button class="btn btn-sm btn-ghost" onclick={() => (editingName = false)}
              >Cancel</button
            >
          </div>
        {:else}
          <p class="detail-value">{store.activeProject.name}</p>
        {/if}
      </div>

      <!-- Instructions -->
      <div class="detail-section">
        <div class="detail-section-header">
          <h3 class="detail-section-title">Custom Instructions</h3>
          {#if !editingInstructions}
            <button
              class="btn-inline"
              onclick={() => {
                editInstructionsText = store.activeProject?.instructions ?? "";
                editingInstructions = true;
              }}
            >
              {store.activeProject.instructions ? "Edit" : "Add"}
            </button>
          {/if}
        </div>
        {#if editingInstructions}
          <textarea
            class="form-textarea"
            rows="5"
            placeholder="Instructions that apply to all conversations in this project…"
            bind:value={editInstructionsText}
          ></textarea>
          <div class="form-actions">
            <button class="btn btn-sm btn-ghost" onclick={() => (editingInstructions = false)}
              >Cancel</button
            >
            <button class="btn btn-sm" onclick={handleSaveInstructions}>Save</button>
          </div>
        {:else if store.activeProject.instructions}
          <p class="detail-value instructions-text">{store.activeProject.instructions}</p>
        {:else}
          <p class="detail-value muted">No custom instructions</p>
        {/if}
      </div>

      <!-- Files -->
      <div class="detail-section">
        <div class="detail-section-header">
          <h3 class="detail-section-title">
            Files
            <span class="badge">{store.files.length}</span>
          </h3>
          <button class="btn-inline" onclick={handleFileUpload} disabled={uploading}>
            {uploading ? "Uploading…" : "+ Add File"}
          </button>
          <input
            type="file"
            multiple
            class="hidden-file-input"
            bind:this={fileInputEl}
            onchange={handleFilesSelected}
          />
        </div>

        {#if uploadError}
          <div class="form-error">{uploadError}</div>
        {/if}

        {#if store.files.length === 0}
          <p class="detail-value muted">
            No files attached. Text files are included as context in chats.
          </p>
        {:else}
          <div class="file-list" role="list">
            {#each store.files as file (file.id)}
              <div class="file-item" role="listitem">
                <span class="file-icon">{fileIcon(file.contentType)}</span>
                <div class="file-info">
                  <span class="file-name">{truncate(file.name, 40)}</span>
                  <span class="file-meta">{formatBytes(file.size)} · {file.contentType}</span>
                </div>
                <button
                  class="btn-icon-remove"
                  aria-label="Remove file"
                  title="Remove file"
                  onclick={() => handleRemoveFile(file.id)}
                >
                  ✕
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Conversations -->
      <div class="detail-section">
        <div class="detail-section-header">
          <h3 class="detail-section-title">
            Conversations
            <span class="badge">{store.conversations.length}</span>
          </h3>
          <button
            class="btn-inline"
            onclick={() => (assigningConversation = !assigningConversation)}
          >
            {assigningConversation ? "Done" : "+ Assign"}
          </button>
        </div>

        {#if assigningConversation}
          <div class="assign-list">
            {#if unassignedConversations.length === 0}
              <p class="detail-value muted">All conversations are already assigned to projects.</p>
            {:else}
              {#each unassignedConversations.slice(0, 20) as conv (conv.id)}
                <button class="assign-item" onclick={() => handleAssignConversation(conv.id)}>
                  <span class="assign-item-title">{conv.title ?? "Untitled"}</span>
                  <span class="assign-item-action">+ Assign</span>
                </button>
              {/each}
            {/if}
          </div>
        {/if}

        {#if store.conversations.length === 0 && !assigningConversation}
          <p class="detail-value muted">
            No conversations linked. Use "+ Assign" to add existing chats.
          </p>
        {:else}
          <div class="conversation-list" role="list">
            {#each store.conversations as conv (conv.id)}
              <div class="conv-item" role="listitem">
                <button class="conv-item-link" onclick={() => handleOpenConversation(conv.id)}>
                  {conv.title ?? "Untitled"}
                </button>
                <button
                  class="btn-icon-remove"
                  aria-label="Unassign conversation"
                  title="Unassign from project"
                  onclick={() => handleUnassignConversation(conv.id)}
                >
                  ✕
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Danger zone -->
      <div class="detail-section danger-zone">
        {#if confirmingDelete}
          <p class="danger-text">
            Delete "{store.activeProject.name}" and all its files? Conversations will be unlinked
            but not deleted.
          </p>
          <div class="form-actions">
            <button class="btn btn-ghost" onclick={() => (confirmingDelete = false)}>Cancel</button>
            <button class="btn btn-danger" onclick={handleDelete}>Delete Project</button>
          </div>
        {:else}
          <button class="btn btn-danger-outline" onclick={() => (confirmingDelete = true)}>
            Delete Project
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .projects-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* ── Header ── */

  .panel-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
    min-height: 52px;
  }

  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .back-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .panel-title {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-normal);
    color: var(--color-text-primary);
    margin: 0;
  }

  /* ── Body ── */

  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
  }

  /* ── Empty state ── */

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-3xl) var(--spacing-lg);
    text-align: center;
  }

  .empty-icon {
    font-size: 2.5rem;
    opacity: 0.7;
  }

  .empty-title {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-lg);
    color: var(--color-text-primary);
    margin: 0;
  }

  .empty-desc {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: 0;
    max-width: 320px;
    line-height: 1.5;
  }

  /* ── Project list ── */

  .project-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .project-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
    width: 100%;
  }

  .project-card:hover {
    border-color: var(--color-border-primary);
    background: var(--color-bg-hover);
    transform: translateX(2px);
  }

  .project-card-icon {
    font-size: 1.25rem;
    flex-shrink: 0;
  }

  .project-card-body {
    flex: 1;
    min-width: 0;
  }

  .project-card-name {
    display: block;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-primary);
  }

  .project-card-meta {
    display: block;
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    margin-top: 2px;
  }

  .project-card-chevron {
    flex-shrink: 0;
    color: var(--color-text-tertiary);
  }

  .list-footer {
    display: flex;
    justify-content: center;
    padding-top: var(--spacing-lg);
  }

  /* ── Form ── */

  .form-section {
    margin-bottom: var(--spacing-lg);
  }

  .form-label {
    display: block;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-primary);
    margin-bottom: var(--spacing-xs);
  }

  .form-hint {
    font-weight: var(--font-weight-normal);
    color: var(--color-text-tertiary);
    font-size: var(--font-size-xs);
    display: block;
    margin-top: 2px;
  }

  .form-input {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    color: var(--color-text-primary);
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    outline: none;
    transition: all var(--transition-fast);
    box-sizing: border-box;
  }

  .form-input:focus {
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .form-textarea {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    color: var(--color-text-primary);
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    outline: none;
    resize: vertical;
    min-height: 80px;
    transition: all var(--transition-fast);
    box-sizing: border-box;
  }

  .form-textarea:focus {
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .form-error {
    font-size: var(--font-size-xs);
    color: var(--color-error);
    padding: var(--spacing-xs) 0;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-md);
  }

  /* ── Buttons ── */

  .btn {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    font-weight: var(--font-weight-medium);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    border: 1px solid transparent;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
    border-color: var(--color-text-primary);
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.85;
  }

  .btn-secondary {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    border-color: var(--color-border-primary);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .btn-ghost {
    background: none;
    color: var(--color-text-secondary);
    border: none;
    padding: var(--spacing-xs) var(--spacing-sm);
  }

  .btn-ghost:hover {
    color: var(--color-text-primary);
  }

  .btn-sm {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-xs);
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
  }

  .btn-sm:hover:not(:disabled) {
    opacity: 0.85;
  }

  .btn-danger {
    background: var(--color-error);
    color: white;
    border-color: var(--color-error);
  }

  .btn-danger:hover:not(:disabled) {
    opacity: 0.85;
  }

  .btn-danger-outline {
    background: none;
    color: var(--color-error);
    border: 1px solid var(--color-error);
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    font-weight: var(--font-weight-medium);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-danger-outline:hover {
    background: var(--color-error);
    color: white;
  }

  .btn-inline {
    background: none;
    border: none;
    color: var(--color-accent-copper);
    font-size: var(--font-size-xs);
    font-family: var(--font-body);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    padding: 0;
    transition: opacity var(--transition-fast);
  }

  .btn-inline:hover {
    opacity: 0.7;
  }

  .btn-inline:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ── Detail sections ── */

  .detail-section {
    padding-bottom: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
    border-bottom: 1px solid var(--color-border-secondary);
  }

  .detail-section:last-child {
    border-bottom: none;
  }

  .detail-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-sm);
  }

  .detail-section-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
    margin: 0;
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .detail-value {
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    margin: 0;
    line-height: 1.5;
  }

  .detail-value.muted {
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .instructions-text {
    white-space: pre-wrap;
    background: var(--color-bg-secondary);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-secondary);
  }

  .inline-edit {
    display: flex;
    gap: var(--spacing-xs);
    align-items: center;
  }

  .inline-edit .form-input {
    flex: 1;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    font-size: 11px;
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-tertiary);
    background: var(--color-bg-hover);
    border-radius: 10px;
  }

  /* ── File list ── */

  .file-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-sm);
  }

  .file-icon {
    font-size: 1rem;
    flex-shrink: 0;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    display: block;
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    display: block;
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .btn-icon-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: var(--radius-sm);
    font-size: 12px;
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .btn-icon-remove:hover {
    background: var(--color-bg-hover);
    color: var(--color-error);
  }

  /* ── Conversation list ── */

  .conversation-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .conv-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-sm);
  }

  .conv-item-link {
    flex: 1;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    text-align: left;
    padding: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color var(--transition-fast);
  }

  .conv-item-link:hover {
    color: var(--color-accent-copper);
  }

  /* ── Assign list ── */

  .assign-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    max-height: 200px;
    overflow-y: auto;
  }

  .assign-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-xs) var(--spacing-sm);
    background: none;
    border: none;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
    width: 100%;
    text-align: left;
    font-family: var(--font-body);
  }

  .assign-item:hover {
    background: var(--color-bg-hover);
  }

  .assign-item-title {
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .assign-item-action {
    font-size: var(--font-size-xs);
    color: var(--color-accent-copper);
    font-weight: var(--font-weight-medium);
    flex-shrink: 0;
    margin-left: var(--spacing-sm);
  }

  /* ── Danger zone ── */

  .danger-zone {
    border-top: 1px solid var(--color-border-secondary);
    padding-top: var(--spacing-lg);
  }

  .danger-text {
    font-size: var(--font-size-sm);
    color: var(--color-error);
    margin: 0 0 var(--spacing-sm) 0;
  }

  .hidden-file-input {
    display: none;
  }
</style>
