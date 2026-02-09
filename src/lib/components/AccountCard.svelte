<script lang="ts">
  import { Play, Star, Trash2, Square } from "lucide-svelte";
  import { accounts, launcher, ui } from "$lib/stores";
  import type { Profile } from "$lib/types";

  let { account, isSelected = false }: { account: Profile; isSelected?: boolean } = $props();

  let isRunning = $derived($launcher.instances.some(i => i.accountId === account.id));
  let instance = $derived($launcher.instances.find(i => i.accountId === account.id));

  function select() {
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

  async function kill(e: MouseEvent) {
    e.stopPropagation();
    if (instance) await launcher.kill(instance.pid);
  }

  async function remove(e: MouseEvent) {
    e.stopPropagation();
    if (confirm(`Delete ${account.displayName}?`)) {
      await accounts.delete(account.id);
    }
  }
</script>

<div
  class="card"
  class:selected={isSelected}
  class:running={isRunning}
  onclick={select}
  onkeydown={(e) => e.key === "Enter" && select()}
  role="button"
  tabindex="0"
>
  {#if isRunning}
    <span class="badge online">Online</span>
  {/if}

  <div class="header">
    {#if account.thumbnail}
      <img src={account.thumbnail} alt="" class="avatar" />
    {:else}
      <div class="avatar-placeholder">?</div>
    {/if}

    <div class="info">
      <span class="name">{account.displayName}</span>
      <span class="username">@{account.username}</span>
    </div>
  </div>

  <div class="actions">
    <button class="action" class:favorited={account.isFavorite} onclick={toggleFavorite} title="Favorite">
      <Star size={14} fill={account.isFavorite ? "currentColor" : "none"} />
    </button>
    {#if isRunning}
      <button class="action stop" onclick={kill} title="Stop">
        <Square size={14} />
      </button>
    {:else}
      <button class="action play" onclick={openLaunch} title="Launch">
        <Play size={14} />
      </button>
    {/if}
    <button class="action delete" onclick={remove} title="Delete">
      <Trash2 size={14} />
    </button>
  </div>
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.15s;
    position: relative;
    text-align: left;
  }

  .card:hover {
    border-color: var(--color-border-hover);
  }

  .card.selected {
    border-color: var(--color-accent);
  }

  .card.running {
    border-color: var(--color-success);
  }

  .badge {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge.online {
    background: rgba(48, 209, 88, 0.2);
    color: var(--color-success);
  }

  .header {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--color-bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary);
    font-size: 14px;
  }

  .info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-weight: 600;
    font-size: 14px;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .username {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .actions {
    display: flex;
    gap: 6px;
    padding-top: 8px;
    border-top: 1px solid var(--color-separator);
  }

  .action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .action:hover {
    color: var(--color-text-primary);
  }

  .action.favorited {
    color: #ffd60a;
  }

  .action.play:hover {
    background: rgba(48, 209, 88, 0.2);
    color: var(--color-success);
  }

  .action.stop:hover {
    background: rgba(255, 214, 10, 0.2);
    color: var(--color-warning);
  }

  .action.delete:hover {
    background: rgba(255, 69, 58, 0.2);
    color: var(--color-error);
  }
</style>
