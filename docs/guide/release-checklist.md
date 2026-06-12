# Release Checklist

本清单用于 VoxType beta 安装包发布。V13 的发布基线是 clipboard-first：`Clipboard` 和 `Auto 安全` 是可靠路径，`SendInput 实验` 只作为可选观察项。

## Automated Checks

从仓库根目录运行：

```bash
npm run verify:v13
```

该命令会运行：

```bash
bash init.sh
npm test -- --run
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml insertion --no-run
python -m json.tool docs/harness/feature_list.json
git diff --check
```

`npm run verify:v13` 只做安全自动验证：不会录音、不会发送全局快捷键、不会抢焦点、不会操作真实用户窗口。

## Beta Build

```bash
npm run tauri -- build
```

预期发布产物在：

```text
src-tauri/target/release/bundle/
```

Windows 下常见产物：

```text
src-tauri/target/release/bundle/nsis/*.exe
src-tauri/target/release/bundle/msi/*.msi
```

Beta 测试优先分发 `nsis/*.exe`。如果 NSIS 产物不存在，再使用 MSI。

## Beta 限制

- 当前安装包预计没有代码签名，Windows SmartScreen 可能提示未知发布者。
- 本地 ASR runtime、模型文件和云端 API Key 不应打进仓库或默认安装包。
- 百度 ASR 需要测试者自己配置 API Key / Secret Key。
- 本地模型路线需要测试者在 App 内完成安装或配置。
- Beta 目标是验证真实安装、启动、录音、快捷键和 clipboard-first 上屏，不承诺正式发布质量。

## Privacy Scan

提交或打 tag 前检查 diff 不包含：

- API keys, secret keys, access tokens, cookies, or full environment variable values.
- Local user profile paths or machine-specific absolute paths.
- Audio files, exported diagnostic WAV files, model binaries, or downloaded ASR runtimes.
- Raw clipboard contents outside intentional tests.
- Personal author identity in Git metadata; use `VoxType <maintainers@voxtype.dev>`.

可用快速扫描：

```bash
git diff --cached | grep -Ei 'C:/Users|C:\\Users|API_KEY=|SECRET_KEY=|access_token|cookie' || true
```

文档里可以出现变量名，不能出现真实 secret 值。

## Manual Desktop Gate

V13 beta 给别人测试前，至少完成：

- `Clipboard` 和 `Auto 安全` 在维护者本机通过。
- `Ctrl+Alt+Space` 和 `Ctrl+Alt+V` 均能触发录音。
- Notepad 或 VS Code 中目标文本与识别记录一致。
- 底部 native overlay 不再右侧/底部截断。
- 说话开头一两个字不再高频丢失。
- 麦克风准备状态不再在可录音时长期显示等待设备。

## Clean-machine Smoke

给外部测试者前，建议在另一台 Windows 或干净用户环境验证：

1. 安装 beta 包。
2. 启动 VoxType。
3. 配置一个可用模型或云端 ASR。
4. 打开 Notepad。
5. 选择 `Auto 安全`。
6. 测试 `Ctrl+Alt+Space` 和 `Ctrl+Alt+V`。
7. 检查识别记录、metadata 和目标窗口文本是否一致。
## Latest Beta Artifacts

2026-06-06 构建通过，当前 beta 安装包：

```text
src-tauri/target/release/bundle/nsis/VoxType_0.1.0_x64-setup.exe
src-tauri/target/release/bundle/msi/VoxType_0.1.0_x64_en-US.msi
```

优先分发 NSIS `.exe`；MSI 作为备用安装包。
