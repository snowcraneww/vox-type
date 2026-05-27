use std::{
    fs::{self, File},
    io::{self, copy},
    path::{Path, PathBuf},
    time::Duration,
};

use sha1::Digest as Sha1Digest;

use crate::{
    asr_config::{self, AsrConfig, AsrConfigStatus},
    error::VoxError,
};

pub const WHISPER_RELEASE_URL: &str =
    "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.4/whisper-bin-x64.zip";
pub const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin";
const WHISPER_ZIP_SHA256: &str = "74f973345cb52ef5ba3ec9e7e7af8e48cc8c71722d1528603b80588a11f82e3e";
const MODEL_SHA1: &str = "465707469ff3a37a2b9b8d8f89f2f99de7299dac";
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
const DOWNLOAD_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedAsrInstallPlan {
    pub root_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub models_dir: PathBuf,
    pub downloads_dir: PathBuf,
    pub binary_zip_path: PathBuf,
    pub whisper_binary_path: PathBuf,
    pub model_path: PathBuf,
}

impl ManagedAsrInstallPlan {
    pub fn asr_config(&self) -> AsrConfig {
        AsrConfig {
            whisper_binary_path: Some(self.whisper_binary_path.to_string_lossy().to_string()),
            whisper_model_path: Some(self.model_path.to_string_lossy().to_string()),
            language: "zh".to_string(),
        }
    }
}

pub fn managed_install_plan(app_data_dir: PathBuf) -> ManagedAsrInstallPlan {
    let root_dir = app_data_dir.join("managed-asr").join("whisper.cpp");
    let bin_dir = root_dir.join("bin");
    let models_dir = root_dir.join("models");
    let downloads_dir = root_dir.join("downloads");
    ManagedAsrInstallPlan {
        binary_zip_path: downloads_dir.join("whisper-bin-x64.zip"),
        whisper_binary_path: bin_dir.join("whisper-cli.exe"),
        model_path: models_dir.join("ggml-base.bin"),
        root_dir,
        bin_dir,
        models_dir,
        downloads_dir,
    }
}

pub fn install_managed_asr(
    app_data_dir: PathBuf,
    config_dir: PathBuf,
) -> Result<AsrConfigStatus, VoxError> {
    let plan = managed_install_plan(app_data_dir);
    fs::create_dir_all(&plan.bin_dir)
        .map_err(|error| VoxError::Config(format!("创建 whisper.cpp 安装目录失败：{error}")))?;
    fs::create_dir_all(&plan.models_dir)
        .map_err(|error| VoxError::Config(format!("创建模型目录失败：{error}")))?;
    fs::create_dir_all(&plan.downloads_dir)
        .map_err(|error| VoxError::Config(format!("创建下载目录失败：{error}")))?;

    if !plan.whisper_binary_path.is_file() {
        download_file(
            WHISPER_RELEASE_URL,
            &plan.binary_zip_path,
            "下载 whisper.cpp 失败",
        )?;
        verify_sha256(
            &plan.binary_zip_path,
            WHISPER_ZIP_SHA256,
            "whisper.cpp ZIP 校验失败",
        )?;
        extract_zip_to_dir(&plan.binary_zip_path, &plan.bin_dir)?;
    }
    if !plan.whisper_binary_path.is_file() {
        return Err(VoxError::Config(
            "安装完成后没有找到 whisper-cli.exe".to_string(),
        ));
    }

    if !plan.model_path.is_file() {
        download_file(MODEL_URL, &plan.model_path, "下载模型失败")?;
        verify_sha1(&plan.model_path, MODEL_SHA1, "模型文件校验失败")?;
    }
    if !plan.model_path.is_file() {
        return Err(VoxError::Config(
            "安装完成后没有找到 ggml-base.bin".to_string(),
        ));
    }

    asr_config::save_asr_config(config_dir, plan.asr_config())
}

fn download_file(url: &str, target_path: &Path, context: &str) -> Result<(), VoxError> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| VoxError::Config(format!("创建下载目录失败：{error}")))?;
    }
    let mut response = download_client()?
        .get(url)
        .send()
        .map_err(|error| VoxError::Config(format!("{context}：{error}")))?;
    if !response.status().is_success() {
        return Err(VoxError::Config(format!(
            "{context}：HTTP {}",
            response.status()
        )));
    }
    let mut file = File::create(target_path)
        .map_err(|error| VoxError::Config(format!("创建下载文件失败：{error}")))?;
    copy(&mut response, &mut file)
        .map_err(|error| VoxError::Config(format!("写入下载文件失败：{error}")))?;
    Ok(())
}

fn download_client() -> Result<reqwest::blocking::Client, VoxError> {
    let (timeout, connect_timeout) = download_timeouts();
    reqwest::blocking::Client::builder()
        .timeout(timeout)
        .connect_timeout(connect_timeout)
        .build()
        .map_err(|error| VoxError::Config(format!("创建下载客户端失败：{error}")))
}

fn download_timeouts() -> (Duration, Duration) {
    (DOWNLOAD_TIMEOUT, DOWNLOAD_CONNECT_TIMEOUT)
}

