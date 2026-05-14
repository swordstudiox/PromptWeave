import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ExportFormat } from "../lib/exportFormats";
import { listGenerationHistory } from "../lib/services/creatorService";
import type { GenerationHistoryRecord } from "../types/backend";
import { defaultCreationSettings, type CreationSettings } from "../types/prompt";
import { EmptyState } from "./EmptyState";
import { FeedbackMessage } from "./FeedbackMessage";

export interface HistoryLoadPayload {
  input: string;
  format: ExportFormat;
  settings: CreationSettings;
  imagePaths: string[];
}

function parseHistorySettings(settingsJson: string): CreationSettings {
  try {
    return { ...defaultCreationSettings, ...JSON.parse(settingsJson) };
  } catch {
    return defaultCreationSettings;
  }
}

function historyImagePaths(record: GenerationHistoryRecord): string[] {
  return record.imagePaths?.length ? record.imagePaths : record.imagePath ? [record.imagePath] : [];
}

export function HistoryPanel({ onLoad }: { onLoad: (payload: HistoryLoadPayload) => void }) {
  const [records, setRecords] = useState<GenerationHistoryRecord[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  async function loadHistory() {
    setIsLoading(true);
    setError(null);
    try {
      const history = await listGenerationHistory(50);
      setRecords(history);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadHistory();
  }, []);

  return (
    <section className="panel">
      <div className="panel-heading">
        <h2>历史</h2>
        <button disabled={isLoading} onClick={() => void loadHistory()}>
          {isLoading ? "刷新中..." : "刷新"}
        </button>
      </div>
      <div className="status-stack">
        {error ? <FeedbackMessage variant="error">{error}</FeedbackMessage> : null}
      </div>
      {!records.length && !isLoading ? (
        <EmptyState title="还没有历史记录" description="复制提示词、API 优化或生成图片后会自动记录。" />
      ) : null}
      <div className="template-list">
        {records.map((record) => (
          <article key={record.id} className="template-row">
            <strong>{record.userInput}</strong>
            <span>
              {record.exportFormat} · {new Date(Number(record.createdAt)).toLocaleString()}
            </span>
            <p>{record.promptZh || record.promptEn}</p>
            {record.matchedTemplates.length ? <small>参考模板：{record.matchedTemplates.join(", ")}</small> : null}
            {historyImagePaths(record).length ? (
              <div className="history-image">
                {historyImagePaths(record).map((imagePath) => (
                  <figure key={imagePath}>
                    <img alt="历史生成图" src={convertFileSrc(imagePath)} />
                    <figcaption>{imagePath}</figcaption>
                  </figure>
                ))}
              </div>
            ) : null}
            <button
              onClick={() =>
                onLoad({
                  input: record.userInput,
                  format: record.exportFormat,
                  settings: parseHistorySettings(record.settingsJson),
                  imagePaths: historyImagePaths(record),
                })
              }
            >
              载入到创作页
            </button>
          </article>
        ))}
      </div>
    </section>
  );
}
