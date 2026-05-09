import { useEffect, useState } from "react";
import { CreatorWorkspace } from "./components/CreatorWorkspace";
import { HistoryPanel, type HistoryLoadPayload } from "./components/HistoryPanel";
import { ImportPanel } from "./components/ImportPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { TemplateLibrary } from "./components/TemplateLibrary";
import { initWorkspace, type WorkspaceInfo } from "./lib/workspace";

type View = "creator" | "templates" | "history" | "imports" | "settings";

const navItems: Array<{ id: View; label: string }> = [
  { id: "creator", label: "创作" },
  { id: "templates", label: "模板库" },
  { id: "history", label: "历史" },
  { id: "imports", label: "导入" },
  { id: "settings", label: "设置" },
];

export default function App() {
  const [view, setView] = useState<View>("creator");
  const [historyPayload, setHistoryPayload] = useState<HistoryLoadPayload | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWorkspace().then(setWorkspace).catch((err) => setError(String(err)));
  }, []);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <strong>PromptWeave</strong>
          <span>织语</span>
        </div>
        <nav>
          {navItems.map((item) => (
            <button
              key={item.id}
              className={item.id === view ? "nav-item active" : "nav-item"}
              onClick={() => setView(item.id)}
            >
              {item.label}
            </button>
          ))}
        </nav>
      </aside>
      <section className="main-panel">
        {error ? <div className="error-banner">{error}</div> : null}
        {workspace ? <div className="workspace-path">工作区：{workspace.dataDir}</div> : null}
        {view === "creator" ? <CreatorWorkspace historyPayload={historyPayload} /> : null}
        {view === "templates" ? <TemplateLibrary /> : null}
        {view === "history" ? (
          <HistoryPanel
            onLoad={(payload) => {
              setHistoryPayload(payload);
              setView("creator");
            }}
          />
        ) : null}
        {view === "imports" ? <ImportPanel /> : null}
        {view === "settings" ? <SettingsPanel /> : null}
      </section>
    </main>
  );
}
