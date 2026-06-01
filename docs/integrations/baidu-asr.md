# 百度短语音识别 ASR 接入记录

本文记录 VoxType 对百度智能云语音识别文档的当前理解。V6 已接入百度短语音识别真实调用；V7 将百度短语音作为可选转写模型，并为实时 WebSocket API 保留 V8 配置入口。

## 官方来源

- 免费测试资源：https://cloud.baidu.com/doc/SPEECH/s/Wl9mh4doe
- 短语音识别标准版 API：https://cloud.baidu.com/doc/SPEECH/s/Jlbxdezuf
- 实时语音识别 WebSocket API：https://cloud.baidu.com/doc/SPEECH/s/jlbxejt2i

## 当前建议路线

VoxType 当前主要流程是停止录音后统一转写，与百度“短语音识别标准版 API”更匹配。实时语音识别 WebSocket API 更适合后续连续输入模式的真正流式识别、会议记录或长句实时字幕，真实接入放到 V8。

## 短语音识别标准版

官方描述：将 60 秒以内的语音识别为文字，适用于语音输入、语音交互、语音指令、语音搜索等短语音场景。

请求地址：

```text
POST http://vop.baidu.com/server_api
```

百度云服务器内网场景可用内网域名替换 `vop.baidu.com`，但 VoxType 桌面应用默认使用公网地址。

## 鉴权

官方文档说明可在 `access_token` 或 API Key 两种鉴权方式中选择一种。VoxType 配置层先按 API Key 管理，真实密钥只写入用户环境变量：

```text
BAIDU_ASR_API_KEY
```

项目配置文件只保存非密钥字段，例如 endpoint、dev_pid、cuid、format、sample rate。日志、文档和测试快照不得写入真实 Key。

## 音频要求

短语音识别支持格式：

- `pcm`，推荐
- `wav`，不压缩 PCM 编码
- `amr`
- `m4a`

VoxType 后续真实调用应优先提交 `pcm`：

- 采样率：`16000`
- 声道：`1`
- 位深：`16bit`
- 编码：单声道 PCM

这与当前本地 ASR 链路已经准备的 16 kHz mono ASR samples 基本一致。

## JSON 请求字段

官方 JSON 上传方式要求 `Content-Type: application/json`，音频以 base64 放在 `speech` 字段中。字段要点：

| 字段 | 类型 | 要求 | 说明 |
| --- | --- | --- | --- |
| `format` | string | 必填 | 音频格式，推荐 `pcm` |
| `rate` | int | 必填 | `16000` 或 `8000` |
| `channel` | int | 必填 | 固定 `1` |
| `cuid` | string | 必填 | 用户唯一标识，VoxType 默认可用 `voxtype-local` |
| `token` | string | 和 API Key 二选一 | access_token 鉴权时使用 |
| `dev_pid` | int | 选填 | 默认 `1537`，可按模型调整 |
| `lm_id` | int | 选填 | 自训练平台模型 ID |
| `speech` | string | 必填 | 音频二进制 base64 |
| `len` | int | 必填 | 原始音频字节数 |

RAW 上传方式可直接把音频二进制放入 body，`Content-Type` 需带格式和采样率，例如：

```text
Content-Type: audio/pcm;rate=16000
```

VoxType 后续实现时优先评估 JSON 方式，因为字段显式、调试更直接；如性能或体积成为问题，再改 RAW。

## dev_pid 选择

文档中短语音标准版默认 `dev_pid` 是 `1537`。实时 WebSocket 文档中还列出常用模型：

| dev_pid | 模型 | 标点 | 场景 |
| --- | --- | --- | --- |
| `1537` | 中文普通话 | 弱标点 | 手机近场输入 |
| `15372` | 中文普通话 | 加强标点 | 手机近场输入 |
| `15376` | 中文多方言 | 弱标点 | 手机近场输入 |
| `1737` | 英语 | 无标点 | 手机近场输入 |
| `17372` | 英语 | 加强标点 | 手机近场输入 |

VoxType 默认先用 `1537`，设置页允许维护者改成 `15372` 等值。

## 返回结果

短语音识别成功返回 JSON，文本在 `result` 数组中，通常取第一个候选：

```json
{
  err_no: 0,
  err_msg: success.,
  corpus_no: 15984125203285346378,
  sn: 481D633F-73BA-726F-49EF-8659ACCC2F3D,
  result: [北京天气]
}
```

错误返回同样是 JSON，包含 `err_no`、`err_msg` 和 `sn`。

## VoxType 当前实现边界

V7 已做：

