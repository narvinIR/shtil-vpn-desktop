//! Значения по умолчанию, которые должны быть верны на первом же запуске (04.08.2026).
//!
//! Форк приходил в ручном режиме: приложение писало «Подключено», а трафик
//! системы шёл мимо VPN — владелец так и не смог понять, работает оно или нет.
//! С 1.2.0 по умолчанию включён системный режим, а установки прошлых версий
//! переводятся один раз.
//!
//! Здесь же снимается системный прокси, осиротевший после падения: ядро на
//! старте уже снято, значит настройка в системе никуда не ведёт и человек
//! остался бы без интернета вовсе.

use tauri::AppHandle;
use tracing::{info, warn};

use crate::app::storage::enhanced_storage_service::get_enhanced_storage;
use crate::utils::app_util::{LEGACY_WORK_DIR_NAME, WORK_DIR_NAME};
use crate::utils::proxy_util::disable_system_proxy;

/// Ключ записи в базе настроек: дефолты какой версии уже применены.
const APPLIED_DEFAULTS_VERSION_KEY: &str = "applied_defaults_version";

/// Версия, с которой системный режим стал значением по умолчанию.
const SYSTEM_MODE_SINCE: (u32, u32, u32) = (1, 2, 0);

fn parse_version(raw: &str) -> Option<(u32, u32, u32)> {
    let mut parts = raw.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// Нужно ли перевести эту установку в системный режим.
///
/// Переводим только тех, у кого не включено ничего: в ручном режиме VPN не
/// работал вовсе, поэтому осознанным выбором это быть не могло.
fn needs_system_mode(
    applied_version: Option<&str>,
    system_proxy_enabled: bool,
    tun_enabled: bool,
) -> bool {
    if system_proxy_enabled || tun_enabled {
        return false;
    }
    match applied_version {
        None => true,
        Some(raw) => parse_version(raw).is_some_and(|version| version < SYSTEM_MODE_SINCE),
    }
}

/// Наш ли системный прокси остался в системе после прошлого запуска.
fn should_clear_leftover_proxy(system_proxy_enabled: bool) -> bool {
    system_proxy_enabled
}

/// Путь из старой папки форка — в нашу. `None`, если правка не нужна.
fn rebase_legacy_path(path: &str) -> Option<String> {
    if path.is_empty() || !path.contains(LEGACY_WORK_DIR_NAME) {
        return None;
    }
    Some(path.replace(LEGACY_WORK_DIR_NAME, WORK_DIR_NAME))
}

/// Переписать сохранённые пути ключей после переименования рабочей папки.
async fn rebase_saved_paths(
    storage: &crate::app::storage::EnhancedStorageService,
    app_config: &mut crate::app::storage::state_model::AppConfig,
) -> Result<bool, String> {
    let mut changed = false;

    if let Some(path) = app_config.active_config_path.as_deref() {
        if let Some(fixed) = rebase_legacy_path(path) {
            app_config.active_config_path = Some(fixed);
            changed = true;
        }
    }

    let mut subscriptions = storage
        .get_subscriptions()
        .await
        .map_err(|e| e.to_string())?;
    let mut subscriptions_changed = false;
    for subscription in subscriptions.iter_mut() {
        if let Some(path) = subscription.config_path.as_deref() {
            if let Some(fixed) = rebase_legacy_path(path) {
                subscription.config_path = Some(fixed);
                subscriptions_changed = true;
            }
        }
    }
    if subscriptions_changed {
        storage
            .save_subscriptions(&subscriptions)
            .await
            .map_err(|e| e.to_string())?;
        info!("Пути ключей переписаны на новую рабочую папку");
    }

    Ok(changed || subscriptions_changed)
}

pub async fn apply_startup_defaults(app: &AppHandle) {
    if let Err(e) = run_startup_defaults(app).await {
        warn!(
            "Не удалось применить значения по умолчанию при старте: {}",
            e
        );
    }
}

async fn run_startup_defaults(app: &AppHandle) -> Result<(), String> {
    let storage = get_enhanced_storage(app).await?;
    let mut app_config = storage.get_app_config().await.map_err(|e| e.to_string())?;

    if should_clear_leftover_proxy(app_config.system_proxy_enabled) {
        if let Err(e) = disable_system_proxy() {
            warn!(
                "Не удалось снять системный прокси, оставшийся от прошлого запуска: {}",
                e
            );
        } else {
            info!("Системный прокси снят: ядро на старте ещё не запущено");
        }
    }

    let applied_version = storage
        .load_generic_config::<String>(APPLIED_DEFAULTS_VERSION_KEY)
        .await
        .map_err(|e| e.to_string())?;

    let mut config_changed = rebase_saved_paths(&storage, &mut app_config).await?;

    if needs_system_mode(
        applied_version.as_deref(),
        app_config.system_proxy_enabled,
        app_config.tun_enabled,
    ) {
        app_config.system_proxy_enabled = true;
        app_config.proxy_mode = "system".to_string();
        config_changed = true;
        info!("Установка переведена в системный режим: до этого трафик шёл мимо VPN");
    }

    if config_changed {
        storage
            .save_app_config(&app_config)
            .await
            .map_err(|e| e.to_string())?;
    }

    let current_version = app.package_info().version.to_string();
    if applied_version.as_deref() != Some(current_version.as_str()) {
        storage
            .save_generic_config(APPLIED_DEFAULTS_VERSION_KEY, &current_version)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "startup_defaults_service.tests.rs"]
mod tests;
