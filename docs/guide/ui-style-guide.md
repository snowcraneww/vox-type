# VoxType UI Style Guide

## 适用范围

本指南用于所有主窗口、模型配置、诊断、文本优化和后续设置类 UI。新增 UI 前必须先阅读现有 `src/styles.css` 与本指南，优先复用已有组件结构和 class，不要为单个功能临时创造一套视觉语言。

V10 音频增强开关、预处理 metadata、后续配置项都应按这里执行。

## 当前产品风格

VoxType 当前主界面是浅绿产品化控制台，不是营销页，也不是深色调试台。整体观感应保持安静、紧凑、可扫描：淡绿白背景、轻微玻璃面板、低饱和绿色文字、圆角但不过度可爱。

核心基调：

- 页面背景使用浅绿径向渐变和近白底色，参考 `body:has(.control-center.v51)`、`body:has(.model-shell)`。
- 主面板使用半透明白绿玻璃层，参考 `.control-center.v51`、`.model-panel`。
- 内容区使用浅绿白小面板，参考 `.model-routing-section`、`.model-config-section`、`.active-config-panel`。
- 记录项和小卡片使用更轻的白绿背景，参考 `.history-list article`、`.mode-model-row`。

## 字体与密度

基础字体沿用系统栈：`-apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", "Microsoft YaHei", sans-serif`。

新增 UI 默认遵守：

- 正文和输入框使用 `12px` 到 `13px`。
- 小标题、字段标签、pill 使用 `11px` 到 `12px`，字重约 `760` 到 `840`。
- 面板标题不使用 hero 大字，设置页标题参考 `.model-config-title span:first-child`。
- 字距保持 `0`，不要使用负字距。
- 设置面板保持紧凑，优先用 `gap: 10px` 到 `14px`、`padding: 10px` 到 `16px`。

## 颜色规则

新增 UI 应复用当前浅绿语义：

- 主要文字：`#102519`、`#102818` 或接近的深绿黑。
- 次级文字：`rgba(43, 70, 53, 0.60-0.72)`。
- 标签文字：`rgba(19, 116, 45, 0.72-0.82)`。
- 面板边框：`rgba(50, 116, 62, 0.13-0.16)`。
- 输入边框：`rgba(54, 107, 75, 0.15-0.16)`。
- 主操作按钮：浅绿渐变，参考 `.model-panel .button-row button:first-child`。

不要新增大面积蓝紫、纯深色卡片、强阴影、彩色渐变按钮，除非是在复用录音声波/浮窗动效语言。

## 组件复用

新增设置页面或设置块优先复用：

- 页面外壳：`.app-shell.model-shell` + `.model-panel`。
- 顶部：`.model-header`、`.model-title-line`、`.secondary-button`。
- 分页/分段控制：`.model-page-tabs`、`.config-model-switch`、`.segmented-models.route-models`。
- 设置面板：`.model-config-section`、`.active-config-panel`、`.model-config-title`。
- 字段：`.field`、`.compact-field-grid`。
- 按钮行：`.button-row`。
- 状态点：`.ready-dot`。
- 记录 metadata：`.record-meta .stat-pill`。
- 质量警告：`.quality-pill`。

只有当现有结构无法表达新行为时才添加新 class。新增 class 应补在相关历史注释附近，并与同层组件共享字体、边框、背景和间距。

## 开关与 Metadata

开关类 UI 不要使用突兀的大卡片。优先做成现有设置面板里的紧凑行：左侧标题和说明，右侧按钮或分段控件。按钮应沿用 `.config-switch-button` 或普通 `.model-panel button` 风格。

记录 metadata 应优先进入 `.record-meta`，用 `.stat-pill` 的尺寸和字重。只有警告才使用 `.quality-pill` 的红色提示。不要在记录列表中新增大块说明文本，不要让 metadata 挤压正文。

## 禁止事项

- 不要新增与当前浅绿主题冲突的孤立 UI 样式。
- 不要把设置项做成嵌套卡片套卡片。
- 不要使用大字号说明文字解释功能。
- 不要新增宽松留白导致主窗口信息密度下降。
- 不要让按钮、pill 或长模型名撑开布局；必须使用 `min-width: 0`、`text-overflow` 或换行策略。
- 不要只在桌面宽度检查，新 UI 必须考虑 `max-width: 820px` 和 `max-width: 520px` 的已有断点。

## 变更检查

涉及 UI 的改动完成前至少检查：

1. 新增元素是否复用了本指南列出的现有 class。
2. 字体大小、字重、圆角、边框和按钮风格是否与相邻区域一致。
3. 长文本、中文标签、模型名和 metadata 是否不会溢出或遮挡。
4. 浅绿页面、模型配置页、识别记录列表在视觉上是否属于同一套系统。
5. 若新增 CSS，是否有明确理由，且只覆盖当前功能所需范围。
