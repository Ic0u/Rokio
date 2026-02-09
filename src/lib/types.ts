// ROKIO TypeScript Types

export interface Profile {
  id: string;
  cookie: string;
  userId: number;
  username: string;
  displayName: string;
  thumbnail: string | null;
  alias: string;
  description: string;
  isFavorite: boolean;
  lastPlayedAt: number;
  password?: string;
  createdAt?: number;
  isPremium?: boolean;
}

export interface VaultStatus {
  exists: boolean;
  unlocked: boolean;
}

export interface AppInfo {
  version: string;
  platform: string;
  hwid: string;
}

export interface RobloxUserData {
  id: number;
  name: string;
  displayName: string;
  thumbnail: string | null;
}

export interface ActiveInstance {
  pid: number;
  accountId: string;
  username: string;
  placeId: number;
  startedAt: number;
}

export interface AppSettings {
  autoLockTimeout: string;  // "never" | "1min" | "5min" | "15min"
  launchOnStartup: boolean;
  alwaysOnTop: boolean;
  theme: string;  // "dark" | "light" | "system"
  compactMode: boolean;
  accentColor: string;  // "red" | "orange" | "yellow" | "green" | "teal" | "blue" | "indigo" | "purple" | "pink"
  multiInstance: boolean;  // Enable multi-instance launching
  launcherPreference: string;  // "default" | "bloxstrap" | "fishstrap" | "froststrap" | "client"
  quarantineInstallers: boolean;  // Prevent Roblox update popups (Windows)
  saveLogs: boolean;  // Save session logs
  forceHandleClosure: boolean;  // Aggressive handle resolution
  lowCpuMode: boolean;  // Reduce CPU usage
}
