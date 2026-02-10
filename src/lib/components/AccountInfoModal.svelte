<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { animate } from "motion";
  import { X, Copy, ExternalLink, Star, Trash2, Play, Gamepad2, Hammer, Wifi, WifiOff, Users, Coins } from "lucide-svelte";
  import { ui, accounts } from "$lib/stores";
  import { toasts } from "$lib/stores/toasts";
  import type { Profile } from "$lib/types";

  let { account }: { account: Profile } = $props();

  // Modal ref for animation
  let modalRef: HTMLDivElement | null = $state(null);

  // Account status
  type PresenceStatus = "offline" | "online" | "ingame" | "studio" | "unknown" | "unchecked";
  let presence = $state<PresenceStatus>("unchecked");
  let lastLocation = $state<string | null>(null);
  let checking = $state(false);

  // Extended user details
  interface GroupInfo { id: number; name: string; memberCount?: number; }
  interface ExtendedDetails {
    robux: number | null;
    createdAt: string | null;
    isBanned: boolean;
    groupsCount: number;
    groups: GroupInfo[];
    profileUrl: string;
    loginLink: string;
  }
  let extendedDetails = $state<ExtendedDetails | null>(null);
  let loadingDetails = $state(true);
  let showGroups = $state(false);

  async function checkStatus() {
    checking = true;
    try {
      const result = await invoke<{ status: string, lastLocation: string | null }>("get_user_presence", { userId: account.userId });
      presence = result.status as PresenceStatus;
      lastLocation = result.lastLocation;
    } catch (e) {
      console.error("Presence check failed:", e);
      presence = "unknown";
    }
    checking = false;
  }

  async function fetchExtendedDetails() {
    loadingDetails = true;
    try {
      const result = await invoke<ExtendedDetails>("get_user_details", {
        cookie: account.cookie,
        userId: account.userId,
        username: account.username
      });
      extendedDetails = result;
    } catch (e) {
      console.error("Extended details failed:", e);
    }
    loadingDetails = false;
  }

  // Auto-check on mount with animation
  $effect(() => {
    checkStatus();
    fetchExtendedDetails();
    // Animate modal on mount
    if (modalRef) {
      animate(modalRef, { opacity: [0, 1], scale: [0.95, 1] }, { duration: 0.2, easing: "ease-out" });
    }
  });

  // Format Robux with commas
  function formatRobux(robux: number | null): string {
    if (robux === null) return "---";
    return robux.toLocaleString();
  }

  // Format created date
  function formatCreatedDate(dateStr: string | null): string {
    if (!dateStr) return "Unknown";
    const date = new Date(dateStr);
    return date.toLocaleDateString("en-US", { year: "numeric", month: "short", day: "numeric" });
  }

  function copy(text: string, label: string) {
    navigator.clipboard.writeText(text);
    toasts.success(`${label} copied!`);
  }

  function openProfile() {
    invoke("open_in_browser", { url: `https://www.roblox.com/users/${account.userId}/profile` });
  }

  function openLogin() {
    invoke("open_in_browser", { url: "https://www.roblox.com/login" });
  }

  function close() {
    ui.closeAccountInfo();
  }

  function openEdit() {
    ui.closeAccountInfo();
    setTimeout(() => ui.openEditAccount(account), 100);
  }

  function toggleFavorite() {
    accounts.toggleFavorite(account.id);
  }

  async function deleteAccount() {
    if (confirm(`Delete account ${account.username}?`)) {
      try {
        await accounts.delete(account.id);
        ui.closeAccountInfo();
        toasts.success("Account deleted");
      } catch (err) {
        toasts.error(`Failed to delete: ${err}`);
      }
    }
  }

  async function launchAccount() {
    try {
      await invoke("launch_roblox", { accountId: account.id });
      toasts.success(`Launching ${account.username}...`);
      ui.closeAccountInfo();
    } catch (e) {
      toasts.error(`Failed to launch: ${e}`);
    }
  }

  // Combo format
  let combo = $derived(`${account.username}:${account.password || "N/A"}`);

  // Created date
  let createdDate = $derived(
    (account.createdAt ?? 0) > 0 
      ? new Date((account.createdAt ?? 0) * 1000).toLocaleDateString() 
      : "Unknown"
  );
</script>

