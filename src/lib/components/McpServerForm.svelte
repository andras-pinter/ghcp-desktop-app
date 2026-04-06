<script lang="ts">
  import type { McpServerConfig, McpConnectionInfo, RegistryServer } from "$lib/types/mcp";
  import { addServer, editServer, testConnection } from "$lib/stores/mcp.svelte";

  interface Props {
    /** If set, we're editing this server. */
    editInfo?: McpConnectionInfo | null;
    /** If set, pre-fill from a registry entry. */
    registryEntry?: RegistryServer | null;
    /** Go back to the list view. */
    onBack: () => void;
  }

  let { editInfo = null, registryEntry = null, onBack }: Props = $props();

  // ── Form fields ──────────────────────────────────────────────

  let formName = $state("");
  let formTransport = $state<"http" | "stdio">("http");
  let formUrl = $state("");
  let formBinaryPath = $state("");
  let formArgs = $state("");
  let formAuthHeader = $state("");
  let formError = $state("");
  let submitting = $state(false);
  let testing = $state(false);
  let testResult = $state<{ success: boolean; message: string } | null>(null);

  const argsPlaceholder = '["--port", "3000"]';

  const isEditing = editInfo !== null;

  // Pre-fill based on source
  if (editInfo) {
    formName = editInfo.config.name;
    formTransport = editInfo.config.transport;
    formUrl = editInfo.config.url ?? "";
    formBinaryPath = editInfo.config.binaryPath ?? "";
    formArgs = editInfo.config.args ?? "";
    formAuthHeader = editInfo.config.authHeader ?? "";
  } else if (registryEntry) {
    formName = registryEntry.displayName;
    if (registryEntry.isStdioOnly) {
      formTransport = "stdio";
      // Auto-fill npx/uvx/dotnet command from packages
      const npmPkg = registryEntry.packages.find((p) => p.registryType === "npm");
      const pypiPkg = registryEntry.packages.find((p) => p.registryType === "pypi");
      const nugetPkg = registryEntry.packages.find((p) => p.registryType === "nuget");
      if (npmPkg) {
        formBinaryPath = "npx";
        const pkgRef = npmPkg.version
          ? `${npmPkg.identifier}@${npmPkg.version}`
          : npmPkg.identifier;
        formArgs = JSON.stringify(["-y", pkgRef]);
      } else if (pypiPkg) {
        formBinaryPath = "uvx";
        const pkgRef = pypiPkg.version
          ? `${pypiPkg.identifier}==${pypiPkg.version}`
          : pypiPkg.identifier;
        formArgs = JSON.stringify([pkgRef]);
      } else if (nugetPkg) {
        formBinaryPath = "dotnet";
        formArgs = JSON.stringify(["tool", "run", nugetPkg.identifier]);
      }
    } else {
      formTransport = "http";
      formUrl = registryEntry.remotes[0]?.url ?? "";
    }
    formAuthHeader = "";
  }

  // ── Validation + Submit ──────────────────────────────────────

  function validate(): boolean {
    formError = "";

    if (!formName.trim()) {
      formError = "Server name is required";
      return false;
    }

    if (formTransport === "http") {
      if (!formUrl.trim()) {
        formError = "URL is required for HTTP transport";
        return false;
      }
      try {
        const parsed = new URL(formUrl);
        if (!["http:", "https:"].includes(parsed.protocol)) {
          formError = "Only HTTP and HTTPS URLs are allowed";
          return false;
        }
      } catch {
        formError = "Invalid URL format";
        return false;
      }
    }

    if (formTransport === "stdio") {
      if (!formBinaryPath.trim()) {
        formError = "Binary path is required for stdio transport";
        return false;
      }
      // Allow bare command names (npx, uvx, dotnet) resolved via PATH,
      // and absolute paths (starting with /)
      const trimmed = formBinaryPath.trim();
      if (trimmed.includes("/") && !trimmed.startsWith("/")) {
        formError = "Binary path must be absolute (start with /) or a command name (e.g., npx)";
        return false;
      }
    }

    if (formArgs.trim()) {
      try {
        const parsed = JSON.parse(formArgs);
        if (!Array.isArray(parsed)) {
          formError = "Arguments must be a JSON array of strings";
          return false;
        }
      } catch {
        formError = 'Arguments must be valid JSON (e.g. ["--port", "3000"])';
        return false;
      }
    }

    return true;
  }

  function buildConfig(): McpServerConfig {
    return {
      id: editInfo?.config.id ?? `mcp-${Date.now()}`,
      name: formName,
      transport: formTransport,
      url: formTransport === "http" ? formUrl || null : null,
      binaryPath: formTransport === "stdio" ? formBinaryPath || null : null,
      args: formTransport === "stdio" && formArgs ? formArgs : null,
      authHeader: formAuthHeader || null,
      fromCatalog: false,
      enabled: true,
    };
  }

  async function submitForm() {
    if (!validate()) return;
    submitting = true;
    try {
      const config = buildConfig();
      if (isEditing) {
        await editServer(config);
      } else {
        await addServer(config);
      }
      onBack();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function handleTest() {
    if (!validate()) return;
    testing = true;
    testResult = null;
    try {
      const count = await testConnection(buildConfig());
      testResult = {
        success: true,
        message: `Connected successfully — ${count} tool${count !== 1 ? "s" : ""} discovered`,
      };
    } catch (e) {
      testResult = {
        success: false,
        message: e instanceof Error ? e.message : String(e),
      };
    } finally {
      testing = false;
    }
  }
</script>

<div class="form-page">
  <header class="form-header">
    <button class="back-btn" onclick={onBack} aria-label="Go back">← Back</button>
    <h2 class="form-title">{isEditing ? "Edit Server" : "Add MCP Server"}</h2>
  </header>

  <div class="form-content">
    {#if registryEntry && !isEditing}
      <div class="prefill-notice">
        From MCP Registry: <strong>{registryEntry.displayName}</strong>
        {#if registryEntry.isStdioOnly}
          <span class="stdio-notice"
            >— This is a stdio-only server. You'll need to install and provide the binary path.</span
          >
        {/if}
        {#if registryEntry.description}
          <p class="prefill-desc">{registryEntry.description}</p>
        {/if}
      </div>
    {/if}

    <form
      class="server-form"
      onsubmit={(e) => {
        e.preventDefault();
        submitForm();
      }}
    >
      <label class="form-field">
        <span class="field-label">Name</span>
        <input type="text" bind:value={formName} placeholder="My MCP Server" required />
      </label>

      <fieldset class="form-field">
        <legend class="field-label">Transport</legend>
        <div class="radio-group">
          <label>
            <input type="radio" bind:group={formTransport} value="http" /> HTTP
          </label>
          <label>
            <input type="radio" bind:group={formTransport} value="stdio" /> Stdio
          </label>
        </div>
      </fieldset>

      {#if formTransport === "http"}
        <label class="form-field">
          <span class="field-label">URL</span>
          <input type="url" bind:value={formUrl} placeholder="https://example.com/mcp" required />
        </label>
        <label class="form-field">
          <span class="field-label">Auth Header (optional)</span>
          <input type="text" bind:value={formAuthHeader} placeholder="Bearer your-token" />
        </label>
      {:else}
        <label class="form-field">
          <span class="field-label">Binary Path</span>
          <input
            type="text"
            bind:value={formBinaryPath}
            placeholder="npx or /usr/local/bin/mcp-server"
            required
          />
          <span class="field-hint">
            A command name (e.g., npx, uvx) or absolute path to the MCP server binary.
          </span>
        </label>
        <label class="form-field">
          <span class="field-label">Arguments (JSON array, optional)</span>
          <input type="text" bind:value={formArgs} placeholder={argsPlaceholder} />
        </label>
      {/if}

      {#if formError}
        <div class="form-error">{formError}</div>
      {/if}

      {#if testResult}
        <div
          class="test-result"
          class:success={testResult.success}
          class:failure={!testResult.success}
        >
          {testResult.success ? "✓" : "✗"}
          {testResult.message}
        </div>
      {/if}

      <div class="form-actions">
        <button type="button" class="action-btn" onclick={handleTest} disabled={testing}>
          {testing ? "Testing..." : "Test Connection"}
        </button>
        <div class="actions-right">
          <button type="button" class="action-btn" onclick={onBack}>Cancel</button>
          <button type="submit" class="action-btn primary" disabled={submitting}>
            {#if submitting}
              Saving...
            {:else}
              {isEditing ? "Save Changes" : "Add Server"}
            {/if}
          </button>
        </div>
      </div>
    </form>
  </div>
</div>

<style>
  .form-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .form-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--color-border-primary);
    flex-shrink: 0;
  }

  .back-btn {
    background: none;
    border: none;
    color: var(--color-accent-copper);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }
  .back-btn:hover {
    background: var(--color-bg-hover);
  }

  .form-title {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-xl);
    color: var(--color-text-primary);
    margin: 0;
  }

  .form-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
    max-width: 640px;
    margin: 0 auto;
    width: 100%;
  }

  .prefill-notice {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    padding: var(--spacing-sm) var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }
  .prefill-desc {
    margin: var(--spacing-xs) 0 0;
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }
  .stdio-notice {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .server-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    border: none;
    padding: 0;
    margin: 0;
  }

  .field-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
  }

  .field-hint {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .form-field input[type="text"],
  .form-field input[type="url"] {
    padding: var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
  }
  .form-field input:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .radio-group {
    display: flex;
    gap: var(--spacing-md);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }
  .radio-group label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    cursor: pointer;
  }

  .form-error {
    color: var(--color-danger, #dc2626);
    font-size: var(--font-size-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: color-mix(in srgb, var(--color-danger, #dc2626) 8%, transparent);
    border-radius: var(--radius-sm);
  }

  .test-result {
    font-size: var(--font-size-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
  }
  .test-result.success {
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 8%, transparent);
  }
  .test-result.failure {
    color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
  }

  .form-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
  }

  .actions-right {
    display: flex;
    gap: var(--spacing-sm);
  }

  .action-btn {
    font-size: var(--font-size-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }
  .action-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }
  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .action-btn.primary {
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
    border-color: var(--color-text-primary);
  }
  .action-btn.primary:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>
