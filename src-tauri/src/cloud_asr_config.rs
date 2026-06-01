use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::error::VoxError;

const CONFIG_FILE_NAME: &str = "cloud-asr-config.json";
pub const ENV_MINIMAX_API_KEY: &str = "MINIMAX_API_KEY";
pub const ENV_BAIDU_ASR_API_KEY: &str = "BAIDU_ASR_API_KEY";
pub const ENV_BAIDU_ASR_SECRET_KEY: &str = "BAIDU_ASR_SECRET_KEY";
const MINIMAX_PROVIDER: &str = "minimax";
const BAIDU_PROVIDER: &str = "baidu";
const DEFAULT_MINIMAX_BASE_URL: &str = "https://api.minimax.io";
const DEFAULT_MINIMAX_MODEL: &str = "speech-to-text";
const DEFAULT_LANGUAGE: &str = "zh";
const DEFAULT_BAIDU_ENDPOINT: &str = "http://vop.baidu.com/server_api";
const DEFAULT_BAIDU_DEV_PID: &str = "1537";
const DEFAULT_BAIDU_CUID: &str = "voxtype-local";
const DEFAULT_BAIDU_FORMAT: &str = "pcm";
const DEFAULT_BAIDU_SAMPLE_RATE: u32 = 16000;
const DEFAULT_BAIDU_REALTIME_ENDPOINT: &str = "wss://vop.baidu.com/realtime_asr";
const DEFAULT_BAIDU_REALTIME_DEV_PID: &str = "15372";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CloudAsrConfig {
    pub provider: String,
    pub group_id: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub language: Option<String>,
    pub baidu_cuid: Option<String>,
    pub baidu_format: Option<String>,
    pub baidu_sample_rate: Option<u32>,
    pub baidu_lm_id: Option<String>,
    pub baidu_realtime_endpoint: Option<String>,
    pub baidu_realtime_dev_pid: Option<String>,
    pub baidu_realtime_cuid: Option<String>,
    pub baidu_realtime_format: Option<String>,
    pub baidu_realtime_sample_rate: Option<u32>,
    pub baidu_realtime_user: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderApiKeys {
    pub minimax_api_key: Option<String>,
    pub baidu_asr_api_key: Option<String>,
    pub baidu_asr_secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CloudAsrConfigStatus {
    pub config: CloudAsrConfig,
    pub api_key_configured: bool,
    pub api_key_source: String,
    pub api_key_preview: Option<String>,
    pub secret_key_configured: bool,
    pub secret_key_source: String,
    pub secret_key_preview: Option<String>,
    pub ready: bool,
    pub message: String,
}

impl Default for CloudAsrConfig {
    fn default() -> Self {
        normalize_config(CloudAsrConfig {
            provider: BAIDU_PROVIDER.to_string(),
            group_id: None,
            base_url: None,
            model: None,
            language: None,
            baidu_cuid: None,
            baidu_format: None,
            baidu_sample_rate: None,
            baidu_lm_id: None,
            baidu_realtime_endpoint: None,
            baidu_realtime_dev_pid: None,
            baidu_realtime_cuid: None,
            baidu_realtime_format: None,
            baidu_realtime_sample_rate: None,
            baidu_realtime_user: None,
        })
    }
}

pub fn get_cloud_asr_config_status(config_dir: PathBuf) -> CloudAsrConfigStatus {
    let config = load_cloud_asr_config(config_dir);
    status_from_config(config, provider_api_keys_from_env())
}

pub fn save_cloud_asr_config(
    config_dir: PathBuf,
    config: CloudAsrConfig,
) -> Result<CloudAsrConfigStatus, VoxError> {
    save_cloud_asr_config_with_provider_keys(config_dir, config, provider_api_keys_from_env())
}

pub fn save_minimax_api_key_to_user_env(
    config_dir: PathBuf,
    api_key: String,
) -> Result<CloudAsrConfigStatus, VoxError> {
    let normalized_key = api_key.trim().to_string();
    if normalized_key.is_empty() {
        return Err(VoxError::Config("MiniMax API Key 不能为空。".to_string()));
    }
    persist_user_environment_api_key(ENV_MINIMAX_API_KEY, &normalized_key)?;
    std::env::set_var(ENV_MINIMAX_API_KEY, &normalized_key);
    let config = load_cloud_asr_config(config_dir);
    let keys = ProviderApiKeys {
        minimax_api_key: Some(normalized_key),
        baidu_asr_api_key: baidu_asr_api_key_from_env(),
        baidu_asr_secret_key: baidu_asr_secret_key_from_env(),
    };
    Ok(status_from_config(config, keys))
}

pub fn save_baidu_asr_api_key_to_user_env(
    config_dir: PathBuf,
    api_key: String,
) -> Result<CloudAsrConfigStatus, VoxError> {
    let normalized_key = api_key.trim().to_string();
    if normalized_key.is_empty() {
        return Err(VoxError::Config("百度 ASR API Key 不能为空。".to_string()));
    }
    persist_user_environment_api_key(ENV_BAIDU_ASR_API_KEY, &normalized_key)?;
    std::env::set_var(ENV_BAIDU_ASR_API_KEY, &normalized_key);
    let existing_config = load_cloud_asr_config(config_dir.clone());
    let config = baidu_config_from_existing_config(existing_config);
    save_cloud_asr_config_with_provider_keys(
        config_dir,
        config,
        ProviderApiKeys {
            minimax_api_key: minimax_api_key_from_env(),
            baidu_asr_api_key: Some(normalized_key),
            baidu_asr_secret_key: baidu_asr_secret_key_from_env(),
        },
    )
}

pub fn save_baidu_asr_secret_key_to_user_env(
    config_dir: PathBuf,
    secret_key: String,
) -> Result<CloudAsrConfigStatus, VoxError> {
    let normalized_key = secret_key.trim().to_string();
    if normalized_key.is_empty() {
        return Err(VoxError::Config(
            "百度 ASR Secret Key 不能为空。".to_string(),
        ));
    }
    persist_user_environment_api_key(ENV_BAIDU_ASR_SECRET_KEY, &normalized_key)?;
    std::env::set_var(ENV_BAIDU_ASR_SECRET_KEY, &normalized_key);
    let existing_config = load_cloud_asr_config(config_dir.clone());
    let config = baidu_config_from_existing_config(existing_config);
    save_cloud_asr_config_with_provider_keys(
        config_dir,
        config,
        ProviderApiKeys {
            minimax_api_key: minimax_api_key_from_env(),
            baidu_asr_api_key: baidu_asr_api_key_from_env(),
            baidu_asr_secret_key: Some(normalized_key),
        },
    )
}

pub fn save_cloud_asr_config_with_api_key(
    config_dir: PathBuf,
    config: CloudAsrConfig,
    api_key: Option<String>,
) -> Result<CloudAsrConfigStatus, VoxError> {
    save_cloud_asr_config_with_provider_keys(
        config_dir,
        config,
        ProviderApiKeys {
            minimax_api_key: api_key,
            baidu_asr_api_key: None,
            baidu_asr_secret_key: None,
        },
    )
}

pub fn save_cloud_asr_config_with_provider_keys(
    config_dir: PathBuf,
    config: CloudAsrConfig,
    keys: ProviderApiKeys,
) -> Result<CloudAsrConfigStatus, VoxError> {
    let normalized = normalize_config(config);
    fs::create_dir_all(&config_dir)
        .map_err(|error| VoxError::Config(format!("创建云端 ASR 配置目录失败：{error}")))?;
    let payload = serde_json::to_string_pretty(&normalized)
        .map_err(|error| VoxError::Config(format!("序列化云端 ASR 配置失败：{error}")))?;
    fs::write(config_dir.join(CONFIG_FILE_NAME), payload)
        .map_err(|error| VoxError::Config(format!("写入云端 ASR 配置失败：{error}")))?;
    Ok(status_from_config(normalized, keys))
}

pub fn load_cloud_asr_config(config_dir: PathBuf) -> CloudAsrConfig {
    let path = config_dir.join(CONFIG_FILE_NAME);
    fs::read_to_string(path)
        .ok()
        .and_then(|payload| serde_json::from_str::<CloudAsrConfig>(&payload).ok())
        .map(normalize_config)
        .unwrap_or_default()
}

pub fn minimax_api_key_from_env() -> Option<String> {
    api_key_from_env(ENV_MINIMAX_API_KEY)
}

pub fn baidu_asr_api_key_from_env() -> Option<String> {
    api_key_from_env(ENV_BAIDU_ASR_API_KEY)
}

pub fn baidu_asr_secret_key_from_env() -> Option<String> {
    api_key_from_env(ENV_BAIDU_ASR_SECRET_KEY)
}

fn provider_api_keys_from_env() -> ProviderApiKeys {
    ProviderApiKeys {
        minimax_api_key: minimax_api_key_from_env(),
        baidu_asr_api_key: baidu_asr_api_key_from_env(),
        baidu_asr_secret_key: baidu_asr_secret_key_from_env(),
    }
}

fn api_key_from_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| api_key_from_user_environment_registry(name))
}

