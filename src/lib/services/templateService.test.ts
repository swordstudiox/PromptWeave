import { describe, expect, it } from "vitest";
import { mapPromptTemplateRecordToReference } from "./templateService";
import type { PromptTemplateRecord } from "../../types/backend";

const templateRecord: PromptTemplateRecord = {
  id: "template-1",
  title: "Cinematic Snow Portrait",
  category: "Portrait",
  sourceRepo: "owner/repo",
  sourceUrl: "https://github.com/owner/repo/blob/main/prompt.md",
  modelHint: "gpt-image",
  language: "en",
  promptOriginal: "A cinematic portrait with dramatic rim light and snow particles.",
  negativePrompt: "watermark, distorted hands",
  aspectRatio: "1:1",
  tags: ["cinematic", "snow"],
  importedAt: "1700000000000",
  isFavorite: true,
};

describe("mapPromptTemplateRecordToReference", () => {
  it("keeps only prompt fields needed by local optimizer", () => {
    expect(mapPromptTemplateRecordToReference(templateRecord)).toEqual({
      title: "Cinematic Snow Portrait",
      category: "Portrait",
      promptOriginal: "A cinematic portrait with dramatic rim light and snow particles.",
      negativePrompt: "watermark, distorted hands",
      tags: ["cinematic", "snow"],
    });
  });
});
