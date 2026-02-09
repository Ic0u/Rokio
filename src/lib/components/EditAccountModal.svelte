<script lang="ts">
  import { X, Save } from "lucide-svelte";
  import { accounts, ui } from "$lib/stores";
  import { toasts } from "$lib/stores/toasts";
  import type { Profile } from "$lib/types";

  let { account }: { account: Profile } = $props();

  let alias = $state(account.alias || "");
  let description = $state(account.description || "");
  let saving = $state(false);

  async function save() {
    saving = true;
    try {
      const updated: Profile = {
        ...account,
        alias,
        description,
      };
      await accounts.update(updated);
      toasts.success("Account updated!");
      close();
    } catch (err) {
      toasts.error(String(err));
    } finally {
      saving = false;
    }
  }

  function close() {
    ui.closeEditAccount();
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
      <h2>Edit Account</h2>
      <button class="close-btn" onclick={close}>
        <X size={20} />
      </button>
    </div>

    <div class="modal-content">
      <!-- Account Preview -->
      <div class="account-preview">
        {#if account.thumbnail}
          <img src={account.thumbnail} alt={account.displayName} class="avatar" />
        {:else}
          <div class="avatar-placeholder">?</div>
        {/if}
        <div class="account-info">
          <h3>{account.displayName}</h3>
          <p>@{account.username}</p>
        </div>
      </div>

      <!-- Alias Input -->
      <div class="input-group">
        <label for="alias">Alias (Custom Name)</label>
        <input
          type="text"
          id="alias"
          bind:value={alias}
          placeholder="e.g., Main Account, Alt 1..."
        />
        <span class="hint">Leave empty to use display name</span>
      </div>

      <!-- Description Input -->
      <div class="input-group">
        <label for="description">Description</label>
        <textarea
          id="description"
          bind:value={description}
          placeholder="Add notes about this account..."
          rows="3"
        ></textarea>
      </div>
    </div>

    <div class="modal-footer">
      <button class="btn btn-secondary" onclick={close}>Cancel</button>
      <button class="btn btn-primary" onclick={save} disabled={saving}>
        <Save size={16} />
        {saving ? "Saving..." : "Save"}
      </button>
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
    width: 420px;
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
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--color-border);
  }

  /* Account Preview */
  .account-preview {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: var(--color-bg-tertiary);
    border-radius: 10px;
  }

  .avatar {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--color-bg-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary);
    font-size: 18px;
  }

  .account-info h3 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }

  .account-info p {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin: 2px 0 0 0;
  }

  /* Input Group */
  .input-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  input, textarea {
    padding: 10px 12px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    color: var(--color-text-primary);
    font-size: 13px;
    font-family: inherit;
    transition: border-color 0.2s;
    resize: none;
  }

  input:focus, textarea:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  input::placeholder, textarea::placeholder {
    color: var(--color-text-tertiary);
  }

  .hint {
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  /* Buttons */
  .btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 16px;
    border: none;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
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
    filter: brightness(1.1);
  }

  .btn-secondary {
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-border);
    color: var(--color-text-primary);
  }
</style>
