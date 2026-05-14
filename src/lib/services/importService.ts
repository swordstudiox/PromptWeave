import { invoke } from "@tauri-apps/api/core";
import type {
  ImportPreview,
  ImportResult,
  PromptLibrarySourceRecord,
} from "../../types/backend";

export function listPromptLibrarySources(): Promise<PromptLibrarySourceRecord[]> {
  return invoke<PromptLibrarySourceRecord[]>("list_prompt_library_sources");
}

export function previewImportUrl(url: string): Promise<ImportPreview> {
  return invoke<ImportPreview>("preview_import_url", { url });
}

export function importPromptLibrary(url: string): Promise<ImportResult> {
  return invoke<ImportResult>("import_prompt_library", { url });
}

export function syncPromptLibrarySource(sourceId: string): Promise<ImportResult> {
  return invoke<ImportResult>("sync_prompt_library_source", { sourceId });
}
