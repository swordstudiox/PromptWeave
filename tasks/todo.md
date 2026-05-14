# 当前修复任务

## 范围

- 修复模板库默认导入源：默认使用 `https://github.com/EvoLinkAI/awesome-gpt-image-2-API-and-Prompts/tree/main/cases`。
- 修复用户粘贴目标仓库根地址时的导入行为：后端自动归一化到 `cases` 目录。
- 默认导入简体中文模板，跳过明显非简体中文路径，并在解析后优先保留简体中文条目。
- 清洗导入后的提示词正文，只保留提示词文本，去掉 Markdown 符号、标签、编号、代码围栏和元数据行。

## 可选项

- 不新增语言选择 UI，本轮只做目标仓库的默认简体中文导入。
- 不修改数据库结构，继续使用现有 `prompt_original`、`language` 等字段。
- 不引入外部繁简识别依赖，使用轻量启发式识别。
- 如环境缺少浏览器自动化能力，记录自动化验证结果和手工检查路径。

## 执行清单

- [x] 前端：更新导入面板默认 URL。
- [x] 后端：目标仓库根地址归一化为 `/tree/main/cases`。
- [x] 后端：默认筛选/优先保留简体中文模板。
- [x] 后端：清洗 Markdown/JSON 提示词正文。
- [x] 测试：补充 URL 归一化、语言筛选、正文清洗回归测试。
- [x] 验证：运行 Rust 导入模块测试。
- [x] 验证：运行 Rust 全量测试。
- [x] 验证：运行前端类型检查和测试。
- [x] 评审：记录最终结果与未验证项。

## 评审结果

已完成模板库导入修复：导入面板默认 URL 已改为 `https://github.com/EvoLinkAI/awesome-gpt-image-2-API-and-Prompts/tree/main/cases`；后端会把目标仓库根地址归一化到 `cases` 目录；默认导入时会跳过明显非简体中文路径，并在解析后优先保留简体中文提示词；Markdown/JSON 导入会在入库前清洗提示词正文，去掉标签、编号、代码围栏、Markdown 包裹符号和元数据行。已补充 URL 归一化、简体中文筛选、Markdown/JSON 正文清洗回归测试。验证通过：`cargo test --manifest-path "F:/mySoftwareTools/PromptWeave/src-tauri/Cargo.toml" imports`、`cargo test --manifest-path "F:/mySoftwareTools/PromptWeave/src-tauri/Cargo.toml"`、`corepack pnpm --dir "F:/mySoftwareTools/PromptWeave" typecheck`、`corepack pnpm --dir "F:/mySoftwareTools/PromptWeave" test`。已启动 Vite 开发服务器并确认 `http://127.0.0.1:5173/` 返回 200；当前环境没有浏览器自动化工具，未完成真实浏览器交互验证。
