<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { X, Loader2, Smartphone, QrCode } from "lucide-svelte";
  import { toasts } from "$lib/stores/toasts";
  import { ui, accounts } from "$lib/stores";

  let status: "loading" | "ready" | "scanned" | "confirmed" | "error" = $state("loading");
  let qrCodeUrl = $state("");
  let code = $state("");
  let errorMessage = $state("");
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await createSession();
  });

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
  });

  async function createSession() {
    status = "loading";
    try {
      const session = await invoke<{ code: string; qr_code_url: string }>("quick_login_create");
      code = session.code;
      qrCodeUrl = session.qr_code_url;
      status = "ready";
      startPolling();
    } catch (err) {
      status = "error";
      errorMessage = String(err);
    }
  }

  function startPolling() {
    pollInterval = setInterval(async () => {
      try {
        const result = await invoke<string>("quick_login_poll", { code });
        
        if (result === "SCANNED") {
          status = "scanned";
        } else if (result === "CONFIRMED") {
          status = "confirmed";
          if (pollInterval) clearInterval(pollInterval);
          await completeLogin();
        } else if (result === "EXPIRED" || result === "CANCELLED") {
          status = "error";
          errorMessage = `Session ${result.toLowerCase()}`;
          if (pollInterval) clearInterval(pollInterval);
        }
      } catch (err) {
        console.error("Poll error:", err);
      }
    }, 2000);
  }

  async function completeLogin() {
    try {
      const result = await invoke<string>("quick_login_complete", { code });
      // Result format: "userId|username|displayName|cookie"
      const [userId, username, displayName, cookie] = result.split("|");
      
      // Add account via existing flow
      await invoke("add_account", { cookie });
      
      toasts.success(`Logged in as ${displayName}!`);
      await accounts.load();
      close();
    } catch (err) {
      status = "error";
      errorMessage = String(err);
    }
  }

  function close() {
    if (pollInterval) clearInterval(pollInterval);
    ui.closeQuickLogin();
  }

  function retry() {
    createSession();
  }
</script>

<div class="modal-backdrop" onclick={close} onkeydown={() => {}} role="presentation">
  <div
    class="modal"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="modal-header">
      <h2><Smartphone size={20} /> Quick Login</h2>
      <button class="close-btn" onclick={close}>
        <X size={20} />
      </button>
    </div>

    <div class="modal-content">
      {#if status === "loading"}
        <div class="state">
          <Loader2 size={32} class="spinner" />
          <p>Generating QR code...</p>
        </div>
      {:else if status === "error"}
        <div class="state error">
          <p>❌ {errorMessage}</p>
          <button class="btn-primary" onclick={retry}>Try Again</button>
        </div>
      {:else}
        <div class="qr-section">
          {#if qrCodeUrl}
            <img src={qrCodeUrl} alt="QR Code" class="qr-image" />
          {:else}
            <div class="qr-placeholder">
              <QrCode size={100} />
            </div>
          {/if}
        </div>

        <div class="instructions">
          {#if status === "ready"}
            <p><strong>Scan with Roblox App</strong></p>
            <p class="hint">Open the Roblox app and scan this QR code to log in.</p>
          {:else if status === "scanned"}
            <p class="scanned"><Loader2 size={16} class="spinner" /> Code scanned! Waiting for confirmation...</p>
          {:else if status === "confirmed"}
            <p class="success">✅ Login confirmed!</p>
          {/if}
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
  }

  .modal {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 16px;
    width: 360px;
    max-width: 90vw;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
  }

  .modal-header h2 {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 8px;
    transition: all 0.15s;
  }

  .close-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .modal-content {
    padding: 24px;
  }

  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 20px;
    color: var(--color-text-secondary);
  }

  .state.error {
    color: var(--color-error);
  }

  .qr-section {
    display: flex;
    justify-content: center;
    margin-bottom: 20px;
  }

  .qr-image {
    width: 200px;
    height: 200px;
    border-radius: 12px;
    background: white;
    padding: 8px;
  }

  .qr-placeholder {
    width: 200px;
    height: 200px;
    background: var(--color-bg-tertiary);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary);
  }

  .instructions {
    text-align: center;
    color: var(--color-text-secondary);
  }

  .instructions p {
    margin: 0 0 8px;
  }

  .hint {
    font-size: 12px;
    color: var(--color-text-tertiary);
  }

  .scanned {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--color-accent);
  }

  .success {
    color: var(--color-success);
    font-weight: 600;
  }

  .btn-primary {
    padding: 10px 20px;
    background: var(--color-accent);
    border: none;
    border-radius: 8px;
    color: white;
    font-weight: 600;
    cursor: pointer;
  }

  :global(.spinner) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
