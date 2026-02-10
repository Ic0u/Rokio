<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Search, Loader2, Server, Gamepad2, Star } from "lucide-svelte";
  import { ui, accounts, launcher } from "$lib/stores";
  import type { ServerInfo, GameInfo, BrowsedGame, FavoriteGame } from "$lib/types/roblox";

  // Components
  import GamesGrid from "./server-browser/GamesGrid.svelte";
  import FavoritesList from "./server-browser/FavoritesList.svelte";
  import GameHeader from "./server-browser/GameHeader.svelte";
  import ServerList from "./server-browser/ServerList.svelte";

  // State
  type SubTab = "servers" | "games" | "favorites";
  let activeSubTab = $state<SubTab>("servers");

  let placeIdInput = $state("");
  let gameInfo = $state<GameInfo | null>(null);
  let servers = $state<ServerInfo[]>([]);
  let loadingSearch = $state(false);
  let loadingServers = $state(false);
  let error = $state("");
  let nextCursor = $state<string | null>(null);
  
  let selectedAccount = $derived($accounts.accounts.find(a => a.id === $accounts.selectedId));

  // Favorites logic
  let favorites = $state<FavoriteGame[]>([]);

  $effect(() => {
    const saved = localStorage.getItem("rokio_favorites");
    if (saved) {
      try { favorites = JSON.parse(saved); } catch (e) { favorites = []; }
    }
  });

  function saveFavorites() {
    localStorage.setItem("rokio_favorites", JSON.stringify(favorites));
  }

  function isFavorite(placeId: number): boolean {
    return favorites.some(f => f.placeId === placeId);
  }

  function toggleFavorite() {
    if (!gameInfo) return;
    if (isFavorite(gameInfo.placeId)) {
      favorites = favorites.filter(f => f.placeId !== gameInfo!.placeId);
    } else {
      favorites = [...favorites, {
        placeId: gameInfo.placeId,
        universeId: gameInfo.universeId,
        name: gameInfo.name,
        thumbnail: gameInfo.thumbnail
      }];
    }
    saveFavorites();
  }

  function removeFavorite(placeId: number) {
    favorites = favorites.filter(f => f.placeId !== placeId);
    saveFavorites();
  }

  // Server actions
  async function searchGame() {
    if (!placeIdInput.trim()) return;
    
    loadingSearch = true;
    error = "";
    gameInfo = null;
    servers = [];
    nextCursor = null;

    try {
      let placeId = placeIdInput.trim();
      const urlMatch = placeId.match(/games\/(\d+)/);
      if (urlMatch) placeId = urlMatch[1];

      if (!/^\d+$/.test(placeId)) throw new Error("Invalid Place ID");

      const universeId = await invoke<number>("get_universe_id", { placeId: parseInt(placeId) });
      const info = await invoke<GameInfo>("get_game_info", { universeId });
      gameInfo = info;

      await loadServers(parseInt(placeId));
    } catch (e) {
      error = String(e);
    } finally {
      loadingSearch = false;
    }
  }

  async function loadServers(placeId?: number) {
    const pid = placeId ?? gameInfo?.placeId;
    if (!pid) return;

    loadingServers = true;
    try {
      const [serverList, cursor] = await invoke<[ServerInfo[], string | null]>("get_game_servers", { 
        placeId: pid, 
        cursor: nextCursor 
      });
      
      if (nextCursor) {
        servers = [...servers, ...serverList];
      } else {
        servers = serverList;
      }
      nextCursor = cursor;
    } catch (e) {
      console.error("Failed to load servers:", e);
    } finally {
      loadingServers = false;
    }
  }

  async function joinServer(serverId: string, placeId?: number) {
    const pid = placeId ?? gameInfo?.placeId;
    if (!pid || !selectedAccount) {
      error = "Please select an account first";
      return;
    }
    
    try {
      await launcher.launch(selectedAccount.id, pid, serverId);
      error = "";
    } catch (e) {
      error = `Failed to join: ${e}`;
    }
  }

  async function quickJoin(placeId: number) {
    if (!selectedAccount) {
      error = "Please select an account first";
      return;
    }
    
    try {
      await launcher.launch(selectedAccount.id, placeId);
    } catch (e) {
      console.error(e);
      // Optional: show toast/notification
    }
  }

  function handleSelectGame(game: BrowsedGame) {
    placeIdInput = String(game.placeId);
    activeSubTab = "servers";
    searchGame();
  }

  function handleSearchFromFavorite(placeId: number) {
    placeIdInput = String(placeId);
    activeSubTab = "servers";
    searchGame();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") searchGame();
  }
