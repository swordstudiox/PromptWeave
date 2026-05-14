import { invoke } from "@tauri-apps/api/core";
import type {
  GenerationHistoryDraft,
  GenerationHistoryRecord,
  ImageGenerationOptions,
  ImageGenerationResult,
  PromptOptimizationResult,
} from "../../types/backend";

export function generateImagePreview(
  prompt: string,
  options: ImageGenerationOptions,
): Promise<ImageGenerationResult> {
  return invoke<ImageGenerationResult>("generate_image_preview", { prompt, options });
}

export function optimizePromptWithApi(localPrompt: string): Promise<PromptOptimizationResult> {
  return invoke<PromptOptimizationResult>("optimize_prompt_with_api", { localPrompt });
}

export function saveGenerationHistory(draft: GenerationHistoryDraft): Promise<void> {
  return invoke("save_generation_history", { draft });
}

export function listGenerationHistory(limit = 50): Promise<GenerationHistoryRecord[]> {
  return invoke<GenerationHistoryRecord[]>("list_generation_history", { limit });
}
