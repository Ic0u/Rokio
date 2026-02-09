/**
 * Launcher Store
 * Manages active Roblox instances and launch operations.
 */

import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ActiveInstance {
  pid: number;
  accountId: string;
  username: string;
  placeId: number;
  startedAt: number;
}

interface LauncherState {
  instances: ActiveInstance[];
  launching: string | null; // account ID currently launching
  error: string | null;
}

const initialState: LauncherState = {
  instances: [],
  launching: null,
  error: null,
};

function createLauncherStore() {
  const { subscribe, set, update } = writable<LauncherState>(initialState);

  // Listen for instance-closed events
  let unlistenFn: (() => void) | null = null;
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function setupListener() {
    if (unlistenFn) return;

    unlistenFn = await listen<{
      pid: number;
      accountId: string;
      username: string;
    }>("instance-closed", (event) => {
      update((state) => ({
        ...state,
        instances: state.instances.filter((i) => i.pid !== event.payload.pid),
      }));
    });
  }

  // Start polling for process status (to detect when Roblox closes)
  function startPolling() {
    if (pollInterval) return;
    pollInterval = setInterval(async () => {
      // Only poll when there are active instances
      let hasInstances = false;
      subscribe((state) => {
        hasInstances = state.instances.length > 0;
      })();
      
      if (hasInstances) {
        // Refresh will clean up dead instances on the backend
        const instances = await invoke<ActiveInstance[]>("get_active_instances");
        update((state) => ({ ...state, instances }));
      }
    }, 2000); // Poll every 2 seconds
  }

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  return {
    subscribe,

    /**
     * Initialize the launcher store and start listening for events
     */
    async init() {
      await setupListener();
      startPolling();
      await this.refresh();
    },

    /**
     * Refresh the list of active instances
     */
    async refresh() {
      try {
        const instances = await invoke<ActiveInstance[]>("get_active_instances");
        update((state) => ({ ...state, instances }));
      } catch (err) {
        console.error("Failed to refresh instances:", err);
      }
    },

    /**
     * Launch Roblox for an account
     */
    async launch(
      accountId: string,
      placeId: number,
      jobId?: string
    ): Promise<ActiveInstance | null> {
      update((state) => ({ ...state, launching: accountId, error: null }));

      try {
        const instance = await invoke<ActiveInstance>("launch_game", {
          accountId,
          placeId,
          jobId,
        });

        update((state) => ({
          ...state,
          instances: [...state.instances, instance],
          launching: null,
        }));

        // Reload accounts to get updated lastPlayedAt
        invoke("get_accounts").then((accounts) => {
          // Update accounts store via dispatch (handled by accounts store)
          import("./accounts").then(({ accounts: accountsStore }) => {
            accountsStore.load();
          });
        }).catch(() => {});

        return instance;
      } catch (err) {
        update((state) => ({
          ...state,
          launching: null,
          error: String(err),
        }));
        return null;
      }
    },

    /**
     * Kill a running instance
     */
    async kill(pid: number): Promise<boolean> {
      try {
        await invoke("kill_instance", { pid });
        update((state) => ({
          ...state,
          instances: state.instances.filter((i) => i.pid !== pid),
        }));
        return true;
      } catch (err) {
        update((state) => ({ ...state, error: String(err) }));
        return false;
      }
    },

    /**
     * Bypass the singleton mutex for multi-instance
     */
    async bypassMutex(): Promise<number> {
      try {
        return await invoke<number>("bypass_mutex");
      } catch (err) {
        console.error("Mutex bypass failed:", err);
        return 0;
      }
    },

    /**
     * Check if an account is currently running
     */
    isRunning(accountId: string): boolean {
      let running = false;
      subscribe((state) => {
        running = state.instances.some((i) => i.accountId === accountId);
      })();
      return running;
    },

    /**
     * Get instance for an account
     */
    getInstance(accountId: string): ActiveInstance | undefined {
      let instance: ActiveInstance | undefined;
      subscribe((state) => {
        instance = state.instances.find((i) => i.accountId === accountId);
      })();
      return instance;
    },

    /**
     * Clear error
     */
    clearError() {
      update((state) => ({ ...state, error: null }));
    },
  };
}

export const launcher = createLauncherStore();

// Derived stores
export const activeInstances = derived(launcher, ($launcher) => $launcher.instances);
export const instanceCount = derived(launcher, ($launcher) => $launcher.instances.length);
export const isLaunching = derived(launcher, ($launcher) => $launcher.launching !== null);
