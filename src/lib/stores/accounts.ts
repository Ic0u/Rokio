// ROKIO Accounts Store
// Manages account state with Tauri IPC

import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Profile } from "$lib/types";

interface AccountsState {
  accounts: Profile[];
  loading: boolean;
  error: string | null;
  selectedId: string | null;
}

function createAccountsStore() {
  const { subscribe, update, set } = writable<AccountsState>({
    accounts: [],
    loading: false,
    error: null,
    selectedId: null,
  });

  return {
    subscribe,

    // Load all accounts from vault
    load: async () => {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        const accounts = await invoke<Profile[]>("get_accounts");
        update((s) => ({ ...s, accounts, loading: false }));
      } catch (err) {
        update((s) => ({ ...s, error: String(err), loading: false }));
      }
    },

    // Add a new account
    add: async (cookie: string): Promise<Profile> => {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        const profile = await invoke<Profile>("add_account", { cookie });
        update((s) => ({
          ...s,
          accounts: [...s.accounts, profile],
          loading: false,
        }));
        return profile;
      } catch (err) {
        update((s) => ({ ...s, error: String(err), loading: false }));
        throw err;
      }
    },

    // Update an existing account
    update: async (profile: Profile) => {
      try {
        await invoke("update_account", { profile });
        update((s) => ({
          ...s,
          accounts: s.accounts.map((a) => (a.id === profile.id ? profile : a)),
        }));
      } catch (err) {
        update((s) => ({ ...s, error: String(err) }));
        throw err;
      }
    },

    // Delete an account
    delete: async (id: string) => {
      try {
        await invoke("delete_account", { id });
        update((s) => ({
          ...s,
          accounts: s.accounts.filter((a) => a.id !== id),
        }));
      } catch (err) {
        update((s) => ({ ...s, error: String(err) }));
        throw err;
      }
    },

    // Toggle favorite
    toggleFavorite: async (id: string) => {
      update((s) => {
        const account = s.accounts.find((a) => a.id === id);
        if (account) {
          const updated = { ...account, isFavorite: !account.isFavorite };
          invoke("update_account", { profile: updated }).catch(console.error);
          return {
            ...s,
            accounts: s.accounts.map((a) => (a.id === id ? updated : a)),
          };
        }
        return s;
      });
    },

    // Select an account
    select: (id: string | null) => {
      update((s) => ({ ...s, selectedId: id }));
    },

    // Clear error
    clearError: () => {
      update((s) => ({ ...s, error: null }));
    },

    // Reset store
    reset: () => {
      set({
        accounts: [],
        loading: false,
        error: null,
        selectedId: null,
      });
    },

    // Export accounts to JSON
    exportAccounts: async (): Promise<string> => {
      return await invoke<string>("export_accounts");
    },

    // Import accounts from JSON
    importAccounts: async (data: string, merge: boolean = true): Promise<number> => {
      const count = await invoke<number>("import_accounts", { data, merge });
      // Reload accounts after import
      const accounts = await invoke<Profile[]>("get_accounts");
      update((s) => ({ ...s, accounts }));
      return count;
    },

    // Clear all accounts
    clearAll: async () => {
      if (confirm("Are you sure you want to delete ALL accounts? This cannot be undone.")) {
        await invoke("clear_accounts");
        update((s) => ({ ...s, accounts: [] }));
      }
    },
  };
}

export const accounts = createAccountsStore();

// Derived stores
export const favoriteAccounts = derived(accounts, ($accounts) =>
  $accounts.accounts.filter((a) => a.isFavorite)
);

export const accountCount = derived(
  accounts,
  ($accounts) => $accounts.accounts.length
);
