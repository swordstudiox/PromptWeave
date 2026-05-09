import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceInfo {
  root: string;
  dataDir: string;
  databasePath: string;
}

export async function initWorkspace(): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("init_workspace");
}
