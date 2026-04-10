<script lang="ts">
  import type { McpServerConfig, McpConnectionInfo, RegistryServer } from "$lib/types/mcp";
  import {
    addServer,
    editServer,
    testConnection,
    testConnectionConfig,
  } from "$lib/stores/mcp.svelte";
  import { approveMcpBinary } from "$lib/utils/commands";
  import { untrack } from "svelte";

  interface Props {
    /** If set, we're editing this server. */
    editInfo?: McpConnectionInfo | null;
    /** If set, pre-fill from a registry entry. */
    registryEntry?: RegistryServer | null;
    /** Go back to the list view. */
    onBack: () => void;
  }

  let { editInfo = null, registryEntry = null, onBack }: Props = $props();

  // Snapshot props once — they never change after mount.
  // untrack() suppresses Svelte's "captured initial value" warning.
  const initialEdit = untrack(() => editInfo);
  const initialRegistry = untrack(() => registryEntry);

  // ── Form fields ──────────────────────────────────────────────

  let formName = $state("");
  let formTransport = $state<"http" | "stdio">("http");
  let formUrl = $state("");
  let formBinaryPath = $state("");
  let formArgs = $state("");
  let formAuthHeader = $state("");
  let formError = $state("");
  let formValid = $derived(
    formName.trim().length > 0 &&
      (formTransport === "http" ? formUrl.trim().length > 0 : formBinaryPath.trim().length > 0),
  );
  let submitting = $state(false);
  let testing = $state(false);
  let testResult = $state<{ success: boolean; message: string } | null>(null);

  const argsPlaceholder = '["--port", "3000"]';

  const isEditing = initialEdit !== null;

  // Pre-fill based on source
  if (initialEdit) {
    formName = initialEdit.config.name;
    formTransport = initialEdit.config.transport;
    formUrl = initialEdit.config.url ?? "";
    formBinaryPath = initialEdit.config.binaryPath ?? "";
    formArgs = initialEdit.config.args ?? "";
    formAuthHeader = initialEdit.config.authHeader ?? "";
  } else if (initialRegistry) {
    formName = initialRegistry.displayName;
    if (initialRegistry.isStdioOnly) {
      formTransport = "stdio";
      // Auto-fill npx/uvx/dotnet command from packages
      const npmPkg = initialRegistry.packages.find((p) => p.registryType === "npm");
      const pypiPkg = initialRegistry.packages.find((p) => p.registryType === "pypi");
      const nugetPkg = initialRegistry.packages.find((p) => p.registryType === "nuget");
      if (npmPkg) {
        formBinaryPath = "npx";
        const pkgRef = npmPkg.version
          ? `${npmPkg.identifier}@${npmPkg.version}`
          : npmPkg.identifier;
        formArgs = JSON.stringify(["-y", pkgRef, ...npmPkg.arguments]);
      } else if (pypiPkg) {
        formBinaryPath = "uvx";
        const pkgRef = pypiPkg.version
          ? `${pypiPkg.identifier}==${pypiPkg.version}`
          : pypiPkg.identifier;
        formArgs = JSON.stringify([pkgRef, ...pypiPkg.arguments]);
      } else if (nugetPkg) {
        formBinaryPath = "dotnet";
        formArgs = JSON.stringify(["tool", "run", nugetPkg.identifier, ...nugetPkg.arguments]);
      }
    } else {
      formTransport = "http";
      formUrl = initialRegistry.remotes[0]?.url ?? "";
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
      id: initialEdit?.config.id ?? `mcp-${Date.now()}`,
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
      // Use server_id for existing servers (loads real auth from keychain),
      // or raw config for unsaved servers (auth is the real value from form).
      const count = isEditing
        ? await testConnection(initialEdit!.config.id)
        : await testConnectionConfig(buildConfig());
      testResult = {
        success: true,
        message: `Connected successfully — ${count} tool${count !== 1 ? "s" : ""} discovered`,
      };
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.startsWith("BINARY_NOT_APPROVED:")) {
        const binaryPath = msg.slice("BINARY_NOT_APPROVED:".length);
        try {
          await approveMcpBinary(binaryPath);
          // Approval succeeded — retry the test
          const count = isEditing
            ? await testConnection(initialEdit!.config.id)
            : await testConnectionConfig(buildConfig());
          testResult = {
            success: true,
            message: `Connected successfully — ${count} tool${count !== 1 ? "s" : ""} discovered`,
          };
        } catch (approveErr) {
          testResult = {
            success: false,
            message: approveErr instanceof Error ? approveErr.message : String(approveErr),
          };
        }
      } else {
        testResult = { success: false, message: msg };
      }
    } finally {
      testing = false;
    }
  }