fn status_from_config(config: CloudAsrConfig, keys: ProviderApiKeys) -> CloudAsrConfigStatus {
    if config.provider.eq_ignore_ascii_case(BAIDU_PROVIDER) {
        return baidu_status_from_config(config, keys.baidu_asr_api_key, keys.baidu_asr_secret_key);
    }
    minimax_status_from_config(config, keys.minimax_api_key)
}

fn baidu_config_from_existing_config(existing_config: CloudAsrConfig) -> CloudAsrConfig {
    if existing_config
        .provider
        .eq_ignore_ascii_case(BAIDU_PROVIDER)
    {
        return existing_config;
    }
    normalize_config(CloudAsrConfig {
        provider: BAIDU_PROVIDER.to_string(),
        group_id: None,
        base_url: None,
        model: None,
        language: existing_config.language,
        baidu_cuid: None,
        baidu_format: None,
        baidu_sample_rate: None,
        baidu_lm_id: existing_config.baidu_lm_id,
        baidu_realtime_endpoint: existing_config.baidu_realtime_endpoint,
        baidu_realtime_dev_pid: existing_config.baidu_realtime_dev_pid,
        baidu_realtime_cuid: existing_config.baidu_realtime_cuid,
        baidu_realtime_format: existing_config.baidu_realtime_format,
        baidu_realtime_sample_rate: existing_config.baidu_realtime_sample_rate,
        baidu_realtime_user: existing_config.baidu_realtime_user,
    })
}

