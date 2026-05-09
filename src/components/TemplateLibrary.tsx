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
  isFavorite: boolean;
}

interface TemplateEditDraft {
  id: string;
  title: string;
  category: string;
  promptOriginal: string;
  negativePrompt: string;
  aspectRatio: string;
  tagsText: string;
}

export function TemplateLibrary() {
  const [query, setQuery] = useState("");
  const [templates, setTemplates] = useState<PromptTemplateRecord[]>([]);
  const [showFavoritesOnly, setShowFavoritesOnly] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState<TemplateEditDraft | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);

  useEffect(() => {
    void loadTemplates("");
  }, []);

  async function loadTemplates(nextQuery: string) {
    setIsLoading(true);
    setError(null);
    setStatus(null);
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

  function startEditing(template: PromptTemplateRecord) {
    setError(null);
    setStatus(null);
    setEditingId(template.id);
    setEditDraft({
      id: template.id,
      title: template.title,
      category: template.category,
      promptOriginal: template.promptOriginal,
      negativePrompt: template.negativePrompt ?? "",
      aspectRatio: template.aspectRatio ?? "",
      tagsText: template.tags.join(", "),
    });
  }

  function updateDraft(field: keyof TemplateEditDraft, value: string) {
    setEditDraft((current) => (current ? { ...current, [field]: value } : current));
  }

  function cancelEditing() {
    setEditingId(null);
    setEditDraft(null);
  }

  async function saveEditing() {
    if (!editDraft) {
      return;
    }
    if (!editDraft.title.trim() || !editDraft.promptOriginal.trim()) {
      setError("标题和提示词不能为空。");
      return;
    }
    setBusyId(editDraft.id);
    setError(null);
    setStatus(null);
    try {
      await invoke("update_prompt_template", {
        draft: {
          id: editDraft.id,
          title: editDraft.title.trim(),
          category: editDraft.category.trim(),
          promptOriginal: editDraft.promptOriginal.trim(),
          negativePrompt: editDraft.negativePrompt.trim() || null,
          aspectRatio: editDraft.aspectRatio.trim() || null,
          tags: editDraft.tagsText
            .split(/[,，]/)
            .map((tag) => tag.trim())
            .filter(Boolean),
        },
      });
      setStatus("模板已保存。");
      cancelEditing();
      await loadTemplates(query);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  async function toggleFavorite(template: PromptTemplateRecord) {
    setBusyId(template.id);
    setError(null);
    setStatus(null);
    try {
      await invoke("toggle_prompt_template_favorite", {
        id: template.id,
        isFavorite: !template.isFavorite,
      });
      setTemplates((current) =>
        current.map((item) =>
          item.id === template.id ? { ...item, isFavorite: !template.isFavorite } : item,
        ),
      );
    } catch (err) {
      setError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  async function archiveTemplate(template: PromptTemplateRecord) {
    const confirmed = window.confirm(`归档模板“${template.title}”？归档后它不会出现在模板库和语义检索结果中。`);
    if (!confirmed) {
      return;
    }
    setBusyId(template.id);
    setError(null);
    setStatus(null);
    try {
      await invoke("archive_prompt_template", { id: template.id });
      setTemplates((current) => current.filter((item) => item.id !== template.id));
      if (editingId === template.id) {
        cancelEditing();
      }
      setStatus("模板已归档。");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  const visibleTemplates = showFavoritesOnly
    ? templates.filter((template) => template.isFavorite)
    : templates;

  return (
    <section className="panel">
      <div className="panel-heading">
        <h2>模板库</h2>
        <label className="checkbox-row compact-checkbox">
          <input
            type="checkbox"
            checked={showFavoritesOnly}
            onChange={(event) => setShowFavoritesOnly(event.target.checked)}
          />
          只看收藏
        </label>
      </div>
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
      {status ? <p className="inline-success">{status}</p> : null}
      {!templates.length && !isLoading ? <p>本地模板库为空。先从“导入”页面粘贴 GitHub 链接导入参考库。</p> : null}
      {templates.length && !visibleTemplates.length && showFavoritesOnly ? <p>暂无收藏模板。</p> : null}
      <div className="template-list">
        {visibleTemplates.map((template) => (
          <article key={template.id} className="template-row">
            {editingId === template.id && editDraft ? (
              <div className="template-edit-form">
                <input
                  aria-label="模板标题"
                  value={editDraft.title}
                  onChange={(event) => updateDraft("title", event.target.value)}
                />
                <div className="template-edit-grid">
                  <input
                    aria-label="分类"
                    placeholder="分类"
                    value={editDraft.category}
                    onChange={(event) => updateDraft("category", event.target.value)}
                  />
                  <input
                    aria-label="比例"
                    placeholder="比例，例如 1:1"
                    value={editDraft.aspectRatio}
                    onChange={(event) => updateDraft("aspectRatio", event.target.value)}
                  />
                </div>
                <textarea
                  aria-label="提示词"
                  value={editDraft.promptOriginal}
                  onChange={(event) => updateDraft("promptOriginal", event.target.value)}
                />
                <textarea
                  aria-label="负面提示词"
                  placeholder="Negative prompt，可留空"
                  value={editDraft.negativePrompt}
                  onChange={(event) => updateDraft("negativePrompt", event.target.value)}
                />
                <input
                  aria-label="标签"
                  placeholder="标签，用逗号分隔"
                  value={editDraft.tagsText}
                  onChange={(event) => updateDraft("tagsText", event.target.value)}
                />
                <div className="template-actions">
                  <button disabled={busyId === template.id} onClick={saveEditing}>
                    {busyId === template.id ? "保存中..." : "保存"}
                  </button>
                  <button className="secondary-button" disabled={busyId === template.id} onClick={cancelEditing}>
                    取消
                  </button>
                </div>
              </div>
            ) : (
              <>
                <div className="template-title-row">
                  <strong>{template.isFavorite ? "★ " : ""}{template.title}</strong>
                  <div className="template-actions">
                    <button className="secondary-button" disabled={busyId === template.id} onClick={() => void toggleFavorite(template)}>
                      {template.isFavorite ? "取消收藏" : "收藏"}
                    </button>
                    <button className="secondary-button" disabled={busyId === template.id} onClick={() => startEditing(template)}>
                      编辑
                    </button>
                    <button className="danger-button" disabled={busyId === template.id} onClick={() => void archiveTemplate(template)}>
                      归档
                    </button>
                  </div>
                </div>
                <span>
                  {template.category || "未分类"} · {template.modelHint} · {template.language}
                </span>
                <p>{template.promptOriginal}</p>
                <small>{template.sourceUrl}</small>
                {template.negativePrompt ? <small>Negative: {template.negativePrompt}</small> : null}
                {template.aspectRatio ? <small>比例：{template.aspectRatio}</small> : null}
                {template.tags.length ? <small>标签：{template.tags.join(", ")}</small> : null}
              </>
            )}
          </article>
        ))}
      </div>
    </section>
  );
}
