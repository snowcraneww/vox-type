use crate::{asr::Transcript, error::VoxError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaiduAsrRequest {
    pub endpoint: String,
    pub access_token: String,
    pub dev_pid: i32,
    pub cuid: String,
    pub format: String,
    pub sample_rate: u32,
    pub speech_base64: String,
    pub byte_len: usize,
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
struct BaiduAsrJsonPayload {
    format: String,
    rate: u32,
    channel: u8,
    cuid: String,
    token: String,
    dev_pid: i32,
    speech: String,
    len: usize,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
struct BaiduAccessTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiniMaxAsrRequest {
    pub base_url: String,
    pub group_id: String,
    pub model: String,
    pub language: String,
}

impl MiniMaxAsrRequest {
    pub fn endpoint_url(&self) -> String {
        let base_url = self.base_url.trim_end_matches('/');
        format!(
            "{base_url}/v1/audio/transcriptions?GroupId={}",
            self.group_id
        )
    }
}

pub fn build_minimax_asr_request(
    base_url: &str,
    group_id: &str,
    model: &str,
    language: &str,
) -> Result<MiniMaxAsrRequest, VoxError> {
    let request = MiniMaxAsrRequest {
        base_url: required_field(base_url, "MiniMax Base URL")?,
        group_id: required_field(group_id, "MiniMax Group ID")?,
        model: required_field(model, "MiniMax 模型")?,
        language: required_field(language, "识别语言")?,
    };
    Ok(request)
}

pub fn transcribe_with_minimax_placeholder(
    _pcm_16khz_mono: &[i16],
) -> Result<Transcript, VoxError> {
    Err(VoxError::Asr(
        "MiniMax 云端识别尚未启用：未找到官方 ASR endpoint、上传字段和返回文本字段，V6 仅保存配置。".to_string(),
    ))
}

pub fn build_baidu_asr_request(
    endpoint: &str,
    access_token: &str,
    dev_pid: &str,
    cuid: &str,
    format: &str,
    sample_rate: u32,
    pcm_16khz_mono: &[i16],
) -> Result<BaiduAsrRequest, VoxError> {
    if pcm_16khz_mono.is_empty() {
        return Err(VoxError::Asr("百度 ASR 音频为空。".to_string()));
    }
    let dev_pid = required_field(dev_pid, "百度 ASR dev_pid")?
        .parse::<i32>()
        .map_err(|_| VoxError::Config("百度 ASR dev_pid 必须是数字。".to_string()))?;
    let bytes = pcm_i16_le_bytes(pcm_16khz_mono);
    let endpoint = normalize_baidu_endpoint(&required_field(endpoint, "百度 ASR Endpoint")?);
    Ok(BaiduAsrRequest {
        endpoint,
        access_token: required_field(access_token, "百度 ASR API Key")?,
        dev_pid,
        cuid: required_field(cuid, "百度 ASR cuid")?,
        format: required_field(format, "百度 ASR 音频格式")?,
        sample_rate,
        speech_base64: base64_encode(&bytes),
        byte_len: bytes.len(),
    })
}

fn normalize_baidu_endpoint(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

pub fn transcribe_with_baidu_short_speech(request: BaiduAsrRequest) -> Result<Transcript, VoxError> {
    let payload = BaiduAsrJsonPayload::from(request.clone());
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|error| VoxError::Asr(format!("创建百度 ASR HTTP client 失败：{error}")))?;
    let body = serde_json::to_string(&payload)
        .map_err(|error| VoxError::Asr(format!("序列化百度 ASR 请求失败：{error}")))?;
    let response = client
        .post(&request.endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .map_err(|error| VoxError::Asr(format!("调用百度 ASR 失败：{error}")))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| VoxError::Asr(format!("读取百度 ASR 响应失败：{error}")))?;
    if !status.is_success() {
        return Err(VoxError::Asr(format!("百度 ASR HTTP {status}：{}", body.trim())));
    }
    parse_baidu_asr_response(&body)
}

impl From<BaiduAsrRequest> for BaiduAsrJsonPayload {
    fn from(request: BaiduAsrRequest) -> Self {
        Self {
            format: request.format,
            rate: request.sample_rate,
            channel: 1,
            cuid: request.cuid,
            token: request.access_token,
            dev_pid: request.dev_pid,
            speech: request.speech_base64,
            len: request.byte_len,
        }
    }
}

pub fn fetch_baidu_access_token(api_key: &str, secret_key: &str) -> Result<String, VoxError> {
    let api_key = required_field(api_key, "Baidu ASR API Key")?;
    let secret_key = required_field(secret_key, "Baidu ASR Secret Key")?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| VoxError::Asr(format!("创建百度 OAuth HTTP client 失败：{error}")))?;
    let response = client
        .post("https://aip.baidubce.com/oauth/2.0/token")
        .query(&[
            ("grant_type", "client_credentials"),
            ("client_id", api_key.as_str()),
            ("client_secret", secret_key.as_str()),
        ])
        .send()
        .map_err(|error| VoxError::Asr(format!("获取百度 access_token 失败：{error}")))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| VoxError::Asr(format!("读取百度 OAuth 响应失败：{error}")))?;
    if !status.is_success() {
        return Err(VoxError::Asr(format!("百度 OAuth HTTP {status}：{}", body.trim())));
    }
    parse_baidu_access_token_response(&body)
}

pub fn parse_baidu_access_token_response(body: &str) -> Result<String, VoxError> {
    let response: BaiduAccessTokenResponse = serde_json::from_str(body)
        .map_err(|error| VoxError::Asr(format!("解析百度 OAuth 响应失败：{error}")))?;
    if let Some(token) = response
        .access_token
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(token);
    }
    let detail = response
        .error_description
        .or(response.error)
        .unwrap_or_else(|| "未返回 access_token".to_string());
    Err(VoxError::Asr(format!("百度 OAuth 返回错误：{detail}")))
}

