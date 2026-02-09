// ROKIO Auth Store
// Manages vault lock/unlock state

import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { VaultStatus } from "$lib/types";

interface AuthState {
  vaultExists: boolean;
  unlocked: boolean;
  loading: boolean;
  error: string | null;
}

function createAuthStore() {
  const { subscribe, update, set } = writable<AuthState>({
    vaultExists: false,
    unlocked: false,
    loading: true,
    error: null,
  });

  return {
    subscribe,

    // Check vault status
    checkStatus: async () => {
      update((s) => ({ ...s, loading: true }));
      try {
        const status = await invoke<VaultStatus>("get_vault_status");
        update((s) => ({
          ...s,
          vaultExists: status.exists,
          unlocked: status.unlocked,
          loading: false,
        }));
        return status;
      } catch (err) {
        update((s) => ({ ...s, error: String(err), loading: false }));
        throw err;
      }
    },

    // Create new vault with password
    createVault: async (password: string) => {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        await invoke("create_vault", { password });
        update((s) => ({
          ...s,
          vaultExists: true,
          unlocked: true,
          loading: false,
        }));
      } catch (err) {
        update((s) => ({ ...s, error: String(err), loading: false }));
        throw err;
      }
    },

    // Unlock existing vault
    unlock: async (password: string): Promise<boolean> => {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        const success = await invoke<boolean>("unlock_vault", { password });
        if (success) {
          update((s) => ({ ...s, unlocked: true, loading: false }));
        } else {
          update((s) => ({
            ...s,
            error: "Incorrect password",
            loading: false,
          }));
        }
        return success;
      } catch (err) {
        update((s) => ({ ...s, error: String(err), loading: false }));
        throw err;
      }
    },

    // Lock vault
    lock: async () => {
      try {
        await invoke("lock_vault");
        update((s) => ({ ...s, unlocked: false }));
      } catch (err) {
        update((s) => ({ ...s, error: String(err) }));
      }
    },

    // Clear error
    clearError: () => {
      update((s) => ({ ...s, error: null }));
    },
  };
}

export const auth = createAuthStore();
