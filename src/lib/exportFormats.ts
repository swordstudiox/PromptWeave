import type { OptimizedPrompt } from "../types/prompt";

export type ExportFormat = "gpt-image" | "midjourney" | "stable-diffusion";

export function formatPrompt(prompt: OptimizedPrompt, format: ExportFormat): string {
  if (format === "midjourney") {
    return `${prompt.en} --ar 1:1 --style raw --no watermark, malformed hands, distorted text`;
  }

  if (format === "stable-diffusion") {
    return [
      "Positive Prompt:",
      prompt.en,
      "",
      "Negative Prompt:",
      prompt.structured.negativePrompt,
      "",
      "Suggested Settings:",
      "Size: 1024x1024",
      "Steps: 28",
      "CFG: 6.5",
      "Sampler: DPM++ 2M Karras",
    ].join("\n");
  }

  return `${prompt.en}\n\nAvoid: ${prompt.structured.negativePrompt}`;
}
