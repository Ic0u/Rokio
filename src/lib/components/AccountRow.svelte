<script lang="ts">
  import { Play, Star, Trash2, Square, Edit2, Copy, Eye } from "lucide-svelte";
  import { accounts, launcher, ui } from "$lib/stores";
  import type { Profile } from "$lib/types";

  let { account, isSelected = false }: { account: Profile; isSelected?: boolean } = $props();

  let isRunning = $derived($launcher.instances.some(i => i.accountId === account.id));
  let instance = $derived($launcher.instances.find(i => i.accountId === account.id));

  // Get display name (alias or displayName)
  let displayLabel = $derived(account.alias || account.displayName);

  // Status: offline, studio, online, ingame
  let status = $derived(() => {
    if (!isRunning) return 'offline';
    if (instance?.placeId === 0) return 'studio';
    if (instance?.placeId) return 'ingame';
    return 'online';
  });

  function toggleSelection(e: Event) {
    accounts.select(isSelected ? null : account.id);
  }

  function toggleFavorite(e: MouseEvent) {
    e.stopPropagation();
    accounts.toggleFavorite(account.id);
  }

  function openLaunch(e: MouseEvent) {
    e.stopPropagation();
    ui.openLaunchModal(account);
  }

  function openEdit(e: MouseEvent) {
    e.stopPropagation();
    ui.openEditAccount(account);
  }

  function openInfo(e: MouseEvent) {
    e.stopPropagation();
    ui.openAccountInfo(account);
  }

  async function kill(e: MouseEvent) {
    e.stopPropagation();
    if (instance) await launcher.kill(instance.pid);
  }

  async function remove(e: MouseEvent) {
    e.stopPropagation();
    if (confirm(`Delete ${displayLabel}?`)) {
      await accounts.delete(account.id);
    }
  }

  function copyUsername(e: MouseEvent) {
    e.stopPropagation();
    navigator.clipboard.writeText(account.username);
  }

  // Quick launch on double-click (one-click switching)
  async function quickLaunch() {
    if (isRunning) return;
    await launcher.launch(account.id, 0, 0); // Launch without specific game
  }
</script>

<div 
  class="account-row" 
  class:selected={isSelected} 
  class:running={isRunning}
  ondblclick={quickLaunch}
  role="row"
  tabindex="0"
>
  <!-- Checkbox -->
  <div class="col center">
    <div class="checkbox-wrapper">
      <input type="checkbox" checked={isSelected} onchange={toggleSelection} />
    </div>
  </div>

  <!-- Account Info -->
  <div class="col truncate">
    <div class="account-info">
      {#if account.thumbnail}
        <img src={account.thumbnail} alt="" class="avatar" />
      {:else}
        <div class="avatar-placeholder">?</div>
      {/if}
      <div class="name-col">
        <span class="display-name">{account.displayName}</span>
        <span class="username">@{account.username}</span>
      </div>
    </div>
  </div>

  <!-- Alias -->
  <div class="col truncate">
    <span class="cell-text alias">{account.alias || "—"}</span>
  </div>

  <!-- Description -->
  <div class="col truncate">
    <span class="cell-text description">{account.description || "—"}</span>
  </div>

  <!-- Status -->
  <div class="col">
    {#if status() === 'ingame'}
      <span class="status-badge ingame">InGame</span>
    {:else if status() === 'online'}
      <span class="status-badge online">Online</span>
    {:else if status() === 'studio'}
      <span class="status-badge studio">Studio</span>
    {:else}
      <span class="status-badge offline">Offline</span>
    {/if}
  </div>

  <!-- Last Used -->
  <div class="col">
    <span class="cell-text last-used">
      {#if account.lastPlayedAt > 0}
        {new Date(account.lastPlayedAt).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
      {:else}
        Never
      {/if}
    </span>
  </div>

  <!-- Actions -->
  <div class="col right">
    <div class="actions">
      <!-- Launch/Kill -->
      {#if isRunning}
        <button class="action-btn stop" onclick={kill} title="Stop">
          <Square size={14} />
        </button>
      {:else}
        <button class="action-btn play" onclick={openLaunch} title="Launch">
          <Play size={14} />
        </button>
      {/if}

      <!-- Favorite -->
      <button 
        class="action-btn" 
        class:active={account.isFavorite} 
        onclick={toggleFavorite} 
        title="Favorite"
      >
        <Star size={14} fill={account.isFavorite ? "currentColor" : "none"} />
      </button>

      <!-- View Info -->
      <button class="action-btn" onclick={openInfo} title="View Info">
        <Eye size={14} />
      </button>

      <!-- Edit -->
      <button class="action-btn" onclick={openEdit} title="Edit">
        <Edit2 size={14} />
      </button>

      <!-- Delete -->
      <button class="action-btn delete" onclick={remove} title="Delete">
        <Trash2 size={14} />
      </button>
    </div>
  </div>
</div>

<style>
  .account-row {
    display: grid;
    grid-template-columns: 40px 1.2fr 80px 0.8fr 70px 100px 160px;
    align-items: center;
    border-bottom: 1px solid var(--color-border);
    transition: background 0.1s;
    height: 48px;
  }

  .account-row:hover {
    background: var(--color-bg-secondary);
  }

  .account-row.selected {
    background: rgba(255, 69, 58, 0.1);
    border-color: rgba(255, 69, 58, 0.2);
  }

  .col {
    padding: 0 12px;
    color: var(--color-text-secondary);
    font-size: 13px;
    white-space: nowrap;
  }

  .col.truncate {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .col.center {
    display: flex;
    justify-content: center;
  }
  
  .col.right {
    display: flex;
    justify-content: flex-end;
    padding-right: 16px;
  }

  /* Components */
  .account-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--color-bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary);
    font-size: 12px;
  }

  .name-col {
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .display-name {
    color: var(--color-text-primary);
    font-weight: 500;
  }

  .username {
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  /* Status Badge - Clean pill style */
  .status-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 500;
    border: 1px solid transparent;
  }

  .status-badge.offline {
    background: rgba(255, 69, 58, 0.15);
    color: #ff6b6b;
    border-color: rgba(255, 69, 58, 0.3);
  }

  .status-badge.studio {
    background: rgba(255, 159, 10, 0.15);
    color: #ffb347;
    border-color: rgba(255, 159, 10, 0.3);
  }

  .status-badge.online {
    background: rgba(10, 132, 255, 0.15);
    color: #5dade2;
    border-color: rgba(10, 132, 255, 0.3);
  }

  .status-badge.ingame {
    background: rgba(48, 209, 88, 0.15);
    color: #58d68d;
    border-color: rgba(48, 209, 88, 0.3);
  }

  /* Actions */
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 2px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    color: var(--color-text-tertiary);
    border-radius: 5px;
    cursor: pointer;
    transition: all 0.15s;
    flex-shrink: 0;
  }

  .action-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .action-btn.active {
    color: var(--color-warning);
  }

  .action-btn.play {
    color: var(--color-success);
    background: rgba(48, 209, 88, 0.1);
  }
  
  .action-btn.play:hover {
    background: rgba(48, 209, 88, 0.2);
  }

  .action-btn.stop {
    color: var(--color-warning);
    background: rgba(255, 214, 10, 0.1);
  }

  .action-btn.delete:hover {
    color: var(--color-error);
    background: rgba(255, 69, 58, 0.1);
  }
</style>
