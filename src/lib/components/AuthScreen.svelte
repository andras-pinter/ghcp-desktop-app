<script lang="ts">
  // Auth screen — shown when user is not authenticated
</script>

<div class="auth-screen">
  <div class="auth-card">
    <div class="auth-logo" aria-hidden="true">🐙</div>
    <h1 class="auth-title">Copilot Desktop</h1>
    <p class="auth-subtitle">Sign in with GitHub to get started.</p>

    <button class="auth-btn" aria-label="Sign in with GitHub">Sign in with GitHub →</button>

    <div class="auth-divider">
      <span>or enter code</span>
    </div>

    <div class="device-code">
      <p class="code-label">Your code:</p>
      <p class="code-value">— — — —</p>
      <button class="copy-code-btn" aria-label="Copy code">Copy Code 📋</button>
    </div>

    <p class="auth-note">Requires an active Copilot subscription</p>
  </div>
</div>

<style>
  .auth-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--color-bg-primary);
  }

  .auth-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--spacing-2xl);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    max-width: 380px;
    width: 100%;
    gap: var(--spacing-md);
  }

  .auth-logo {
    font-size: 48px;
  }

  .auth-title {
    font-size: var(--font-size-xl);
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .auth-subtitle {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    text-align: center;
  }

  .auth-btn {
    width: 100%;
    padding: var(--spacing-md);
    background: var(--color-accent);
    color: var(--color-text-inverse);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--font-size-base);
    font-weight: 600;
    transition: background var(--transition-fast);
  }

  .auth-btn:hover {
    background: var(--color-accent-hover);
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
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  .code-value {
    font-family: var(--font-mono);
    font-size: var(--font-size-lg);
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--color-text-primary);
  }

  .copy-code-btn {
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all var(--transition-fast);
  }

  .copy-code-btn:hover {
    background: var(--color-bg-hover);
  }

  .auth-note {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    text-align: center;
    margin-top: var(--spacing-sm);
  }
</style>
