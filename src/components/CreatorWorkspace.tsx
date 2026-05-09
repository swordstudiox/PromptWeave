import { useEffect, useMemo, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { formatPrompt, type ExportFormat } from "../lib/exportFormats";
import { optimizePromptLocally } from "../lib/localOptimizer";
import type { CreationSettings, PromptTemplateReference } from "../types/prompt";

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
  const [settings, setSettings] = useState<CreationSettings>({
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
  });
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
  const localExported = useMemo(() => formatPrompt(result, format, settings), [format, result, settings]);
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
        options: {
          size: settings.imageSize,
          quality: settings.imageQuality,
          n: settings.imageCount,
        },
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
        <div className="control-grid">
          <label>
            比例
            <select value={settings.aspectRatio} onChange={(event) => setSettings({ ...settings, aspectRatio: event.target.value })}>
              <option value="1:1">1:1</option>
              <option value="16:9">16:9</option>
              <option value="9:16">9:16</option>
              <option value="4:3">4:3</option>
              <option value="3:4">3:4</option>
            </select>
          </label>
          <label>
            图片尺寸
            <select value={settings.imageSize} onChange={(event) => setSettings({ ...settings, imageSize: event.target.value })}>
              <option value="1024x1024">1024x1024</option>
              <option value="1536x864">1536x864</option>
              <option value="864x1536">864x1536</option>
              <option value="1536x1024">1536x1024</option>
              <option value="1024x1536">1024x1536</option>
            </select>
          </label>
          <label>
            图片质量
            <select
              value={settings.imageQuality}
              onChange={(event) => setSettings({ ...settings, imageQuality: event.target.value as CreationSettings["imageQuality"] })}
            >
              <option value="auto">auto</option>
              <option value="low">low</option>
              <option value="medium">medium</option>
              <option value="high">high</option>
            </select>
          </label>
          <label>
            生成数量
            <input
              min={1}
              max={4}
              type="number"
              value={settings.imageCount}
              onChange={(event) => setSettings({ ...settings, imageCount: Number(event.target.value) })}
            />
          </label>
        </div>
        <div className="control-grid">
          <label>
            MJ stylize
            <input
              min={0}
              max={1000}
              type="number"
              value={settings.midjourneyStylize}
              onChange={(event) => setSettings({ ...settings, midjourneyStylize: Number(event.target.value) })}
            />
          </label>
          <label>
            MJ chaos
            <input
              min={0}
              max={100}
              type="number"
              value={settings.midjourneyChaos}
              onChange={(event) => setSettings({ ...settings, midjourneyChaos: Number(event.target.value) })}
            />
          </label>
          <label>
            SD steps
            <input
              min={1}
              max={150}
              type="number"
              value={settings.sdSteps}
              onChange={(event) => setSettings({ ...settings, sdSteps: Number(event.target.value) })}
            />
          </label>
          <label>
            SD CFG
            <input
              min={1}
              max={20}
              step={0.5}
              type="number"
              value={settings.sdCfg}
              onChange={(event) => setSettings({ ...settings, sdCfg: Number(event.target.value) })}
            />
          </label>
        </div>
        <label>
          SD Sampler
          <input value={settings.sdSampler} onChange={(event) => setSettings({ ...settings, sdSampler: event.target.value })} />
        </label>
        <label>
          SD Seed
          <input value={settings.sdSeed} onChange={(event) => setSettings({ ...settings, sdSeed: event.target.value })} />
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
