# ASR 一键安装设计

## 背景

维护者确认普通用户不应该手动寻找 `whisper-cli.exe`、下载模型或配置环境变量。VoxType 需要在现有 `ASR 配置` 面板上提供一个可点击的一键安装入口。

## 目标

- 在 Windows x64 上自动下载 whisper.cpp CPU 版 binary。
- 自动下载默认中文可用模型 `ggml-base.bin`。
- 把 binary 和模型保存到 Tauri 应用数据目录，不写入仓库。
- 安装完成后自动保存 `AsrConfig`，让 `转写最近录音` 可直接使用。
- 所有失败原因用中文显示。

## 非目标

- 第一版不做 CUDA/GPU 自动识别。
- 第一版不做模型选择器。
- 第一版不做后台断点续传。
- 第一版不把 binary 或模型提交到 Git。

## 方案

默认安装内容：

| 项 | 默认值 | 来源 |
|---|---|---|
| whisper.cpp binary | `whisper-bin-x64.zip` | `ggml-org/whisper.cpp` GitHub Release |
| 模型 | `ggml-base.bin` | `ggerganov/whisper.cpp` Hugging Face 仓库 |
| 语言 | `zh` | VoxType 默认配置 |

安装目录：

```text
<Tauri app data dir>/managed-asr/whisper.cpp/
  bin/
    whisper-cli.exe
  models/
    ggml-base.bin
  downloads/
    whisper-bin-x64.zip
```

## 用户体验

`ASR 配置` 面板新增按钮：`一键安装 whisper.cpp`。

点击后：

1. 下载官方 Windows x64 CPU binary。
2. 解压 `whisper-cli.exe` 和必要 DLL。
3. 下载 `ggml-base.bin`。
4. 保存 ASR 配置。
5. 刷新配置状态。

成功时显示：`一键安装完成，可以调用 whisper.cpp。`

失败时显示具体原因，例如：

- `下载 whisper.cpp 失败`
- `解压 whisper.cpp 失败`
- `下载模型失败`
- `安装完成后没有找到 whisper-cli.exe`

## 许可证与安全

- whisper.cpp 是第三方开源项目，VoxType 不把其 binary 或模型重新分发到本仓库。
- 下载源使用官方 GitHub Release 和官方文档指向的 Hugging Face 模型仓库。
- 当前版本已校验 `whisper-bin-x64.zip` 的 SHA-256 和 `ggml-base.bin` 的 SHA-1。后续版本应加入下载源镜像选择、模型选择和更完整的许可证说明页。