<div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e) => e.key === 'Escape' && close()}>
  <div class="modal" onclick={(e) => e.stopPropagation()} onkeydown={() => {}} role="dialog" aria-modal="true" tabindex="-1">
    <!-- Header -->
    <div class="modal-header">
      <div class="header-content">
        <div class="avatar-container">
          {#if account.thumbnail}
            <img src={account.thumbnail} alt="" class="avatar" />
          {:else}
            <div class="avatar-placeholder">?</div>
          {/if}
        </div>
        <div class="user-info">
          <div class="name-row">
            {#if account.isPremium}
              <span class="premium-icon">👑</span>
            {/if}
            <h2>{account.displayName}</h2>
            <span class="username">@{account.username}</span>
          </div>
          
          <!-- Row 1: ID, Created, Alias -->
          <div class="tags-row">
            <div class="tag">ID: {account.userId}</div>
            <div class="tag">Created: {formatCreatedDate(extendedDetails?.createdAt ?? null)}</div>
            {#if account.alias}
              <div class="tag red-tag">{account.alias}</div>
            {/if}
          </div>

          <!-- Row 2: Robux, Groups -->
          <div class="tags-row">
            <div class="tag">Robux: {formatRobux(extendedDetails?.robux ?? null)}</div>
            <div class="tag">Groups: {extendedDetails?.groupsCount ?? 0}</div>
          </div>

          <div class="links-row">
            <button class="link-btn" onclick={openProfile}>
              <ExternalLink size={12} /> Roblox Profile
            </button>
            <button class="link-btn" onclick={openLogin}>
              <ExternalLink size={12} /> Roblox Login
            </button>
          </div>
        </div>
        <button class="close-btn" onclick={close}>
          <X size={20} />
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="modal-body">
      <!-- Username -->
      <div class="input-group">
        <label>Username</label>
        <div class="input-wrapper">
          <span class="value">{account.username}</span>
          <button class="copy-icon" onclick={() => copy(account.username, "Username")}>
            <Copy size={16} />
          </button>
        </div>
      </div>

      <!-- Password -->
      <div class="input-group">
        <label>Password</label>
        <div class="input-wrapper">
          <span class="value mono">{account.password || "Not saved"}</span>
          {#if account.password}
            <button class="copy-icon" onclick={() => copy(account.password!, "Password")}>
              <Copy size={16} />
            </button>
          {/if}
        </div>
      </div>

      <!-- Combo -->
      <div class="input-group">
        <label>Combo Format (username:password)</label>
        <div class="input-wrapper">
          <span class="value mono">{combo}</span>
          <button class="copy-icon" onclick={() => copy(combo, "Combo")}>
            <Copy size={16} />
          </button>
        </div>
      </div>

      <!-- Account Status (Presence) -->
      <div class="status-row">
        <span class="status-label">Account Status:</span>
        {#if checking}
           <span class="status-badge checking">Checking...</span>
        {:else if presence === "online"}
           <span class="status-badge online"><Wifi size={12} /> Online</span>
        {:else if presence === "ingame"}
           <span class="status-badge ingame">
             <Gamepad2 size={12} /> In Game 
             {#if lastLocation}<span class="location"> - {lastLocation}</span>{/if}
           </span>
        {:else if presence === "studio"}
           <span class="status-badge studio"><Hammer size={12} /> In Studio</span>
        {:else if presence === "offline"}
           <span class="status-badge offline"><WifiOff size={12} /> Offline</span>
        {:else}
           <span class="status-badge unknown">Unknown</span>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="actions-grid">
        <button 
          class="action-btn favorite" 
          class:active={account.isFavorite}
          onclick={toggleFavorite}
        >
          <Star size={14} fill={account.isFavorite ? "currentColor" : "none"} /> 
          Favorite
        </button>
        <button class="action-btn delete" onclick={deleteAccount}>
          <Trash2 size={14} /> Delete
        </button>
        <button class="action-btn launch" onclick={launchAccount}>
          <Play size={14} /> Launch
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(5px);
    padding: 20px; /* Prevent touching edges on mobile */
  }

  .modal {
    background: #111;
    border-radius: 20px; /* Rounder square */
    width: 100%;
    max-width: 520px; /* More square ratio */
    max-height: 90vh;
    box-shadow: 0 25px 60px -15px rgba(0, 0, 0, 0.9);
    border: 1px solid #2a2a2a;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* --- HEADER --- */
  .modal-header {
    background: linear-gradient(180deg, #2a1010 0%, #120808 100%);
    padding: 28px;
    position: relative;
    flex-shrink: 0;
  }
  
  .header-content {
    display: flex;
    gap: 18px;
    align-items: center;
  }

  .avatar-container {
    flex-shrink: 0;
  }

  .avatar {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    object-fit: cover;
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: transparent;
    box-shadow: 0 4px 16px rgba(0,0,0,0.4);
  }

  .avatar-placeholder {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.05);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #666;
    font-size: 24px;
    border: 2px solid rgba(255, 255, 255, 0.1);
  }

  .user-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .name-row {
    display: flex;
    align-items: baseline;
    gap: 10px;
    flex-wrap: wrap;
  }

  .name-row h2 {
    font-size: 22px;
    font-weight: 700;
    color: #fff;
    margin: 0;
    letter-spacing: -0.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .username {
    font-size: 14px;
    color: #bbb;
    font-weight: 500;
  }

  .tags-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    align-items: center;
    margin-top: 4px;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(0, 0, 0, 0.3);
    color: #aaa;
    font-size: 11px;
    padding: 3px 8px;
    border-radius: 12px;
    font-weight: 500;
    border: 1px solid rgba(255,255,255,0.05);
  }

  .red-tag {
    background: rgba(220, 53, 69, 0.2);
    color: #ff6b6b;
    border: 1px solid rgba(220, 53, 69, 0.3);
  }

  .links-row {
    display: flex;
    gap: 8px;
    margin-top: 4px;
    flex-wrap: wrap;
  }

  .link-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #ddd;
    font-size: 11px;
    padding: 5px 10px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 500;
  }

  .link-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    color: #fff;
  }

  .close-btn {
    position: absolute;
    top: 20px;
    right: 20px;
    background: transparent;
    border: none;
    color: rgba(255,255,255,0.3);
    cursor: pointer;
    padding: 8px;
    display: flex;
    transition: color 0.2s;
  }

  .close-btn:hover {
    color: #fff;
  }

  /* --- BODY --- */
  .modal-body {
    padding: 24px 32px 32px 32px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: #0E0E0E;
    overflow-y: auto;
    flex: 1;
  }

  /* Input fields */
  .input-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .input-group label {
    font-size: 12px;
    text-transform: uppercase;
    color: #888;
    font-weight: 700;
    letter-spacing: 0.5px;
    margin-left: 4px;
  }

  .input-wrapper {
    background: #181818;
    border: 1px solid #2a2a2a;
    border-radius: 10px;
    padding: 0 16px;
    height: 48px; /* Taller inputs */
    display: flex;
    align-items: center;
    justify-content: space-between;
    transition: border-color 0.2s;
  }

  .input-wrapper:hover {
    border-color: #444;
  }

  .value {
    color: #eee;
    font-size: 15px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mono {
    font-family: 'SF Mono', 'Menlo', monospace;
    font-size: 14px;
    color: #ccc;
  }

  .copy-icon {
    background: transparent;
    border: none;
    color: #666;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px;
    border-radius: 6px;
    transition: all 0.2s;
  }

  .copy-icon:hover {
    background: rgba(255,255,255,0.05);
    color: #fff;
  }

  /* Status Row */
  .status-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
  }

  .status-label {
    font-size: 13px;
    color: #888;
    font-weight: 500;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
  }

  .status-badge.checking {
    background: rgba(255, 255, 255, 0.05);
    color: #888;
  }

  .status-badge.online {
    background: rgba(52, 152, 219, 0.15);
    color: #3498db;
  }

  .status-badge.ingame {
    background: rgba(46, 204, 113, 0.15);
    color: #2ecc71;
  }

  .status-badge.studio {
    background: rgba(230, 126, 34, 0.15);
    color: #e67e22;
  }

  .status-badge.offline {
    background: rgba(149, 165, 166, 0.15);
    color: #95a5a6;
  }

  .status-badge.unknown {
    background: rgba(255, 255, 255, 0.05);
    color: #666;
  }

  .location {
    font-weight: 400;
    opacity: 0.8;
  }

  /* Action Buttons Grid */
  .actions-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    margin-top: 16px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 38px;
    background: #181818;
    border: 1px solid #2a2a2a;
    border-radius: 8px;
    color: #aaa;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: #222;
    border-color: #444;
    color: #fff;
  }

  .action-btn.favorite.active {
    background: rgba(255, 215, 0, 0.1);
    border-color: #FFD700;
    color: #FFD700;
  }

  .action-btn.delete:hover {
    background: rgba(231, 76, 60, 0.1);
    border-color: #e74c3c;
    color: #e74c3c;
  }

  .action-btn.launch {
    background: rgba(46, 204, 113, 0.1);
    border-color: rgba(46, 204, 113, 0.3);
    color: #2ecc71;
  }

  .action-btn.launch:hover {
    background: rgba(46, 204, 113, 0.2);
    border-color: #2ecc71;
  }
</style>
