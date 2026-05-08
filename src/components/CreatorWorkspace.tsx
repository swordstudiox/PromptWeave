import { useMemo, useState } from "react";
import { formatPrompt, type ExportFormat } from "../lib/exportFormats";
import { optimizePromptLocally } from "../lib/localOptimizer";

export function CreatorWorkspace() {
  const [input, setInput] = useState("一个穿红色斗篷的女孩站在雪山上，电影感");
  const [format, setFormat] = useState<ExportFormat>("gpt-image");
  const result = useMemo(() => optimizePromptLocally(input), [input]);
  const exported = useMemo(() => formatPrompt(result, format), [format, result]);

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
        <h3>结构化字段</h3>
        <dl className="field-list">
          {Object.entries(result.structured).map(([key, value]) => (
            <div key={key}>
              <dt>{key}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      </div>

      <div className="panel">
        <h2>导出 / 预览</h2>
        <textarea readOnly value={exported} />
        <button>复制提示词</button>
        <button disabled>生成图片：未配置 API</button>
      </div>
    </section>
  );
}
