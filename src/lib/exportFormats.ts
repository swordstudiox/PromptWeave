import type { CreationSettings, OptimizedPrompt } from "../types/prompt";

export type ExportFormat = "gpt-image" | "midjourney" | "stable-diffusion";

const defaultSettings: CreationSettings = {
  aspectRatio: "1:1",
  imageSize: "1024x1024",
  imageQuality: "medium",
  imageCount: 1,
  midjourneyStylize: 100,
  midjourneyChaos: 0,
  sdSteps: 28,
  sdCfg: 6.5,
  sdSampler: "DPM++ 2M Karras",
  sdSeed: "",
};

export function formatPrompt(prompt: OptimizedPrompt, format: ExportFormat, settings: CreationSettings = defaultSettings): string {
  if (format === "midjourney") {
    return `${prompt.en} --ar ${settings.aspectRatio} --style raw --s ${settings.midjourneyStylize} --chaos ${settings.midjourneyChaos} --no watermark, malformed hands, distorted text`;
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
      `Size: ${settings.imageSize}`,
      `Steps: ${settings.sdSteps}`,
      `CFG: ${settings.sdCfg}`,
      `Sampler: ${settings.sdSampler}`,
      settings.sdSeed.trim() ? `Seed: ${settings.sdSeed.trim()}` : "Seed: random",
    ].join("\n");
  }

  return `${prompt.en}\n\nAvoid: ${prompt.structured.negativePrompt}`;
}
