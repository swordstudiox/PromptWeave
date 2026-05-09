import { describe, expect, it } from "vitest";
import { formatPrompt } from "./exportFormats";
import type { CreationSettings, OptimizedPrompt } from "../types/prompt";

const prompt: OptimizedPrompt = {
  zh: "中文提示词",
  en: "A cinematic product poster",
  matchedTemplateTitles: [],
  structured: {
    subject: "product",
    scene: "studio",
    style: "cinematic",
    composition: "centered",
    camera: "medium shot",
    lighting: "soft light",
    color: "red",
    details: "high detail",
    negativePrompt: "watermark",
  },
};

describe("formatPrompt", () => {
  it("uses Midjourney creation settings", () => {
    const settings: CreationSettings = {
      aspectRatio: "16:9",
      imageSize: "1536x864",
      imageQuality: "high",
      imageCount: 1,
      midjourneyStylize: 250,
      midjourneyChaos: 12,
      sdSteps: 32,
      sdCfg: 7,
      sdSampler: "DPM++ 2M Karras",
      sdSeed: "123",
    };

    const formatted = formatPrompt(prompt, "midjourney", settings);

    expect(formatted).toContain("--ar 16:9");
    expect(formatted).toContain("--s 250");
    expect(formatted).toContain("--chaos 12");
  });

  it("uses Stable Diffusion creation settings", () => {
    const settings: CreationSettings = {
      aspectRatio: "1:1",
      imageSize: "1024x1024",
      imageQuality: "medium",
      imageCount: 1,
      midjourneyStylize: 100,
      midjourneyChaos: 0,
      sdSteps: 40,
      sdCfg: 8.5,
      sdSampler: "Euler a",
      sdSeed: "999",
    };

    const formatted = formatPrompt(prompt, "stable-diffusion", settings);

    expect(formatted).toContain("Size: 1024x1024");
    expect(formatted).toContain("Steps: 40");
    expect(formatted).toContain("CFG: 8.5");
    expect(formatted).toContain("Sampler: Euler a");
    expect(formatted).toContain("Seed: 999");
  });
});
