import type { OptimizedPrompt, PromptTemplateReference, StructuredPrompt } from "../types/prompt";

const qualityTermsZh = ["高细节", "清晰主体", "自然光影", "专业构图"];
const qualityTermsEn = ["high detail", "clear subject", "natural lighting", "professional composition"];
const negativeTermsEn = ["low resolution", "distorted anatomy", "broken text", "extra limbs", "watermark"];

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
    negativePrompt: [negativeTermsEn.join(", "), ...templateNegativeTerms].join(", "),
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
    renderEnglishSubject(input),
    `scene: ${translateScene(prompt.scene)}`,
    `style: ${translateStyle(prompt.style)}`,
    "balanced composition",
    "medium shot with subtle depth of field",
    "soft natural lighting",
    qualityTermsEn.join(", "),
  ].join(", ");
}

function renderEnglishSubject(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) return "A clearly defined visual subject";

  const subject = inferEnglishSubject(trimmed);
  const wardrobe: string[] = [];
  const locations: string[] = [];
  const descriptors: string[] = [];

  if ((trimmed.includes("红色") || trimmed.includes("红")) && trimmed.includes("斗篷")) {
    wardrobe.push("wearing a red cloak");
  } else if (trimmed.includes("斗篷")) {
    wardrobe.push("wearing a cloak");
  } else if (trimmed.includes("红色") || trimmed.includes("红")) {
    descriptors.push("with red accents");
  }
  if (trimmed.includes("雪山")) locations.push("on a snowy mountain");
  else if (trimmed.includes("雪")) locations.push("in a snowy environment");
  if (trimmed.includes("城市")) locations.push("in a city");
  if (trimmed.includes("街")) locations.push("on a street");
  if (trimmed.includes("室内")) locations.push("indoors");

  if (subject === "user-described subject" && !wardrobe.length && !locations.length && !descriptors.length) {
    return `A polished image based on this user concept: ${trimmed}`;
  }
  return ["A", subject, ...wardrobe, ...locations, ...descriptors].join(" ");
}

function inferEnglishSubject(input: string): string {
  if (input.includes("女孩")) return "girl";
  if (input.includes("男孩")) return "boy";
  if (input.includes("女性") || input.includes("女人")) return "woman";
  if (input.includes("男性") || input.includes("男人")) return "man";
  if (input.includes("猫")) return "cat";
  if (input.includes("狗")) return "dog";
  if (input.includes("机器人")) return "robot";
  if (input.includes("海报")) return "poster design";
  if (input.includes("产品") || input.includes("商品")) return "product";
  return "user-described subject";
}

function translateScene(scene: string): string {
  const map: Record<string, string> = {
    "雪地或雪山环境": "snowy landscape or mountain environment",
    "城市街景": "urban street scene",
    "室内空间": "interior space",
    "与主体匹配的自然场景": "natural setting that supports the subject",
  };
  return map[scene] ?? scene;
}

function translateStyle(style: string): string {
  const map: Record<string, string> = {
    "电影感": "cinematic",
    "赛博朋克": "cyberpunk",
    "水彩插画": "watercolor illustration",
    "写实摄影": "realistic photography",
    "精致商业视觉": "refined commercial visual style",
  };
  return map[style] ?? style;
}
