# V13 浮窗截断复发与 VAD 开头裁剪

日期：2026-06-06
范围：V13 桌面验证、native Win32 overlay、V10 audio preprocessing

## 现象

维护者在 V13 桌面测试中确认 clipboard-first 上屏基本通过，但发现两个复发/质量问题：

1. 底部胶囊状桌面浮窗右侧和底部又出现截断。
2. 多数识别结果开头少一两个字，且缺失发生在 ASR 返回文本阶段，不是上屏阶段。

同时主界面准备状态里的麦克风显示“等待设备”，但实际可以录音。

## 为什么复杂

这不是单一链路问题：

- 上屏文本和识别记录基本一致，说明 insertion 层不是主因。
- 浮窗截断属于 native Win32 overlay 绘制/窗口边界问题，历史上 WebView fallback 已经修过类似问题。
- 开头少字属于音频进入 ASR 前的处理问题，最可能在 VAD trim，而不是文本后处理或 clipboard。
- 麦克风状态是 UI readiness 和真实 recorder runtime 状态不同步。

## 根因

### 浮窗截断

2026-06-02 曾把 overlay 视窗扩大到 `132 x 44`，WebView fallback 里的 SVG 胶囊放在 `x=6.5, y=6.5, width=119, height=31`，因此有安全内边距。

但 native Win32 overlay 仍然把胶囊从 `x=0` 画到整个 `WIDTH`，圆角和抗锯齿边缘贴着透明 layered window 的边界，右侧和底部容易被裁掉。

### 开头少字

V10 audio preprocessing 的 VAD trim 使用 20 ms frame、RMS 阈值和 120 ms padding。对真实说话里较轻的第一个音节，弱起音可能低于 speech RMS 阈值；裁剪从第一个“强语音帧”前 120 ms 开始，仍可能吃掉开头一两个字。

### 麦克风等待状态

主界面 readiness 的麦克风项只看 `recorderInfo`。如果初始 `getDefaultInputInfo` 失败、慢返回或被设备枚举竞态影响，即使后续 `startRecording` 已证明录音可用，主界面仍显示等待设备。

## 修复

- native overlay 增加 `native_overlay_layout()`，并用测试锁定窗口和胶囊安全边距。
- native overlay 绘制改为在 `132 x 44` 视窗内绘制 `x=6.5, y=6.5, width=119, height=31` 的胶囊，与 WebView fallback 对齐。
- VAD padding 从 120 ms 提高到 300 ms，降低真实语音开头被裁掉的概率。
- 增加弱起音回归测试，要求 VAD trim 保留更多前置上下文。
- 前端在 `startRecording` 成功后，如果 `recorderInfo` 仍为空，会用录音运行时的 sample rate/channels 自愈麦克风准备状态。

## 验证

已通过：

```bash
cargo test --manifest-path src-tauri/Cargo.toml audio_preprocess --no-run
cargo test --manifest-path src-tauri/Cargo.toml overlay --no-run
npm test -- --run src/App.test.tsx -t self-heals
npm run typecheck
cargo check --manifest-path src-tauri/Cargo.toml --lib
npm run verify:v13
```

`npm run verify:v13` 通过，包含 63 个前端测试。

仍需真实桌面验证：

1. 按住说话时底部 native overlay 右侧和底部不再截断。
2. 开启音频增强后，说话开头一两个字的丢失明显减少。
3. 初始麦克风状态不应在可录音时长期显示等待设备。

## 经验

- overlay 视窗扩大不等于 native 绘制也安全；native 绘制也必须有同样的 layout 测试。
- 对语音输入法，VAD trim 宁可保守多留 100-300 ms，也不要裁掉开头音节。
- readiness UI 不应只信启动探测；真实录音成功是更强的麦克风可用证据。
