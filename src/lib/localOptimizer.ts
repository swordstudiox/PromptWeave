import type { OptimizedPrompt, PromptTemplateReference, StructuredPrompt } from "../types/prompt";

const qualityTermsZh = ["高细节", "清晰主体", "自然光影", "专业构图"];
const qualityTermsEn = ["high detail", "clear subject", "natural lighting", "professional composition"];

export function optimizePromptLocally(input: string, templates: PromptTemplateReference[] = []): OptimizedPrompt {
  const trimmed = input.trim();
  const matchedTemplates = templates.slice(0, 3);
  const templateDetails = matchedTemplates.length
    ? `参考模板：${matchedTemplates.map((template) => template.title).join("、")}`
    : "";
  const templateNegativeTerms = matchedTemplates
    .map((template) => template.negativePrompt)
    .filter((value): value is string => Boolean(value?.trim()));
  const structured: StructuredPrompt = {
    subject: trimmed || "未指定主体",
    scene: inferScene(trimmed),
    style: inferStyle(trimmed, matchedTemplates),
    composition: "主体明确，画面层次清晰",
    camera: "中景，轻微景深",
    lighting: "柔和自然光",
    color: "色彩协调，对比适中",
    details: [qualityTermsZh.join("，"), templateDetails].filter(Boolean).join("，"),
    negativePrompt: ["低清晰度，画面变形，错误文字，多余肢体，水印", ...templateNegativeTerms].join("，"),
  };

  return {
    structured,
    zh: renderChinesePrompt(structured),
    en: renderEnglishPrompt(trimmed, structured),
    matchedTemplateTitles: matchedTemplates.map((template) => template.title),
  };
}

function inferScene(input: string): string {
  if (input.includes("雪")) return "雪地或雪山环境";
  if (input.includes("街") || input.includes("城市")) return "城市街景";
  if (input.includes("室内")) return "室内空间";
  return "与主体匹配的自然场景";
}

function inferStyle(input: string, templates: PromptTemplateReference[]): string {
  if (input.includes("电影")) return "电影感";
  if (input.includes("赛博")) return "赛博朋克";
  if (input.includes("水彩")) return "水彩插画";
  if (input.includes("写实")) return "写实摄影";
  if (templates.some((template) => template.tags.includes("cinematic") || template.promptOriginal.toLowerCase().includes("cinematic"))) {
    return "电影感";
  }
  return "精致商业视觉";
}

function renderChinesePrompt(prompt: StructuredPrompt): string {
  return [
    prompt.subject,
    prompt.scene,
    prompt.style,
    prompt.composition,
    prompt.camera,
    prompt.lighting,
    prompt.color,
    prompt.details,
  ].join("，");
}

function renderEnglishPrompt(input: string, prompt: StructuredPrompt): string {
  return [
    input || "A clearly defined visual subject",
    `scene: ${prompt.scene}`,
    `style: ${prompt.style}`,
    "balanced composition",
    "medium shot with subtle depth of field",
    "soft natural lighting",
    qualityTermsEn.join(", "),
  ].join(", ");
}
