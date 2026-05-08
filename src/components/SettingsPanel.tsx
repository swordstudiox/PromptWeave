export function SettingsPanel() {
  return (
    <section className="settings-grid">
      <div className="panel">
        <h2>提示词优化 API</h2>
        <select defaultValue="local">
          <option value="local">本地规则模式</option>
          <option value="openai">OpenAI</option>
          <option value="claude">Claude</option>
          <option value="compatible">自定义兼容接口</option>
        </select>
      </div>
      <div className="panel">
        <h2>图片生成 API</h2>
        <select defaultValue="disabled">
          <option value="disabled">未启用</option>
          <option value="gpt-image">GPT Image</option>
          <option value="compatible">自定义图像接口</option>
        </select>
      </div>
    </section>
  );
}
