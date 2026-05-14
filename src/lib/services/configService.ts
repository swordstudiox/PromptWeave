import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../../types/backend";

export function getAppConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_app_config");
}

export function saveAppConfig(config: AppConfig): Promise<AppConfig> {
  return invoke<AppConfig>("save_app_config", { config });
}
