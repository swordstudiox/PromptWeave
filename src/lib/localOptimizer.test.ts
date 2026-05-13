import { describe, expect, it } from "vitest";
import { optimizePromptLocally } from "./localOptimizer";
import type { PromptTemplateReference } from "../types/prompt";

describe("optimizePromptLocally", () => {
  it("uses matched template references when optimizing", () => {
    const templates: PromptTemplateReference[] = [
      {
        title: "Cinematic Snow Portrait",
        category: "Portrait",
        promptOriginal: "A cinematic portrait with dramatic rim light and snow particles.",
        negativePrompt: "watermark, distorted hands",
        tags: ["cinematic", "snow"],
      },
    ];

    const result = optimizePromptLocally("红色斗篷女孩在雪山上", templates);

    expect(result.matchedTemplateTitles).toEqual(["Cinematic Snow Portrait"]);
    expect(result.structured.details).toContain("参考模板：Cinematic Snow Portrait");
    expect(result.structured.negativePrompt).toContain("watermark");
    expect(result.zh).toContain("参考模板");
  });

  it("renders common Chinese concepts as an English export prompt", () => {
    const result = optimizePromptLocally("一个穿红色斗篷的女孩站在雪山上，电影感");

    expect(result.en).toContain("girl");
    expect(result.en).toContain("wearing a red cloak");
    expect(result.en).toContain("snowy mountain");
    expect(result.en).toContain("cinematic");
    expect(result.en).not.toContain("雪山");
    expect(result.structured.negativePrompt).toContain("low resolution");
  });
});