pub fn parse_baidu_asr_response(body: &str) -> Result<Transcript, VoxError> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| VoxError::Asr(format!("解析百度 ASR 响应失败：{error}")))?;
    let err_no = value.get("err_no").and_then(serde_json::Value::as_i64).unwrap_or(-1);
    if err_no != 0 {
        let err_msg = value.get("err_msg").and_then(serde_json::Value::as_str).unwrap_or("未知错误");
        return Err(VoxError::Asr(format!("百度 ASR 返回错误 {err_no}：{err_msg}")));
    }
    let text = value
        .get("result")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.first())
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| VoxError::Asr("百度 ASR 没有返回识别文本。".to_string()))?;
    Ok(Transcript { text: text.to_string(), engine: "baidu-short-speech".to_string() })
}

fn pcm_i16_le_bytes(samples: &[i16]) -> Vec<u8> {
    samples.iter().flat_map(|sample| sample.to_le_bytes()).collect()
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(b0 >> 2) as usize] as char);
        encoded.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn required_field(value: &str, label: &str) -> Result<String, VoxError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(VoxError::Config(format!("{label} 未配置。")));
    }
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_endpoint_does_not_include_api_key() {
        let request =
            build_minimax_asr_request("https://api.minimax.io/", "group-1", "speech-to-text", "zh")
                .unwrap();

        assert_eq!(
            request.endpoint_url(),
            "https://api.minimax.io/v1/audio/transcriptions?GroupId=group-1"
        );
        assert!(!request.endpoint_url().contains("secret"));
    }

    #[test]
    fn placeholder_refuses_to_claim_real_cloud_transcription() {
        let error = transcribe_with_minimax_placeholder(&[0, 1, -1]).unwrap_err();

        assert!(error.to_string().contains("MiniMax 云端识别尚未启用"));
        assert!(error.to_string().contains("官方 ASR endpoint"));
    }

    #[test]
    fn builds_baidu_json_request_without_exposing_key_in_url() {
        let request = build_baidu_asr_request(
            "http://vop.baidu.com/server_api",
            "baidu-access-token",
            "1537",
            "voxtype-local",
            "pcm",
            16_000,
            &[1, -2],
        ).unwrap();

        assert_eq!(request.endpoint, "http://vop.baidu.com/server_api");
        assert!(!request.endpoint.contains("baidu-access-token"));
        assert_eq!(request.dev_pid, 1537);
        assert_eq!(request.byte_len, 4);
        assert_eq!(request.speech_base64, "AQD+/w==");
    }

    #[test]
    fn baidu_json_payload_uses_access_token_not_ak_or_sk() {
        let request = build_baidu_asr_request(
            "http://vop.baidu.com/server_api",
            "access-token-from-oauth",
            "1537",
            "voxtype-local",
            "pcm",
            16_000,
            &[1, -2],
        ).unwrap();

        assert_eq!(request.access_token, "access-token-from-oauth");
        let payload = BaiduAsrJsonPayload::from(request);
        let body = serde_json::to_string(&payload).unwrap();

        assert!(body.contains("access-token-from-oauth"));
        assert!(!body.contains("baidu-api-key"));
        assert!(!body.contains("baidu-secret-key"));
    }

    #[test]
    fn parses_baidu_success_response() {
        let transcript = parse_baidu_asr_response(r#"{"err_no":0,"err_msg":"success.","result":["测试文本"]}"#).unwrap();

        assert_eq!(transcript.engine, "baidu-short-speech");
        assert_eq!(transcript.text, "测试文本");
    }

    #[test]
    fn surfaces_baidu_error_response() {
        let error = parse_baidu_asr_response(r#"{"err_no":3302,"err_msg":"authentication failed."}"#).unwrap_err();

        assert!(error.to_string().contains("3302"));
        assert!(error.to_string().contains("authentication failed"));
    }

    #[test]
    fn parses_baidu_access_token_response() {
        let token = parse_baidu_access_token_response(r#"{"access_token":"oauth-token","expires_in":2592000}"#).unwrap();

        assert_eq!(token, "oauth-token");
    }

    #[test]
    fn surfaces_baidu_access_token_error_without_secrets() {
        let error = parse_baidu_access_token_response(r#"{"error":"invalid_client","error_description":"bad credentials"}"#).unwrap_err();

        assert!(error.to_string().contains("bad credentials"));
        assert!(!error.to_string().contains("baidu-api-key"));
        assert!(!error.to_string().contains("baidu-secret-key"));
    }
}
