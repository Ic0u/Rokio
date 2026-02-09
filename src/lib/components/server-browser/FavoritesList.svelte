<script lang="ts">
  import { Play, Server, Star, X } from "lucide-svelte";
  import type { FavoriteGame } from "$lib/types/roblox";

  let { favorites, selectedAccount, onRemove, onJoin, onSearch } = $props<{
    favorites: FavoriteGame[];
    selectedAccount: any;
    onRemove: (placeId: number) => void;
    onJoin: (placeId: number) => void;
    onSearch: (placeId: number) => void;
  }>();
</script>

<div class="favorites-container">
  {#if favorites.length === 0}
    <div class="empty-state">
      <Star size={40} class="icon-muted" />
      <h3>No favorite games yet</h3>
      <p class="hint">Star a game from the Servers tab to add it here</p>
    </div>
  {:else}
    <div class="list-header">
      <Star size={16} class="icon-accent" />
      <span>Your Favorites ({favorites.length})</span>
    </div>
    <div class="favorites-list">
      {#each favorites as fav (fav.placeId)}
        <div class="favorite-row">
          <div class="thumb-wrapper">
            {#if fav.thumbnail}
              <img src={fav.thumbnail} alt={fav.name} />
            {:else}
              <div class="placeholder">🎮</div>
            {/if}
          </div>
          
          <div class="details">
            <span class="name" title={fav.name}>{fav.name}</span>
            <span class="id">ID: {fav.placeId}</span>
          </div>

          <div class="actions">
            <button class="action-btn accent" onclick={() => onJoin(fav.placeId)} title="Quick Join">
              <Play size={14} />
            </button>
            <button class="action-btn secondary" onclick={() => onSearch(fav.placeId)} title="Browse Servers">
              <Server size={14} />
            </button>
            <button class="action-btn danger" onclick={() => onRemove(fav.placeId)} title="Remove from favorites">
              <X size={14} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .favorites-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 4px;
    gap: 12px;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: var(--color-text-secondary);
  }

  .empty-state h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
  }

  .hint {
    font-size: 13px;
    color: var(--color-text-tertiary);
    margin: 0;
  }

  .icon-muted { opacity: 0.2; }
  .icon-accent { color: var(--color-accent); }

  .list-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-border);
    font-size: 14px;
    font-weight: 600;
  }

  .favorites-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow-y: auto;
    padding-right: 4px;
  }

  .favorite-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    transition: all 0.2s;
  }

  .favorite-row:hover {
    background: var(--color-bg-tertiary);
    border-color: var(--color-border-hover);
  }

  .thumb-wrapper {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--color-bg-primary);
  }

  .thumb-wrapper img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
  }

  .details {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .id {
    font-size: 11px;
    color: var(--color-text-tertiary);
    font-family: monospace;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px;
    border-radius: 6px;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn.accent {
    background: var(--color-accent);
    color: white;
  }
  
  .action-btn.accent:hover {
    filter: brightness(1.1);
  }

  .action-btn.accent:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.secondary {
    background: transparent;
    border-color: var(--color-border);
    color: var(--color-text-secondary);
  }

  .action-btn.secondary:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .action-btn.danger {
    background: transparent;
    color: var(--color-text-tertiary);
  }

  .action-btn.danger:hover {
    background: rgba(239, 68, 68, 0.1);
    color: var(--color-error);
  }
</style>
