<script lang="ts">
  import { onMount } from "svelte";
  import { Plus, Search, RefreshCw, Download, Upload, Trash2, Smartphone, Globe, ArrowUpDown } from "lucide-svelte";
  import { accounts, favoriteAccounts, accountCount, ui, launcher } from "$lib/stores";
  import { toasts } from "$lib/stores/toasts";
  import AccountRow from "$lib/components/AccountRow.svelte";
  import { stagger, animate } from "motion";

  let searchQuery = $state("");
  let filterMode = $state<"all" | "favorites" | "online">("all");
  let sortMode = $state<"name" | "created" | "lastUsed" | "favorite">("name");
  let sortAsc = $state(true);

  onMount(() => {
    accounts.load();
    // Animate account rows on first load
    setTimeout(() => animateRows(), 100);
  });

  function animateRows() {
    const rows = document.querySelectorAll('.account-row');
    if (rows.length > 0) {
      animate(rows, 
        { opacity: [0, 1], y: [10, 0] },
        { delay: stagger(0.03), duration: 0.3, easing: "ease-out" }
      );
    }
  }

  let filteredAccounts = $derived(() => {
    let list = filterMode === "favorites"
      ? $favoriteAccounts
      : filterMode === "online"
        ? $accounts.accounts.filter(a => $launcher.instances.some(i => i.accountId === a.id))
        : $accounts.accounts;

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = list.filter(a =>
        a.displayName.toLowerCase().includes(q) ||
        a.username.toLowerCase().includes(q)
      );
    }

    // Sorting
    list = [...list].sort((a, b) => {
      let cmp = 0;
      switch (sortMode) {
        case "name":
          cmp = a.displayName.localeCompare(b.displayName);
          break;
        case "created":
          cmp = (a.createdAt || 0) - (b.createdAt || 0);
          break;
        case "lastUsed":
          cmp = (a.lastPlayedAt || 0) - (b.lastPlayedAt || 0);
          break;
        case "favorite":
          cmp = (b.isFavorite ? 1 : 0) - (a.isFavorite ? 1 : 0);
          break;
      }
      return sortAsc ? cmp : -cmp;
    });

    return list;
  });

  async function handleExport() {
    try {
      const data = await accounts.exportAccounts();
      const blob = new Blob([data], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `rokio-backup-${new Date().toISOString().split("T")[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toasts.success("Backup exported successfully");
    } catch (e) {
      toasts.error(String(e));
    }
  }

  async function handleImport() {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      
      const text = await file.text();
      try {
        const count = await accounts.importAccounts(text, true);
        toasts.success(`Imported ${count} accounts`);
      } catch (err) {
        toasts.error(String(err));
      }
    };
    input.click();
  }
</script>

<div class="page">
  <!-- Search Bar -->
  <div class="search-bar">
    <Search size={14} />
    <input type="text" placeholder="Search accounts..." bind:value={searchQuery} />
  </div>

  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-left">
      <div class="filter-dropdown">
        <select bind:value={filterMode}>
          <option value="all">All</option>
          <option value="favorites">Favorites</option>
          <option value="online">Running</option>
        </select>
      </div>
      <div class="filter-dropdown">
        <select bind:value={sortMode}>
          <option value="name">Name</option>
          <option value="created">Created</option>
          <option value="lastUsed">Last Used</option>
          <option value="favorite">⭐ First</option>
        </select>
      </div>
      <button class="sort-btn" onclick={() => sortAsc = !sortAsc} title="Toggle order">
        <ArrowUpDown size={12} />
        {sortAsc ? "↑" : "↓"}
      </button>
      <span class="stat-badge">{$accountCount}</span>
    </div>

    <div class="toolbar-right">
      <button class="icon-btn" onclick={() => accounts.load()} title="Reload">
        <RefreshCw size={14} />
      </button>
      <button class="icon-btn" onclick={handleImport} title="Import">
        <Download size={14} />
      </button>
      <button class="icon-btn" onclick={handleExport} title="Export">
        <Upload size={14} />
      </button>
      <button class="icon-btn danger" onclick={() => accounts.clearAll()} title="Clear">
        <Trash2 size={14} />
      </button>
      <button class="icon-btn" onclick={() => ui.openQuickLogin()} title="QR Login">
        <Smartphone size={14} />
      </button>
      <button class="icon-btn" onclick={() => ui.openBrowserLogin()} title="Browser Login">
        <Globe size={14} />
      </button>
      <button class="btn-primary" onclick={() => ui.openAddAccount()}>
        <Plus size={14} />
        Add
      </button>
    </div>
  </div>

  <!-- Table Container -->
  {#if $accounts.loading}
    <div class="state">Loading accounts...</div>
  {:else if filteredAccounts().length === 0}
    <div class="state empty">
      <div class="empty-icon">
        {#if searchQuery}🔍{:else}👤{/if}
      </div>
      <p>
        {#if searchQuery}
          No results found for "{searchQuery}"
        {:else if filterMode === "favorites"}
          No favorite accounts
        {:else if filterMode === "online"}
          No running instances
        {:else}
          No accounts added yet
        {/if}
      </p>
    </div>
  {:else}
    <div class="table-container">
      <div class="accounts-table">
        <div class="thead-row">
          <div class="th center">
            <input type="checkbox" disabled />
          </div>
          <div class="th">Account</div>
          <div class="th">Alias</div>
          <div class="th">Description</div>
          <div class="th">Status</div>
          <div class="th">Last Used</div>
          <div class="th right">Actions</div>
        </div>
        <div class="tbody">
          {#each filteredAccounts() as account (account.id)}
            <AccountRow 
              {account} 
              isSelected={$accounts.selectedId === account.id} 
            />
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 8px;
    padding-bottom: 16px;
  }

  /* Compact Search Bar */
  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    color: var(--color-text-tertiary);
  }

  .search-bar:focus-within {
    border-color: var(--color-accent);
  }

  .search-bar input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--color-text-primary);
    font-size: 13px;
    outline: none;
  }

  .search-bar input::placeholder {
    color: var(--color-text-tertiary);
  }

  /* Toolbar */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  /* Filter Dropdown */
  .filter-dropdown select {
    appearance: none;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 6px 24px 6px 10px;
    color: var(--color-text-primary);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 6px center;
  }

  .filter-dropdown select:hover {
    border-color: var(--color-text-tertiary);
  }

  .filter-dropdown select:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  /* Stat Badge */
  .stat-badge {
    padding: 6px 10px;
    background: var(--color-bg-tertiary);
    border-radius: 6px;
    font-size: 12px;
    color: var(--color-text-secondary);
    font-weight: 500;
    border: 1px solid var(--color-border);
  }

  /* Sort Button */
  .sort-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 11px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .sort-btn:hover {
    border-color: var(--color-text-tertiary);
    color: var(--color-text-primary);
  }

  /* Icon Button - SF Symbol style */
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .icon-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .icon-btn.danger:hover {
    background: rgba(255, 69, 58, 0.15);
    color: #ff6b6b;
  }

  /* Primary Button */
  .btn-primary {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    background: var(--color-accent);
    border: none;
    border-radius: 8px;
    color: #fff;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-primary:hover {
    background: var(--color-accent-hover);
  }

  /* Secondary Button */
  .btn-secondary {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-secondary:hover {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    border-color: var(--color-text-tertiary);
  }

  /* Danger Button */
  .btn-danger {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: rgba(255, 69, 58, 0.15);
    border: 1px solid rgba(255, 69, 58, 0.3);
    border-radius: 6px;
    color: #ff6b6b;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-danger:hover {
    background: rgba(255, 69, 58, 0.25);
    border-color: rgba(255, 69, 58, 0.5);
  }

  /* Table Container */
  .table-container {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border-radius: 12px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  /* Grid Layout for Header and Rows */
  .accounts-table {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }

  .thead-row {
    display: grid;
    grid-template-columns: 40px 1.2fr 80px 0.8fr 70px 100px 160px;
    background: var(--color-bg-tertiary);
    border-bottom: 1px solid var(--color-border);
    align-items: center;
  }

  .th {
    padding: 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    letter-spacing: 0.5px;
    white-space: nowrap;
  }

  .th.center { text-align: center; }
  .th.right { text-align: right; padding-right: 16px; }

  .tbody {
    flex: 1;
    overflow-y: auto;
    width: 100%;
  }

  /* States */
  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-tertiary);
    gap: 12px;
  }

  .empty-icon {
    font-size: 32px;
    opacity: 0.5;
  }
</style>
