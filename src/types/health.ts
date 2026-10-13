/** DTOs returned by Rust commands (see src-tauri/src/services). */

export type StorageHealth = {
  ready: boolean;
  schemaVersion: number;
};

export type HealthReport = {
  appName: string;
  appVersion: string;
  storage: StorageHealth;
};
