import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PromptTemplateRecord {
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
}

export function TemplateLibrary() {
  const [query, setQuery] = useState("");
  const [templates, setTemplates] = useState<PromptTemplateRecord[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    void loadTemplates("");
  }, []);

  async function loadTemplates(nextQuery: string) {
    setIsLoading(true);
    setError(null);
    try {
      const command = nextQuery.trim() ? "search_prompt_templates" : "list_prompt_templates";
      const payload = nextQuery.trim()
        ? { query: nextQuery, limit: 50 }
        : { limit: 50 };
      const records = await invoke<PromptTemplateRecord[]>(command, payload);
      setTemplates(records);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <section className="panel">
      <h2>模板库</h2>
      <div className="search-row">
        <input
          placeholder="搜索标题、分类、prompt 或标签"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              void loadTemplates(query);
            }
          }}
        />
        <button disabled={isLoading} onClick={() => void loadTemplates(query)}>
          {isLoading ? "搜索中..." : "搜索"}
        </button>
      </div>
      {error ? <p className="inline-error">{error}</p> : null}
      {!templates.length && !isLoading ? <p>本地模板库为空。先从“导入”页面粘贴 GitHub 链接导入参考库。</p> : null}
      <div className="template-list">
        {templates.map((template) => (
          <article key={template.id} className="template-row">
            <strong>{template.title}</strong>
            <span>
              {template.category || "未分类"} · {template.modelHint} · {template.language}
            </span>
            <p>{template.promptOriginal}</p>
            <small>{template.sourceUrl}</small>
            {template.negativePrompt ? <small>Negative: {template.negativePrompt}</small> : null}
            {template.aspectRatio ? <small>比例：{template.aspectRatio}</small> : null}
            {template.tags.length ? <small>标签：{template.tags.join(", ")}</small> : null}
          </article>
        ))}
      </div>
    </section>
  );
}
