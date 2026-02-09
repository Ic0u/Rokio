<script lang="ts">
  import { Star, Play, Trash2, Plus, Loader2 } from "lucide-svelte";
  import { favorites, type FavoriteGame } from "$lib/stores/favorites";
  import { launcher, ui } from "$lib/stores";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let loading = $state(false);
  let games = $state<FavoriteGame[]>([]);
  let showAddModal = $state(false);
  let newPlaceId = $state("");
  let newGameName = $state("");
  let addLoading = $state(false);
  let error = $state("");

  // Selected account for launching
  let selectedAccountId = $state("");

  onMount(async () => {
    await favorites.load();
    favorites.subscribe((state) => {
      games = state.games;
      loading = state.loading;
    });
  });

  async function addGame() {
    if (!newPlaceId || !newGameName) {
      error = "Please enter both Place ID and Game Name";
      return;
    }

    addLoading = true;
    error = "";

    try {
      // Fetch game thumbnail
      const gameInfo = await invoke<{ name: string; thumbnail?: string }>(
        "get_game_info",
        { placeId: parseInt(newPlaceId) }
      ).catch(() => null);

      await favorites.add({
        placeId: parseInt(newPlaceId),
        name: gameInfo?.name || newGameName,
        thumbnail: gameInfo?.thumbnail,
      });

      showAddModal = false;
      newPlaceId = "";
      newGameName = "";
    } catch (err) {
      error = String(err);
    } finally {
      addLoading = false;
    }
  }

  async function launchGame(game: FavoriteGame) {
    if (!selectedAccountId) {
      error = "Please select an account first";
      return;
    }

    try {
      await launcher.launch(selectedAccountId, game.placeId);
    } catch (err) {
      error = String(err);
    }
  }

  async function removeGame(gameId: string) {
    await favorites.remove(gameId);
  }
</script>

<div class="favorites-container">
  <div class="header">
    <div class="title">
      <Star size={24} class="icon" />
      <h2>Favorite Games</h2>
    </div>
    <button class="btn btn-primary add-btn" onclick={() => showAddModal = true}>
      <Plus size={16} />
      Add Game
    </button>
  </div>

  {#if loading}
    <div class="loading">
      <Loader2 size={24} class="spin" />
      <p>Loading favorites...</p>
    </div>
  {:else if games.length === 0}
    <div class="empty-state">
      <Star size={48} class="icon-muted" />
      <h3>No Favorite Games</h3>
      <p>Add your frequently played games for quick launching</p>
      <button class="btn btn-primary" onclick={() => showAddModal = true}>
        <Plus size={16} />
        Add Your First Game
      </button>
    </div>
  {:else}
    <div class="games-grid">
      {#each games as game (game.id)}
        <div class="game-card">
          {#if game.thumbnail}
            <img src={game.thumbnail} alt={game.name} class="thumbnail" />
          {:else}
            <div class="thumbnail-placeholder">🎮</div>
          {/if}
          <div class="game-info">
            <h4>{game.name}</h4>
            <p class="place-id">Place ID: {game.placeId}</p>
          </div>
          <div class="game-actions">
            <button class="btn btn-success btn-sm" onclick={() => launchGame(game)}>
              <Play size={14} />
              Launch
            </button>
            <button class="btn btn-danger btn-sm" onclick={() => removeGame(game.id)}>
              <Trash2 size={14} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if error}
    <div class="error-toast">{error}</div>
  {/if}
</div>

<!-- Add Game Modal -->
{#if showAddModal}
  <div class="modal-backdrop" onclick={() => showAddModal = false}>
    <div class="modal glass" onclick={(e) => e.stopPropagation()}>
      <h3>Add Favorite Game</h3>
      <div class="input-group">
        <label for="place-id">Place ID</label>
        <input
          type="text"
          id="place-id"
          bind:value={newPlaceId}
          placeholder="e.g., 4924922222"
        />
      </div>
      <div class="input-group">
        <label for="game-name">Game Name</label>
        <input
          type="text"
          id="game-name"
          bind:value={newGameName}
          placeholder="e.g., Brookhaven"
        />
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={() => showAddModal = false}>Cancel</button>
        <button class="btn btn-primary" onclick={addGame} disabled={addLoading}>
          {#if addLoading}
            <Loader2 size={16} class="spin" />
            Adding...
          {:else}
            <Plus size={16} />
            Add Game
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .favorites-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 20px;
    height: 100%;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .title h2 {
    font-size: 24px;
    font-weight: 600;
    margin: 0;
  }

  .title :global(.icon) {
    color: var(--color-warning);
  }

  .add-btn {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px;
    color: var(--color-text-secondary);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px;
    text-align: center;
  }

  .empty-state h3 {
    margin: 0;
    font-size: 18px;
  }

  .empty-state p {
    margin: 0;
    color: var(--color-text-secondary);
  }

  .icon-muted {
    opacity: 0.3;
  }

  .games-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .game-card {
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    overflow: hidden;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .game-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  }

  .thumbnail {
    width: 100%;
    height: 150px;
    object-fit: cover;
  }

  .thumbnail-placeholder {
    width: 100%;
    height: 150px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 48px;
    background: rgba(0, 0, 0, 0.3);
  }

  .game-info {
    padding: 12px;
  }

  .game-info h4 {
    margin: 0 0 4px 0;
    font-size: 16px;
    font-weight: 600;
  }

  .place-id {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .game-actions {
    display: flex;
    gap: 8px;
    padding: 0 12px 12px;
  }

  .btn-sm {
    padding: 6px 12px;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .btn-success {
    background: var(--color-success);
    color: white;
    flex: 1;
  }

  .btn-danger {
    background: var(--color-error);
    color: white;
  }

  .error-toast {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--color-error);
    color: white;
    padding: 12px 20px;
    border-radius: 8px;
    font-size: 14px;
    z-index: 1000;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal {
    width: 90%;
    max-width: 400px;
    padding: 24px;
    border-radius: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .modal h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  input {
    padding: 12px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    color: var(--color-text-primary);
    font-size: 14px;
  }

  input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 8px;
  }

  .btn {
    padding: 10px 16px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    transition: opacity 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-secondary);
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
