<script lang="ts">
  import { Users, Play, Loader2, Server, Share2 } from "lucide-svelte";
  import type { ServerInfo } from "$lib/types/roblox";

  let { servers, loading, onJoin, onLoadMore, hasMore, selectedAccount, placeId } = $props<{
    servers: ServerInfo[];
    loading: boolean;
    onJoin: (serverId: string) => void;
    onLoadMore: () => void;
    hasMore: boolean;
    selectedAccount: any;
    placeId?: number;
  }>();

  // Filter & Sort state
  let hideFull = $state(false);
  let sortBy = $state<"players" | "ping">("players");
  let sortAsc = $state(true);

  // Filtered and sorted servers
  let filteredServers = $derived(() => {
    let result = [...servers];
    
    if (hideFull) {
      result = result.filter(s => s.playing < s.maxPlayers);
    }
    
    result.sort((a, b) => {
      if (sortBy === "players") {
        return sortAsc ? a.playing - b.playing : b.playing - a.playing;
      } else {
        const pingA = a.ping ?? 9999;
        const pingB = b.ping ?? 9999;
        return sortAsc ? pingA - pingB : pingB - pingA;
      }
    });
    
    return result;
  });

  function getPingClass(ping?: number): string {
    if (!ping) return "unknown";
    if (ping < 100) return "good";
    if (ping < 200) return "mid";
    return "bad";
  }
</script>

<div class="list-container">
  <div class="list-header">
    <span class="title">
      <Server size={14} />
      Servers ({filteredServers().length})
    </span>

    <div class="controls">
      <label class="toggle">
        <input type="checkbox" bind:checked={hideFull} />
        <span>Hide Full</span>
      </label>
      <div class="sort-pill">
        <select bind:value={sortBy}>
          <option value="players">Players</option>
          <option value="ping">Ping</option>
        </select>
        <button class="order-btn" onclick={() => sortAsc = !sortAsc}>
          {sortAsc ? "↑" : "↓"}
        </button>
      </div>
    </div>
  </div>

  <div class="server-rows">
    {#if filteredServers().length === 0 && !loading}
      <div class="empty">No servers found</div>
    {:else}
      {#each filteredServers() as server (server.id)}
        <div class="row">
          <div class="players" class:full={server.playing >= server.maxPlayers}>
            <Users size={12} />
            <span>{server.playing}/{server.maxPlayers}</span>
          </div>
          <span class="ping {getPingClass(server.ping)}">
            {server.ping ? `${server.ping}ms` : "—"}
          </span>
          <span class="fps">
            {server.fps ? `${Math.round(server.fps)} fps` : "—"}
          </span>
          <button class="job-id" onclick={() => navigator.clipboard.writeText(server.id)} title="Click to copy">
            {server.id}
          </button>
          <div class="actions">
            <button class="share-btn" onclick={() => navigator.clipboard.writeText(`roblox://experiences/start?placeId=${placeId}&gameInstanceId=${server.id}`)} title="Copy server link">
              <Share2 size={12} />
            </button>
            <button class="join-btn" onclick={() => onJoin(server.id)} title="Join">
              Join
            </button>
          </div>
        </div>
      {/each}
    {/if}

    {#if hasMore}
      <div class="load-more">
        <button onclick={onLoadMore} disabled={loading}>
          {#if loading}
            <Loader2 size={14} class="spin" />
          {:else}
            Load More
          {/if}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .list-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    overflow: hidden;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    background: var(--color-bg-tertiary);
    border-bottom: 1px solid var(--color-border);
  }

  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--color-text-tertiary);
    cursor: pointer;
  }

  .toggle input {
    accent-color: var(--color-accent);
    width: 12px;
    height: 12px;
  }

  .sort-pill {
    display: flex;
    align-items: center;
    background: var(--color-bg-primary);
    border-radius: 6px;
    border: 1px solid var(--color-border);
    overflow: hidden;
  }

  .sort-pill select {
    padding: 3px 6px;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    font-size: 11px;
    cursor: pointer;
    outline: none;
  }

  .order-btn {
    padding: 3px 6px;
    background: transparent;
    border: none;
    border-left: 1px solid var(--color-border);
    color: var(--color-text-tertiary);
    font-size: 11px;
    cursor: pointer;
  }

  .order-btn:hover {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .server-rows {
    flex: 1;
    overflow-y: auto;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--color-border);
    transition: background 0.15s;
  }

  .row:last-child { border-bottom: none; }
  .row:hover { background: var(--color-bg-tertiary); }

  .players {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 500;
    min-width: 50px;
  }

  .players.full { color: var(--color-error); }

  .ping {
    flex: 1;
    font-size: 11px;
    font-weight: 500;
    opacity: 0.7;
  }

  .ping.good { color: #34d399; }
  .ping.mid { color: #fbbf24; }
  .ping.bad { color: #f87171; }
  .ping.unknown { color: var(--color-text-tertiary); }

  .fps {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-tertiary);
    min-width: 50px;
  }

  .job-id {
    flex: 1;
    font-size: 10px;
    font-family: monospace;
    color: var(--color-text-tertiary);
    background: transparent;
    border: none;
    padding: 2px 4px;
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s;
  }

  .job-id:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
  }

  .job-id:active {
    background: var(--color-accent);
    color: white;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .share-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text-tertiary);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .share-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .share-btn:active {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
  }

  .join-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: filter 0.15s;
  }

  .join-btn:hover { filter: brightness(1.15); }

  .empty {
    padding: 32px;
    text-align: center;
    color: var(--color-text-tertiary);
    font-size: 13px;
  }

  .load-more {
    padding: 8px;
    display: flex;
    justify-content: center;
  }

  .load-more button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-tertiary);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .load-more button:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  :global(.spin) { animation: spin 1s linear infinite; }
</style>