fn minimax_status_from_config(
    config: CloudAsrConfig,
    api_key: Option<String>,
) -> CloudAsrConfigStatus {
    let api_key_preview = api_key.as_deref().map(mask_api_key);
    let api_key_configured = api_key.is_some();
    let has_provider = config.provider.eq_ignore_ascii_case(MINIMAX_PROVIDER);
    let has_base_url = has_value(config.base_url.as_ref());
    let has_model = has_value(config.model.as_ref());
    let has_group_id = has_value(config.group_id.as_ref());
    let ready = api_key_configured && has_provider && has_base_url && has_model && has_group_id;
    let message = if ready {
        "MiniMax 云端识别配置完整。".to_string()
    } else if !api_key_configured {
        "未读取到 MINIMAX_API_KEY 环境变量。".to_string()
    } else if !has_group_id {
        "MiniMax Group ID 未配置。".to_string()
    } else if !has_base_url || !has_model {
        "MiniMax Base URL 或模型未配置。".to_string()
    } else {
        "仅支持 MiniMax 或百度短语音识别 provider。".to_string()
    };

    CloudAsrConfigStatus {
        config,
        api_key_configured,
        api_key_source: if api_key_configured {
            "env:MINIMAX_API_KEY"
        } else {
            "missing"
        }
        .to_string(),
        api_key_preview,
        secret_key_configured: true,
        secret_key_source: "not-required".to_string(),
        secret_key_preview: None,
        ready,
        message,
    }
}

