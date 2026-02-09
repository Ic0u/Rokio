<script lang="ts">
  import { X, Loader2, Cookie, Globe } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { accounts, ui } from "$lib/stores";
  import type { RobloxUserData } from "$lib/types";
  import { animate } from "motion";

  type AddMethod = "cookie" | "browser";

  let addMethod = $state<AddMethod>("cookie");
  let cookie = $state("");
  let loading = $state(false);
  let error = $state("");
  let previewUser = $state<RobloxUserData | null>(null);
  let step = $state<"select" | "input" | "preview" | "success">("select");
  let modalRef = $state<HTMLElement | null>(null);

  async function validateCookie() {
    if (!cookie.trim()) {
      error = "Please enter a cookie";
      return;
    }

    loading = true;
    error = "";

    try {
      previewUser = await invoke<RobloxUserData>("validate_cookie", {
        cookie: cookie.trim(),
      });
      step = "preview";
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function confirmAdd() {
    loading = true;
    error = "";

    try {
      await accounts.add(cookie.trim());
      step = "success";
      setTimeout(() => {
        close();
      }, 1500);
    } catch (err) {
      error = String(err);
      step = "input";
    } finally {
      loading = false;
    }
  }

  function selectMethod(method: AddMethod) {
    addMethod = method;
    if (method === "cookie") {
      step = "input";
    } else {
      // Open browser login modal
      close(); // Close add account modal
      ui.openBrowserLogin(); // Open browser login modal
    }
  }

  function close() {
    ui.closeAddAccount();
    // Reset state
    cookie = "";
    error = "";
    previewUser = null;
    step = "select";
    addMethod = "cookie";
  }

  function goBack() {
    if (step === "preview") {
      step = "input";
      previewUser = null;
    } else if (step === "input") {
      step = "select";
      cookie = "";
    }
    error = "";
  }

  // Animate modal on mount
  $effect(() => {
    if (modalRef) {
      animate(modalRef,
        { opacity: [0, 1], scale: [0.95, 1] },
        { duration: 0.2, easing: "ease-out" }
      );
    }
  });
</script>

<div class="modal-backdrop" onclick={close} onkeydown={() => {}} role="presentation">
  <div
    bind:this={modalRef}
    class="modal glass"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
  >
    <div class="modal-header">
      <h2>Add Account</h2>
      <button class="close-btn" onclick={close}>
        <X size={20} />
      </button>
    </div>

    <div class="modal-content">
      {#if step === "select"}
        <div class="method-select">
          <p class="select-hint">Choose how to add your account:</p>

          <div class="method-grid">
            <button class="method-card" onclick={() => selectMethod("cookie")}>
              <div class="method-icon">
                <Cookie size={28} />
              </div>
              <h3>Paste Cookie</h3>
              <p>Paste your .ROBLOSECURITY cookie directly</p>
            </button>

            <button class="method-card" onclick={() => selectMethod("browser")}>
              <div class="method-icon">
                <Globe size={28} />
              </div>
              <h3>Browser Login</h3>
              <p>Login to Roblox in a browser window</p>
            </button>
          </div>

          {#if error}
            <div class="error-message">{error}</div>
          {/if}
        </div>
      {:else if step === "input"}
        <div class="input-step">
          <label for="cookie">Roblox Cookie</label>
          <textarea
            id="cookie"
            bind:value={cookie}
            placeholder="Paste your .ROBLOSECURITY cookie here..."
            rows="4"
          ></textarea>
          <p class="hint">
            You can find this in your browser's developer tools → Application →
            Cookies → .roblox.com
          </p>

          {#if error}
            <div class="error-message">{error}</div>
          {/if}
        </div>
      {:else if step === "preview" && previewUser}
        <div class="preview-step">
          <div class="user-preview">
            {#if previewUser.thumbnail}
              <img
                src={previewUser.thumbnail}
                alt={previewUser.displayName}
                class="avatar"
              />
            {:else}
              <div class="avatar-placeholder">👤</div>
            {/if}
            <div class="user-info">
              <h3>{previewUser.displayName}</h3>
              <p>@{previewUser.name}</p>
              <p class="user-id">ID: {previewUser.id}</p>
            </div>
          </div>

          {#if error}
            <div class="error-message">{error}</div>
          {/if}
        </div>
      {:else if step === "success"}
        <div class="success-step">
          <div class="success-icon">✅</div>
          <h3>Account Added!</h3>
          <p>{previewUser?.displayName} has been added to your vault.</p>
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      {#if step === "select"}
        <button class="btn btn-secondary" onclick={close}>Cancel</button>
      {:else if step === "input"}
        <button class="btn btn-secondary" onclick={goBack}>Back</button>
        <button class="btn btn-primary" onclick={validateCookie} disabled={loading}>
          {#if loading}
            <Loader2 size={16} class="spin" />
            Validating...
          {:else}
            Continue
          {/if}
        </button>
      {:else if step === "preview"}
        <button class="btn btn-secondary" onclick={goBack} disabled={loading}>Back</button>
        <button class="btn btn-primary" onclick={confirmAdd} disabled={loading}>
          {#if loading}
            <Loader2 size={16} class="spin" />
            Adding...
          {:else}
            Add Account
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 0.2s ease-out;
  }

  .modal {
    width: 90%;
    max-width: 520px;
    border-radius: 16px;
    animation: slideUp 0.3s ease-out;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .modal-header h2 {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--color-text-primary);
  }

  .modal-content {
    padding: 24px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 24px;
    border-top: 1px solid var(--color-border);
  }

  /* Method Select Step */
  .select-hint {
    color: var(--color-text-secondary);
    margin: 0 0 16px 0;
  }

  .method-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .method-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    cursor: pointer;
    text-align: center;
    transition: all 0.2s;
    position: relative;
  }

  .method-card:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--color-accent);
  }

  .method-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    background: rgba(59, 130, 246, 0.15);
    border-radius: 14px;
    color: var(--color-accent);
    margin-bottom: 12px;
  }

  .method-card h3 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 4px 0;
  }

  .method-card p {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* Input Step */
  label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 8px;
  }

  textarea {
    width: 100%;
    padding: 12px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    color: var(--color-text-primary);
    font-family: "Monaco", "Consolas", monospace;
    font-size: 12px;
    resize: vertical;
    transition: border-color 0.2s;
  }

  textarea:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .hint {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin-top: 8px;
  }

  .error-message {
    margin-top: 16px;
    padding: 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    color: var(--color-error);
    font-size: 13px;
  }

  /* Preview Step */
  .user-preview {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 12px;
  }

  .avatar {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 28px;
  }

  .user-info h3 {
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 4px 0;
  }

  .user-info p {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .user-id {
    font-size: 12px !important;
    margin-top: 4px !important;
  }

  /* Success Step */
  .success-step {
    text-align: center;
    padding: 24px 0;
  }

  .success-icon {
    font-size: 48px;
    margin-bottom: 16px;
  }

  .success-step h3 {
    font-size: 20px;
    font-weight: 600;
    margin: 0 0 8px 0;
  }

  .success-step p {
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* Buttons */
  .btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 18px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--color-accent-hover);
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-secondary);
  }

  .btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
    color: var(--color-text-primary);
  }

  /* Animations */
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
