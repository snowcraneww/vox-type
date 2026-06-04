use std::{fs, path::PathBuf};

use crate::{
    error::VoxError,
    sensevoice_config::{self, SenseVoiceConfig, SenseVoiceConfigStatus},
};

const RUNTIME_ARCHIVE_NAME: &str = "sherpa-onnx-v1.13.2-win-x64-static-MT-Release-no-tts.tar.bz2";
const RUNTIME_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.2/sherpa-onnx-v1.13.2-win-x64-static-MT-Release-no-tts.tar.bz2";
const RUNTIME_BINARY_NAME: &str = "sherpa-onnx-offline.exe";
const DEPRECATED_GUI_RUNTIME_NAME: &str = "sherpa-onnx-non-streaming-asr-x64-v1.13.2.exe";
const MODEL_URL: &str = "https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/resolve/main/model.int8.onnx";
const TOKENS_URL: &str = "https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/resolve/main/tokens.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedSenseVoiceInstallPlan {
    pub root_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub models_dir: PathBuf,
    pub downloads_dir: PathBuf,
    pub runtime_archive_path: PathBuf,
    pub runtime_path: PathBuf,
    pub deprecated_gui_runtime_path: PathBuf,
    pub model_path: PathBuf,
    pub tokens_path: PathBuf,
}

impl ManagedSenseVoiceInstallPlan {
    pub fn config(&self) -> SenseVoiceConfig {
        SenseVoiceConfig {
            runtime_path: Some(self.runtime_path.to_string_lossy().to_string()),
            model_path: Some(self.model_path.to_string_lossy().to_string()),
            tokens_path: Some(self.tokens_path.to_string_lossy().to_string()),
            language: "zh".to_string(),
        }
    }
}

pub fn managed_sensevoice_install_plan(app_data_dir: PathBuf) -> ManagedSenseVoiceInstallPlan {
    let root_dir = app_data_dir.join("managed-asr").join("sensevoice-small");
    let bin_dir = root_dir.join("bin");
    let models_dir = root_dir.join("models");
    let downloads_dir = root_dir.join("downloads");
    ManagedSenseVoiceInstallPlan {
        runtime_archive_path: downloads_dir.join(RUNTIME_ARCHIVE_NAME),
        runtime_path: bin_dir.join(RUNTIME_BINARY_NAME),
        deprecated_gui_runtime_path: bin_dir.join(DEPRECATED_GUI_RUNTIME_NAME),
        model_path: models_dir.join("model.int8.onnx"),
        tokens_path: models_dir.join("tokens.txt"),
        root_dir,
        bin_dir,
        models_dir,
        downloads_dir,
    }
}

pub fn install_managed_sensevoice(
    app_data_dir: PathBuf,
    config_dir: PathBuf,
) -> Result<SenseVoiceConfigStatus, VoxError> {
    let plan = managed_sensevoice_install_plan(app_data_dir);
    fs::create_dir_all(&plan.bin_dir).map_err(|error| {
        VoxError::Config(format!(
            "Create SenseVoice runtime directory failed: {error}"
        ))
    })?;
    fs::create_dir_all(&plan.models_dir).map_err(|error| {
        VoxError::Config(format!("Create SenseVoice model directory failed: {error}"))
    })?;
    fs::create_dir_all(&plan.downloads_dir).map_err(|error| {
        VoxError::Config(format!(
            "Create SenseVoice download directory failed: {error}"
        ))
    })?;

    remove_deprecated_gui_runtime(&plan)?;
    if !plan.runtime_path.is_file() {
        download_file(
            RUNTIME_URL,
            &plan.runtime_archive_path,
            "Download sherpa-onnx CLI runtime failed",
        )?;
        extract_runtime_archive(&plan.runtime_archive_path, &plan.root_dir)?;
    }
    if !plan.runtime_path.is_file() {
        return Err(VoxError::Config(format!(
            "SenseVoice install finished but {RUNTIME_BINARY_NAME} was not found"
        )));
    }

    if !plan.model_path.is_file() {
        download_file(
            MODEL_URL,
            &plan.model_path,
            "Download SenseVoice ONNX model failed",
        )?;
    }
    if !plan.tokens_path.is_file() {
        download_file(
            TOKENS_URL,
            &plan.tokens_path,
            "Download SenseVoice tokens failed",
        )?;
    }
    sensevoice_config::save_sensevoice_config(config_dir, plan.config())
}