fn baidu_status_from_config(
    config: CloudAsrConfig,
    api_key: Option<String>,
    secret_key: Option<String>,
) -> CloudAsrConfigStatus {
    let api_key_preview = api_key.as_deref().map(mask_api_key);
    let api_key_configured = api_key.is_some();
    let secret_key_preview = secret_key.as_deref().map(mask_api_key);
    let secret_key_configured = secret_key.is_some();
    let has_base_url = has_value(config.base_url.as_ref());
    let has_supported_endpoint = config
        .base_url
        .as_ref()
        .is_some_and(|value| value.trim().trim_end_matches('/').ends_with("/server_api"));
    let has_model = has_value(config.model.as_ref());
    let has_cuid = has_value(config.baidu_cuid.as_ref());
    let has_format = has_value(config.baidu_format.as_ref());
    let has_sample_rate = config.baidu_sample_rate.is_some_and(|rate| rate > 0);
    let ready = api_key_configured
        && secret_key_configured
        && has_base_url
        && has_supported_endpoint
        && has_model
        && has_cuid
        && has_format
        && has_sample_rate;
    let message = if ready {
        "百度短语音识别配置完整。".to_string()
    } else if !api_key_configured {
        "未读取到 BAIDU_ASR_API_KEY 环境变量。".to_string()
    } else if !secret_key_configured {
        "Missing BAIDU_ASR_SECRET_KEY environment variable.".to_string()
    } else if !has_base_url || !has_model {
        "百度 ASR Endpoint 或 dev_pid 未配置。".to_string()
    } else if !has_supported_endpoint {
        "百度 ASR Endpoint 应为短语音识别 server_api 地址。".to_string()
    } else if !has_cuid {
        "百度 ASR cuid 未配置。".to_string()
    } else if !has_format || !has_sample_rate {
        "百度 ASR 音频格式或采样率未配置。".to_string()
    } else {
        "百度短语音识别配置未就绪。".to_string()
    };

    CloudAsrConfigStatus {
        config,
        api_key_configured,
        api_key_source: if api_key_configured {
            "env:BAIDU_ASR_API_KEY"
        } else {
            "missing"
        }
        .to_string(),
        api_key_preview,
        secret_key_configured,
        secret_key_source: if secret_key_configured {
            "env:BAIDU_ASR_SECRET_KEY"
        } else {
            "missing"
        }
        .to_string(),
        secret_key_preview,
        ready,
        message,
    }
}

