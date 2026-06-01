use crate::error::VoxError;
use crate::recorder::RecorderManager;
use serde::{Deserialize, Serialize};
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

pub const BAIDU_REALTIME_ENDPOINT: &str = "wss://vop.baidu.com/realtime_asr";
pub const BAIDU_REALTIME_SAMPLE_RATE: u32 = 16_000;
pub const BAIDU_REALTIME_FRAME_SAMPLES: usize = 2_560;
pub const BAIDU_REALTIME_FRAME_BYTES: usize = 5_120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaiduRealtimeConfig {
    pub endpoint: String,
    pub appid: String,
    pub appkey: String,
    pub dev_pid: i32,
    pub lm_id: Option<String>,
    pub cuid: String,
    pub user: Option<String>,
    pub format: String,
    pub sample: u32,
}

impl BaiduRealtimeConfig {
    pub fn start_frame(&self) -> BaiduRealtimeStartFrame {
        BaiduRealtimeStartFrame {
            appid: self.appid.clone(),
            appkey: self.appkey.clone(),
            dev_pid: self.dev_pid,
            lm_id: self.lm_id.clone(),
            cuid: self.cuid.clone(),
            user: self.user.clone(),
            format: self.format.clone(),
            sample: self.sample,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaiduRealtimeStartFrame {
    pub appid: String,
    pub appkey: String,
    pub dev_pid: i32,
    pub lm_id: Option<String>,
    pub cuid: String,
    pub user: Option<String>,
    pub format: String,
    pub sample: u32,
}

#[derive(Debug, Serialize)]
struct StartFrameEnvelope<'a> {
    #[serde(rename = "type")]
    frame_type: &'static str,
    data: StartFrameData<'a>,
}

#[derive(Debug, Serialize)]
struct StartFrameData<'a> {
    appid: &'a str,
    appkey: &'a str,
    dev_pid: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    lm_id: Option<&'a str>,
    cuid: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<&'a str>,
    format: &'a str,
    sample: u32,
}

impl BaiduRealtimeStartFrame {
    pub fn to_json(&self) -> Result<String, VoxError> {
        let envelope = StartFrameEnvelope {
            frame_type: "START",
            data: StartFrameData {
                appid: &self.appid,
                appkey: &self.appkey,
                dev_pid: self.dev_pid,
                lm_id: self.lm_id.as_deref(),
                cuid: &self.cuid,
                user: self.user.as_deref(),
                format: &self.format,
                sample: self.sample,
            },
        };
        serde_json::to_string(&envelope).map_err(|error| {
            VoxError::Asr(format!(
                "serialize Baidu realtime START frame failed: {error}"
            ))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaiduRealtimeControlFrame {
    frame_type: &'static str,
}

#[derive(Debug, Serialize)]
struct ControlFrameEnvelope {
    #[serde(rename = "type")]
    frame_type: &'static str,
}

impl BaiduRealtimeControlFrame {
    pub fn finish() -> Self {
        Self {
            frame_type: "FINISH",
        }
    }

    pub fn cancel() -> Self {
        Self {
            frame_type: "CANCEL",
        }
    }

    pub fn heartbeat() -> Self {
        Self {
            frame_type: "HEARTBEAT",
        }
    }

    pub fn to_json(&self) -> Result<String, VoxError> {
        serde_json::to_string(&ControlFrameEnvelope {
            frame_type: self.frame_type,
        })
        .map_err(|error| {
            VoxError::Asr(format!(
                "serialize Baidu realtime control frame failed: {error}"
            ))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaiduRealtimeEvent {
    Started,
    Partial { text: String, sequence: Option<u64> },
    Final { text: String, sequence: Option<u64> },
    Finished,
    Error { code: i64, message: String },
    Ignored,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PcmFrameBuffer {
    pending_samples: Vec<i16>,
}

impl PcmFrameBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_samples(&mut self, samples: &[i16]) {
        self.pending_samples.extend_from_slice(samples);
    }

    pub fn drain_full_frames(&mut self) -> Vec<Vec<u8>> {
        let full_frame_count = self.pending_samples.len() / BAIDU_REALTIME_FRAME_SAMPLES;
        let mut frames = Vec::with_capacity(full_frame_count);
        for _ in 0..full_frame_count {
            let frame_samples: Vec<i16> = self
                .pending_samples
                .drain(..BAIDU_REALTIME_FRAME_SAMPLES)
                .collect();
            frames.push(pcm_i16_le_bytes(&frame_samples));
        }
        frames
    }

    pub fn pending_sample_count(&self) -> usize {
        self.pending_samples.len()
    }
}

pub fn config_from_cloud_status(
    status: &crate::cloud_asr_config::CloudAsrConfigStatus,
    appkey: String,
) -> Result<BaiduRealtimeConfig, VoxError> {
    let config = &status.config;
    let endpoint = required_field(
        config.baidu_realtime_endpoint.as_deref(),
        "Baidu realtime WebSocket endpoint is not configured.",
    )?;
    if endpoint.trim() != BAIDU_REALTIME_ENDPOINT {
        return Err(VoxError::Config(
            "Baidu realtime WebSocket endpoint must be wss://vop.baidu.com/realtime_asr."
                .to_string(),
        ));
    }
    let appid = required_field(
        config.baidu_realtime_app_id.as_deref(),
        "Baidu realtime WebSocket AppID is not configured.",
    )?;
    let appkey = required_field(
        Some(appkey.as_str()),
        "Missing BAIDU_ASR_API_KEY environment variable.",
    )?;
    let dev_pid_text = required_field(
        config.baidu_realtime_dev_pid.as_deref(),
        "Baidu realtime WebSocket dev_pid is not configured.",
    )?;
    let dev_pid = dev_pid_text.parse::<i32>().map_err(|_| {
        VoxError::Config("Baidu realtime WebSocket dev_pid must be numeric.".to_string())
    })?;
    let cuid = required_field(
        config.baidu_realtime_cuid.as_deref(),
        "Baidu realtime WebSocket cuid is not configured.",
    )?;
    let format = required_field(
        config.baidu_realtime_format.as_deref(),
        "Baidu realtime WebSocket audio format is not configured.",
    )?;
    if format != "pcm" {
        return Err(VoxError::Config(
            "Baidu realtime WebSocket audio format must be pcm.".to_string(),
        ));
    }
    let sample = config.baidu_realtime_sample_rate.ok_or_else(|| {
        VoxError::Config("Baidu realtime WebSocket sample rate is not configured.".to_string())
    })?;
    if sample != BAIDU_REALTIME_SAMPLE_RATE {
        return Err(VoxError::Config(
            "Baidu realtime WebSocket sample rate must be 16000.".to_string(),
        ));
    }

    Ok(BaiduRealtimeConfig {
        endpoint,
        appid,
        appkey,
        dev_pid,
        lm_id: config
            .baidu_lm_id
            .clone()
            .filter(|value| !value.trim().is_empty()),
        cuid,
        user: config
            .baidu_realtime_user
            .clone()
            .filter(|value| !value.trim().is_empty()),
        format,
        sample,
    })
}

fn required_field(value: Option<&str>, message: &str) -> Result<String, VoxError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| VoxError::Config(message.to_string()))
}

pub fn pcm_i16_le_bytes(samples: &[i16]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect()
}

pub fn parse_baidu_realtime_message(body: &str) -> Result<BaiduRealtimeEvent, VoxError> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| VoxError::Asr(format!("parse Baidu realtime response failed: {error}")))?;
    let frame_type = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let sequence = value
        .get("sn")
        .or_else(|| value.get("sequence"))
        .and_then(serde_json::Value::as_u64);

    if frame_type.eq_ignore_ascii_case("ERROR") || value.get("err_no").is_some() {
        let code = value
            .get("err_no")
            .or_else(|| value.get("code"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-1);
        let message = value
            .get("err_msg")
            .or_else(|| value.get("message"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown Baidu realtime error")
            .to_string();
        return Ok(BaiduRealtimeEvent::Error { code, message });
    }

    if frame_type.eq_ignore_ascii_case("START") || frame_type.eq_ignore_ascii_case("STARTED") {
        return Ok(BaiduRealtimeEvent::Started);
    }
    if frame_type.eq_ignore_ascii_case("FINISH") || frame_type.eq_ignore_ascii_case("FINISHED") {
        return Ok(BaiduRealtimeEvent::Finished);
    }

    let text = extract_result_text(&value);
    if frame_type.eq_ignore_ascii_case("MID_TEXT")
        || frame_type.eq_ignore_ascii_case("PARTIAL")
        || frame_type.eq_ignore_ascii_case("RESULT")
    {
        return Ok(BaiduRealtimeEvent::Partial { text, sequence });
    }
    if frame_type.eq_ignore_ascii_case("FIN_TEXT") || frame_type.eq_ignore_ascii_case("FINAL") {
        return Ok(BaiduRealtimeEvent::Final { text, sequence });
    }

    Ok(BaiduRealtimeEvent::Ignored)
}

fn extract_result_text(value: &serde_json::Value) -> String {
    value
        .get("result")
        .or_else(|| value.get("data").and_then(|data| data.get("result")))
        .and_then(text_from_json_value)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn text_from_json_value(value: &serde_json::Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    value
        .as_array()
        .and_then(|items| items.first())
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BaiduRealtimeResultEvent {
    pub text: String,
    pub is_final: bool,
    pub sequence: u64,
    pub started_at_ms: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BaiduRealtimeSessionStatus {
    pub state: String,
    pub message: String,
    pub started_at_ms: Option<u64>,
    pub duration_ms: u64,
    pub final_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BaiduRealtimeSessionSummary {
    pub status: BaiduRealtimeSessionStatus,
    pub text: String,
    pub duration_ms: u64,
    pub char_count: usize,
}

#[derive(Default)]
pub struct BaiduRealtimeSessionManager {
    active: Mutex<Option<ActiveRealtimeSession>>,
}

struct ActiveRealtimeSession {
    control_tx: mpsc::Sender<RealtimeControl>,
    result_rx: mpsc::Receiver<WorkerResult>,
    started_at: Instant,
    started_at_ms: u64,
    final_texts: Vec<String>,
    last_state: String,
}

enum RealtimeControl {
    Finish,
    Cancel,
}

enum WorkerResult {
    Event(BaiduRealtimeResultEvent),
    Finished,
    Failed(String),
}

impl BaiduRealtimeSessionManager {
    pub fn start(
        &self,
        app: AppHandle,
        recorder: &RecorderManager,
        config: BaiduRealtimeConfig,
    ) -> Result<BaiduRealtimeSessionStatus, VoxError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| VoxError::Asr("Baidu realtime session lock is poisoned.".to_string()))?;
        if active.is_some() {
            return Err(VoxError::Asr(
                "Baidu realtime session is already running.".to_string(),
            ));
        }

        recorder.start()?;
        let (control_tx, control_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let started_at = Instant::now();
        let started_at_ms = unix_time_ms();
        thread::spawn(move || {
            run_realtime_worker(app, config, control_rx, result_tx, started_at_ms)
        });
        let status = BaiduRealtimeSessionStatus {
            state: "streaming".to_string(),
            message: "Baidu realtime WebSocket session started.".to_string(),
            started_at_ms: Some(started_at_ms),
            duration_ms: 0,
            final_text: String::new(),
        };
        *active = Some(ActiveRealtimeSession {
            control_tx,
            result_rx,
            started_at,
            started_at_ms,
            final_texts: Vec::new(),
            last_state: "streaming".to_string(),
        });
        Ok(status)
    }

    pub fn finish(
        &self,
        recorder: &RecorderManager,
    ) -> Result<BaiduRealtimeSessionSummary, VoxError> {
        self.stop_with_control(recorder, RealtimeControl::Finish)
    }

    pub fn cancel(
        &self,
        recorder: &RecorderManager,
    ) -> Result<BaiduRealtimeSessionStatus, VoxError> {
        let summary = self.stop_with_control(recorder, RealtimeControl::Cancel)?;
        Ok(summary.status)
    }

    pub fn status(&self) -> Result<BaiduRealtimeSessionStatus, VoxError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| VoxError::Asr("Baidu realtime session lock is poisoned.".to_string()))?;
        let Some(session) = active.as_mut() else {
            return Ok(BaiduRealtimeSessionStatus {
                state: "idle".to_string(),
                message: "No Baidu realtime session is running.".to_string(),
                started_at_ms: None,
                duration_ms: 0,
                final_text: String::new(),
            });
        };
        drain_worker_results(session);
        Ok(session_status(
            session,
            "Baidu realtime session is running.",
        ))
    }

    fn stop_with_control(
        &self,
        recorder: &RecorderManager,
        control: RealtimeControl,
    ) -> Result<BaiduRealtimeSessionSummary, VoxError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| VoxError::Asr("Baidu realtime session lock is poisoned.".to_string()))?;
        let Some(mut session) = active.take() else {
            return Err(VoxError::Asr(
                "No Baidu realtime session is running.".to_string(),
            ));
        };
        let _ = session.control_tx.send(control);
        let stop_deadline = Instant::now() + Duration::from_secs(8);
        while Instant::now() < stop_deadline {
            drain_worker_results(&mut session);
            if session.last_state == "finished" || session.last_state == "failed" {
                break;
            }
            thread::sleep(Duration::from_millis(40));
        }
        drain_worker_results(&mut session);
        let recorded = recorder.stop();
        if let Err(error) = recorded {
            if !error.to_string().contains("没有正在进行的录音") {
                return Err(error);
            }
        }
        let text = session
            .final_texts
            .join(" ")
            .replace("  ", " ")
            .trim()
            .to_string();
        let duration_ms = session.started_at.elapsed().as_millis() as u64;
        let state = if session.last_state == "failed" {
            "failed"
        } else {
            "finished"
        }
        .to_string();
        let status = BaiduRealtimeSessionStatus {
            state,
            message: "Baidu realtime WebSocket session stopped.".to_string(),
            started_at_ms: Some(session.started_at_ms),
            duration_ms,
            final_text: text.clone(),
        };
        Ok(BaiduRealtimeSessionSummary {
            status,
            char_count: text.chars().count(),
            text,
            duration_ms,
        })
    }
}

fn drain_worker_results(session: &mut ActiveRealtimeSession) {
    while let Ok(result) = session.result_rx.try_recv() {
        match result {
            WorkerResult::Event(event) => {
                if event.is_final && !event.text.trim().is_empty() {
                    session.final_texts.push(event.text.trim().to_string());
                }
            }
            WorkerResult::Finished => session.last_state = "finished".to_string(),
            WorkerResult::Failed(message) => {
                session.last_state = "failed".to_string();
                if !message.trim().is_empty() {
                    session.final_texts.push(String::new());
                }
            }
        }
    }
}

fn session_status(session: &ActiveRealtimeSession, message: &str) -> BaiduRealtimeSessionStatus {
    BaiduRealtimeSessionStatus {
        state: session.last_state.clone(),
        message: message.to_string(),
        started_at_ms: Some(session.started_at_ms),
        duration_ms: session.started_at.elapsed().as_millis() as u64,
        final_text: session.final_texts.join(" ").trim().to_string(),
    }
}

fn run_realtime_worker(
    app: AppHandle,
    config: BaiduRealtimeConfig,
    control_rx: mpsc::Receiver<RealtimeControl>,
    result_tx: mpsc::Sender<WorkerResult>,
    started_at_ms: u64,
) {
    if let Err(error) =
        run_realtime_worker_inner(&app, config, control_rx, &result_tx, started_at_ms)
    {
        let _ = result_tx.send(WorkerResult::Failed(error.to_string()));
    }
}

fn run_realtime_worker_inner(
    app: &AppHandle,
    config: BaiduRealtimeConfig,
    control_rx: mpsc::Receiver<RealtimeControl>,
    result_tx: &mpsc::Sender<WorkerResult>,
    started_at_ms: u64,
) -> Result<(), VoxError> {
    let (mut socket, _) = connect(config.endpoint.as_str()).map_err(|error| {
        VoxError::Asr(format!("Baidu realtime WebSocket connect failed: {error}"))
    })?;
    set_read_timeout(socket.get_mut(), Duration::from_millis(20));
    socket
        .send(Message::Text(config.start_frame().to_json()?))
        .map_err(|error| VoxError::Asr(format!("Baidu realtime START frame failed: {error}")))?;
    let mut sequence = 0u64;
    let mut source_cursor = 0usize;
    let mut pcm_buffer = PcmFrameBuffer::new();
    loop {
        match control_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(RealtimeControl::Finish) => {
                send_available_audio_frames(app, &mut socket, &mut source_cursor, &mut pcm_buffer)?;
                let _ = socket.send(Message::Text(
                    BaiduRealtimeControlFrame::finish().to_json()?,
                ));
                break;
            }
            Ok(RealtimeControl::Cancel) => {
                let _ = socket.send(Message::Text(
                    BaiduRealtimeControlFrame::cancel().to_json()?,
                ));
                let _ = socket.close(None);
                let _ = result_tx.send(WorkerResult::Finished);
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                send_available_audio_frames(app, &mut socket, &mut source_cursor, &mut pcm_buffer)?;
                let _ = socket.send(Message::Text(
                    BaiduRealtimeControlFrame::heartbeat().to_json()?,
                ));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        while let Ok(message) = socket.read() {
            if let Some(event) = worker_event_from_message(message, started_at_ms, sequence)? {
                sequence += 1;
                let _ = app.emit("voxtype-baidu-realtime-result", event.clone());
                let _ = result_tx.send(WorkerResult::Event(event));
            }
        }
    }
    let finish_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < finish_deadline {
        match socket.read() {
            Ok(message) => {
                if let Some(event) = worker_event_from_message(message, started_at_ms, sequence)? {
                    sequence += 1;
                    let _ = app.emit("voxtype-baidu-realtime-result", event.clone());
                    let is_final = event.is_final;
                    let _ = result_tx.send(WorkerResult::Event(event));
                    if is_final {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }
    let _ = socket.close(None);
    let _ = result_tx.send(WorkerResult::Finished);
    Ok(())
}

fn send_available_audio_frames(
    app: &AppHandle,
    socket: &mut WebSocket<MaybeTlsStream<std::net::TcpStream>>,
    source_cursor: &mut usize,
    pcm_buffer: &mut PcmFrameBuffer,
) -> Result<(), VoxError> {
    let recorder = app.state::<RecorderManager>();
    if let Ok((samples, end_cursor)) = recorder.active_asr_samples_from(*source_cursor) {
        *source_cursor = end_cursor;
        pcm_buffer.push_samples(&samples);
        for frame in pcm_buffer.drain_full_frames() {
            socket.send(Message::Binary(frame)).map_err(|error| {
                VoxError::Asr(format!("Baidu realtime audio frame failed: {error}"))
            })?;
        }
    }
    Ok(())
}

fn set_read_timeout(stream: &mut MaybeTlsStream<std::net::TcpStream>, timeout: Duration) {
    match stream {
        MaybeTlsStream::Plain(tcp) => {
            let _ = tcp.set_read_timeout(Some(timeout));
        }
        MaybeTlsStream::Rustls(tls) => {
            let _ = tls.sock.set_read_timeout(Some(timeout));
        }
        _ => {}
    }
}

fn worker_event_from_message(
    message: Message,
    started_at_ms: u64,
    sequence: u64,
) -> Result<Option<BaiduRealtimeResultEvent>, VoxError> {
    let body = match message {
        Message::Text(text) => text,
        Message::Binary(_) => return Ok(None),
        Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => {
            return Ok(None)
        }
    };
    match parse_baidu_realtime_message(&body)? {
        BaiduRealtimeEvent::Partial {
            text,
            sequence: server_sequence,
        } => Ok(Some(BaiduRealtimeResultEvent {
            text,
            is_final: false,
            sequence: server_sequence.unwrap_or(sequence),
            started_at_ms,
            duration_ms: unix_time_ms().saturating_sub(started_at_ms),
        })),
        BaiduRealtimeEvent::Final {
            text,
            sequence: server_sequence,
        } => Ok(Some(BaiduRealtimeResultEvent {
            text,
            is_final: true,
            sequence: server_sequence.unwrap_or(sequence),
            started_at_ms,
            duration_ms: unix_time_ms().saturating_sub(started_at_ms),
        })),
        BaiduRealtimeEvent::Error { code, message } => Err(VoxError::Asr(format!(
            "Baidu realtime error {code}: {message}"
        ))),
        BaiduRealtimeEvent::Started
        | BaiduRealtimeEvent::Finished
        | BaiduRealtimeEvent::Ignored => Ok(None),
    }
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_frame_serializes_official_shape_without_secret_logging_fields() {
        let frame = BaiduRealtimeStartFrame {
            appid: "10500017".to_string(),
            appkey: "test-api-key".to_string(),
            dev_pid: 15372,
            lm_id: Some("9001".to_string()),
            cuid: "voxtype-local".to_string(),
            user: Some("dialect-user".to_string()),
            format: "pcm".to_string(),
            sample: 16000,
        };

        let value: serde_json::Value = serde_json::from_str(&frame.to_json().unwrap()).unwrap();

        assert_eq!(value["type"], "START");
        assert_eq!(value["data"]["appid"], "10500017");
        assert_eq!(value["data"]["appkey"], "test-api-key");
        assert_eq!(value["data"]["dev_pid"], 15372);
        assert_eq!(value["data"]["lm_id"], "9001");
        assert_eq!(value["data"]["cuid"], "voxtype-local");
        assert_eq!(value["data"]["user"], "dialect-user");
        assert_eq!(value["data"]["format"], "pcm");
        assert_eq!(value["data"]["sample"], 16000);
    }

    #[test]
    fn control_frames_serialize_official_types() {
        assert_eq!(
            BaiduRealtimeControlFrame::finish().to_json().unwrap(),
            r#"{"type":"FINISH"}"#
        );
        assert_eq!(
            BaiduRealtimeControlFrame::cancel().to_json().unwrap(),
            r#"{"type":"CANCEL"}"#
        );
        assert_eq!(
            BaiduRealtimeControlFrame::heartbeat().to_json().unwrap(),
            r#"{"type":"HEARTBEAT"}"#
        );
    }

    #[test]
    fn pcm_frame_buffer_drains_only_complete_160ms_frames() {
        let mut buffer = PcmFrameBuffer::new();
        buffer.push_samples(&vec![1; 2559]);
        assert!(buffer.drain_full_frames().is_empty());

        buffer.push_samples(&[0x1234]);
        let frames = buffer.drain_full_frames();

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].len(), 5120);
        assert_eq!(&frames[0][5118..5120], &0x1234i16.to_le_bytes());
    }

    #[test]
    fn parser_accepts_partial_and_final_result_shapes() {
        assert_eq!(
            parse_baidu_realtime_message(
                r#"{"type":"MID_TEXT","result":"\u6b63\u5728\u8bc6\u522b"}"#
            )
            .unwrap(),
            BaiduRealtimeEvent::Partial {
                text: "\u{6b63}\u{5728}\u{8bc6}\u{522b}".to_string(),
                sequence: None
            }
        );
        assert_eq!(
            parse_baidu_realtime_message(
                r#"{"type":"FIN_TEXT","result":["\u6700\u7ec8\u6587\u672c"]}"#
            )
            .unwrap(),
            BaiduRealtimeEvent::Final {
                text: "\u{6700}\u{7ec8}\u{6587}\u{672c}".to_string(),
                sequence: None
            }
        );
    }

    #[test]
    fn parser_accepts_nested_data_result_and_errors() {
        assert_eq!(
            parse_baidu_realtime_message(
                r#"{"type":"FIN_TEXT","data":{"result":"\u5d4c\u5957\u6587\u672c"},"sn":7}"#
            )
            .unwrap(),
            BaiduRealtimeEvent::Final {
                text: "\u{5d4c}\u{5957}\u{6587}\u{672c}".to_string(),
                sequence: Some(7)
            }
        );
        assert_eq!(
            parse_baidu_realtime_message(
                r#"{"type":"ERROR","err_no":3301,"err_msg":"invalid appkey"}"#
            )
            .unwrap(),
            BaiduRealtimeEvent::Error {
                code: 3301,
                message: "invalid appkey".to_string()
            }
        );
    }

    #[test]
    fn build_realtime_config_requires_official_start_fields() {
        let config = BaiduRealtimeConfig {
            endpoint: "wss://vop.baidu.com/realtime_asr".to_string(),
            appid: "10500017".to_string(),
            appkey: "test-api-key".to_string(),
            dev_pid: 15372,
            lm_id: None,
            cuid: "voxtype-local".to_string(),
            user: None,
            format: "pcm".to_string(),
            sample: 16000,
        };

        let frame = config.start_frame();

        assert_eq!(frame.appid, "10500017");
        assert_eq!(frame.appkey, "test-api-key");
        assert_eq!(frame.dev_pid, 15372);
    }

    fn cloud_status_for_realtime(
        secret_key_configured: bool,
    ) -> crate::cloud_asr_config::CloudAsrConfigStatus {
        crate::cloud_asr_config::CloudAsrConfigStatus {
            config: crate::cloud_asr_config::CloudAsrConfig {
                provider: "baidu".to_string(),
                group_id: None,
                base_url: Some("http://vop.baidu.com/server_api".to_string()),
                model: Some("1537".to_string()),
                language: Some("zh".to_string()),
                baidu_cuid: Some("voxtype-local".to_string()),
                baidu_format: Some("pcm".to_string()),
                baidu_sample_rate: Some(16000),
                baidu_lm_id: None,
                baidu_realtime_app_id: Some("10500017".to_string()),
                baidu_realtime_endpoint: Some("wss://vop.baidu.com/realtime_asr".to_string()),
                baidu_realtime_dev_pid: Some("15372".to_string()),
                baidu_realtime_cuid: Some("voxtype-local".to_string()),
                baidu_realtime_format: Some("pcm".to_string()),
                baidu_realtime_sample_rate: Some(16000),
                baidu_realtime_user: None,
            },
            api_key_configured: true,
            api_key_source: "env:BAIDU_ASR_API_KEY".to_string(),
            api_key_preview: Some("ba***ey".to_string()),
            secret_key_configured,
            secret_key_source: if secret_key_configured {
                "env:BAIDU_ASR_SECRET_KEY"
            } else {
                "missing"
            }
            .to_string(),
            secret_key_preview: None,
            ready: secret_key_configured,
            message: "short speech status".to_string(),
        }
    }

    #[test]
    fn realtime_config_uses_api_key_and_does_not_require_secret_key() {
        let config = config_from_cloud_status(
            &cloud_status_for_realtime(false),
            "baidu-api-key".to_string(),
        )
        .unwrap();

        assert_eq!(config.endpoint, "wss://vop.baidu.com/realtime_asr");
        assert_eq!(config.appid, "10500017");
        assert_eq!(config.appkey, "baidu-api-key");
        assert_eq!(config.dev_pid, 15372);
        assert_eq!(config.cuid, "voxtype-local");
        assert_eq!(config.format, "pcm");
        assert_eq!(config.sample, 16000);
    }

    #[test]
    fn realtime_config_requires_app_id_without_leaking_key() {
        let mut status = cloud_status_for_realtime(true);
        status.config.baidu_realtime_app_id = None;

        let error = config_from_cloud_status(&status, "baidu-api-key".to_string())
            .unwrap_err()
            .to_string();

        assert!(error.contains("AppID"));
        assert!(!error.contains("baidu-api-key"));
    }
}
