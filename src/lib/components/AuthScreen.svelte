<script lang="ts">
  import { startDeviceFlow, pollAuthToken } from "$lib/stores/auth.svelte";
  import type { DeviceCodeResponse } from "$lib/types/auth";
  import { open } from "@tauri-apps/plugin-shell";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  let deviceCode = $state<DeviceCodeResponse | null>(null);
  let error = $state<string | null>(null);
  let polling = $state(false);
  let copied = $state(false);

  async function handleSignIn() {
    error = null;
    try {
      deviceCode = await startDeviceFlow();
      // Copy device code to clipboard for easy pasting
      await writeText(deviceCode.user_code);
      copied = true;
      setTimeout(() => (copied = false), 2000);
      // Open verification URL in browser
      await open(deviceCode.verification_uri);
      // Start polling
      startPolling();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function startPolling() {
    if (!deviceCode) return;
    polling = true;

    const interval = Math.max((deviceCode.interval || 5) * 1000, 1000);
    const expiresAt = Date.now() + deviceCode.expires_in * 1000;

    while (Date.now() < expiresAt && polling) {
      await new Promise((r) => setTimeout(r, interval));
      if (!polling) break;

      try {
        await pollAuthToken(deviceCode.device_code);
        // Success — auth store will update, App.svelte will transition
        polling = false;
        return;
      } catch (e) {
        const msg = String(e).toLowerCase();
        if (msg.includes("authorization pending") || msg.includes("authorization_pending")) {
          continue; // Keep polling
        } else if (msg.includes("slow down") || msg.includes("slow_down")) {
          await new Promise((r) => setTimeout(r, 5000)); // Extra wait
          continue;
        } else if (msg.includes("expired") || msg.includes("expired_token")) {
          error = "Device code expired. Please try again.";
          polling = false;
          deviceCode = null;
          return;
        } else {
          error = String(e);
          polling = false;
          return;
        }
      }
    }

    if (polling) {
      error = "Device code expired. Please try again.";
      polling = false;
      deviceCode = null;
    }
  }

  async function handleCopyCode() {
    if (!deviceCode) return;
    try {
      await writeText(deviceCode.user_code);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      // Fallback: navigator clipboard (may not work in all webviews)
      try {
        await navigator.clipboard.writeText(deviceCode.user_code);
        copied = true;
        setTimeout(() => (copied = false), 2000);
      } catch {
        // Ignore
      }
    }
  }
</script>

<div class="auth-screen">
  <div class="auth-card">
    <div class="auth-mark" aria-hidden="true">
      <svg width="36" height="36" viewBox="0 0 16 16" fill="currentColor">
        <path
          d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z"
        />
      </svg>
    </div>

    <div class="auth-header">
      <p class="auth-eyebrow">Welcome to</p>
      <h1 class="auth-title">Chuck</h1>
    </div>

    <p class="auth-tagline">First one past the sound barrier.</p>
    <p class="auth-subtitle">You'd be the second.</p>

    <button
      class="auth-btn"
      onclick={handleSignIn}
      disabled={polling}
      aria-label="Sign in with GitHub"
    >
      <span>{polling ? "Waiting for authorization..." : "Sign in with GitHub"}</span>
      {#if !polling}
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M6.5 3.5L11 8l-4.5 4.5" stroke="currentColor" stroke-width="1.5" fill="none" />
        </svg>
      {/if}
    </button>

    <div class="auth-divider">
      <span>or enter code manually</span>
    </div>

    <div class="device-code">
      <p class="code-label">Your device code</p>
      <p class="code-value">{deviceCode ? deviceCode.user_code : "— — — —"}</p>
      <button
        class="copy-code-btn"
        onclick={handleCopyCode}
        disabled={!deviceCode}
        aria-label="Copy code"
      >
        <svg
          width="13"
          height="13"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <rect x="5" y="5" width="9" height="9" rx="1.5" />
          <path d="M11 5V3.5A1.5 1.5 0 009.5 2h-6A1.5 1.5 0 002 3.5v6A1.5 1.5 0 003.5 11H5" />
        </svg>
        <span>{copied ? "Copied!" : "Copy Code"}</span>
      </button>
    </div>

    {#if polling}
      <div class="polling-indicator">
        <div class="spinner"></div>
        <span>Waiting for authorization...</span>
      </div>
    {/if}

    {#if error}
      <p class="auth-error">{error}</p>
    {/if}

    <p class="auth-note">Requires an active GitHub Copilot subscription</p>
  </div>
</div>

<style>
  .auth-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--color-bg-primary);
    animation: fadeIn 500ms ease both;
  }

  .auth-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--spacing-3xl) var(--spacing-2xl);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-lg);
    max-width: 380px;
    width: 100%;
    gap: var(--spacing-lg);
    animation: scaleIn 500ms ease both;
    animation-delay: 100ms;
  }

  .auth-mark {
    color: var(--color-text-primary);
    opacity: 0.85;
  }

  .auth-header {
    text-align: center;
  }

  .auth-eyebrow {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: var(--spacing-xs);
  }

  .auth-title {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-xl);
    font-weight: 400;
    color: var(--color-text-primary);
    letter-spacing: var(--letter-spacing-tight);
    line-height: var(--line-height-tight);
  }

  .auth-tagline {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    text-align: center;
    line-height: var(--line-height-relaxed);
    margin-bottom: var(--spacing-sm);
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .auth-subtitle {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-xl);
    color: var(--color-accent);
    text-align: center;
    line-height: var(--line-height-tight);
  }

  .auth-btn {
    width: 100%;
    padding: var(--spacing-md) var(--spacing-xl);
    background: var(--color-accent);
    color: var(--color-text-inverse);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    font-family: var(--font-sans);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    transition: all var(--transition-normal);
    letter-spacing: var(--letter-spacing-normal);
  }

  .auth-btn:hover {
    background: var(--color-accent-hover);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .auth-btn:active {
    transform: translateY(0);
  }

  .auth-divider {
    width: 100%;
    text-align: center;
    border-bottom: 1px solid var(--color-border-primary);
    line-height: 0;
  }

  .auth-divider span {
    background: var(--color-bg-secondary);
    padding: 0 var(--spacing-md);
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .device-code {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .code-label {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .code-value {
    font-family: var(--font-mono);
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-bold);
    letter-spacing: 0.12em;
    color: var(--color-text-primary);
  }

  .copy-code-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-md);
    background: transparent;
    color: var(--color-accent-copper);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    transition: all var(--transition-fast);
  }

  .copy-code-btn:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-accent-copper);
  }

  .auth-note {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    text-align: center;
    margin-top: var(--spacing-xs);
  }

  .polling-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .auth-error {
    font-size: var(--font-size-sm);
    color: var(--color-error, #dc2626);
    text-align: center;
    max-width: 100%;
    word-break: break-word;
  }

  .auth-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    transform: none;
  }
</style>
