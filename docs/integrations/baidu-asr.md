# 百度 ASR 接入记录

本文记录 VoxType 对百度智能云语音识别文档的当前理解。V6 已接入百度短语音识别真实调用；V7 将百度短语音作为可选转写模型；V8 已接入百度实时 WebSocket API 连续输入链路，真实桌面流式验证仍需维护者执行。

## 官方来源

- 免费测试资源：https://cloud.baidu.com/doc/SPEECH/s/Wl9mh4doe
- 短语音识别标准版 API：https://cloud.baidu.com/doc/SPEECH/s/Jlbxdezuf
- 实时语音识别 WebSocket API：https://cloud.baidu.com/doc/SPEECH/s/jlbxejt2i

## 当前建议路线

VoxType 当前支持两条百度 ASR 路线：按住说话和停止后整段识别优先使用“短语音识别标准版 API”；连续输入模式可以选择“实时语音识别 WebSocket API”，用于边录音边接收 partial/final 结果并把 final 片段上屏。

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

VoxType 真实调用优先提交 `pcm`：

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

VoxType 当前短语音调用使用 JSON 方式，因为字段显式、调试更直接；如性能或体积成为问题，再评估 RAW。

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

已完成：

- 模型选择页移除 MiniMax 用户配置入口。
- 按住说话和连续输入可分别选择默认模型。
- 百度短语音配置保存 endpoint、`dev_pid`、`cuid`、`format`、sample rate 和可选 `lm_id`。
- 百度 API Key 与 Secret Key 通过密码输入框写入用户环境变量 `BAIDU_ASR_API_KEY` / `BAIDU_ASR_SECRET_KEY`。
- 百度短语音真实调用使用 OAuth `access_token` 路径。
- 百度实时 WebSocket API 已作为连续输入模型接入：`Ctrl+Alt+V` 启动 session，final 片段通过现有剪贴板路径上屏，停止时合并为一条识别记录。

当前不做：

- 按住说话模式不使用百度实时 WebSocket API；如果选择该模型，会返回明确的不支持错误。
- 不接入 LLM 纠错。

残余风险：实时 WebSocket 自动测试已覆盖协议帧、分帧、配置校验、前端路由和记录合并，但真实网络、账号权限、麦克风和目标应用上屏仍需要维护者在 Tauri 桌面环境验证。

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

### Official realtime WebSocket protocol notes

The official document describes realtime ASR as a WebSocket flow that uploads audio while receiving recognition results. The intended use cases include long-sentence voice input, subtitles, live-stream quality inspection, and meeting notes.

Connection and frame sequence:

1. Connect to `wss://vop.baidu.com/realtime_asr?sn=<request-id>`.
2. Send one text START frame as JSON.
3. Send binary PCM audio frames while recording.
4. Receive text JSON result frames from Baidu.
5. Send a text FINISH frame when audio ends, or CANCEL when abandoning the session.
6. Close the WebSocket after the server finishes or closes the connection.

The `sn` query parameter is caller-defined and is used for log lookup. It should contain only English letters, digits, `-`, and `_`, up to 128 characters. VoxType keeps the configured endpoint as `wss://vop.baidu.com/realtime_asr` and appends a generated `sn=voxtype-<pid>-<timestamp>` at runtime.

START frame shape:

```json
{
  "type": "START",
  "data": {
    "appid": 10500017,
    "appkey": "<BAIDU_ASR_API_KEY>",
    "dev_pid": 15372,
    "cuid": "voxtype-local",
    "format": "pcm",
    "sample": 16000
  }
}
```

Optional START fields:

- `lm_id`: numeric self-trained model ID. It must align with the chosen base `dev_pid`.
- `user`: required by Baidu when using Chinese multi-dialect model `dev_pid=15376`; otherwise optional.

Audio frame requirements:

- WebSocket opcode: Binary.
- Audio format: 16 kHz, 16-bit, mono PCM.
- Each normal audio frame should contain 20-200 ms of audio.
- Official recommendation: 160 ms frames, `160 * (16000 * 2 / 1000) = 5120` bytes.
- Baidu may disconnect and report an error if it receives no audio for 5 seconds. VoxType sends full 160 ms frames while recording and can send HEARTBEAT frames during waits.

Control frames:

```json
{ "type": "FINISH" }
```

```json
{ "type": "CANCEL" }
```

```json
{ "type": "HEARTBEAT" }
```

Result frames:

- `MID_TEXT`: temporary result for one utterance.
- `FIN_TEXT`: final result for one utterance; may include `start_time` and `end_time` in milliseconds.
- `HEARTBEAT`: server heartbeat; VoxType ignores it.
- `err_no != 0`: error result. The WebSocket request may still continue for other utterances unless the server closes the connection.

### VoxType realtime configuration model

百度短语音 API and 百度实时 WebSocket API use the same Baidu application credentials:

- `BAIDU_ASR_API_KEY`: shared API Key. Short speech uses it with Secret Key to fetch an OAuth `access_token`; realtime WebSocket sends it as START frame `appkey`.
- `BAIDU_ASR_SECRET_KEY`: shared Secret Key for short speech OAuth. Realtime WebSocket does not send Secret Key in the START frame, but it still belongs to the same Baidu application credential pair.
- `AppID`: realtime WebSocket additionally requires the same Baidu application AppID as START frame `appid`. VoxType defaults this non-secret field to the official sample-style value `10500017`, but users should replace it with their own console AppID when available.

The Baidu Short Speech API and Baidu Realtime WebSocket API config panels both show the same shared credential status and password inputs for `BAIDU_ASR_API_KEY` and `BAIDU_ASR_SECRET_KEY`. The raw values are never loaded back into the UI; VoxType only shows masked previews such as `ba***ey` after reading the user environment. Saving either credential from either panel writes to the same user environment variable.

The realtime WebSocket config panel stores only non-secret realtime fields:

| Field | Default | Notes |
| --- | --- | --- |
| `baiduRealtimeAppId` | `10500017` | Numeric Baidu application AppID; required by START frame. |
| `baiduRealtimeEndpoint` | `wss://vop.baidu.com/realtime_asr` | Runtime appends `sn`. |
| `baiduRealtimeDevPid` | `15372` | Mandarin with stronger punctuation; official recommended model for input-like scenarios. |
| `baiduRealtimeCuid` | `voxtype-local` | Device/user unique ID for UV stats and troubleshooting. |
| `baiduRealtimeFormat` | `pcm` | Fixed by official docs. |
| `baiduRealtimeSampleRate` | `16000` | Fixed by official docs. |
| `baiduRealtimeUser` | empty | Required only for `dev_pid=15376`. |

Realtime ready means `BAIDU_ASR_API_KEY` is readable, AppID and dev_pid are numeric, endpoint is the official realtime endpoint, cuid is present, format is `pcm`, and sample rate is `16000`. It intentionally does not require `BAIDU_ASR_SECRET_KEY` for the WebSocket START frame; the UI still displays Secret Key because short speech and realtime belong to the same Baidu application credential set, and short speech ready still requires both API Key and Secret Key.

Key protocol details recorded for implementation:

- WebSocket URI: `wss://vop.baidu.com/realtime_asr?sn=<generated>`.
- START frame follows the official JSON example: top-level `type: "START"`, nested `data` containing numeric `appid`, `appkey`, numeric `dev_pid`, optional numeric `lm_id`, `cuid`, optional `user`, `format`, and `sample`.
- `appid` is non-secret Baidu application AppID config. `appkey` uses the Baidu console API Key from `BAIDU_ASR_API_KEY`; VoxType must never log the value.
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