</script>

<div class="panel-body-narrow">
  {#if initialRegistry && !isEditing}
    <div class="banner banner--info prefill-notice">
      From MCP Registry: <strong>{initialRegistry.displayName}</strong>
      {#if initialRegistry.isStdioOnly && !initialRegistry.packages.length}
        <span class="prefill-sub"
          >— This is a stdio-only server. You'll need to provide the binary path.</span
        >
      {/if}
      {#if initialRegistry.description}
        <p class="prefill-sub">{initialRegistry.description}</p>
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
      <span class="form-label">Name</span>
      <input
        class="form-input"
        type="text"
        bind:value={formName}
        placeholder="My MCP Server"
        required
      />
    </label>

    <fieldset class="form-field fieldset-reset">
      <legend class="form-label">Transport</legend>
      <div class="form-radio-group">
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
        <span class="form-label">URL</span>
        <input
          class="form-input"
          type="url"
          bind:value={formUrl}
          placeholder="https://example.com/mcp"
          required
        />
      </label>
      <label class="form-field">
        <span class="form-label">Auth Header (optional)</span>
        <input
          class="form-input"
          type="text"
          bind:value={formAuthHeader}
          placeholder="Bearer your-token"
        />
      </label>
    {:else}
      <label class="form-field">
        <span class="form-label">Binary Path</span>
        <input
          class="form-input"
          type="text"
          bind:value={formBinaryPath}
          placeholder="npx or /usr/local/bin/mcp-server"
          required
        />
        <span class="form-hint">
          A command name (e.g., npx, uvx) or absolute path to the MCP server binary.
        </span>
      </label>
      <label class="form-field">
        <span class="form-label">Arguments (JSON array, optional)</span>
        <input class="form-input" type="text" bind:value={formArgs} placeholder={argsPlaceholder} />
      </label>
    {/if}

    {#if formError}
      <div class="form-error">{formError}</div>
    {/if}

    {#if testResult}
      <div
        class="banner"
        class:banner--success={testResult.success}
        class:banner--error={!testResult.success}
      >
        {testResult.success ? "✓" : "✗"}
        {testResult.message}
      </div>
    {/if}

    <div class="form-actions form-actions--split">
      <button type="button" class="btn" onclick={handleTest} disabled={testing || !formValid}>
        {testing ? "Testing..." : "Test Connection"}
      </button>
      <div class="actions-right">
        <button type="button" class="btn" onclick={onBack}>Cancel</button>
        <button type="submit" class="btn btn--primary" disabled={submitting || !formValid}>
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

<style>
  .prefill-notice {
    margin-bottom: var(--spacing-lg);
  }
  .prefill-sub {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    display: block;
    margin-top: var(--spacing-xs);
  }

  .server-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .fieldset-reset {
    border: none;
    padding: 0;
    margin: 0;
  }

  .form-actions--split {
    justify-content: space-between;
    border-top: none;
    padding-top: var(--spacing-sm);
    margin-top: var(--spacing-sm);
  }

  .actions-right {
    display: flex;
    gap: var(--spacing-sm);
  }
</style>
