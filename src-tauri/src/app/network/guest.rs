//! Гостевой доступ без Telegram (04.08.2026).
//!
//! Человек поставил приложение, ключа у него нет, а дойти до бота он не может —
//! именно потому, что нет VPN. Одна кнопка даёт связь на два часа: хватает, чтобы
//! открыть бота и купить подписку.
//!
//! Сервер тот же, что у телефона: `POST /guest/start`. Отказы приходят кодом
//! (`already_issued`, `ip_limit`, `daily_cap`, `disabled`), слова человеку
//! подбирает приложение — сырой ответ сервера на экран не выходит.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::app::network::device_link::{backend, device_name, ensure_device_id};
use crate::app::storage::enhanced_storage_service::get_enhanced_storage;
use crate::utils::http_client;

/// Ключ записи в базе настроек.
const STORAGE_KEY: &str = "guest_access";

/// Как называем себя серверу.
const PLATFORM: &str = "desktop";

/// Что помним о выданном гостевом доступе.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuestRecord {
    /// Когда доступ кончится — строкой от сервера, свои часы не в счёт.
    #[serde(default)]
    pub expires_at: String,
    #[serde(default)]
    pub traffic_gb: u64,
}

/// Выданный гостевой доступ.
#[derive(Debug, Clone, Serialize)]
pub struct GuestResult {
    pub sub_url: String,
    pub expires_at: String,
    pub ttl_hours: u64,
    pub traffic_gb: u64,
}

/// Имя отказа для экрана: сначала слово сервера, иначе — по коду ответа.
fn reason_for(status: u16, body: &serde_json::Value) -> String {
    if let Some(error) = body.get("error").and_then(|v| v.as_str()) {
        if !error.is_empty() {
            return error.to_string();
        }
    }
    match status {
        429 => "rate_limited".to_string(),
        503 => "disabled".to_string(),
        _ => "server".to_string(),
    }
}

/// Разбор ответа `POST /guest/start`. Без ссылки подписки гостя нет.
fn parse_guest(body: &serde_json::Value) -> Option<GuestResult> {
    let sub_url = body
        .get("sub_url")
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())?;

    Some(GuestResult {
        sub_url: sub_url.to_string(),
        expires_at: body
            .get("expires_at")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        ttl_hours: body.get("ttl_hours").and_then(|v| v.as_u64()).unwrap_or(0),
        traffic_gb: body.get("traffic_gb").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

async fn save_record(app: &AppHandle, record: &GuestRecord) -> Result<(), String> {
    let storage = get_enhanced_storage(app).await?;
    storage
        .save_config(STORAGE_KEY, record)
        .await
        .map_err(|_| "storage".to_string())
}

/// Попросить гостевой доступ. Отказ возвращается кодом, а не текстом сервера.
#[tauri::command]
pub async fn guest_start(app: AppHandle) -> Result<GuestResult, String> {
    let device_id = ensure_device_id(&app).await?;
    let payload = serde_json::json!({
        "device_id": device_id,
        "platform": PLATFORM,
        "device_name": device_name(),
        "app_version": env!("CARGO_PKG_VERSION"),
    });

    let response = http_client::get_client()
        .post(backend("/guest/start"))
        .json(&payload)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!("гость: сервер не ответил: {}", error);
            "network".to_string()
        })?;

    let status = response.status().as_u16();
    let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);

    if !(200..300).contains(&status) {
        let reason = reason_for(status, &body);
        tracing::warn!("гость: отказано ({})", reason);
        return Err(reason);
    }

    let issued = parse_guest(&body).ok_or_else(|| "server".to_string())?;
    save_record(
        &app,
        &GuestRecord {
            expires_at: issued.expires_at.clone(),
            traffic_gb: issued.traffic_gb,
        },
    )
    .await?;

    Ok(issued)
}

/// Что известно о госте локально, без похода в сеть.
#[tauri::command]
pub async fn guest_snapshot(app: AppHandle) -> Result<GuestRecord, String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .get_config::<GuestRecord>(STORAGE_KEY)
        .await
        .map(|value| value.unwrap_or_default())
        .map_err(|_| "storage".to_string())
}

#[cfg(test)]
#[path = "guest.tests.rs"]
mod tests;
