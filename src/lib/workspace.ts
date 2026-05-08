import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceInfo {
  root: string;
  data_dir: string;
  database_path: string;
}

export async function initWorkspace(): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("init_workspace");
}
