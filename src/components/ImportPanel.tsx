import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PromptTemplateDraft {
  id: string;
  title: string;
  category: string;
  sourceRepo: string;
  sourceUrl: string;
  promptOriginal: string;
  negativePrompt?: string;
  aspectRatio?: string;
  tags: string[];
}

interface ImportPreview {
  items: PromptTemplateDraft[];
  warnings: string[];
}

interface ImportResult {
  importedCount: number;
  skippedCount: number;
  warnings: string[];
}

function classifyGitHubUrl(url: string): string {
  if (url.includes("raw.githubusercontent.com")) return "GitHub raw 文件";
  if (url.includes("/blob/")) return "GitHub blob 文件";
  if (url.includes("/tree/")) return "GitHub 目录";
  if (url.includes("github.com")) return "GitHub 仓库";
  return "未知链接";
}

export function ImportPanel() {
  const [url, setUrl] = useState("https://github.com/EvoLinkAI/awesome-gpt-image-2-prompts");
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);
  const type = useMemo(() => classifyGitHubUrl(url), [url]);

  async function previewImport() {
    setIsBusy(true);
    setError(null);
    setResult(null);
    try {
      const nextPreview = await invoke<ImportPreview>("preview_import_url", { url });
      setPreview(nextPreview);
    } catch (err) {
      setPreview(null);
      setError(String(err));
    } finally {
      setIsBusy(false);
    }
  }

  async function importLibrary() {
    setIsBusy(true);
    setError(null);
    try {
      const importResult = await invoke<ImportResult>("import_prompt_library", { url });
      setResult(importResult);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="panel">
      <h2>参考库导入</h2>
      <input value={url} onChange={(event) => setUrl(event.target.value)} />
      <p>识别结果：{type}</p>
      <button
        disabled={isBusy || !url.trim()}
        onClick={previewImport}
      >
        {isBusy ? "处理中..." : "预览导入"}
      </button>
      <button disabled={isBusy || !preview?.items.length} onClick={importLibrary}>
        导入到本地库
      </button>
      {error ? <p className="inline-error">{error}</p> : null}
      {result ? (
        <p className="inline-success">
          已导入 {result.importedCount} 条，跳过重复 {result.skippedCount} 条。
        </p>
      ) : null}
      {preview?.warnings.length ? (
        <div className="warning-list">
          {preview.warnings.map((warning) => (
            <p key={warning}>{warning}</p>
          ))}
        </div>
      ) : null}
      {preview ? (
        <div className="import-preview">
          <h3>预览条目：{preview.items.length}</h3>
          {preview.items.slice(0, 12).map((item) => (
            <article key={item.id} className="template-row">
              <strong>{item.title}</strong>
              <span>{item.category || "未分类"}</span>
              <p>{item.promptOriginal}</p>
              {item.negativePrompt ? <small>Negative: {item.negativePrompt}</small> : null}
              {item.aspectRatio ? <small>比例：{item.aspectRatio}</small> : null}
            </article>
          ))}
        </div>
      ) : null}
    </section>
  );
}
