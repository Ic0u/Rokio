// ROKIO UI Store
// Manages modals and UI state

import { writable } from "svelte/store";
import type { Profile } from "$lib/types";

interface UIState {
  addAccountModalOpen: boolean;
  settingsModalOpen: boolean;
  launchModalOpen: boolean;
  launchModalAccount: Profile | null;
  quickLoginModalOpen: boolean;
  browserLoginModalOpen: boolean;
  editAccountModalOpen: boolean;
  editModalAccount: Profile | null;
  accountInfoModalOpen: boolean;
  infoModalAccount: Profile | null;
}

function createUIStore() {
  const { subscribe, update } = writable<UIState>({
    addAccountModalOpen: false,
    settingsModalOpen: false,
    launchModalOpen: false,
    launchModalAccount: null,
    quickLoginModalOpen: false,
    browserLoginModalOpen: false,
    editAccountModalOpen: false,
    editModalAccount: null,
    accountInfoModalOpen: false,
    infoModalAccount: null,
  });

  return {
    subscribe,

    openAddAccount: () => {
      update((s) => ({ ...s, addAccountModalOpen: true }));
    },

    closeAddAccount: () => {
      update((s) => ({ ...s, addAccountModalOpen: false }));
    },

    openSettings: () => {
      update((s) => ({ ...s, settingsModalOpen: true }));
    },

    closeSettings: () => {
      update((s) => ({ ...s, settingsModalOpen: false }));
    },

    openLaunchModal: (account: Profile) => {
      update((s) => ({ ...s, launchModalOpen: true, launchModalAccount: account }));
    },

    closeLaunchModal: () => {
      update((s) => ({ ...s, launchModalOpen: false, launchModalAccount: null }));
    },

    openQuickLogin: () => {
      update((s) => ({ ...s, quickLoginModalOpen: true }));
    },

    closeQuickLogin: () => {
      update((s) => ({ ...s, quickLoginModalOpen: false }));
    },

    openBrowserLogin: () => {
      update((s) => ({ ...s, browserLoginModalOpen: true }));
    },

    closeBrowserLogin: () => {
      update((s) => ({ ...s, browserLoginModalOpen: false }));
    },

    openEditAccount: (account: Profile) => {
      update((s) => ({ ...s, editAccountModalOpen: true, editModalAccount: account }));
    },

    closeEditAccount: () => {
      update((s) => ({ ...s, editAccountModalOpen: false, editModalAccount: null }));
    },

    openAccountInfo: (account: Profile) => {
      update((s) => ({ ...s, accountInfoModalOpen: true, infoModalAccount: account }));
    },

    closeAccountInfo: () => {
      update((s) => ({ ...s, accountInfoModalOpen: false, infoModalAccount: null }));
    },
  };
}

export const ui = createUIStore();
