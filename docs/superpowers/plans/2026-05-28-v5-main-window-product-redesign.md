# VoxType V5 主窗体产品化重设计实施计划

## 目标

把默认主窗体从“测试台 + 装饰动效”重构为“语音输入控制中心”。V5 不改录音、ASR、剪贴板上屏和原生浮窗核心链路，只重构主窗体信息架构、组件边界和视觉层级。

## 执行顺序

1. **组件拆分**
   - 从 `App.tsx` 中拆出 `MainWindow`、`DiagnosticView`、`ModeSelector`、`ReadinessPanel`、`RecentTranscript`。
   - 保持现有 command 调用和状态机不变，只移动 JSX 和 view model。

2. **主界面重排**
   - 顶部展示产品名、可用状态、设置/诊断入口。
   - 中部展示两种输入模式：`Ctrl+Alt+Space` 和 `Ctrl+Alt+V`。
   - 下方展示四项准备状态和最近识别文本。

3. **诊断模式整理**
   - 保留现有测试能力。
   - 将主界面不该出现的技术动作全部放回诊断模式。
   - 保持日志滚动和复制全部日志能力。

4. **视觉系统落地**
   - 改为深色精密控制台风格。
   - 统一字号、间距、圆角、状态色。
   - 主窗体只使用轻量语音品牌符号，不复刻完整浮窗动效。

5. **测试与文档**
   - 更新前端测试，覆盖主界面核心文案、模式入口、诊断入口和最近文本动作。
   - 更新 `docs/harness/progress.md` 和 `docs/harness/feature_list.json`。
   - 如验证发现 bug，追加到 `docs/harness/debugging-log.md`。

## 风险控制

- 不改 Rust 录音、ASR、overlay 和上屏逻辑，降低回归风险。
- 每一步先跑 `npm test -- --run src/App.test.tsx` 和 `npm run typecheck`。
- 完成后跑全量验证，再提交。

## 完成定义

- 主窗体不再呈现测试台观感。
- 两种快捷键模式入口清晰，功能不回退。
- 诊断模式能力完整保留。
- 文档和 feature list 与代码状态一致。
