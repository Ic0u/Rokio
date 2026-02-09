<script lang="ts">
  import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from "lucide-svelte";
  import { toasts } from "$lib/stores/toasts";
  import { fade, fly } from "svelte/transition";

  const icons = {
    success: CheckCircle,
    error: AlertCircle,
    warning: AlertTriangle,
    info: Info,
  };
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div
      class="toast {toast.type}"
      in:fly={{ x: 100, duration: 200 }}
      out:fade={{ duration: 150 }}
    >
      <svelte:component this={icons[toast.type]} size={18} />
      <span class="message">{toast.message}</span>
      <button class="close" onclick={() => toasts.dismiss(toast.id)}>
        <X size={14} />
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 20px;
    right: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 9999;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    color: var(--color-text-primary);
    font-size: 13px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    pointer-events: auto;
    max-width: 320px;
  }

  .toast.success {
    border-color: var(--color-success);
    color: var(--color-success);
  }

  .toast.error {
    border-color: var(--color-error);
    color: var(--color-error);
  }

  .toast.warning {
    border-color: var(--color-warning);
    color: var(--color-warning);
  }

  .toast.info {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .message {
    flex: 1;
    color: var(--color-text-primary);
  }

  .close {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    background: transparent;
    border: none;
    color: var(--color-text-tertiary);
    cursor: pointer;
    border-radius: 4px;
    transition: color 0.15s;
  }

  .close:hover {
    color: var(--color-text-primary);
  }
</style>
