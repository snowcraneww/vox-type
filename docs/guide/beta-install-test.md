# VoxType Beta 安装测试说明

本文面向 beta 测试者。当前版本是内部测试包，不是正式公开 release。

## 安装

优先使用 `.exe` 安装包：

```text
src-tauri/target/release/bundle/nsis/VoxType_0.1.0_x64-setup.exe
```

如果没有 `.exe`，使用：

```text
src-tauri/target/release/bundle/msi/VoxType_0.1.0_x64_en-US.msi
```

当前安装包可能没有代码签名。Windows 如果提示未知发布者，需要测试者确认继续安装。

## 首次配置

启动 VoxType 后：

1. 打开 `设置`。
2. 在 `输入` 里确认上屏策略为 `Auto 安全`。
3. 在 `模型` 里选择可用模型。
4. 如果使用百度 ASR，配置自己的 API Key / Secret Key。
5. 如果使用本地模型，按 App 内提示安装或配置 runtime/model。

不要把 API Key、Secret Key、模型文件或诊断 WAV 发回给维护者，除非明确确认脱敏和授权。

## 最小测试

1. 打开 Notepad 或 VS Code。
2. 点击目标输入框，让光标在里面。
3. 按住 `Ctrl+Alt+Space` 说一句话，松开后等待上屏。
4. 按 `Ctrl+Alt+V` 开始连续输入，再按一次停止。
5. 检查 VoxType 识别记录和目标窗口文字是否一致。

## 期望结果

- `Auto 安全` 应走 clipboard 路径。
- 目标窗口文字应与识别记录基本一致。
- 底部小胶囊浮窗不应被右侧或底部截断。
- 开头一两个字不应频繁丢失。
- 麦克风状态不应在实际可录音时长期显示等待设备。

## 反馈格式

```text
系统：Windows 版本 / 是否远程桌面
目标应用：Notepad / VS Code / 浏览器 / 其他
模型：SenseVoice Small / 百度短语音 API / 百度实时 WebSocket API / whisper.cpp
输入方式：Ctrl+Alt+Space / Ctrl+Alt+V
上屏策略：Auto 安全 / Clipboard
结果：通过 / 失败
识别记录文本：
目标窗口文本：
metadata：
备注：
```
