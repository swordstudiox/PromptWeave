import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ApiProviderConfig {
  enabled: boolean;
  provider: string;
  baseUrl: string;
  model: string;
  apiKey: string;
}

interface AppConfig {
  promptOptimization: ApiProviderConfig;
  imageGeneration: ApiProviderConfig;
}

const defaultConfig: AppConfig = {
  promptOptimization: {
    enabled: false,
    provider: "local-rules",
    baseUrl: "",
    model: "",
    apiKey: "",
  },
  imageGeneration: {
    enabled: false,
    provider: "disabled",
    baseUrl: "",
    model: "",
    apiKey: "",
  },
};

export function SettingsPanel() {
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_app_config")
      .then(setConfig)
      .catch((err) => setError(String(err)));
  }, []);

  function updateProvider(section: keyof AppConfig, patch: Partial<ApiProviderConfig>) {
    setConfig((current) => ({
      ...current,
      [section]: {
        ...current[section],
        ...patch,
      },
    }));
  }

  async function save() {
    setIsSaving(true);
    setError(null);
    setStatus(null);
    try {
      const saved = await invoke<AppConfig>("save_app_config", { config });
      setConfig(saved);
      setStatus("配置已保存到当前工作区。");
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <>
      <section className="settings-grid">
        <ProviderSettings
          title="提示词优化 API"
          value={config.promptOptimization}
          providers={[
            ["local-rules", "本地规则模式"],
            ["openai", "OpenAI"],
            ["claude", "Claude"],
            ["compatible", "自定义兼容接口"],
          ]}
          onChange={(patch) => updateProvider("promptOptimization", patch)}
        />
        <ProviderSettings
          title="图片生成 API"
          value={config.imageGeneration}
          providers={[
            ["disabled", "未启用"],
            ["gpt-image", "GPT Image"],
            ["compatible", "自定义图像接口"],
          ]}
          onChange={(patch) => updateProvider("imageGeneration", patch)}
        />
      </section>
      <div className="settings-actions">
        <button disabled={isSaving} onClick={save}>
          {isSaving ? "保存中..." : "保存配置"}
        </button>
        {status ? <span className="inline-success">{status}</span> : null}
        {error ? <span className="inline-error">{error}</span> : null}
      </div>
    </>
  );
}

function ProviderSettings({
  title,
  value,
  providers,
  onChange,
}: {
  title: string;
  value: ApiProviderConfig;
  providers: Array<[string, string]>;
  onChange: (patch: Partial<ApiProviderConfig>) => void;
}) {
  return (
    <div className="panel">
      <h2>{title}</h2>
      <label className="checkbox-row">
        <input
          checked={value.enabled}
          type="checkbox"
          onChange={(event) => onChange({ enabled: event.target.checked })}
        />
        启用
      </label>
      <label>
        服务类型
        <select value={value.provider} onChange={(event) => onChange({ provider: event.target.value })}>
          {providers.map(([id, label]) => (
            <option key={id} value={id}>
              {label}
            </option>
          ))}
        </select>
      </label>
      <label>
        Base URL
        <input value={value.baseUrl} onChange={(event) => onChange({ baseUrl: event.target.value })} />
      </label>
      <label>
        模型 ID
        <input value={value.model} onChange={(event) => onChange({ model: event.target.value })} />
      </label>
      <label>
        API Key
        <input
          type="password"
          value={value.apiKey}
          onChange={(event) => onChange({ apiKey: event.target.value })}
        />
      </label>
    </div>
  );
}