</script>

<div class="browser-layout">
  <!-- Tabs -->
  <div class="tabs-header segmented-control">
    <button class="tab-btn segment" class:active={activeSubTab === "servers"} onclick={() => activeSubTab = "servers"}>
      <Server size={14} /> Servers
    </button>
    <button class="tab-btn segment" class:active={activeSubTab === "favorites"} onclick={() => activeSubTab = "favorites"}>
      <Star size={14} /> Favorites
    </button>
  </div>

  <div class="content-area">
    <!-- SERVERS TAB -->
    {#if activeSubTab === "servers"}
      <div class="search-bar">
        <Search size={14} class="search-icon" />
        <input 
          type="text" 
          placeholder="Enter Place ID or Game URL..." 
          bind:value={placeIdInput} 
          onkeydown={handleKeydown}
        />
        {#if loadingSearch}
          <Loader2 size={14} class="spin search-loader" />
        {/if}
      </div>

      {#if error}
        <div class="error-banner">{error}</div>
      {/if}

      <div class="account-select-row">
        <span class="label">Launch with:</span>
        {#if $accounts.accounts.length > 0}
          <select class="account-select" value={$accounts.selectedId || ""} onchange={(e) => accounts.select(e.currentTarget.value || null)}>
            <option value="">-- Select Account --</option>
            {#each $accounts.accounts as acc}
              <option value={acc.id}>{acc.displayName} (@{acc.username})</option>
            {/each}
          </select>
        {:else}
          <span class="no-accounts">No accounts added</span>
        {/if}
      </div>

      {#if gameInfo}
        <div class="game-section">
          <GameHeader 
            {gameInfo}
            {loadingServers}
            isFavorite={isFavorite(gameInfo.placeId)}
            onToggleFavorite={toggleFavorite}
            onRefresh={() => loadServers()}
          />
          
          <ServerList 
            {servers} 
            loading={loadingServers}
            onJoin={(serverId) => joinServer(serverId)}
            onLoadMore={() => loadServers()}
            hasMore={!!nextCursor}
            {selectedAccount}
            placeId={gameInfo?.placeId}
          />
        </div>
      {:else if !loadingSearch}
        <div class="empty-state">
          <Server size={48} class="icon-muted" />
          <p>Search for a game to view servers</p>
        </div>
      {/if}

    <!-- GAMES TAB -->
    {:else if activeSubTab === "games"}
      <GamesGrid onSelectGame={handleSelectGame} />

    <!-- FAVORITES TAB -->
    {:else if activeSubTab === "favorites"}
      <FavoritesList 
        {favorites}
        {selectedAccount}
        onRemove={removeFavorite}
        onJoin={quickJoin}
        onSearch={handleSearchFromFavorite}
      />
    {/if}
  </div>
</div>

<style>
  .browser-layout {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px;
    gap: 16px;
  }

  /* Tabs - Segmented Control Style */
  .tabs-header.segmented-control {
    display: flex;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
    padding: 3px;
    gap: 2px;
    border: none; /* Override previous border */
    flex-shrink: 0;
  }

  .tab-btn.segment {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    font-weight: 500;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .tab-btn.segment:hover {
    color: var(--color-text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .tab-btn.segment.active {
    background: rgba(255, 255, 255, 0.12);
    color: var(--color-text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  /* Content */
  .content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow: hidden; /* Important for inner scrolling */
    min-height: 0;
  }

  /* Search */
  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    flex-shrink: 0;
  }

  .search-bar input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--color-text-primary);
    font-size: 14px;
    outline: none;
  }

  :global(.search-icon) { color: var(--color-text-tertiary); }
  :global(.search-loader) { color: var(--color-text-tertiary); }

  /* Account Select */
  .account-select-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 4px;
    flex-shrink: 0;
  }

  .label {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .account-select {
    flex: 1;
    padding: 8px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    color: var(--color-text-primary);
    font-size: 13px;
  }

  .game-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow: hidden;
  }

  .error-banner {
    padding: 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);
    border-radius: 8px;
    color: var(--color-error);
    font-size: 13px;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary);
    gap: 16px;
  }



  :global(.spin) { animation: spin 1s linear infinite; }
</style>