fn remove_deprecated_gui_runtime(plan: &ManagedSenseVoiceInstallPlan) -> Result<(), VoxError> {
    if plan.deprecated_gui_runtime_path.is_file() {
        fs::remove_file(&plan.deprecated_gui_runtime_path).map_err(|error| {
            VoxError::Config(format!(
                "Remove deprecated SenseVoice GUI runtime failed: {error}"
            ))
        })?;
    }
    Ok(())
}

fn extract_runtime_archive(archive_path: &PathBuf, target_dir: &PathBuf) -> Result<(), VoxError> {
    fs::create_dir_all(target_dir).map_err(|error| {
        VoxError::Config(format!(
            "Create SenseVoice extract directory failed: {error}"
        ))
    })?;
    let status = std::process::Command::new("tar")
        .args([
            "-xjf",
            &archive_path.to_string_lossy(),
            "-C",
            &target_dir.to_string_lossy(),
            "--strip-components=1",
        ])
        .status()
        .map_err(|error| {
            VoxError::Config(format!("Start tar for SenseVoice runtime failed: {error}"))
        })?;
    if !status.success() {
        return Err(VoxError::Config(format!(
            "Extract SenseVoice runtime failed: tar exited with {status}"
        )));
    }
    Ok(())
}

fn download_file(url: &str, target_path: &PathBuf, context: &str) -> Result<(), VoxError> {
    let mut response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .connect_timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| {
            VoxError::Config(format!("Create SenseVoice download client failed: {error}"))
        })?
        .get(url)
        .send()
        .map_err(|error| VoxError::Config(format!("{context}: {error}")))?;
    if !response.status().is_success() {
        return Err(VoxError::Config(format!(
            "{context}: HTTP {}",
            response.status()
        )));
    }
    let mut file = fs::File::create(target_path).map_err(|error| {
        VoxError::Config(format!("Create SenseVoice download file failed: {error}"))
    })?;
    std::io::copy(&mut response, &mut file).map_err(|error| {
        VoxError::Config(format!("Write SenseVoice download file failed: {error}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_plan_uses_managed_sensevoice_directory() {
        let root = PathBuf::from("C:/voxtype-data");
        let plan = managed_sensevoice_install_plan(root.clone());
        assert_eq!(
            plan.root_dir,
            root.join("managed-asr").join("sensevoice-small")
        );
        assert_eq!(plan.bin_dir, plan.root_dir.join("bin"));
        assert_eq!(plan.models_dir, plan.root_dir.join("models"));
        assert_eq!(
            plan.runtime_path,
            plan.bin_dir.join("sherpa-onnx-offline.exe")
        );
        assert_eq!(
            plan.runtime_archive_path,
            plan.downloads_dir
                .join("sherpa-onnx-v1.13.2-win-x64-static-MT-Release-no-tts.tar.bz2")
        );
        assert_eq!(
            plan.deprecated_gui_runtime_path,
            plan.bin_dir
                .join("sherpa-onnx-non-streaming-asr-x64-v1.13.2.exe")
        );
        assert_eq!(plan.model_path, plan.models_dir.join("model.int8.onnx"));
        assert_eq!(plan.tokens_path, plan.models_dir.join("tokens.txt"));
    }

    #[test]
    fn install_plan_converts_to_ready_shape_config() {
        let plan = managed_sensevoice_install_plan(PathBuf::from("C:/voxtype-data"));
        let config = plan.config();
        let runtime_path = config.runtime_path.unwrap();
        assert!(runtime_path.contains("sherpa-onnx-offline.exe"));
        assert!(!runtime_path.contains("non-streaming-asr-x64"));
        assert!(config.model_path.unwrap().contains("model.int8.onnx"));
        assert!(config.tokens_path.unwrap().contains("tokens.txt"));
        assert_eq!(config.language, "zh");
    }

    #[test]
    fn removes_deprecated_gui_runtime_from_managed_bin_dir() {
        let temp = tempfile::tempdir().unwrap();
        let plan = managed_sensevoice_install_plan(temp.path().to_path_buf());
        fs::create_dir_all(&plan.bin_dir).unwrap();
        fs::write(&plan.deprecated_gui_runtime_path, b"gui runtime").unwrap();

        remove_deprecated_gui_runtime(&plan).unwrap();

        assert!(!plan.deprecated_gui_runtime_path.exists());
    }
}
