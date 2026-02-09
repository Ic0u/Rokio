<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X, Globe, Loader2, CheckCircle } from "lucide-svelte";
  import { ui, accounts } from "$lib/stores";
  import { toasts } from "$lib/stores/toasts";

  let status = $state<"ready" | "waiting" | "success" | "error">("ready");
  let errorMsg = $state("");
  let pollInterval: number | null = null;

  async function startLogin() {
    status = "waiting";
    try {
      // Open embedded webview for Roblox login
      await invoke("browser_login_open");
      
      // Start polling for cookie
      pollInterval = setInterval(checkForCookie, 1000) as unknown as number;
    } catch (e) {
      errorMsg = String(e);
      status = "error";
    }
  }

  async function checkForCookie() {
    try {
      const cookie = await invoke<string | null>("browser_login_check");
      
      if (cookie) {
        // Stop polling
        if (pollInterval) clearInterval(pollInterval);
        
        // Add account with extracted cookie
        await accounts.add(cookie);
        status = "success";
        toasts.success("Account added successfully!");
        
        // Close modal after delay
        setTimeout(() => ui.closeBrowserLogin(), 1500);
      }
    } catch (e) {
      // Window might be closed
      if (pollInterval) clearInterval(pollInterval);
      status = "ready";
    }
  }

  function close() {
    if (pollInterval) clearInterval(pollInterval);
    invoke("browser_login_close").catch(() => {});
    ui.closeBrowserLogin();
  }
</script>

<div class="modal-backdrop" onclick={close}>
  <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
    <div class="modal-header">
      <h2><Globe size={16} /> Browser Login</h2>
      <button class="close-btn" onclick={close}>
        <X size={16} />
      </button>
    </div>

    <div class="modal-content">
      {#if status === "success"}
        <div class="state success">
          <CheckCircle size={36} />
          <p>Account added!</p>
        </div>
      {:else if status === "error"}
        <div class="state error">
          <p>{errorMsg}</p>
          <button class="btn-primary" onclick={() => status = "ready"}>Try Again</button>
        </div>
      {:else if status === "waiting"}
        <div class="state waiting">
          <Loader2 size={36} class="spin" />
          <p>Waiting for login...</p>
          <span class="hint">Log in to Roblox in the browser window</span>
          <span class="hint">Cookie will be extracted automatically</span>
        </div>
      {:else}
        <div class="info">
          <p>A browser window will open for you to log in to Roblox.</p>
          <p class="hint">Your cookie will be extracted automatically after login.</p>
          <button class="btn-primary full" onclick={startLogin}>
            <Globe size={14} />
            Open Roblox Login
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    width: 320px;
    max-width: 90vw;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--color-border);
  }

  .modal-header h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 6px;
  }

  .close-btn:hover {
    background: var(--color-bg-tertiary);
  }

  .modal-content {
    padding: 16px;
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .info p {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-primary);
    line-height: 1.5;
  }

  .hint {
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 20px;
    text-align: center;
  }

  .state p {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
  }

  .state.success { color: var(--color-success); }
  .state.error { color: var(--color-error); }
  .state.waiting { color: var(--color-text-secondary); }

  .btn-primary {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 14px;
    background: var(--color-accent);
    border: none;
    border-radius: 8px;
    color: #fff;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-primary.full {
    width: 100%;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