fn extract_zip_to_dir(zip_path: &PathBuf, target_dir: &PathBuf) -> Result<(), VoxError> {
    fs::create_dir_all(target_dir)
        .map_err(|error| VoxError::Config(format!("创建解压目录失败：{error}")))?;
    let file = File::open(zip_path)
        .map_err(|error| VoxError::Config(format!("打开 whisper.cpp ZIP 失败：{error}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|error| VoxError::Config(format!("读取 whisper.cpp ZIP 失败：{error}")))?;

    for index in 0..archive.len() {
        let mut item = archive
            .by_index(index)
            .map_err(|error| VoxError::Config(format!("读取 ZIP 条目失败：{error}")))?;
        if item.is_dir() {
            continue;
        }
        let Some(name) = item.enclosed_name() else {
            continue;
        };
        let file_name = name.file_name().map(|value| value.to_os_string());
        let Some(file_name) = file_name else {
            continue;
        };
        let output_path = target_dir.join(file_name);
        let mut output = File::create(&output_path)
            .map_err(|error| VoxError::Config(format!("写入解压文件失败：{error}")))?;
        io::copy(&mut item, &mut output)
            .map_err(|error| VoxError::Config(format!("解压 whisper.cpp 失败：{error}")))?;
    }
    Ok(())
}

fn verify_sha256(path: &Path, expected: &str, context: &str) -> Result<(), VoxError> {
    let bytes = fs::read(path)
        .map_err(|error| VoxError::Config(format!("{context}：读取文件失败：{error}")))?;
    let actual = format!("{:x}", sha2::Sha256::digest(&bytes));
    if actual != expected {
        return Err(VoxError::Config(format!(
            "{context}：期望 {expected}，实际 {actual}"
        )));
    }
    Ok(())
}

fn verify_sha1(path: &Path, expected: &str, context: &str) -> Result<(), VoxError> {
    let bytes = fs::read(path)
        .map_err(|error| VoxError::Config(format!("{context}：读取文件失败：{error}")))?;
    let actual = format!("{:x}", sha1::Sha1::digest(&bytes));
    if actual != expected {
        return Err(VoxError::Config(format!(
            "{context}：期望 {expected}，实际 {actual}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_plan_uses_app_data_managed_asr_directory() {
        let root = PathBuf::from("C:/Users/example/AppData/Roaming/dev.voxtype.desktop");

        let plan = managed_install_plan(root.clone());

        assert_eq!(plan.root_dir, root.join("managed-asr").join("whisper.cpp"));
        assert_eq!(plan.bin_dir, plan.root_dir.join("bin"));
        assert_eq!(plan.models_dir, plan.root_dir.join("models"));
        assert_eq!(plan.downloads_dir, plan.root_dir.join("downloads"));
        assert_eq!(
            plan.binary_zip_path,
            plan.downloads_dir.join("whisper-bin-x64.zip")
        );
        assert_eq!(
            plan.whisper_binary_path,
            plan.bin_dir.join("whisper-cli.exe")
        );
        assert_eq!(plan.model_path, plan.models_dir.join("ggml-base.bin"));
    }

    #[test]
    fn install_plan_can_be_converted_to_asr_config() {
        let root = PathBuf::from("C:/voxtype");
        let plan = managed_install_plan(root);

        let config = plan.asr_config();

        assert_eq!(
            config.whisper_binary_path,
            Some(plan.whisper_binary_path.to_string_lossy().to_string())
        );
        assert_eq!(
            config.whisper_model_path,
            Some(plan.model_path.to_string_lossy().to_string())
        );
        assert_eq!(config.language, "zh");
    }

    #[test]
    fn extracts_zip_contents_to_target_directory() {
        let temp = tempfile::tempdir().unwrap();
        let zip_path = temp.path().join("test.zip");
        let target_dir = temp.path().join("out");
        create_test_zip(&zip_path);

        extract_zip_to_dir(&zip_path, &target_dir).unwrap();

        assert!(target_dir.join("whisper-cli.exe").is_file());
        assert!(target_dir.join("libwhisper.dll").is_file());
    }

    #[test]
    fn verifies_sha256_hash() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"abc").unwrap();

        verify_sha256(
            temp.path(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "测试 SHA-256 失败",
        )
        .unwrap();
    }

    #[test]
    fn reports_sha1_mismatch() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"abc").unwrap();

        let error = verify_sha1(
            temp.path(),
            "0000000000000000000000000000000000000000",
            "测试 SHA-1 失败",
        )
        .unwrap_err();

        assert!(error.to_string().contains("测试 SHA-1 失败"));
    }

    #[test]
    fn download_client_uses_bounded_timeouts() {
        let (timeout, connect_timeout) = download_timeouts();

        assert_eq!(timeout, Duration::from_secs(300));
        assert_eq!(connect_timeout, Duration::from_secs(20));
    }

    fn create_test_zip(path: &PathBuf) {
        let file = File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("whisper-cli.exe", options).unwrap();
        std::io::Write::write_all(&mut zip, b"fake exe").unwrap();
        zip.start_file("nested/libwhisper.dll", options).unwrap();
        std::io::Write::write_all(&mut zip, b"fake dll").unwrap();
        zip.finish().unwrap();
    }
}
