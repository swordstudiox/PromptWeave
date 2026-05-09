export interface StructuredPrompt {
  subject: string;
  scene: string;
  style: string;
  composition: string;
  camera: string;
  lighting: string;
  color: string;
  details: string;
  negativePrompt: string;
}

export interface OptimizedPrompt {
  zh: string;
  en: string;
  structured: StructuredPrompt;
  matchedTemplateTitles: string[];
}

export interface PromptTemplateReference {
  title: string;
  category: string;
  promptOriginal: string;
  negativePrompt?: string;
  tags: string[];
}

export interface CreationSettings {
  aspectRatio: string;
  imageSize: string;
  imageQuality: "low" | "medium" | "high" | "auto";
  imageCount: number;
  midjourneyStylize: number;
  midjourneyChaos: number;
  sdSteps: number;
  sdCfg: number;
  sdSampler: string;
  sdSeed: string;
}
