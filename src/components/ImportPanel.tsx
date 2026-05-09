import { useEffect, useMemo, useState } from "react";
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
  sourceId: string;
  importedCount: number;
  skippedCount: number;
  warnings: string[];
}

interface PromptLibrarySourceRecord {
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
  const [sources, setSources] = useState<PromptLibrarySourceRecord[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);
  const [syncingSourceId, setSyncingSourceId] = useState<string | null>(null);
  const type = useMemo(() => classifyGitHubUrl(url), [url]);

  useEffect(() => {
    void loadSources();
  }, []);

  async function loadSources() {
    try {
      const records = await invoke<PromptLibrarySourceRecord[]>("list_prompt_library_sources");
      setSources(records);
    } catch (err) {
      setError(String(err));
    }
  }

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
      await loadSources();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsBusy(false);
    }
  }

  async function syncSource(source: PromptLibrarySourceRecord) {
    setSyncingSourceId(source.id);
    setError(null);
    setResult(null);
    try {
      const syncResult = await invoke<ImportResult>("sync_prompt_library_source", {
        sourceId: source.id,
      });
      setResult(syncResult);
      await loadSources();
    } catch (err) {
      setError(String(err));
      await loadSources();
    } finally {
      setSyncingSourceId(null);
    }
  }

  return (
    <section className="panel">
      <div className="panel-heading">
        <h2>参考库导入</h2>
        <button className="secondary-button" disabled={isBusy || Boolean(syncingSourceId)} onClick={() => void loadSources()}>
          刷新来源
        </button>
      </div>
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
          已同步来源 {result.sourceId.slice(0, 8)}，导入 {result.importedCount} 条，跳过重复 {result.skippedCount} 条。
        </p>
      ) : null}
      {sources.length ? (
        <div className="source-list">
          <h3>已保存参考库：{sources.length}</h3>
          {sources.map((source) => (
            <article key={source.id} className="template-row source-row">
              <div className="template-title-row">
                <strong>{source.name}</strong>
                <div className="template-actions">
                  <button
                    className="secondary-button"
                    disabled={isBusy || Boolean(syncingSourceId)}
                    onClick={() => {
                      setUrl(source.url);
                      setPreview(null);
                      setResult(null);
                    }}
                  >
                    载入链接
                  </button>
                  <button
                    disabled={isBusy || Boolean(syncingSourceId)}
                    onClick={() => void syncSource(source)}
                  >
                    {syncingSourceId === source.id ? "同步中..." : "同步"}
                  </button>
                </div>
              </div>
              <span>{source.sourceType} · {source.lastSyncedAt ? `上次同步 ${source.lastSyncedAt}` : "尚未同步"}</span>
              <small>{source.url}</small>
              <small>
                最近结果：导入 {source.lastImportedCount} 条，跳过 {source.lastSkippedCount} 条
              </small>
              {source.lastError ? <small className="inline-error">错误：{source.lastError}</small> : null}
            </article>
          ))}
        </div>
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
