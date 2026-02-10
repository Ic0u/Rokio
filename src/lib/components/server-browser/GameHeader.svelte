<script lang="ts">
  import { Star, RefreshCw, Users } from "lucide-svelte";
  import type { GameInfo } from "$lib/types/roblox";

  let { gameInfo, loadingServers, isFavorite, onToggleFavorite, onRefresh } = $props<{
    gameInfo: GameInfo;
    loadingServers: boolean;
    isFavorite: boolean;
    onToggleFavorite: () => void;
    onRefresh: () => void;
  }>();
</script>

<div class="game-header">
  <div class="thumb-wrapper">
    {#if gameInfo.thumbnail}
      <img src={gameInfo.thumbnail} alt={gameInfo.name} />
    {:else}
      <div class="placeholder">🎮</div>
    {/if}
  </div>
  
  <div class="details">
    <h3>{gameInfo.name}</h3>
    <p class="creator">by {gameInfo.creatorName}</p>
    <div class="stats">
      <span class="stat">
        <Users size={12} />
        {gameInfo.playing.toLocaleString()} playing
      </span>
      <span class="stat separator">•</span>
      <span class="stat">{gameInfo.visits.toLocaleString()} visits</span>
    </div>
  </div>

  <div class="actions">
    <button class="action-btn" class:active={isFavorite} onclick={onToggleFavorite} title={isFavorite ? "Remove from favorites" : "Add to favorites"}>
      <Star size={18} fill={isFavorite ? "gold" : "none"} class={isFavorite ? "star-filled" : ""} />
    </button>
    <button class="action-btn" onclick={onRefresh} disabled={loadingServers} title="Refresh Servers">
      <RefreshCw size={18} class={loadingServers ? "spin" : ""} />
    </button>
  </div>
</div>

<style>
  .game-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 12px;
  }

  .thumb-wrapper {
    width: 64px;
    height: 64px;
    border-radius: 10px;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
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
    font-size: 28px;
    color: var(--color-text-tertiary);
  }

  .details {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .details h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-text-primary);
  }

  .creator {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-tertiary);
  }

  .stats {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .separator { color: var(--color-border); }

  .actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
    border-color: var(--color-border-hover);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.active {
    border-color: gold;
    background: rgba(255, 215, 0, 0.1);
  }



  :global(.spin) { animation: spin 1s linear infinite; }
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
