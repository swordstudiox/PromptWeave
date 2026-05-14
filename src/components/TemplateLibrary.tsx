import { useEffect, useState } from "react";
import {
  cleanupDuplicatePromptTemplates,
  deletePromptTemplate,
  listPromptTemplates,
  searchPromptTemplates,
  togglePromptTemplateFavorite,
  updatePromptTemplate,
} from "../lib/services/templateService";
import type { PromptTemplateRecord } from "../types/backend";
import { EmptyState } from "./EmptyState";
import { FeedbackMessage } from "./FeedbackMessage";

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
  const [expandedIds, setExpandedIds] = useState<string[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState<TemplateEditDraft | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isCleaningDuplicates, setIsCleaningDuplicates] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);

  useEffect(() => {
    void loadTemplates("");
  }, []);

  async function loadTemplates(nextQuery: string) {
    setIsLoading(true);
    setError(null);
    setStatus(null);
    try {
      const records = nextQuery.trim()
        ? await searchPromptTemplates(nextQuery, 50)
        : await listPromptTemplates(50);
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

  function toggleExpanded(id: string) {
    setExpandedIds((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
    );
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
      await updatePromptTemplate({
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
      await togglePromptTemplateFavorite(template.id, !template.isFavorite);
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

  async function deleteTemplate(template: PromptTemplateRecord) {
    const confirmed = window.confirm(`永久删除模板“${template.title}”？删除后不能从模板库恢复。`);
    if (!confirmed) {
      return;
    }
    setBusyId(template.id);
    setError(null);
    setStatus(null);
    try {
      await deletePromptTemplate(template.id);
      setTemplates((current) => current.filter((item) => item.id !== template.id));
      if (editingId === template.id) {
        cancelEditing();
      }
      setStatus("模板已删除。");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  async function cleanupDuplicates() {
    const confirmed = window.confirm("清理重复模板会永久删除重复记录，并保留收藏或较新的那一条。继续吗？");
    if (!confirmed) {
      return;
    }
    setIsCleaningDuplicates(true);
    setError(null);
    setStatus(null);
    try {
      const result = await cleanupDuplicatePromptTemplates();
      setStatus(result.deletedCount ? `已删除 ${result.deletedCount} 条重复模板。` : "没有发现重复模板。");
      await loadTemplates(query);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsCleaningDuplicates(false);
    }
  }

  const visibleTemplates = showFavoritesOnly
    ? templates.filter((template) => template.isFavorite)
    : templates;
  const hasActiveSearch = Boolean(query.trim());

  return (
    <section className="panel">
      <div className="panel-heading">
        <h2>模板库</h2>
        <div className="template-toolbar">
          <label className="checkbox-row compact-checkbox">
            <input
              type="checkbox"
              checked={showFavoritesOnly}
              onChange={(event) => setShowFavoritesOnly(event.target.checked)}
            />
            只看收藏
          </label>
          <button className="secondary-button" disabled={isCleaningDuplicates || isLoading} onClick={() => void cleanupDuplicates()}>
            {isCleaningDuplicates ? "清理中..." : "清理重复"}
          </button>
        </div>
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
      <div className="status-stack">
        {error ? <FeedbackMessage variant="error">{error}</FeedbackMessage> : null}
        {status ? <FeedbackMessage variant="success">{status}</FeedbackMessage> : null}
      </div>
      {!templates.length && !isLoading && hasActiveSearch ? (
        <EmptyState title="没有匹配模板" description="换一个关键词，或清空搜索后查看全部本地模板。" />
      ) : null}
      {!templates.length && !isLoading && !hasActiveSearch ? (
        <EmptyState title="本地模板库为空" description="先从“导入”页面粘贴 GitHub 链接导入参考库。" />
      ) : null}
      {templates.length && !visibleTemplates.length && showFavoritesOnly ? (
        <EmptyState title="暂无收藏模板" description="取消“只看收藏”，或在模板卡片中收藏常用模板。" />
      ) : null}
      <div className="template-list">
        {visibleTemplates.map((template) => {
          const isExpanded = expandedIds.includes(template.id);
          const promptText = cleanPromptText(template.promptOriginal);
          const sourceLabel = formatSourceLabel(template.sourceUrl);
          const title = cleanTemplateTitle(template.title);
          const visibleTags = template.tags.slice(0, 6);
          const hiddenTagCount = Math.max(0, template.tags.length - visibleTags.length);
          const hasLongPrompt = promptText.length > 420 || promptText.includes("\n");

          return (
            <article key={template.id} className="template-card">
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
                  <div className="template-card-top">
                    <div className="template-summary">
                      <h3>{template.isFavorite ? "★ " : ""}{title}</h3>
                      <div className="template-meta">
                        <span>{template.category || "未分类"}</span>
                        <span>{template.modelHint}</span>
                        <span>{template.language}</span>
                        {template.aspectRatio ? <span>{template.aspectRatio}</span> : null}
                      </div>
                    </div>
                    <div className="template-actions compact-actions">
                      <button className="secondary-button" disabled={busyId === template.id} onClick={() => void toggleFavorite(template)}>
                        {template.isFavorite ? "已收藏" : "收藏"}
                      </button>
                      <button className="secondary-button" disabled={busyId === template.id} onClick={() => startEditing(template)}>
                        编辑
                      </button>
                      <button className="danger-button" disabled={busyId === template.id} onClick={() => void deleteTemplate(template)}>
                        删除
                      </button>
                    </div>
                  </div>

                  <p className={isExpanded ? "template-prompt expanded" : "template-prompt"}>
                    {promptText || "暂无提示词内容"}
                  </p>

                  <div className="template-footer">
                    <div className="template-tags">
                      {visibleTags.map((tag) => (
                        <span key={tag}>{tag}</span>
                      ))}
                      {hiddenTagCount ? <span>+{hiddenTagCount}</span> : null}
                    </div>
                    <div className="template-footer-actions">
                      {hasLongPrompt ? (
                        <button className="text-button" onClick={() => toggleExpanded(template.id)}>
                          {isExpanded ? "收起" : "展开"}
                        </button>
                      ) : null}
                      <small title={template.sourceUrl}>{sourceLabel}</small>
                    </div>
                  </div>
                  {template.negativePrompt ? <small className="template-negative">Negative: {cleanPromptText(template.negativePrompt)}</small> : null}
                </>
              )}
            </article>
          );
        })}
      </div>
    </section>
  );
}

function cleanTemplateTitle(title: string): string {
  return normalizeWhitespace(
    title
      .replace(/^\d+[\s.)-]*/, "")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      .replace(/<!--|-->/g, ""),
  ) || "Imported Prompt";
}

function cleanPromptText(prompt: string): string {
  const text = prompt.trim();
  if (!text) {
    return "";
  }

  const jsonSummary = summarizeJsonPrompt(text);
  if (jsonSummary) {
    return jsonSummary;
  }

  return normalizeWhitespace(
    text
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/<img\b[^>]*>/gi, "")
      .replace(/\|:?-+:?\|/g, " ")
      .replace(/\*\*Prompt:\*\*/gi, "Prompt:")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      .replace(/!\[[^\]]*]\([^)]+\)/g, ""),
  );
}

function summarizeJsonPrompt(text: string): string | null {
  if (!text.startsWith("{") && !text.startsWith("[")) {
    return null;
  }

  try {
    const value = JSON.parse(text);
    return summarizeJsonValue(value);
  } catch {
    return null;
  }
}

function summarizeJsonValue(value: unknown): string {
  if (Array.isArray(value)) {
    return value.slice(0, 4).map(summarizeJsonValue).filter(Boolean).join(" · ");
  }

  if (value && typeof value === "object") {
    const record = value as Record<string, unknown>;
    const parts = [
      stringValue(record.type),
      stringValue(record.title),
      stringValue(record.name),
      stringValue(record.prompt),
      summarizeNamedObject("brand", record.brand),
      summarizeNamedObject("subject", record.subject),
      summarizeSectionTitles(record.sections),
    ].filter(Boolean);
    return parts.join(" · ");
  }

  return stringValue(value);
}

function summarizeNamedObject(label: string, value: unknown): string {
  if (!value || typeof value !== "object") {
    return "";
  }
  const record = value as Record<string, unknown>;
  const name = stringValue(record.name) || stringValue(record.description);
  return name ? `${label}: ${name}` : "";
}

function summarizeSectionTitles(value: unknown): string {
  if (!Array.isArray(value)) {
    return "";
  }
  const titles = value
    .map((item) => (item && typeof item === "object" ? stringValue((item as Record<string, unknown>).title) : ""))
    .filter(Boolean)
    .slice(0, 6);
  return titles.length ? `sections: ${titles.join(" / ")}` : "";
}

function stringValue(value: unknown): string {
  return typeof value === "string" ? normalizeWhitespace(value) : "";
}

function normalizeWhitespace(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function formatSourceLabel(url: string): string {
  try {
    const parsed = new URL(url);
    const fileName = parsed.pathname.split("/").filter(Boolean).at(-1) || parsed.hostname;
    return `${parsed.hostname} · ${fileName}`;
  } catch {
    return url;
  }
}
