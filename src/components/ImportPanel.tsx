import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function classifyGitHubUrl(url: string): string {
  if (url.includes("raw.githubusercontent.com")) return "GitHub raw 文件";
  if (url.includes("/blob/")) return "GitHub blob 文件";
  if (url.includes("github.com")) return "GitHub 仓库";
  return "未知链接";
}

export function ImportPanel() {
  const [url, setUrl] = useState("https://github.com/EvoLinkAI/awesome-gpt-image-2-prompts");
  const type = useMemo(() => classifyGitHubUrl(url), [url]);

  return (
    <section className="panel">
      <h2>参考库导入</h2>
      <input value={url} onChange={(event) => setUrl(event.target.value)} />
      <p>识别结果：{type}</p>
      <button
        onClick={async () => {
          await invoke("classify_import_url", { url });
        }}
      >
        预览导入
      </button>
    </section>
  );
}
