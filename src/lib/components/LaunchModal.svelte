<script lang="ts">
  import { X, Loader2, Play, Gamepad2, Users, Lock, Search } from "lucide-svelte";
  import { launcher, ui } from "$lib/stores";
  import type { Profile } from "$lib/types";
  import { animate } from "motion";
  import { invoke } from "@tauri-apps/api/core";

  let { account }: { account: Profile } = $props();

  // Tab state
  type LaunchMode = "game" | "vip" | "join";
  let mode = $state<LaunchMode>("game");

  // Game mode
  let placeId = $state("");
  let jobId = $state("");
  
  // VIP Server mode
  let vipPlaceId = $state("");
  let vipLinkCode = $state("");
  
  // Join User mode
  let targetUsername = $state("");
  let targetUser = $state<{ id: number; name: string; displayName: string; thumbnail?: string } | null>(null);
  let gameInfo = $state<{ isInGame: boolean; placeId?: number; gameId?: string; gameName?: string } | null>(null);
  let searchLoading = $state(false);
  
  let loading = $state(false);
  let error = $state("");
  let modalRef = $state<HTMLElement | null>(null);

  // Popular games with their universe IDs for thumbnail fetching
  const popularPlaces = [
    { name: "Adopt Me!", id: "920587237", universeId: "418238370" },
    { name: "Brookhaven", id: "4924922222", universeId: "2040407060" },
    { name: "Blox Fruits", id: "2753915549", universeId: "1217110035" },
    { name: "Murder Mystery 2", id: "142823291", universeId: "66654135" },
    { name: "MeepCity", id: "370731277", universeId: "137877284" },
    { name: "Arsenal", id: "286090429", universeId: "111958650" },
    { name: "Pet Simulator 99", id: "8737899170", universeId: "4823684776" },
    { name: "Jailbreak", id: "606849621", universeId: "220552094" },
  ];

  // Game icons state (loaded dynamically via backend)
  let gameIcons = $state<Record<string, string>>({});
  let iconsLoading = $state(true);

  // Fetch game icons on mount via backend (avoids CORS)
  $effect(() => {
    loadGameIcons();
  });

  async function loadGameIcons() {
    iconsLoading = true;
    try {
      // Get universe IDs from popular places
      const universeIds = popularPlaces.map(p => parseInt(p.universeId));
      
      // Fetch from backend (no CORS issues)
      // Note: Rust HashMap<u64, String> serializes to JSON with string keys
      const icons = await invoke<Record<string, string>>("batch_get_game_icons", { universeIds });
      
      // Map universe IDs back to place IDs (use string keys for JSON lookup)
      const iconsByPlaceId: Record<string, string> = {};
      for (const place of popularPlaces) {
        const iconUrl = icons[place.universeId]; // JSON keys are strings
        if (iconUrl) {
          iconsByPlaceId[place.id] = iconUrl;
        }
      }
      gameIcons = iconsByPlaceId;
    } catch (e) {
      console.warn("Failed to load game icons:", e);
    } finally {
      iconsLoading = false;
    }
  }

  async function launchGame() {
    if (!placeId.trim()) {
      error = "Please enter a Place ID";
      return;
    }

    const placeIdNum = parseInt(placeId.trim(), 10);
    if (isNaN(placeIdNum)) {
      error = "Place ID must be a number";
      return;
    }

    loading = true;
    error = "";

    try {
      const instance = await launcher.launch(account.id, placeIdNum, jobId.trim() || undefined);
      if (instance) close();
      else error = "Failed to launch game";
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function launchVipServer() {
    // Parse VIP server link if pasted
    let parsedPlaceId = vipPlaceId.trim();
    let parsedLinkCode = vipLinkCode.trim();
    
    // Try to parse from full URL
    if (vipLinkCode.includes("roblox.com") || vipLinkCode.includes("privateServerLinkCode=")) {
      const placeMatch = vipLinkCode.match(/games\/(\d+)/);
      const codeMatch = vipLinkCode.match(/privateServerLinkCode=(\d+)/);
      if (placeMatch) parsedPlaceId = placeMatch[1];
      if (codeMatch) parsedLinkCode = codeMatch[1];
    }
    
    if (!parsedPlaceId || !parsedLinkCode) {
      error = "Please enter Place ID and VIP Server Link Code";
      return;
    }

    loading = true;
    error = "";

    try {
      const instance = await invoke("launch_vip_server", {
        accountId: account.id,
        placeId: parseInt(parsedPlaceId),
        linkCode: parsedLinkCode
      });
      if (instance) close();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function searchUser() {
    if (!targetUsername.trim()) {
      error = "Please enter a username";
      return;
    }

    searchLoading = true;
    error = "";
    targetUser = null;
    gameInfo = null;

    try {
      // Look up user
      const user = await invoke<{ id: number; name: string; displayName: string; thumbnail?: string }>(
        "get_user_by_username", 
        { username: targetUsername.trim() }
      );
      targetUser = user;
      
      // Get their game info
      const info = await invoke<{ isInGame: boolean; placeId?: number; gameId?: string; gameName?: string }>(
        "get_user_game_info",
        { cookie: account.cookie, targetUserId: user.id, targetUsername: user.name }
      );
      gameInfo = info;
    } catch (err) {
      error = String(err);
    } finally {
      searchLoading = false;
    }
  }

  async function joinUser() {
    if (!gameInfo?.isInGame || !gameInfo.placeId || !gameInfo.gameId) {
      error = "User is not in a joinable game";
      return;
    }

    loading = true;
    error = "";

    try {
      const instance = await launcher.launch(account.id, gameInfo.placeId, gameInfo.gameId);
      if (instance) close();
      else error = "Failed to join user";
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  function selectPopular(id: string) {
    placeId = id;
  }

  function close() {
    ui.closeLaunchModal();
    placeId = "";
    jobId = "";
    vipPlaceId = "";
    vipLinkCode = "";
    targetUsername = "";
    targetUser = null;
    gameInfo = null;
    error = "";
    mode = "game";
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

<div class="modal-backdrop" onclick={close} role="presentation">
  <div
    bind:this={modalRef}
    class="modal glass"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
  >
    <div class="modal-header">
      <h2>Launch Game</h2>
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
          <div class="avatar-placeholder">👤</div>
        {/if}
        <div class="account-info">
          <h3>{account.displayName}</h3>
          <p>@{account.username}</p>
        </div>
      </div>

      <!-- SwiftUI-style Segmented Control -->
      <div class="segmented-control">
        <button 
          class="segment" 
          class:active={mode === "game"}
          onclick={() => mode = "game"}
        >
          <Gamepad2 size={16} />
          Game
        </button>
        <button 
          class="segment" 
          class:active={mode === "vip"}
          onclick={() => mode = "vip"}
        >
          <Lock size={16} />
          VIP Server
        </button>
        <button 
          class="segment" 
          class:active={mode === "join"}
          onclick={() => mode = "join"}
        >
          <Users size={16} />
          Join User
        </button>
      </div>

      <!-- Game Mode -->
      {#if mode === "game"}
        <div class="input-group">
          <label for="place-id">Place ID</label>
          <input
            type="text"
            id="place-id"
            bind:value={placeId}
            placeholder="Enter Roblox Place ID..."
          />
        </div>

        <div class="popular-places">
          <p class="section-label">Quick Launch:</p>
          <div class="games-carousel">
            {#each popularPlaces as place}
              <button
                class="game-tile"
                class:active={placeId === place.id}
                onclick={() => selectPopular(place.id)}
                title={place.name}
              >
                {#if gameIcons[place.id]}
                  <img src={gameIcons[place.id]} alt={place.name} class="game-icon" />
                {:else}
                  <div class="game-icon-placeholder">
                    <Gamepad2 size={24} />
                  </div>
                {/if}
                <span class="game-name">{place.name}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="input-group">
          <label for="job-id">Job ID (Optional - for specific server)</label>
          <input
            type="text"
            id="job-id"
            bind:value={jobId}
            placeholder="Leave empty for random server..."
          />
        </div>
      {/if}

      <!-- VIP Server Mode -->
      {#if mode === "vip"}
        <div class="input-group">
          <label for="vip-place-id">Place ID</label>
          <input
            type="text"
            id="vip-place-id"
            bind:value={vipPlaceId}
            placeholder="e.g., 4924922222"
          />
        </div>

        <div class="input-group">
          <label for="vip-link">VIP Server Link or Code</label>
          <input
            type="text"
            id="vip-link"
            bind:value={vipLinkCode}
            placeholder="Paste full URL or linkCode..."
          />
          <p class="hint">Paste the VIP server URL or just the privateServerLinkCode</p>
        </div>
      {/if}

      <!-- Join User Mode -->
      {#if mode === "join"}
        <div class="input-group">
          <label for="target-username">Username to Join</label>
          <div class="search-input">
            <input
              type="text"
              id="target-username"
              bind:value={targetUsername}
              placeholder="Enter Roblox username..."
              onkeydown={(e) => e.key === "Enter" && searchUser()}
            />
            <button class="search-btn" onclick={searchUser} disabled={searchLoading}>
              {#if searchLoading}
                <Loader2 size={16} class="spin" />
              {:else}
                <Search size={16} />
              {/if}
            </button>
          </div>
        </div>

        {#if targetUser}
          <div class="user-preview">
            {#if targetUser.thumbnail}
              <img src={targetUser.thumbnail} alt={targetUser.displayName} class="avatar" />
            {:else}
              <div class="avatar-placeholder">👤</div>
            {/if}
            <div class="user-info">
              <h4>{targetUser.displayName}</h4>
              <p>@{targetUser.name}</p>
              {#if gameInfo}
                {#if gameInfo.isInGame}
                  <span class="status ingame">🎮 In Game: {gameInfo.gameName || "Unknown"}</span>
                {:else}
                  <span class="status offline">⚪ Not in a joinable game</span>
                {/if}
              {/if}
            </div>
          </div>
        {/if}
      {/if}

      {#if error}
        <div class="error-message">{error}</div>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn btn-secondary" onclick={close}>Cancel</button>
      
      {#if mode === "game"}
        <button class="btn btn-primary" onclick={launchGame} disabled={loading}>
          {#if loading}
            <Loader2 size={16} class="spin" />
            Launching...
          {:else}
            <Play size={16} />
            Launch
          {/if}
        </button>
      {:else if mode === "vip"}
        <button class="btn btn-primary" onclick={launchVipServer} disabled={loading}>
          {#if loading}
            <Loader2 size={16} class="spin" />
            Joining...
          {:else}
            <Lock size={16} />
            Join VIP
          {/if}
        </button>
      {:else if mode === "join"}
        <button 
          class="btn btn-primary" 
          onclick={joinUser} 
          disabled={loading || !gameInfo?.isInGame}
        >
          {#if loading}
            <Loader2 size={16} class="spin" />
            Joining...
          {:else}
            <Users size={16} />
            Join User
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
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 0.15s ease-out;
    padding: 16px;
  }

  .modal {
    width: 100%;
    max-width: 380px;
    border-radius: 12px;
    background: rgba(30, 30, 32, 0.95);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
    animation: slideUp 0.2s ease-out;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
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
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-height: 60vh;
    overflow-y: auto;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 14px 16px;
    border-top: 1px solid var(--color-border);
  }

  /* Account Preview */
  .account-preview {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 10px;
  }

  .avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
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
    transition: border-color 0.2s;
  }

  input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  input::placeholder {
    color: var(--color-text-secondary);
  }

  /* Popular Places */
  .popular-places {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* PS4/PS5 Style Games Carousel */
  .games-carousel {
    display: flex;
    gap: 12px;
    overflow-x: auto;
    overflow-y: hidden;
    padding: 8px 0;
    scroll-snap-type: x mandatory;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .games-carousel::-webkit-scrollbar {
    display: none;
  }

  .game-tile {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 8px;
    background: rgba(255, 255, 255, 0.04);
    border: 2px solid transparent;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    scroll-snap-align: start;
    width: 90px;
  }

  .game-tile:hover {
    transform: translateY(-4px) scale(1.05);
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.4);
  }

  .game-tile.active {
    border-color: var(--color-accent);
    background: rgba(59, 130, 246, 0.15);
    transform: translateY(-4px) scale(1.05);
    box-shadow: 0 0 20px rgba(59, 130, 246, 0.3);
  }

  .game-icon {
    width: 64px;
    height: 64px;
    border-radius: 12px;
    object-fit: cover;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    transition: transform 0.2s;
  }

  .game-tile:hover .game-icon {
    transform: scale(1.05);
  }

  .game-icon-placeholder {
    width: 64px;
    height: 64px;
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.3), rgba(139, 92, 246, 0.3));
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-secondary);
  }

  .game-name {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-align: center;
    line-height: 1.2;
    max-width: 80px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color 0.2s;
  }

  .game-tile:hover .game-name,
  .game-tile.active .game-name {
    color: var(--color-text-primary);
  }


  /* Error */
  .error-message {
    padding: 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    color: var(--color-error);
    font-size: 13px;
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
    background: var(--color-success);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #16a34a;
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

  /* SwiftUI-style Segmented Control */
  .segmented-control {
    display: flex;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
    padding: 3px;
    gap: 2px;
    margin-bottom: 8px;
  }

  .segment {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 12px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    font-weight: 500;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .segment:hover {
    color: var(--color-text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .segment.active {
    background: rgba(255, 255, 255, 0.12);
    color: var(--color-text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  /* Search Input */
  .search-input {
    display: flex;
    gap: 8px;
  }

  .search-input input {
    flex: 1;
  }

  .search-btn {
    padding: 12px;
    background: var(--color-accent);
    border: none;
    border-radius: 8px;
    color: white;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .search-btn:hover:not(:disabled) {
    opacity: 0.85;
  }

  .search-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* User Preview */
  .user-preview {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    border: 1px solid var(--color-border);
  }

  .user-preview .avatar {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    object-fit: cover;
  }

  .user-info h4 {
    margin: 0 0 2px 0;
    font-size: 15px;
    font-weight: 600;
  }

  .user-info p {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .status {
    display: inline-block;
    margin-top: 6px;
    font-size: 12px;
    padding: 3px 8px;
    border-radius: 12px;
  }

  .status.ingame {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  .status.offline {
    background: rgba(107, 114, 128, 0.15);
    color: #9ca3af;
  }

  /* Hint text */
  .hint {
    font-size: 11px;
    color: var(--color-text-secondary);
    opacity: 0.7;
    margin: 4px 0 0 0;
  }
</style>