fn normalize_config(config: CloudAsrConfig) -> CloudAsrConfig {
    let provider = trim_or_default(Some(config.provider), BAIDU_PROVIDER).to_lowercase();
    if provider == BAIDU_PROVIDER {
        return CloudAsrConfig {
            provider,
            group_id: trim_optional(config.group_id),
            base_url: trim_optional(config.base_url)
                .or_else(|| Some(DEFAULT_BAIDU_ENDPOINT.to_string())),
            model: trim_optional(config.model).or_else(|| Some(DEFAULT_BAIDU_DEV_PID.to_string())),
            language: trim_optional(config.language).or_else(|| Some(DEFAULT_LANGUAGE.to_string())),
            baidu_cuid: trim_optional(config.baidu_cuid)
                .or_else(|| Some(DEFAULT_BAIDU_CUID.to_string())),
            baidu_format: trim_optional(config.baidu_format)
                .or_else(|| Some(DEFAULT_BAIDU_FORMAT.to_string())),
            baidu_sample_rate: config.baidu_sample_rate.or(Some(DEFAULT_BAIDU_SAMPLE_RATE)),
            baidu_lm_id: trim_optional(config.baidu_lm_id),
            baidu_realtime_endpoint: trim_optional(config.baidu_realtime_endpoint)
                .or_else(|| Some(DEFAULT_BAIDU_REALTIME_ENDPOINT.to_string())),
            baidu_realtime_dev_pid: trim_optional(config.baidu_realtime_dev_pid)
                .or_else(|| Some(DEFAULT_BAIDU_REALTIME_DEV_PID.to_string())),
            baidu_realtime_cuid: trim_optional(config.baidu_realtime_cuid)
                .or_else(|| Some(DEFAULT_BAIDU_CUID.to_string())),
            baidu_realtime_format: trim_optional(config.baidu_realtime_format)
                .or_else(|| Some(DEFAULT_BAIDU_FORMAT.to_string())),
            baidu_realtime_sample_rate: config
                .baidu_realtime_sample_rate
                .or(Some(DEFAULT_BAIDU_SAMPLE_RATE)),
            baidu_realtime_user: trim_optional(config.baidu_realtime_user),
        };
    }

    CloudAsrConfig {
        provider: MINIMAX_PROVIDER.to_string(),
        group_id: trim_optional(config.group_id),
        base_url: trim_optional(config.base_url)
            .or_else(|| Some(DEFAULT_MINIMAX_BASE_URL.to_string())),
        model: trim_optional(config.model).or_else(|| Some(DEFAULT_MINIMAX_MODEL.to_string())),
        language: trim_optional(config.language).or_else(|| Some(DEFAULT_LANGUAGE.to_string())),
        baidu_cuid: trim_optional(config.baidu_cuid),
        baidu_format: trim_optional(config.baidu_format),
        baidu_sample_rate: config.baidu_sample_rate,
        baidu_lm_id: trim_optional(config.baidu_lm_id),
        baidu_realtime_endpoint: trim_optional(config.baidu_realtime_endpoint),
        baidu_realtime_dev_pid: trim_optional(config.baidu_realtime_dev_pid),
        baidu_realtime_cuid: trim_optional(config.baidu_realtime_cuid),
        baidu_realtime_format: trim_optional(config.baidu_realtime_format),
        baidu_realtime_sample_rate: config.baidu_realtime_sample_rate,
        baidu_realtime_user: trim_optional(config.baidu_realtime_user),
    }
}

fn has_value(value: Option<&String>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn trim_or_default(value: Option<String>, fallback: &str) -> String {
    trim_optional(value).unwrap_or_else(|| fallback.to_string())
}

fn mask_api_key(api_key: &str) -> String {
    let chars: Vec<char> = api_key.chars().collect();
    if chars.len() <= 4 {
        return "****".to_string();
    }
    let prefix: String = chars.iter().take(2).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}***{suffix}")
}

#[cfg(windows)]
fn persist_user_environment_api_key(name: &str, api_key: &str) -> Result<(), VoxError> {
    write_user_environment_variable(name, api_key)
}

