// Toast notification store for UI feedback
import { writable, derived } from "svelte/store";

export interface Toast {
  id: string;
  type: "success" | "error" | "info" | "warning";
  message: string;
  duration?: number;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  let counter = 0;

  return {
    subscribe,

    // Add a toast notification
    show(message: string, type: Toast["type"] = "info", duration = 3000) {
      const id = `toast-${++counter}`;
      const toast: Toast = { id, type, message, duration };

      update((toasts) => [...toasts, toast]);

      // Auto-remove after duration
      if (duration > 0) {
        setTimeout(() => {
          this.dismiss(id);
        }, duration);
      }

      return id;
    },

    // Convenience methods
    success(message: string, duration = 3000) {
      return this.show(message, "success", duration);
    },

    error(message: string, duration = 5000) {
      return this.show(message, "error", duration);
    },

    info(message: string, duration = 3000) {
      return this.show(message, "info", duration);
    },

    warning(message: string, duration = 4000) {
      return this.show(message, "warning", duration);
    },

    // Remove a toast by ID
    dismiss(id: string) {
      update((toasts) => toasts.filter((t) => t.id !== id));
    },

    // Clear all toasts
    clear() {
      update(() => []);
    },
  };
}

export const toasts = createToastStore();