- 模型选择页移除 MiniMax 用户配置入口。
- 按住说话和连续输入可分别选择默认模型。
- 百度短语音配置保存 endpoint、`dev_pid`、`cuid`、`format`、sample rate 和可选 `lm_id`。
- 百度 API Key 与 Secret Key 通过密码输入框写入用户环境变量 `BAIDU_ASR_API_KEY` / `BAIDU_ASR_SECRET_KEY`。
- 百度短语音真实调用使用 OAuth `access_token` 路径。
- 百度实时 WebSocket 只保留 V8 占位配置，当前不可用于真实转写。

V7 不做：

- 不实现实时 WebSocket ASR 音频帧发送、增量结果合并和流式上屏。
- 不接入 LLM 纠错。

后续 V8 真实 WebSocket 接入应新增独立任务，先写协议帧、鉴权、错误恢复和增量结果合并测试，再接入连续输入录音链路。

## 2026-06-01 OAuth access_token update

Official auth document: https://cloud.baidu.com/doc/SPEECH/s/cm8sn2bii

The short speech endpoint remains:

```text
POST http://vop.baidu.com/server_api
```

VoxType now uses Baidu's `access_token` auth path for the short speech JSON request:

1. Read `BAIDU_ASR_API_KEY` and `BAIDU_ASR_SECRET_KEY` from the user environment.
2. Request an access token from `https://aip.baidubce.com/oauth/2.0/token` with `grant_type=client_credentials`, `client_id=<API Key>`, and `client_secret=<Secret Key>`.
3. Put the returned `access_token` into the ASR JSON payload's `token` field.

Do not put the raw API Key or Secret Key into the ASR `server_api` URL or JSON body. Do not write either real value to project config, diagnostics, docs, or tests.

## 2026-06-01 V8 realtime WebSocket implementation update

Official realtime WebSocket API source: https://cloud.baidu.com/doc/SPEECH/s/jlbxejt2i

V8 implementation treats Baidu Realtime WebSocket API as a continuous-input feature, not as the default push-to-talk path.

Key protocol details recorded for implementation:

- WebSocket URI: `wss://vop.baidu.com/realtime_asr`.
- START frame follows the official JSON example: top-level `type: "START"`, nested `data` containing `appid`, `appkey`, `dev_pid`, optional `lm_id`, `cuid`, optional `user`, `format`, and `sample`.
- `appid` is non-secret Baidu application AppID config. `appkey` uses the Baidu console API Key; VoxType should read it from `BAIDU_ASR_API_KEY` and never log the value.
- `format` is fixed to `pcm`.
- `sample` is fixed to `16000`.
- Recommended Mandarin realtime model is `dev_pid=15372` for stronger punctuation.
- Official frame guidance uses 160 ms PCM chunks: `160 * (16000 * 2 / 1000) = 5120` bytes.
- Control frames include `FINISH`, `CANCEL`, and `HEARTBEAT`.

Design and implementation plan:

- `docs/superpowers/specs/2026-06-01-v8-baidu-realtime-websocket-design.md`
- `docs/superpowers/plans/2026-06-01-v8-baidu-realtime-websocket.md`


### V8 implementation

- Backend adds `src-tauri/src/baidu_realtime.rs` and uses `tungstenite` to connect to `wss://vop.baidu.com/realtime_asr`.
- START frames follow the official Baidu shape: top-level `type: "START"` with nested `data` containing `appid`, `appkey`, `dev_pid`, optional `lm_id`, `cuid`, optional `user`, `format`, and `sample`.
- `appid` is non-secret config. `appkey` is read from `BAIDU_ASR_API_KEY` and must not be logged or saved to project files.
- Audio is 16 kHz, 16-bit mono PCM. Full 160 ms / 5120-byte frames are sent; before `FINISH`, the worker drains any currently available complete frames.
- Control frames support `FINISH`, `CANCEL`, and `HEARTBEAT`.
- Frontend continuous input with model `baidu-realtime` starts `start_baidu_realtime_session`, inserts final text events through the existing clipboard path, then calls `finish_baidu_realtime_session` and records one merged transcript row.
- Push-to-talk rejects Baidu Realtime WebSocket API with a clear unsupported-mode error in V8.

### V8 residual risks

- Automated tests cover protocol serialization, PCM chunking, config validation, frontend routing, and transcript merging. Real WebSocket streaming still requires maintainer verification with real Baidu credentials, network, and microphone input.
- The frontend inserts final text only. Partial text is shown as transient status so unstable partial hypotheses do not rewrite the target app.
- Finish uses a bounded wait. If Baidu does not return final text in time, the transcript row can only include final fragments already received.
