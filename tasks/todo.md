# 当前修复任务

## 范围

- 修复后端稳定性问题：FTS 空查询、GitHub 带斜杠分支解析、compatible 图片 API endpoint 归一化。
- 收敛前端服务层：集中 DTO、封装 Tauri invoke、修复创作页模板搜索竞态。
- 优化 UI：设计 token、基础交互态、统一反馈与空态。
- 明确不处理 API Key 明文保存。

## 可选项

- 原生 `window.confirm` 本轮保留，不做完整弹窗系统。
- UI 人工验证如环境受限，可只记录自动化验证结果并列出手动检查路径。

## 执行清单

- [x] 后端：补测试并修复 FTS 清洗后空查询。
- [x] 后端：补测试并修复 GitHub tree/blob 带斜杠分支解析。
- [x] 后端：补测试并修复 compatible 图片 API endpoint 归一化。
- [x] 前端：新增 `src/types/backend.ts`。
- [x] 前端：新增 `src/lib/services/*` 并替换组件直接 `invoke`。
- [x] 前端：修复 `CreatorWorkspace` 模板搜索竞态。
- [x] UI：新增反馈/空态组件。
- [x] UI：补齐 CSS token 与控件交互态。
- [x] UI：接入主要页面反馈与空态。
- [x] 验证：运行 TypeScript 类型检查。
- [x] 验证：运行前端测试。
- [x] 验证：运行 Rust 测试。
- [x] 评审：记录最终结果与未验证项。

## 评审结果

阶段 1 已完成：修复 FTS 清洗后空查询、GitHub tree/blob 带斜杠分支解析、compatible 图片 API endpoint 归一化，并补充 Rust 测试。阶段 2 已完成：集中后端 DTO 到 `src/types/backend.ts`，新增 template/creator/import/config service 封装，相关组件已移除直接 `invoke` 调用；`CreatorWorkspace` 模板搜索已使用 request id 防止旧请求覆盖新输入；新增模板记录到 `PromptTemplateReference` 映射单元测试。阶段 3 已完成：新增轻量反馈/空态组件，补齐基础设计 token 与交互态，并在创作、模板库、导入、历史、设置和全局错误区域统一反馈与空态。已通过 `corepack pnpm typecheck`、`corepack pnpm test -- --run`、`cargo test --manifest-path "src-tauri/Cargo.toml"` 与 `corepack pnpm vite build`。已启动 Vite 开发服务器并确认 `http://127.0.0.1:5173` 可访问；由于当前环境没有可用浏览器自动化工具，未做真实视觉交互验证。未触碰 API Key 存储策略，原生 `window.confirm` 保留。
