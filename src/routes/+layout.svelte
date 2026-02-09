<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Lock, Smartphone, Terminal } from "lucide-svelte";
  import { auth, ui, launcher } from "$lib/stores";
  import type { AppInfo, AppSettings } from "$lib/types";
  import LockScreen from "$lib/components/LockScreen.svelte";
  import AddAccountModal from "$lib/components/AddAccountModal.svelte";
  import LaunchModal from "$lib/components/LaunchModal.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import QuickLoginModal from "$lib/components/QuickLoginModal.svelte";
  import BrowserLoginModal from "$lib/components/BrowserLoginModal.svelte";
  import EditAccountModal from "$lib/components/EditAccountModal.svelte";
  import AccountInfoModal from "$lib/components/AccountInfoModal.svelte";
  import ServerBrowser from "$lib/components/ServerBrowser.svelte";

  let { children } = $props();

  let appInfo: AppInfo | null = $state(null);
  let bridgeConnected = $state(false);
  let currentTab = $state("accounts");

  // Console logs (Raptor-style)
  let consoleLogs = $state<{time: string; type: string; msg: string}[]>([]);

  // About modal
  let showAboutModal = $state(false);
  
  const changelog = [
    { version: "1.1.0", date: "Feb 2026", highlight: true, changes: [
      "Refactored codebase for better maintainability",
      "Cross-platform linker optimization (sold/mold/lld)",
      "Removed Game Tab in Serverlist",
      "Native CPU optimization for all platforms",
      "Redesign Settings Tab with sandboxed feature and polish"
    ]},
    { version: "1.0.0", date: "Feb 2026", highlight: false, changes: [
      "Initial public release",
      "Cross-platform support (macOS, Windows, Linux)",
      "Encrypted vault for account security",
      "Multi-instance launching",
      "Server browser with region info",
      "Dynamic themes with accent colors"
    ]},
    { version: "0.9.0", date: "Jan 2026", highlight: false, changes: [
      "Beta release",
      "Cookie-based account import",
      "Quick login feature",
      "Favorite games system"
    ]}
  ];

  let settings = $state<AppSettings>({
    autoLockTimeout: "never",
    launchOnStartup: false,
    alwaysOnTop: false,
    theme: "dark",
    compactMode: false,
    accentColor: "red",
    multiInstance: false,
    launcherPreference: "default",
    quarantineInstallers: false,
    saveLogs: false,
    forceHandleClosure: false,
    lowCpuMode: false
  });

  // Accent color map - 16 colors
  const accentColors: Record<string, string> = {
    red: '#ff4d4d',
    orange: '#ff9500',
    amber: '#ffb300',
    yellow: '#ffcc00',
    lime: '#a3e635',
    green: '#34c759',
    mint: '#00c7be',
    teal: '#5ac8fa',
    cyan: '#00bcd4',
    blue: '#007aff',
    indigo: '#5856d6',
    purple: '#af52de',
    pink: '#ff2d55',
    rose: '#f43f5e',
    brown: '#a18072',
    graphite: '#6b7280'
  };

  function log(type: string, msg: string) {
    const now = new Date();
    // Format: YYYY-MM-DD HH:mm:ss.ms
    const time = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')} ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}.${String(now.getMilliseconds()).padStart(3, '0')}`;
    consoleLogs = [...consoleLogs.slice(-99), { time, type, msg }];
  }

  onMount(async () => {
    log("info", "ROKIO starting...");
    try {
      appInfo = await invoke<AppInfo>("init_app_check");
      bridgeConnected = true;
      log("success", `Connected v${appInfo.version}`);
    } catch (err) {
      console.error("Bridge failed:", err);
      log("error", "Bridge connection failed");
    }

    await auth.checkStatus();
    log("info", "Vault status checked");
    await launcher.init();
    log("info", "Launcher initialized");

    // Load settings
    try {
      settings = await invoke<AppSettings>("get_settings");
      log("info", "Settings loaded");
    } catch (err) {
      log("error", "Failed to load settings");
    }
  });

  async function saveSettings() {
    try {
      await invoke("save_settings", { settings });
      log("success", "Settings saved");
    } catch (err) {
      log("error", "Failed to save settings");
    }
  }

  // Circular reveal theme transition
  async function setThemeWithTransition(newTheme: string, event?: MouseEvent) {
    const root = document.documentElement;
    
    // Calculate the target theme
    let targetTheme = newTheme;
    if (newTheme === 'system') {
      targetTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    
    // If View Transitions API is not supported, just switch instantly
    if (!document.startViewTransition) {
      root.setAttribute('data-theme', targetTheme);
      return;
    }
    
    // Get click position for circular reveal origin
    const x = event?.clientX ?? window.innerWidth / 2;
    const y = event?.clientY ?? window.innerHeight / 2;
    
    // Calculate the maximum radius needed to cover the screen
    const maxRadius = Math.hypot(
      Math.max(x, window.innerWidth - x),
      Math.max(y, window.innerHeight - y)
    );
    
    // Start the view transition
    const transition = document.startViewTransition(() => {
      root.setAttribute('data-theme', targetTheme);
    });
    
    // Set the clip-path origin
    root.style.setProperty('--transition-x', `${x}px`);
    root.style.setProperty('--transition-y', `${y}px`);
    root.style.setProperty('--transition-radius', `${maxRadius}px`);
    
    await transition.finished;
  }

  // Watch theme changes and apply (for initial load and system changes)
  $effect(() => {
    const targetTheme = settings.theme === 'system' 
      ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
      : settings.theme;
    document.documentElement.setAttribute('data-theme', targetTheme);
  });

  // Watch accent color changes and apply
  $effect(() => {
    const color = accentColors[settings.accentColor] || accentColors.red;
    const root = document.documentElement;
    root.style.setProperty('--color-accent', color);
    root.style.setProperty('--color-accent-hover', color);
    root.style.setProperty('--color-accent-dim', `${color}1a`); // 10% opacity
  });

  // Watch always-on-top changes
  $effect(() => {
    invoke('set_always_on_top', { enabled: settings.alwaysOnTop });
  });
</script>

{#if !$auth.loading && !$auth.unlocked}
  <LockScreen vaultExists={$auth.vaultExists} />
{:else}
  <div class="app">
    <!-- Minimal Top Bar -->
    <header class="topbar glass" data-tauri-drag-region>
      <div class="topbar-left">
        <img src="/AppIcon.png" alt="ROKIO" class="app-icon" />
        <div class="app-info">
          <span class="app-name">Rokio Account Manager</span>
          <span class="version">v1.1.0-a67r</span>
        </div>
      </div>

      <nav class="tabs">
        <button
          class="tab"
          class:active={currentTab === "accounts"}
          onclick={() => currentTab = "accounts"}
        >
          Accounts
        </button>
        <button
          class="tab"
          class:active={currentTab === "servers"}
          onclick={() => currentTab = "servers"}
        >
          Servers
        </button>
        <button
          class="tab"
          class:active={currentTab === "console"}
          onclick={() => currentTab = "console"}
        >
          Console
        </button>
        <button
          class="tab"
          class:active={currentTab === "settings"}
          onclick={() => currentTab = "settings"}
        >
          Settings
        </button>
      </nav>

      <div class="topbar-right">
        <button class="icon-btn" onclick={() => auth.lock()} title="Lock">
          <Lock size={16} />
        </button>
      </div>
    </header>

    <!-- Content -->
    <main class="content">
      {#if currentTab === "accounts"}
        {@render children()}
      {:else if currentTab === "servers"}
        <ServerBrowser />
      {:else if currentTab === "console"}
        <div class="console-view">
          <div class="console-header">
            <Terminal size={14} />
            <span>Console Output</span>
            <button class="clear-btn" onclick={() => consoleLogs = []}>Clear</button>
          </div>
          <div class="console-logs">
            {#each consoleLogs as log}
              <div class="log-line {log.type}">
                <span class="log-time">[{log.time}]</span>
                <span class="log-type">[{log.type}]</span>
                <span class="log-arrow">&gt;</span>
                <span class="log-msg">{log.msg}</span>
              </div>
            {:else}
              <div class="empty-logs">No logs yet...</div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="settings-view">
          <h2>Settings</h2>
          
          <!-- General Section -->
          <div class="settings-section">
            <div class="section-header">General</div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Auto-lock timeout</span>
                <span class="setting-desc">Lock app after inactivity</span>
              </div>
              <select class="setting-select" bind:value={settings.autoLockTimeout} onchange={saveSettings}>
                <option value="never">Never</option>
                <option value="1min">1 minute</option>
                <option value="5min">5 minutes</option>
                <option value="15min">15 minutes</option>
              </select>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Launch on startup</span>
                <span class="setting-desc">Start ROKIO when system boots</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.launchOnStartup} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Always on top</span>
                <span class="setting-desc">Keep window above other apps</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.alwaysOnTop} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <!-- Appearance Section - GNOME Style -->
          <div class="settings-section">
            <div class="section-header">Appearance</div>
            
            <!-- Theme Cards -->
            <div class="theme-cards-container">
              <span class="theme-cards-label">Style</span>
              <div class="theme-cards">
                <button 
                  class="theme-card" 
                  class:selected={settings.theme === 'light'}
                  onclick={(e) => { settings.theme = 'light'; setThemeWithTransition('light', e); saveSettings(); }}
                >
                  <div class="theme-preview light-preview">
                    <div class="preview-window light-window">
                      <div class="preview-titlebar"></div>
                      <div class="preview-content"></div>
                    </div>
                  </div>
                  <span class="theme-label">Light</span>
                </button>
                
                <button 
                  class="theme-card" 
                  class:selected={settings.theme === 'dark'}
                  onclick={(e) => { settings.theme = 'dark'; setThemeWithTransition('dark', e); saveSettings(); }}
                >
                  <div class="theme-preview dark-preview">
                    <div class="preview-window dark-window">
                      <div class="preview-titlebar"></div>
                      <div class="preview-content"></div>
                    </div>
                  </div>
                  <span class="theme-label">Dark</span>
                </button>
              </div>
            </div>

            <!-- Accent Color Picker -->
            <div class="accent-colors-container">
              <span class="accent-colors-label">Color</span>
              <div class="accent-colors">
                {#each [
                  { name: 'red', color: '#ff4d4d' },
                  { name: 'orange', color: '#ff9500' },
                  { name: 'amber', color: '#ffb300' },
                  { name: 'yellow', color: '#ffcc00' },
                  { name: 'lime', color: '#a3e635' },
                  { name: 'green', color: '#34c759' },
                  { name: 'mint', color: '#00c7be' },
                  { name: 'teal', color: '#5ac8fa' },
                  { name: 'cyan', color: '#00bcd4' },
                  { name: 'blue', color: '#007aff' },
                  { name: 'indigo', color: '#5856d6' },
                  { name: 'purple', color: '#af52de' },
                  { name: 'pink', color: '#ff2d55' },
                  { name: 'rose', color: '#f43f5e' },
                  { name: 'brown', color: '#a18072' },
                  { name: 'graphite', color: '#6b7280' }
                ] as accent}
                  <button 
                    class="accent-swatch" 
                    class:selected={settings.accentColor === accent.name}
                    style="--swatch-color: {accent.color}"
                    title={accent.name}
                    onclick={() => { settings.accentColor = accent.name; saveSettings(); }}
                  ></button>
                {/each}
              </div>
            </div>

            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Compact mode</span>
                <span class="setting-desc">Reduce row height in account list</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.compactMode} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <!-- Launch Options Section -->
          <div class="settings-section">
            <div class="section-header">Launch Options</div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Launcher</span>
                <span class="setting-desc">Preferred Roblox launcher (Windows)</span>
              </div>
              <select class="setting-select" bind:value={settings.launcherPreference} onchange={saveSettings}>
                <option value="default">Default (Protocol)</option>
                <option value="bloxstrap">Bloxstrap</option>
                <option value="fishstrap">Fishstrap</option>
                <option value="froststrap">Froststrap</option>
                <option value="client">Roblox Client (Vanilla)</option>
              </select>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Multi-Instance Mode</span>
                <span class="setting-desc">Launch multiple Roblox accounts simultaneously</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.multiInstance} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Quarantine Installers</span>
                <span class="setting-desc">Prevent Roblox update popups (Windows)</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.quarantineInstallers} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <!-- Advanced Section (Multiblox-style) -->
          <div class="settings-section">
            <div class="section-header">Advanced</div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Save Logs</span>
                <span class="setting-desc">Auto-save session logs when ROKIO closes</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.saveLogs} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Force Handle Closure</span>
                <span class="setting-desc">Aggressive handle resolution for multi-instance</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.forceHandleClosure} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Low-CPU Mode</span>
                <span class="setting-desc">Reduce scan frequency for lower CPU usage</span>
              </div>
              <label class="toggle">
                <input type="checkbox" bind:checked={settings.lowCpuMode} onchange={saveSettings} />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <!-- Security Section -->
          <div class="settings-section">
            <div class="section-header">Security</div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Change master password</span>
                <span class="setting-desc">Update your vault password</span>
              </div>
              <button class="setting-btn">Change</button>
            </div>
          </div>

          <!-- About Section -->
          <div class="settings-section">
            <div class="section-header">About</div>
            <div class="setting-row clickable" onclick={() => showAboutModal = true}>
              <div class="setting-info">
                <span class="setting-label">ROKIO</span>
                <span class="setting-desc">v{appInfo?.version || '0.0.0'} • Click for changelog</span>
              </div>
              <span class="setting-chevron">›</span>
            </div>
          </div>

          <!-- Credits Section -->
          <div class="settings-section">
            <div class="section-header">Credits</div>
            <div class="setting-row clickable" onclick={() => window.open('https://github.com/Ic0u', '_blank')}>
              <div class="setting-info">
                <span class="setting-label">Developer</span>
                <span class="setting-desc">Nam Nguyễn (@Ic0u)</span>
              </div>
              <span class="setting-chevron">›</span>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Inspired by</span>
                <span class="setting-desc">Roblox Account Manager by evanovar</span>
              </div>
            </div>
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Built with</span>
                <span class="setting-desc">Tauri • SvelteKit • Rust</span>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </main>
  </div>

  <!-- Modals -->
  {#if $ui.addAccountModalOpen}
    <AddAccountModal />
  {/if}
  {#if $ui.launchModalOpen && $ui.launchModalAccount}
    <LaunchModal account={$ui.launchModalAccount} />
  {/if}
  {#if $ui.quickLoginModalOpen}
    <QuickLoginModal />
  {/if}
  {#if $ui.browserLoginModalOpen}
    <BrowserLoginModal />
  {/if}
  {#if $ui.editAccountModalOpen && $ui.editModalAccount}
    <EditAccountModal account={$ui.editModalAccount} />
  {/if}
  {#if $ui.accountInfoModalOpen && $ui.infoModalAccount}
    <AccountInfoModal account={$ui.infoModalAccount} />
  {/if}

  <!-- About Modal -->
  {#if showAboutModal}
    <div class="modal-overlay" onclick={() => showAboutModal = false}>
      <div class="about-modal" onclick={(e) => e.stopPropagation()}>
        <div class="about-header">
          <img src="/AppIcon.png" alt="ROKIO" class="about-icon" />
          <div class="about-title">
            <h2>ROKIO</h2>
            <span class="about-version">Version {appInfo?.version || '0.0.0'}</span>
          </div>
        </div>
        
        <div class="about-tagline">
          Roblox Account Manager
        </div>

        <div class="platform-badges">
          <span class="platform-badge">
            <svg class="platform-icon" viewBox="0 0 24 24" fill="currentColor"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/></svg>
            macOS
          </span>
          <span class="platform-badge">
            <svg class="platform-icon" viewBox="0 0 88 88"><path d="m0 12.402 35.687-4.86.016 34.423-35.67.203zm35.67 33.529.028 34.453L.028 75.48.026 45.7zm4.326-39.025L87.314 0v41.527l-47.318.376zm47.329 39.349-.011 41.34-47.318-6.678-.066-34.739z" fill="#00adef"/></svg>
            Windows
          </span>
          <span class="platform-badge">
            <svg class="platform-icon" viewBox="0 0 24 24" fill="currentColor"><path d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.4v.019c0 .101 0 .2.008.267a.66.66 0 00.167.332c.055.053.109.086.163.12a1.015 1.015 0 00-.106.05c-.136.067-.209.135-.32.2h-.003c-.2.135-.39.333-.58.6-.19.135-.397.265-.598.4-.199.135-.394.265-.597.399-.334.2-.553.465-.74.8-.02.036-.035.07-.053.107-.041.081-.116.182-.187.26-.079.089-.222.138-.333.2-.058.03-.113.06-.163.087-.045.03-.09.055-.132.085-.205.13-.32.265-.57.465-.247.2-.487.4-.68.6-.187.199-.392.398-.525.598-.134.2-.267.465-.334.865-.067.398-.1.797-.066 1.196.033.399.1.798.233 1.196.134.398.333.797.6 1.196.134.199.269.398.469.597.2.2.4.398.665.531.265.134.597.2.861.2.265 0 .53-.066.729-.132a7.46 7.46 0 00-.598 2.924c0 .531.132 1.063.394 1.594.262.532.66.97 1.192 1.304.53.333 1.193.533 1.926.597.733.065 1.465.065 2.197 0 .732-.064 1.396-.264 1.926-.597a3.47 3.47 0 001.192-1.304c.262-.531.394-1.063.394-1.594a7.46 7.46 0 00-.598-2.924c.199.066.464.132.729.132.264 0 .596-.066.861-.2.265-.133.465-.331.665-.531.2-.199.335-.398.469-.598.267-.398.466-.797.6-1.196.133-.398.2-.797.233-1.196.034-.399 0-.798-.066-1.196-.067-.4-.2-.665-.334-.865-.133-.2-.338-.399-.525-.598-.193-.2-.433-.4-.68-.6-.25-.2-.365-.335-.57-.465-.042-.03-.087-.055-.132-.085-.05-.027-.105-.057-.163-.087-.111-.062-.254-.111-.333-.2-.071-.078-.146-.179-.187-.26-.018-.037-.033-.071-.053-.107-.187-.335-.406-.6-.74-.8a9.03 9.03 0 00-.597-.399 7.36 7.36 0 00-.598-.4c-.19-.267-.38-.465-.58-.6h-.003a1.353 1.353 0 00-.32-.2c-.037-.017-.073-.036-.106-.05.054-.034.108-.067.163-.12a.66.66 0 00.167-.332c.008-.067.008-.166.008-.267v-.02a1.18 1.18 0 00-.09-.398.8.8 0 00-.205-.334c-.086-.102-.169-.132-.262-.132h-.016c-.1 0-.183.066-.267.132-.082.133-.138.199-.183.333a1.21 1.21 0 00-.061.4v.019c.006.138.035.274.088.402.037.065.133.138.183.198a1.312 1.312 0 00-.22.066c-.086.069-.18.088-.284.133a.71.71 0 00-.088.042.953.953 0 01-.213-.335 1.807 1.807 0 01-.15-.706l-.004.024a.086.086 0 01-.004.021v-.105c0 .02.006.04.006.06.008-.265.061-.465.166-.724.108-.2.248-.398.438-.533.188-.136.371-.198.584-.198z"/></svg>
            Linux
          </span>
        </div>

        <div class="changelog-section">
          <h3>Changelog</h3>
          <div class="changelog-list">
            {#each changelog as release}
              <div class="changelog-release" class:highlight={release.highlight}>
                <div class="release-header">
                  <span class="release-version">v{release.version}</span>
                  {#if release.highlight}
                    <span class="release-badge">Latest</span>
                  {/if}
                  <span class="release-date">{release.date}</span>
                </div>
                <ul class="release-changes">
                  {#each release.changes as change}
                    <li>{change}</li>
                  {/each}
                </ul>
              </div>
            {/each}
          </div>
        </div>

        <div class="about-footer">
          <div class="footer-left">
            <span class="copyright">© 2026 Nam Nguyễn</span>
          </div>
          <button class="about-close-btn" onclick={() => showAboutModal = false}>Done</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Toast notifications -->
  <ToastContainer />
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--color-bg-primary);
  }

  /* Top Bar - Minimal & Seamless */
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 52px; /* Slightly taller for modern feel */
    padding: 0 20px;
    background: transparent; /* Remove background */
    border-bottom: none; /* Remove border */
    flex-shrink: 0;
    z-index: 50;
  }

  .topbar-left, .topbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
  }

  .app-icon {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    object-fit: cover;
  }

  .app-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .app-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
    letter-spacing: 0.2px;
  }

  .version {
    font-size: 10px;
    color: var(--color-text-tertiary);
  }

  /* Tabs - SwiftUI Segmented Control Style */
  .tabs {
    display: flex;
    background: rgba(255, 255, 255, 0.05); /* Subtle background */
    backdrop-filter: blur(10px);
    border-radius: 8px;
    padding: 3px;
    gap: 2px;
    -webkit-app-region: no-drag;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .tab {
    padding: 6px 16px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.15s ease;
  }

  .tab:hover {
    color: var(--color-text-primary);
  }

  .tab.active {
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  /* Icon Button - Native macOS Style */
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: rgba(255, 255, 255, 0.05);
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  .icon-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--color-text-primary);
    border-color: rgba(255, 255, 255, 0.08);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .icon-btn:active {
    transform: translateY(0);
    box-shadow: none;
  }

  /* Content */
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-tertiary);
  }

  /* Console View - Raptor Style */
  .console-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e; /* VS Code-like dark background */
    border-radius: 12px;
    border: 1px solid var(--color-border);
    overflow: hidden;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
    font-family: "Monaco", "Menlo", "Consolas", monospace;
  }

  .console-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--color-bg-tertiary);
    border-bottom: 1px solid var(--color-border);
    font-size: 13px;
    color: var(--color-text-secondary);
    font-family: var(--font-stack-sans); /* Keep header sans-serif */
  }

  .console-header span {
    flex: 1;
    font-weight: 500;
  }

  .clear-btn {
    padding: 4px 8px;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 11px;
    color: var(--color-text-tertiary);
    cursor: pointer;
  }

  .clear-btn:hover {
    color: var(--color-text-primary);
    border-color: var(--color-text-tertiary);
  }

  .console-logs {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    font-size: 12px;
    line-height: 1.5;
    color: #f0f0f0; /* Brighter text for contrast */
  }

  .log-line {
    display: flex;
    gap: 12px;
    padding: 2px 0;
    border-bottom: none; /* Cleaner look without lines */
  }
  
  .log-line:hover {
    background: rgba(255, 255, 255, 0.05); /* Hover effect */
  }

  .log-time {
    color: #888;
    flex-shrink: 0;
    font-size: 13px;
    opacity: 0.8;
  }

  .log-type {
    font-weight: 500;
    flex-shrink: 0;
    font-size: 13px;
    text-transform: lowercase; /* Wave style uses lowercase */
  }

  .log-arrow {
    color: #888;
    margin: 0 4px;
    font-size: 13px;
  }

  /* Wave-style Colors */
  .log-line.info .log-type { color: #569cd6; } /* Blue */
  .log-line.success .log-type { color: #4ec9b0; } /* Teal */
  .log-line.error .log-type { color: #f44747; } /* Red */
  .log-line.warning .log-type { color: #cca700; } /* Orange/Yellow */
  .log-line.debug .log-type { color: #b180d7; } /* Purple for debug */

  .log-msg {
    color: #cccccc;
    word-break: break-all;
    font-family: inherit;
  }

  .empty-logs {
    color: var(--color-text-tertiary);
    text-align: center;
    padding: 20px;
  }

  /* Settings View - SwiftUI Style */
  .settings-view {
    max-width: 600px;
    margin: 0 auto;
  }

  .settings-view h2 {
    font-size: 22px;
    font-weight: 600;
    margin-bottom: 24px;
    color: var(--color-text-primary);
  }

  .settings-section {
    background: var(--color-bg-secondary);
    border-radius: 12px;
    border: 1px solid var(--color-border);
    margin-bottom: 20px;
    overflow: hidden;
  }

  .section-header {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 12px 16px 8px;
    background: var(--color-bg-tertiary);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .setting-desc {
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  .setting-select {
    appearance: none;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 6px 28px 6px 12px;
    font-size: 13px;
    color: var(--color-text-primary);
    cursor: pointer;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
  }

  .setting-btn {
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-accent);
    cursor: pointer;
    transition: all 0.15s;
  }

  .setting-btn:hover {
    background: var(--color-accent);
    color: #fff;
  }

  /* Toggle Switch */
  .toggle {
    position: relative;
    width: 44px;
    height: 24px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    inset: 0;
    background: var(--color-bg-tertiary);
    border-radius: 24px;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid var(--color-border);
  }

  .toggle-slider::before {
    content: "";
    position: absolute;
    height: 18px;
    width: 18px;
    left: 2px;
    bottom: 2px;
    background: #fff;
    border-radius: 50%;
    transition: all 0.2s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .toggle input:checked + .toggle-slider {
    background: var(--color-accent);
    border-color: var(--color-accent);
  }

  .toggle input:checked + .toggle-slider::before {
    transform: translateX(20px);
  }

  /* Servers View */
  .servers-view {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 40px;
  }

  .coming-soon {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
    padding: 40px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 16px;
    border: 1px solid var(--color-border);
  }

  .coming-soon h3 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
  }

  .coming-soon p {
    margin: 0;
    color: var(--color-text-secondary);
    max-width: 280px;
  }

  .coming-soon .badge {
    padding: 4px 12px;
    background: rgba(59, 130, 246, 0.15);
    color: var(--color-accent);
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }

  :global(.icon-muted) {
    opacity: 0.3;
  }

  /* GNOME-Style Theme Cards */
  .theme-cards-container {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .theme-cards-label,
  .accent-colors-label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-secondary);
    margin-bottom: 12px;
  }

  .theme-cards {
    display: flex;
    gap: 16px;
    justify-content: center;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
  }

  .theme-preview {
    width: 120px;
    height: 80px;
    border-radius: 12px;
    padding: 10px;
    display: flex;
    align-items: flex-end;
    justify-content: flex-end;
    border: 3px solid transparent;
    transition: all 0.2s ease;
  }

  .theme-card.selected .theme-preview {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px var(--color-accent-dim);
  }

  .theme-card:hover .theme-preview {
    transform: scale(1.02);
  }

  .light-preview {
    background: linear-gradient(135deg, #ffb6a3 0%, #ffa07a 100%);
  }

  .dark-preview {
    background: linear-gradient(135deg, #ff8c69 0%, #e07356 100%);
  }

  .preview-window {
    width: 55px;
    height: 40px;
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
  }

  .light-window {
    background: #ffffff;
  }

  .dark-window {
    background: #2d2d2d;
  }

  .preview-titlebar {
    height: 10px;
    background: inherit;
    filter: brightness(0.95);
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  .dark-window .preview-titlebar {
    filter: brightness(1.2);
  }

  .preview-content {
    height: 30px;
  }

  .theme-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .theme-card.selected .theme-label {
    color: var(--color-text-primary);
    font-weight: 600;
  }

  /* Accent Color Swatches */
  .accent-colors-container {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .accent-colors {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: center;
    max-width: 320px;
    margin: 0 auto;
  }

  .accent-swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--swatch-color);
    border: 2px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .accent-swatch:hover {
    transform: scale(1.15);
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.3);
  }

  .accent-swatch.selected {
    border-color: var(--color-text-primary);
    box-shadow: 0 0 0 2px var(--color-bg-primary), 0 0 0 4px var(--swatch-color);
  }

  /* Clickable Setting Row */
  .setting-row.clickable {
    cursor: pointer;
  }

  .setting-row.clickable:hover {
    background: var(--color-bg-tertiary);
  }

  .setting-chevron {
    font-size: 20px;
    font-weight: 300;
    color: var(--color-text-tertiary);
  }

  /* Modal Overlay */
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease-out;
  }

  /* About Modal - SwiftUI Style */
  .about-modal {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 16px;
    width: 380px;
    max-height: 80vh;
    overflow: hidden;
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.5);
    animation: modalSlide 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes modalSlide {
    from {
      opacity: 0;
      transform: scale(0.95) translateY(10px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .about-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 24px 24px 16px;
  }

  .about-icon {
    width: 64px;
    height: 64px;
    border-radius: 14px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .about-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .about-title h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    color: var(--color-text-primary);
    letter-spacing: -0.3px;
  }

  .about-version {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .about-platform {
    font-size: 11px;
    color: var(--color-text-tertiary);
    text-transform: capitalize;
  }

  .about-tagline {
    padding: 0 24px 12px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-align: center;
  }

  /* Platform Badges */
  .platform-badges {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding: 0 24px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .platform-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 16px;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .platform-icon {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  /* Release Badge */
  .release-badge {
    padding: 2px 8px;
    background: var(--color-accent-dim);
    color: var(--color-accent);
    border-radius: 8px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .changelog-release.highlight {
    border: 1px solid var(--color-accent-dim);
  }

  .release-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .release-date {
    margin-left: auto;
  }

  /* Changelog Section */
  .changelog-section {
    padding: 16px 24px;
    max-height: 280px;
    overflow-y: auto;
  }

  .changelog-section h3 {
    margin: 0 0 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .changelog-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .changelog-release {
    background: var(--color-bg-tertiary);
    border-radius: 10px;
    padding: 12px;
  }

  .release-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .release-version {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-accent);
  }

  .release-date {
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  .release-changes {
    margin: 0;
    padding-left: 16px;
    font-size: 12px;
    color: var(--color-text-secondary);
    line-height: 1.6;
  }

  .release-changes li {
    margin-bottom: 2px;
  }

  /* About Footer */
  .about-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 24px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-tertiary);
  }

  .copyright {
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  .about-close-btn {
    background: var(--color-accent);
    border: none;
    border-radius: 8px;
    padding: 8px 20px;
    font-size: 13px;
    font-weight: 600;
    color: white;
    cursor: pointer;
    transition: all 0.15s;
  }

  .about-close-btn:hover {
    background: var(--color-accent-hover);
    transform: scale(1.02);
  }

  .about-close-btn:active {
    transform: scale(0.98);
  }
</style>