#[cfg(windows)]
fn write_user_environment_variable(name: &str, value: &str) -> Result<(), VoxError> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, ptr};

    type Hkey = *mut c_void;
    const ERROR_SUCCESS: i32 = 0;
    const HKEY_CURRENT_USER: Hkey = 0x80000001usize as Hkey;
    const KEY_SET_VALUE: u32 = 0x0002;
    const REG_OPTION_NON_VOLATILE: u32 = 0;
    const REG_SZ: u32 = 1;
    const HWND_BROADCAST: *mut c_void = 0xffffusize as *mut c_void;
    const WM_SETTINGCHANGE: u32 = 0x001A;
    const SMTO_ABORTIFHUNG: u32 = 0x0002;

    extern "system" {
        fn RegCreateKeyExW(
            h_key: Hkey,
            lp_sub_key: *const u16,
            reserved: u32,
            lp_class: *mut u16,
            dw_options: u32,
            sam_desired: u32,
            lp_security_attributes: *mut c_void,
            phk_result: *mut Hkey,
            lpdw_disposition: *mut u32,
        ) -> i32;
        fn RegSetValueExW(
            h_key: Hkey,
            lp_value_name: *const u16,
            reserved: u32,
            dw_type: u32,
            lp_data: *const u8,
            cb_data: u32,
        ) -> i32;
        fn RegCloseKey(h_key: Hkey) -> i32;
        fn SendMessageTimeoutW(
            h_wnd: *mut c_void,
            msg: u32,
            w_param: usize,
            l_param: isize,
            fu_flags: u32,
            u_timeout: u32,
            lpdw_result: *mut usize,
        ) -> usize;
    }

    fn wide(value: &str) -> Vec<u16> {
        std::ffi::OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let sub_key = wide("Environment");
    let value_name = wide(name);
    let value_data = wide(value);
    let mut key: Hkey = ptr::null_mut();
    let create_result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            0,
            ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            ptr::null_mut(),
            &mut key,
            ptr::null_mut(),
        )
    };
    if create_result != ERROR_SUCCESS {
        return Err(VoxError::Config(format!(
            "打开用户环境变量注册表失败：Windows error {create_result}"
        )));
    }

    let set_result = unsafe {
        RegSetValueExW(
            key,
            value_name.as_ptr(),
            0,
            REG_SZ,
            value_data.as_ptr().cast::<u8>(),
            (value_data.len() * std::mem::size_of::<u16>()) as u32,
        )
    };
    let _ = unsafe { RegCloseKey(key) };
    if set_result != ERROR_SUCCESS {
        return Err(VoxError::Config(format!(
            "写入用户环境变量失败：Windows error {set_result}"
        )));
    }

    let environment = wide("Environment");
    let _ = unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            environment.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            ptr::null_mut(),
        )
    };
    Ok(())
}

#[cfg(windows)]
fn api_key_from_user_environment_registry(name: &str) -> Option<String> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, ptr};

    type Hkey = *mut c_void;
    const ERROR_SUCCESS: i32 = 0;
    const ERROR_MORE_DATA: i32 = 234;
    const HKEY_CURRENT_USER: Hkey = 0x80000001usize as Hkey;
    const KEY_QUERY_VALUE: u32 = 0x0001;
    const REG_SZ: u32 = 1;
    const REG_EXPAND_SZ: u32 = 2;

    extern "system" {
        fn RegOpenKeyExW(
            h_key: Hkey,
            lp_sub_key: *const u16,
            ul_options: u32,
            sam_desired: u32,
            phk_result: *mut Hkey,
        ) -> i32;
        fn RegQueryValueExW(
            h_key: Hkey,
            lp_value_name: *const u16,
            lp_reserved: *mut u32,
            lp_type: *mut u32,
            lp_data: *mut u8,
            lpcb_data: *mut u32,
        ) -> i32;
        fn RegCloseKey(h_key: Hkey) -> i32;
    }

    fn wide(value: &str) -> Vec<u16> {
        std::ffi::OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let sub_key = wide("Environment");
    let value_name = wide(name);
    let mut key: Hkey = ptr::null_mut();
    let open_result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        )
    };
    if open_result != ERROR_SUCCESS {
        return None;
    }

    let mut value_type = 0u32;
    let mut byte_len = 0u32;
    let size_result = unsafe {
        RegQueryValueExW(
            key,
            value_name.as_ptr(),
            ptr::null_mut(),
            &mut value_type,
            ptr::null_mut(),
            &mut byte_len,
        )
    };
    if size_result != ERROR_SUCCESS && size_result != ERROR_MORE_DATA {
        let _ = unsafe { RegCloseKey(key) };
        return None;
    }
    if value_type != REG_SZ && value_type != REG_EXPAND_SZ || byte_len < 2 {
        let _ = unsafe { RegCloseKey(key) };
        return None;
    }

    let mut buffer = vec![0u16; (byte_len as usize + 1) / 2];
    let query_result = unsafe {
        RegQueryValueExW(
            key,
            value_name.as_ptr(),
            ptr::null_mut(),
            &mut value_type,
            buffer.as_mut_ptr().cast::<u8>(),
            &mut byte_len,
        )
    };
    let _ = unsafe { RegCloseKey(key) };
    if query_result != ERROR_SUCCESS {
        return None;
    }
    let char_len = (byte_len as usize / 2).saturating_sub(1);
    String::from_utf16(&buffer[..char_len])
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(not(windows))]
fn api_key_from_user_environment_registry(_name: &str) -> Option<String> {
    None
}

