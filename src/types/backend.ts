import type { ExportFormat } from "../lib/exportFormats";

export interface PromptTemplateRecord {
  id: string;
  title: string;
  category: string;
  sourceRepo: string;
  sourceUrl: string;
  modelHint: string;
  language: string;
  promptOriginal: string;
  negativePrompt?: string;
  aspectRatio?: string;
  tags: string[];
  importedAt: string;
  isFavorite: boolean;
}

export interface TemplateUpdateDraft {
  id: string;
  title: string;
  category: string;
  promptOriginal: string;
  negativePrompt?: string | null;
  aspectRatio?: string | null;
  tags: string[];
}

export interface DuplicateCleanupResult {
  deletedCount: number;
}

export interface GenerationHistoryDraft {
  id: string;
  userInput: string;
  promptZh: string;
  promptEn: string;
  exportFormat: ExportFormat;
  matchedTemplatesJson: string;
  settingsJson: string;
  imagePath?: string;
  imagePathsJson: string;
  createdAt: string;
}

export interface GenerationHistoryRecord {
  id: string;
  userInput: string;
  promptZh: string;
  promptEn: string;
  exportFormat: ExportFormat;
  matchedTemplates: string[];
  settingsJson: string;
  imagePath?: string;
  imagePaths: string[];
  createdAt: string;
}

export interface PromptLibrarySourceRecord {
  id: string;
  name: string;
  url: string;
  sourceType: string;
  lastSyncedAt?: string;
  lastImportedCount: number;
  lastSkippedCount: number;
  lastError?: string;
  createdAt: string;
  updatedAt: string;
}

export interface ImportUrlInfo {
  sourceType: string;
  normalizedUrl: string;
  isSupported: boolean;
}

export interface PromptTemplateDraft {
  id: string;
  title: string;
  category: string;
  sourceRepo: string;
  sourceUrl: string;
  sourceLicense?: string;
  author?: string;
  modelHint: string;
  language: string;
  promptOriginal: string;
  promptZh?: string;
  promptEn?: string;
  negativePrompt?: string;
  aspectRatio?: string;
  tags: string[];
  previewImageUrls: string[];
  importedAt: string;
  contentHash: string;
}

export interface ImportPreview {
  source: ImportUrlInfo;
  items: PromptTemplateDraft[];
  warnings: string[];
}

export interface ImportResult {
  sourceId: string;
  importedCount: number;
  skippedCount: number;
  warnings: string[];
}

export interface ApiProviderConfig {
  enabled: boolean;
  provider: string;
  baseUrl: string;
  model: string;
  apiKey: string;
}

export interface AppConfig {
  promptOptimization: ApiProviderConfig;
  imageGeneration: ApiProviderConfig;
}

export interface ImageGenerationOptions {
  size: string;
  quality: string;
  n: number;
}

export interface ImageGenerationResult {
  imagePath?: string;
  imagePaths: string[];
}

export interface PromptOptimizationResult {
  prompt: string;
}
