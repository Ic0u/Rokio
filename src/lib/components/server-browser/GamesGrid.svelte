<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Users, Loader2 } from "lucide-svelte";
  import { onMount } from "svelte";
  import type { BrowsedGame } from "$lib/types/roblox";

  let { onSelectGame } = $props<{ 
    onSelectGame: (game: BrowsedGame) => void 
  }>();

  let games = $state<BrowsedGame[]>([]);
  let loading = $state(true);
  let error = $state("");

  async function loadGames() {
    loading = true;
    error = "";
    try {
      console.log("Calling get_popular_games...");
      const result = await invoke<Array<any>>("get_popular_games");
      console.log("Got games:", result.length);
      
      // Map basic game info first so list appears instantly
      games = result.map(g => ({
        universeId: g.universeId,
        placeId: g.placeId,
        name: g.name,
        playerCount: g.playerCount,
        thumbnail: undefined // will fetch async
      }));

      // Fetch thumbnails client-side (non-blocking)
      const universeIds = games.map(g => g.universeId);
      fetchThumbnails(universeIds);

    } catch (e) {
      console.error("Failed to load games:", e);
      error = `Failed to load games: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function fetchThumbnails(universeIds: number[]) {
    if (universeIds.length === 0) return;
    
    try {
      const url = `https://thumbnails.roblox.com/v1/games/icons?universeIds=${universeIds.join(",")}&size=420x420&format=Png&isCircular=false`;
      const res = await fetch(url);
      const data = await res.json();
      
      if (data.data) {
        const iconMap = new Map(data.data.map((d: any) => [d.targetId, d.imageUrl]));
        games = games.map(g => ({
          ...g,
          thumbnail: (iconMap.get(g.universeId) as string | undefined) || g.thumbnail
        }));
      }
    } catch (err) {
      console.warn("Failed to fetch thumbnails:", err);
    }
  }

  // Use onMount instead of $effect to prevent infinite loops
  onMount(() => {
    loadGames();
  });
</script>

<div class="games-container">
  {#if error}
    <div class="error-state">
      <p>{error}</p>
      <button class="retry-btn" onclick={loadGames}>Retry</button>
    </div>
  {:else if loading && games.length === 0}
    <div class="loading-state">
      <Loader2 size={32} class="spin" />
      <p>Loading games...</p>
    </div>
  {:else}
    <div class="games-grid">
      {#each games as game (game.placeId)}
        <button class="game-tile" onclick={() => onSelectGame(game)} title={game.name}>
          <div class="thumb-wrapper">
            {#if game.thumbnail}
              <img src={game.thumbnail} alt={game.name} loading="lazy" />
            {:else}
              <div class="placeholder">🎮</div>
            {/if}
            <div class="player-badge">
              <Users size={10} />
              <span>{(game.playerCount / 1000).toFixed(1)}k</span>
            </div>
          </div>
          <div class="info">
            <span class="name">{game.name}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .games-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 0 4px 4px 0; /* Slight padding for scrollbar space */
  }

  /* Header removed as requested */

  .games-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); /* Increased from 160px */
    gap: 16px; /* Increased gap */
    overflow-y: auto;
    padding: 4px 6px; /* balanced padding */
  }

  .game-tile {
    display: flex;
    flex-direction: column;
    background: transparent;
    border: none;
    border-radius: 16px; /* More rounded */
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s cubic-bezier(0.2, 0.8, 0.2, 1);
    padding: 0;
    text-align: left;
    font-family: inherit;
    position: relative;
    will-change: transform;
  }

  .game-tile:hover {
    transform: translateY(-6px);
  }

  .thumb-wrapper {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 16px; /* Match tile radius */
    overflow: hidden;
    background: var(--color-bg-secondary);
    position: relative;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    transition: all 0.2s;
  }
  
  .game-tile:hover .thumb-wrapper {
    box-shadow: 0 12px 24px rgba(0,0,0,0.25);
  }

  .thumb-wrapper img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.4s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .game-tile:hover .thumb-wrapper img {
    transform: scale(1.08); /* Smoother zoom */
  }

  .player-badge {
    position: absolute;
    bottom: 6px;
    right: 6px;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    padding: 3px 6px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    gap: 4px;
    color: white;
    font-size: 10px;
    font-weight: 500;
  }

  .placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 32px;
    color: var(--color-text-tertiary);
  }

  .info {
    padding: 8px 2px;
  }

  .name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
    opacity: 0.9;
    transition: opacity 0.2s;
  }

  .game-tile:hover .name {
    opacity: 1;
    color: white;
  }

  .loading-state, .error-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--color-text-secondary);
    min-height: 200px;
  }

  .retry-btn {
    padding: 8px 16px;
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
  }

  :global(.spin) { animation: spin 1s linear infinite; }
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
