<script lang="ts">
  interface Props {
    open: boolean;
    title: string;
    detail?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    loading?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let {
    open,
    title,
    detail,
    confirmLabel = "Delete",
    cancelLabel = "Cancel",
    loading = false,
    onconfirm,
    oncancel,
  }: Props = $props();

  let cancelBtn: HTMLButtonElement | undefined = $state();

  $effect(() => {
    if (open && cancelBtn) {
      cancelBtn.focus();
    }
  });
</script>

{#if open}
  <div
    class="confirm-overlay"
    role="alertdialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    onkeydown={(e) => {
      if (e.key === "Escape") oncancel();
      if (e.key === "Tab") {
        const dialog = e.currentTarget as HTMLElement;
        const focusable = dialog.querySelectorAll<HTMLElement>("button:not(:disabled)");
        if (focusable.length === 0) return;
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }}
  >
    <div class="confirm-dialog">
      <p class="confirm-title">{title}</p>
      {#if detail}
        <p class="confirm-detail">{detail}</p>
      {/if}
      <div class="confirm-actions">
        <button
          bind:this={cancelBtn}
          class="confirm-btn cancel"
          onclick={oncancel}
          disabled={loading}
        >
          {cancelLabel}
        </button>
        <button class="confirm-btn delete" onclick={onconfirm} disabled={loading}>
          {loading ? "Deleting…" : confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .confirm-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.15);
    animation: fadeIn 100ms ease;
  }

  .confirm-dialog {
    background: var(--color-bg-primary);
    border-radius: 16px;
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.12),
      0 2px 8px rgba(0, 0, 0, 0.06);
    padding: 24px 28px 20px;
    min-width: 340px;
    max-width: 420px;
    animation: scaleIn 120ms ease;
  }

  .confirm-title {
    margin: 0 0 6px;
    font-family: var(--font-sans);
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    line-height: 1.4;
  }

  .confirm-detail {
    margin: 0 0 20px;
    font-family: var(--font-sans);
    font-size: 14px;
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .confirm-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .confirm-btn {
    padding: 7px 18px;
    border: 1px solid var(--color-border-primary);
    border-radius: 9999px;
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 120ms ease;
    line-height: 1.3;
  }

  .confirm-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .confirm-btn.cancel {
    background: transparent;
    color: var(--color-text-primary);
  }

  .confirm-btn.cancel:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .confirm-btn.delete {
    background: #b91c1c;
    border-color: #b91c1c;
    color: #fff;
  }

  .confirm-btn.delete:hover:not(:disabled) {
    background: #991b1b;
    border-color: #991b1b;
  }

  .confirm-btn:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  :global([data-theme="dark"]) .confirm-dialog {
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.4),
      0 2px 8px rgba(0, 0, 0, 0.2);
  }
</style>
