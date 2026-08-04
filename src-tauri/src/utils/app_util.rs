use crate::app::constants::messages;
use std::path::{Path, PathBuf};
use tracing::{error, warn};

/// Адрес для журнала: остаётся только «куда», исчезает «по какому ключу».
///
/// Токен подписки в пути — это платный ключ клиента: по нему конфиг забирают
/// без бота и без пароля. Журнал же лежит файлом на диске, уезжает в резервную
/// копию и приходит нам скриншотом в поддержку.
pub fn safe_url(raw: &str) -> String {
    match url::Url::parse(raw) {
        Ok(parsed) => format!(
            "{}://{}/…",
            parsed.scheme(),
            parsed.host_str().unwrap_or("?")
        ),
        Err(_) => "<адрес скрыт>".to_string(),
    }
}

/// Имя нашей рабочей папки: там ядро, конфиги и журналы.
pub const WORK_DIR_NAME: &str = "ShtilVPN";

/// Как папка звалась у форка. Оставшиеся от него данные переносим один раз.
pub const LEGACY_WORK_DIR_NAME: &str = "sing-box-windows";

/// Куда система кладёт данные приложений.
fn data_base_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        // Windows: %LOCALAPPDATA%
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData"))
    } else if cfg!(any(target_os = "linux", target_os = "macos")) {
        // Linux: ~/.local/share · macOS: ~/Library/Application Support
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
    } else {
        dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
    }
}

/// Забрать папку форка под наше имя. Не вышло (файл занят работающим ядром) —
/// работаем со старой: терять ключи и ядро человеку нельзя.
fn adopt_legacy_dir(ours: &Path, legacy: &Path) -> PathBuf {
    if ours.exists() || !legacy.exists() {
        return ours.to_path_buf();
    }

    match std::fs::rename(legacy, ours) {
        Ok(()) => ours.to_path_buf(),
        Err(e) => {
            warn!("Не удалось перенести рабочую папку под наше имя: {}", e);
            legacy.to_path_buf()
        }
    }
}

/// Рабочая папка: наша, а если приложение стоит с прошлых версий — перенесённая.
fn resolve_work_dir() -> PathBuf {
    let base = data_base_dir();
    adopt_legacy_dir(&base.join(WORK_DIR_NAME), &base.join(LEGACY_WORK_DIR_NAME))
}

// 获取工作目录（同步版本）
pub fn get_work_dir_sync() -> String {
    let cache_dir = resolve_work_dir();

    // 确保目录存在
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
    }

    cache_dir.to_str().unwrap_or(".").to_string()
}

// 获取工作目录
pub async fn get_work_dir() -> String {
    let cache_dir = resolve_work_dir();

    // 确保目录存在
    if let Err(e) = tokio::fs::create_dir_all(&cache_dir).await {
        error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
    }

    cache_dir.to_str().unwrap_or(".").to_string()
}

/// 获取服务路径
pub fn get_service_path() -> PathBuf {
    // 获取可执行程序路径
    let exe_path = std::env::current_exe().expect("无法获取可执行程序路径");
    let work_dir = exe_path
        .parent()
        .expect("无法获取可执行程序父目录")
        .to_str()
        .expect("无法将父目录路径转换为字符串");

    // 根据平台确定可执行文件名
    let service_name = if cfg!(target_os = "windows") {
        "sing-box-service.exe"
    } else {
        "sing-box-service"
    };

    PathBuf::from(&work_dir)
        .join("src")
        .join("config")
        .join(service_name)
}

#[cfg(test)]
#[path = "app_util.tests.rs"]
mod tests;
