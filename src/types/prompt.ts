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
