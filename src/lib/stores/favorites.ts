/**
 * Favorites Store - Manages favorite games for quick launch
 */
import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface FavoriteGame {
  id: string;
  placeId: number;
  name: string;
  thumbnail?: string;
  addedAt: number;
}

interface FavoritesState {
  games: FavoriteGame[];
  loading: boolean;
  error: string | null;
}

const initialState: FavoritesState = {
  games: [],
  loading: false,
  error: null,
};

function createFavoritesStore() {
  const { subscribe, set, update } = writable<FavoritesState>(initialState);

  return {
    subscribe,

    // Load favorites from settings
    async load() {
      update((s) => ({ ...s, loading: true }));
      try {
        const settings = await invoke<{ favoriteGames?: FavoriteGame[] }>("get_settings");
        update((s) => ({
          ...s,
          games: settings.favoriteGames || [],
          loading: false,
          error: null,
        }));
      } catch (err) {
        update((s) => ({ ...s, loading: false, error: String(err) }));
      }
    },

    // Add a game to favorites
    async add(game: Omit<FavoriteGame, "id" | "addedAt">) {
      const newGame: FavoriteGame = {
        ...game,
        id: crypto.randomUUID(),
        addedAt: Date.now(),
      };

      update((s) => ({
        ...s,
        games: [...s.games, newGame],
      }));

      await this.save();
    },

    // Remove a game from favorites
    async remove(gameId: string) {
      update((s) => ({
        ...s,
        games: s.games.filter((g) => g.id !== gameId),
      }));

      await this.save();
    },

    // Save to settings
    async save() {
      const state = get({ subscribe });
      try {
        const settings = await invoke<Record<string, unknown>>("get_settings");
        await invoke("save_settings", {
          settings: {
            ...settings,
            favoriteGames: state.games,
          },
        });
      } catch (err) {
        console.error("Failed to save favorites:", err);
      }
    },

    // Check if a game is favorited
    isFavorite(placeId: number): boolean {
      const state = get({ subscribe });
      return state.games.some((g) => g.placeId === placeId);
    },

    // Get all favorites
    getAll(): FavoriteGame[] {
      return get({ subscribe }).games;
    },
  };
}

export const favorites = createFavoritesStore();