#[cfg(not(windows))]
fn persist_user_environment_api_key(name: &str, api_key: &str) -> Result<(), VoxError> {
    std::env::set_var(name, api_key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_status_without_exposing_api_key() {
        let dir = tempfile::tempdir().unwrap();
        let status = save_cloud_asr_config_with_api_key(
            dir.path().to_path_buf(),
            minimax_config(),
            Some("secret-key".to_string()),
        )
        .unwrap();

        assert!(status.ready);
        assert!(status.api_key_configured);
        assert_eq!(status.api_key_source, "env:MINIMAX_API_KEY");
        assert_eq!(status.api_key_preview, Some("se***ey".to_string()));
        let payload = fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap();
        assert!(!payload.contains("secret-key"));
    }

    fn minimax_config() -> CloudAsrConfig {
        CloudAsrConfig {
            provider: "minimax".to_string(),
            group_id: Some("group-1".to_string()),
            base_url: Some("https://api.minimax.io".to_string()),
            model: Some("speech-to-text".to_string()),
            language: Some("zh".to_string()),
            baidu_cuid: None,
            baidu_format: None,
            baidu_sample_rate: None,
            baidu_lm_id: None,
            baidu_realtime_endpoint: None,
            baidu_realtime_dev_pid: None,
            baidu_realtime_cuid: None,
            baidu_realtime_format: None,
            baidu_realtime_sample_rate: None,
            baidu_realtime_user: None,
        }
    }

    fn baidu_config() -> CloudAsrConfig {
        CloudAsrConfig {
            provider: "baidu".to_string(),
            group_id: None,
            base_url: Some("http://vop.baidu.com/server_api".to_string()),
            model: Some("1537".to_string()),
            language: Some("zh".to_string()),
            baidu_cuid: Some("voxtype-local".to_string()),
            baidu_format: Some("pcm".to_string()),
            baidu_sample_rate: Some(16000),
            baidu_lm_id: None,
            baidu_realtime_endpoint: None,
            baidu_realtime_dev_pid: None,
            baidu_realtime_cuid: None,
            baidu_realtime_format: None,
            baidu_realtime_sample_rate: None,
            baidu_realtime_user: None,
        }
    }

    #[test]
    fn baidu_provider_is_ready_with_api_key_and_required_fields() {
        let dir = tempfile::tempdir().unwrap();
        let status = save_cloud_asr_config_with_provider_keys(
            dir.path().to_path_buf(),
            baidu_config(),
            ProviderApiKeys {
                minimax_api_key: None,
                baidu_asr_api_key: Some("baidu-api-key".to_string()),
                baidu_asr_secret_key: Some("baidu-secret-key".to_string()),
            },
        )
        .unwrap();

        assert!(status.ready);
        assert_eq!(status.api_key_source, "env:BAIDU_ASR_API_KEY");
        assert_eq!(status.api_key_preview, Some("ba***ey".to_string()));
        assert!(status.secret_key_configured);
        assert_eq!(status.secret_key_source, "env:BAIDU_ASR_SECRET_KEY");
        assert_eq!(status.secret_key_preview, Some("ba***ey".to_string()));
        assert!(status.message.contains("百度短语音识别配置完整"));
        let payload = fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap();
        assert!(!payload.contains("baidu-api-key"));
        assert!(!payload.contains("baidu-secret-key"));
        assert!(payload.contains("voxtype-local"));
    }

    #[test]
    fn baidu_provider_defaults_to_short_speech_rest_fields() {
        let mut config = baidu_config();
        config.base_url = None;
        config.model = None;
        config.language = None;
        config.baidu_cuid = None;
        config.baidu_format = None;
        config.baidu_sample_rate = None;
        let normalized = normalize_config(config);

        assert_eq!(
            normalized.base_url,
            Some("http://vop.baidu.com/server_api".to_string())
        );
        assert_eq!(normalized.model, Some("1537".to_string()));
        assert_eq!(normalized.baidu_cuid, Some("voxtype-local".to_string()));
        assert_eq!(normalized.baidu_format, Some("pcm".to_string()));
        assert_eq!(normalized.baidu_sample_rate, Some(16000));
        assert_eq!(
            normalized.baidu_realtime_endpoint,
            Some("wss://vop.baidu.com/realtime_asr".to_string())
        );
        assert_eq!(normalized.baidu_realtime_dev_pid, Some("15372".to_string()));
    }

    #[test]
    fn saving_baidu_api_key_switches_minimax_disk_config_to_baidu_status() {
        let dir = tempfile::tempdir().unwrap();
        save_cloud_asr_config_with_provider_keys(
            dir.path().to_path_buf(),
            minimax_config(),
            ProviderApiKeys {
                minimax_api_key: None,
                baidu_asr_api_key: None,
                baidu_asr_secret_key: None,
            },
        )
        .unwrap();

        let existing_config = load_cloud_asr_config(dir.path().to_path_buf());
        let baidu_config = baidu_config_from_existing_config(existing_config);
        let status = save_cloud_asr_config_with_provider_keys(
            dir.path().to_path_buf(),
            baidu_config,
            ProviderApiKeys {
                minimax_api_key: None,
                baidu_asr_api_key: Some("baidu-api-key".to_string()),
                baidu_asr_secret_key: Some("baidu-secret-key".to_string()),
            },
        )
        .unwrap();

        assert_eq!(status.config.provider, "baidu");
        assert!(status.ready);
        assert_eq!(status.api_key_source, "env:BAIDU_ASR_API_KEY");
        let saved = load_cloud_asr_config(dir.path().to_path_buf());
        assert_eq!(saved.provider, "baidu");
        assert_eq!(saved.model, Some("1537".to_string()));
    }

    #[test]
    fn baidu_status_rejects_non_server_api_endpoint() {
        let status = save_cloud_asr_config_with_provider_keys(
            tempfile::tempdir().unwrap().path().to_path_buf(),
            CloudAsrConfig {
                base_url: Some("http://vop.baidu.com".to_string()),
                ..baidu_config()
            },
            ProviderApiKeys {
                minimax_api_key: None,
                baidu_asr_api_key: Some("baidu-api-key".to_string()),
                baidu_asr_secret_key: Some("baidu-secret-key".to_string()),
            },
        )
        .unwrap();

        assert!(!status.ready);
        assert!(status.message.contains("server_api"));
    }

    #[test]
    fn baidu_provider_requires_secret_key_for_access_token_auth() {
        let status = save_cloud_asr_config_with_provider_keys(
            tempfile::tempdir().unwrap().path().to_path_buf(),
            baidu_config(),
            ProviderApiKeys {
                minimax_api_key: None,
                baidu_asr_api_key: Some("baidu-api-key".to_string()),
                baidu_asr_secret_key: None,
            },
        )
        .unwrap();

        assert!(!status.ready);
        assert!(status.api_key_configured);
        assert!(!status.secret_key_configured);
        assert_eq!(status.secret_key_source, "missing");
        assert!(status.message.contains("BAIDU_ASR_SECRET_KEY"));
    }
}
