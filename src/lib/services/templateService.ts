import { invoke } from "@tauri-apps/api/core";
import type {
  DuplicateCleanupResult,
  PromptTemplateRecord,
  TemplateUpdateDraft,
} from "../../types/backend";
import type { PromptTemplateReference } from "../../types/prompt";

export function listPromptTemplates(limit = 50): Promise<PromptTemplateRecord[]> {
  return invoke<PromptTemplateRecord[]>("list_prompt_templates", { limit });
}

export function searchPromptTemplates(query: string, limit = 50): Promise<PromptTemplateRecord[]> {
  return invoke<PromptTemplateRecord[]>("search_prompt_templates", { query, limit });
}

export async function searchPromptTemplateReferences(
  query: string,
  limit = 3,
): Promise<PromptTemplateReference[]> {
  const records = await searchPromptTemplates(query, limit);
  return records.map(mapPromptTemplateRecordToReference);
}

export function updatePromptTemplate(draft: TemplateUpdateDraft): Promise<void> {
  return invoke("update_prompt_template", { draft });
}

export function togglePromptTemplateFavorite(id: string, isFavorite: boolean): Promise<void> {
  return invoke("toggle_prompt_template_favorite", { id, isFavorite });
}

export function deletePromptTemplate(id: string): Promise<void> {
  return invoke("delete_prompt_template", { id });
}

export function cleanupDuplicatePromptTemplates(): Promise<DuplicateCleanupResult> {
  return invoke<DuplicateCleanupResult>("cleanup_duplicate_prompt_templates");
}

export function mapPromptTemplateRecordToReference(
  record: PromptTemplateRecord,
): PromptTemplateReference {
  return {
    title: record.title,
    category: record.category,
    promptOriginal: record.promptOriginal,
    negativePrompt: record.negativePrompt,
    tags: record.tags,
  };
}
