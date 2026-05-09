import { useEffect, useMemo, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { formatPrompt, type ExportFormat } from "../lib/exportFormats";
import { optimizePromptLocally } from "../lib/localOptimizer";
import type { PromptTemplateReference } from "../types/prompt";

interface PromptTemplateRecord {
  title: string;
  category: string;
  promptOriginal: string;
  negativePrompt?: string;
  tags: string[];
}

interface ImageGenerationResult {
  imagePath: string;
}

interface PromptOptimizationResult {
  prompt: string;
}

export function CreatorWorkspace() {
  const [input, setInput] = useState("一个穿红色斗篷的女孩站在雪山上，电影感");
  const [format, setFormat] = useState<ExportFormat>("gpt-image");
  const [templates, setTemplates] = useState<PromptTemplateReference[]>([]);
  const [templateError, setTemplateError] = useState<string | null>(null);
  const [copyStatus, setCopyStatus] = useState<string | null>(null);
  const [generationError, setGenerationError] = useState<string | null>(null);
  const [apiPrompt, setApiPrompt] = useState<string | null>(null);
  const [apiError, setApiError] = useState<string | null>(null);
  const [isOptimizing, setIsOptimizing] = useState(false);
  const [imagePath, setImagePath] = useState<string | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const result = useMemo(() => optimizePromptLocally(input, templates), [input, templates]);
  const localExported = useMemo(() => formatPrompt(result, format), [format, result]);
  const exported = apiPrompt || localExported;

  useEffect(() => {
    const timer = window.setTimeout(async () => {
      if (!input.trim()) {
        setTemplates([]);
        return;
      }

      try {
        const records = await invoke<PromptTemplateRecord[]>("search_prompt_templates", {
          query: input,
          limit: 3,
        });
        setTemplates(records);
        setTemplateError(null);
      } catch (err) {
        setTemplates([]);
        setTemplateError(String(err));
      }
    }, 350);

    return () => window.clearTimeout(timer);
  }, [input]);

  useEffect(() => {
    setApiPrompt(null);
    setApiError(null);
  }, [input, format]);

  async function copyPrompt() {
    await navigator.clipboard.writeText(exported);
    setCopyStatus("已复制");
    window.setTimeout(() => setCopyStatus(null), 1600);
  }

  async function generateImage() {
    setIsGenerating(true);
    setGenerationError(null);
    try {
      const generated = await invoke<ImageGenerationResult>("generate_image_preview", {
        prompt: exported,
      });
      setImagePath(generated.imagePath);
    } catch (err) {
      setGenerationError(String(err));
    } finally {
      setIsGenerating(false);
    }
  }

  async function optimizeWithApi() {
    setIsOptimizing(true);
    setApiError(null);
    try {
      const optimized = await invoke<PromptOptimizationResult>("optimize_prompt_with_api", {
        localPrompt: localExported,
      });
      setApiPrompt(optimized.prompt);
    } catch (err) {
      setApiError(String(err));
    } finally {
      setIsOptimizing(false);
    }
  }

  return (
    <section className="creator-grid">
      <div className="panel">
        <h2>创作输入</h2>
        <textarea value={input} onChange={(event) => setInput(event.target.value)} />
        <label>
          导出格式
          <select value={format} onChange={(event) => setFormat(event.target.value as ExportFormat)}>
            <option value="gpt-image">GPT Image</option>
            <option value="midjourney">Midjourney</option>
            <option value="stable-diffusion">Stable Diffusion / ComfyUI</option>
          </select>
        </label>
      </div>

      <div className="panel">
        <h2>优化结果</h2>
        <h3>中文提示词</h3>
        <p>{result.zh}</p>
        <h3>英文提示词</h3>
        <p>{result.en}</p>
        {apiPrompt ? (
          <>
            <h3>API 优化版</h3>
            <p>{apiPrompt}</p>
          </>
        ) : null}
        <h3>结构化字段</h3>
        <dl className="field-list">
          {Object.entries(result.structured).map(([key, value]) => (
            <div key={key}>
              <dt>{key}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
        {result.matchedTemplateTitles.length ? (
          <>
            <h3>参考模板</h3>
            <ul className="matched-list">
              {result.matchedTemplateTitles.map((title) => (
                <li key={title}>{title}</li>
              ))}
            </ul>
          </>
        ) : null}
        {templateError ? <p className="inline-error">{templateError}</p> : null}
        {apiError ? <p className="inline-error">{apiError}</p> : null}
      </div>

      <div className="panel">
        <h2>导出 / 预览</h2>
        <textarea readOnly value={exported} />
        <button disabled={isOptimizing} onClick={optimizeWithApi}>
          {isOptimizing ? "优化中..." : "API 优化"}
        </button>
        <button onClick={copyPrompt}>复制提示词</button>
        <button disabled={isGenerating} onClick={generateImage}>
          {isGenerating ? "生成中..." : "生成图片"}
        </button>
        {copyStatus ? <p className="inline-success">{copyStatus}</p> : null}
        {generationError ? <p className="inline-error">{generationError}</p> : null}
        {imagePath ? (
          <div className="image-preview">
            <img alt="生成预览" src={convertFileSrc(imagePath)} />
            <small>{imagePath}</small>
          </div>
        ) : null}
      </div>
    </section>
  );
}
