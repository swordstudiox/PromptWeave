# 当前修复任务

## 范围

- 长期内置三个 GPT Image 2 默认模板库来源：EvoLink cases、freestylefly gallery、YouMind README_zh。
- 前端导入面板展示三个内置来源预设，支持一键填入 URL。
- 后端集中 adapter 分发，分别处理 EvoLink、freestylefly、YouMind 的不同 Markdown 结构。
- freestylefly `gallery.md` 自动展开到 `gallery-part-1.md` / `gallery-part-2.md`，并优先提取 `[中文]` 段。
- YouMind `README_zh.md` 只提取 `#### 📝 提示词` 下的代码块，保留 JSON prompt 结构。

## 可选项

- 不修改数据库结构，不 seed 内置来源到 `prompt_library_sources`。
- 已保存参考库仍只展示用户实际导入/同步过的来源。
- 不引入外部 Markdown/中文繁简依赖，使用现有轻量解析和清洗 helper。
- 如环境缺少浏览器自动化能力，记录自动化验证结果和手工检查路径。

## 执行清单

- [x] 任务：更新本文件记录本轮范围和评审区。
- [x] 后端：新增内置来源 adapter 分发。
- [x] 后端：实现 freestylefly gallery 展开与专用解析。
- [x] 后端：实现 YouMind README_zh 专用解析。
- [x] 后端：保留 EvoLink cases 现有归一化、简体优先和清洗行为。
- [x] 前端：展示三个内置来源预设并支持一键填入。
- [x] 测试：补充 adapter、freestylefly、YouMind 和 EvoLink 回归测试。
- [x] 验证：运行 Rust 导入模块测试。
- [x] 验证：运行 Rust 全量测试。
- [x] 验证：运行前端类型检查和测试。
- [x] 评审：记录最终结果与未验证项。

## 评审结果

已完成三个长期内置 GPT Image 2 来源入口与后端专用解析器：EvoLink cases 继续归一化到 cases 并保留简体优先，freestylefly gallery 会从索引展开分册并优先提取 `[中文]` 段，YouMind README_zh 只提取 `#### 📝 提示词` 下的代码块并保留 JSON prompt 结构。

验证结果：

- `cargo test --manifest-path "F:/mySoftwareTools/PromptWeave/src-tauri/Cargo.toml" imports` 通过，30 个导入模块测试全部通过。
- `cargo test --manifest-path "F:/mySoftwareTools/PromptWeave/src-tauri/Cargo.toml"` 通过，56 个 Rust 测试全部通过。
- `corepack pnpm --dir "F:/mySoftwareTools/PromptWeave" typecheck` 通过。
- `corepack pnpm --dir "F:/mySoftwareTools/PromptWeave" test` 通过，3 个测试文件 5 个前端测试全部通过。
- `corepack pnpm --dir "F:/mySoftwareTools/PromptWeave" exec vite --host 127.0.0.1 --port 5173` 启动后首页可访问；当前环境未提供浏览器自动化工具，未执行真实浏览器点击验证。

边缘情况：

- freestylefly 索引无法解析分册链接时回退到固定 `gallery-part-1.md` / `gallery-part-2.md`。
- YouMind 只在当前案例的提示词小节中取第一个 fenced code block，避免描述、图片和详情混入正文。
- 内置来源仍不写入数据库 seed，只有用户实际导入或同步后才出现在已保存参考库。
