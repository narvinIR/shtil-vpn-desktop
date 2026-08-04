//! Привязка компьютера к боту по коду (04.08.2026).
//!
//! Человек видит на экране код из десяти цифр, называет его боту — и ключ
//! приезжает сам. Ссылку подписки руками больше не переносят.
//!
//! Тот же поток уже работает на телефоне (`mobile/app` → `link/DeviceLink.kt`),
//! и сервер для него готов целиком: `POST /link/start`, `GET /link/status`,
//! `GET /device/state`. Здесь — половина компьютера.
//!
//! Два правила, ради которых написаны тесты:
//! * **сеть молчит ≠ подписка кончилась** — не дозвонились, значит прошлое
//!   состояние остаётся как было;
//! * **первый опрос молчит** — иначе человек с действующей подпиской получал бы
//!   при запуске новость «вы оплатили».

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::app::constants::network::api;
use crate::app::storage::enhanced_storage_service::get_enhanced_storage;
use crate::utils::http_client;

/// Ключ записи в базе настроек. Своего файла на диске не заводим.
const STORAGE_KEY: &str = "device_link";

/// Как называем себя серверу. Сервер знает три вида: phone, tv, desktop.
const PLATFORM: &str = "desktop";

/// Сколько ждать между опросами, если сервер молчит о своём периоде.
const DEFAULT_POLL_SEC: u64 = 1800;

/// Чаще этого не спрашиваем даже по просьбе сервера.
const MIN_POLL_SEC: u64 = 60;

/// Сколько ответов «не знаю такого устройства» подряд считаем правдой.
/// Один такой ответ бывает и от сбоя на стороне сервера, а привязку человек
/// восстанавливает только руками — поэтому ждём подтверждения.
const MISSES_BEFORE_FORGET: u32 = 2;

/// Что помним об устройстве между запусками.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkRecord {
    /// Постоянный отпечаток этого компьютера. Живёт от первой привязки.
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub secret: String,
    #[serde(default)]
    pub poll_interval: u64,
    #[serde(default)]
    pub last_poll: i64,
    /// Состояние подписки на прошлом опросе — по нему видно смену.
    #[serde(default)]
    pub signature: String,
    /// Сколько раз подряд сервер ответил «не знаю такого устройства».
    #[serde(default)]
    pub misses: u32,
}

/// Пора ли забыть привязку. Одиночный отказ сервера её не снимает.
pub fn should_forget(misses: u32) -> bool {
    misses >= MISSES_BEFORE_FORGET
}

/// Что отдаём экрану сразу после запроса кода.
#[derive(Debug, Clone, Serialize)]
pub struct LinkStartResult {
    pub code: String,
    pub expires_at: String,
    pub poll_interval: u64,
}

/// Ожидание подтверждения в боте.
#[derive(Debug, Clone, Serialize)]
pub struct LinkStatusResult {
    /// pending | linked | expired | not_found | no_subscription
    pub status: String,
    pub sub_url: Option<String>,
}

/// Состояние подписки, каким его видит сервер.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceStateResult {
    pub linked: bool,
    /// active | expired | none
    pub subscription: String,
    pub is_trial: bool,
    pub days_left: i64,
    pub expires_at: Option<String>,
    pub sub_url: Option<String>,
    /// paid | trial | over — только в момент смены, иначе пусто.
    pub news: Option<String>,
}

/// Что известно локально, без похода в сеть.
#[derive(Debug, Clone, Serialize)]
pub struct LinkSnapshot {
    pub linked: bool,
    pub code: Option<String>,
}

fn backend(path: &str) -> String {
    format!("{}{}", api::BACKEND_URL.trim_end_matches('/'), path)
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Отпечаток компьютера: шестнадцать случайных байт, один раз и навсегда.
fn new_device_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Период опроса: сервер решает, но в разумных границах.
pub fn normalized_interval(value: u64) -> u64 {
    if value == 0 {
        DEFAULT_POLL_SEC
    } else {
        value.max(MIN_POLL_SEC)
    }
}

/// Пора ли спрашивать сервер.
pub fn is_poll_due(last_poll: i64, interval: u64, now: i64) -> bool {
    if last_poll <= 0 {
        return true;
    }
    now - last_poll >= normalized_interval(interval) as i64
}

/// Короткое имя состояния подписки — по нему сравниваем «было» и «стало».
pub fn signature_of(subscription: &str, is_trial: bool) -> String {
    match subscription {
        "active" if is_trial => "trial".to_string(),
        "active" => "active".to_string(),
        "expired" => "expired".to_string(),
        _ => "none".to_string(),
    }
}

/// Что сказать человеку при смене состояния.
///
/// Первый опрос (прошлого нет) молчит всегда: подписка могла существовать и до
/// установки приложения, а «вы оплатили» на ровном месте выглядит ошибкой.
pub fn news_for(previous: &str, current: &str) -> Option<String> {
    if previous.is_empty() || previous == current {
        return None;
    }
    match current {
        "active" => Some("paid".to_string()),
        "trial" => Some("trial".to_string()),
        "expired" | "none" => Some("over".to_string()),
        _ => None,
    }
}

fn text(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
}

/// Разбор ответа `GET /link/status`.
pub fn parse_status(body: &serde_json::Value) -> LinkStatusResult {
    let status = text(body, "status").unwrap_or_else(|| "pending".to_string());
    let sub_url = text(body, "sub_url");
    // Сервер отвечает `linked` и в тот момент, когда ссылки ещё нет: для человека
    // это всё ещё ожидание, а не готовый ключ.
    if status == "linked" && sub_url.is_none() {
        return LinkStatusResult {
            status: "pending".to_string(),
            sub_url: None,
        };
    }
    LinkStatusResult { status, sub_url }
}

/// Разбор ответа `GET /device/state`. Новость считается против прошлой подписи.
pub fn parse_state(body: &serde_json::Value, previous_signature: &str) -> DeviceStateResult {
    let status = text(body, "status").unwrap_or_default();
    if status != "linked" {
        return DeviceStateResult {
            linked: false,
            subscription: "none".to_string(),
            is_trial: false,
            days_left: 0,
            expires_at: None,
            sub_url: None,
            news: None,
        };
    }

    let subscription = text(body, "subscription").unwrap_or_else(|| "none".to_string());
    let is_trial = body
        .get("is_trial")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let signature = signature_of(&subscription, is_trial);

    DeviceStateResult {
        linked: true,
        subscription,
        is_trial,
        days_left: body.get("days_left").and_then(|v| v.as_i64()).unwrap_or(0),
        expires_at: text(body, "expires_at"),
        sub_url: text(body, "sub_url"),
        news: news_for(previous_signature, &signature),
    }
}

async fn load_record(app: &AppHandle) -> Result<LinkRecord, String> {
    let storage = get_enhanced_storage(app).await?;
    storage
        .get_config::<LinkRecord>(STORAGE_KEY)
        .await
        .map(|value| value.unwrap_or_default())
        .map_err(|_| "storage".to_string())
}

async fn save_record(app: &AppHandle, record: &LinkRecord) -> Result<(), String> {
    let storage = get_enhanced_storage(app).await?;
    storage
        .save_config(STORAGE_KEY, record)
        .await
        .map_err(|_| "storage".to_string())
}

/// Имя компьютера, каким его увидит человек в списке устройств бота.
fn device_name() -> String {
    let host = hostname();
    let system = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "Mac"
    } else {
        "Linux"
    };
    let name = match host {
        Some(host) => format!("{} — {}", system, host),
        None => system.to_string(),
    };
    name.chars().take(64).collect()
}

fn hostname() -> Option<String> {
    for key in ["COMPUTERNAME", "HOSTNAME"] {
        if let Ok(value) = std::env::var(key) {
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Запросить у сервера новый код привязки.
#[tauri::command]
pub async fn device_link_start(app: AppHandle) -> Result<LinkStartResult, String> {
    let mut record = load_record(&app).await?;
    if record.device_id.is_empty() {
        record.device_id = new_device_id();
    }

    let payload = serde_json::json!({
        "device_id": record.device_id,
        "platform": PLATFORM,
        "device_name": device_name(),
        "app_version": env!("CARGO_PKG_VERSION"),
    });

    let response = http_client::get_client()
        .post(backend("/link/start"))
        .json(&payload)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!("привязка: сервер не ответил на запрос кода: {}", error);
            "network".to_string()
        })?;

    if !response.status().is_success() {
        tracing::warn!("привязка: сервер отказал кодом {}", response.status());
        return Err("server".to_string());
    }

    let body: serde_json::Value = response.json().await.map_err(|error| {
        tracing::warn!("привязка: ответ сервера нечитаем: {}", error);
        "server".to_string()
    })?;

    let code = text(&body, "code").ok_or_else(|| "server".to_string())?;
    let secret = text(&body, "secret").ok_or_else(|| "server".to_string())?;
    let poll_interval = body
        .get("poll_interval")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    record.code = code.clone();
    record.secret = secret;
    record.poll_interval = poll_interval;
    // Новый код — новая привязка: прошлое состояние подписки к ней отношения не имеет.
    record.last_poll = 0;
    record.signature = String::new();
    save_record(&app, &record).await?;

    Ok(LinkStartResult {
        code,
        expires_at: text(&body, "expires_at").unwrap_or_default(),
        poll_interval: normalized_interval(poll_interval),
    })
}

/// Подтвердили ли код в боте.
#[tauri::command]
pub async fn device_link_status(app: AppHandle) -> Result<LinkStatusResult, String> {
    let record = load_record(&app).await?;
    if record.code.is_empty() || record.secret.is_empty() {
        return Ok(LinkStatusResult {
            status: "not_found".to_string(),
            sub_url: None,
        });
    }

    let url = format!(
        "{}?code={}&secret={}",
        backend("/link/status"),
        urlencoding::encode(&record.code),
        urlencoding::encode(&record.secret),
    );

    let body: serde_json::Value = http_client::get_json(&url).await.map_err(|error| {
        tracing::warn!("привязка: опрос подтверждения не прошёл: {}", error);
        "network".to_string()
    })?;

    Ok(parse_status(&body))
}

/// Спросить сервер о подписке. `force` — не ждать конца периода.
///
/// Возвращает `None`, когда спрашивать ещё рано: экран в этом случае оставляет
/// прошлое состояние, а не рисует пустое.
#[tauri::command]
pub async fn device_link_poll(
    app: AppHandle,
    force: Option<bool>,
) -> Result<Option<DeviceStateResult>, String> {
    let mut record = load_record(&app).await?;
    if record.code.is_empty() || record.secret.is_empty() {
        return Ok(None);
    }
    if !force.unwrap_or(false)
        && !is_poll_due(record.last_poll, record.poll_interval, now_secs())
    {
        return Ok(None);
    }

    let url = format!(
        "{}?code={}&secret={}",
        backend("/device/state"),
        urlencoding::encode(&record.code),
        urlencoding::encode(&record.secret),
    );

    // Сеть молчит — прошлое состояние не трогаем вовсе: «не дозвонились» это не
    // «подписка кончилась».
    let body: serde_json::Value = http_client::get_json(&url).await.map_err(|error| {
        tracing::warn!("привязка: состояние устройства не пришло: {}", error);
        "network".to_string()
    })?;

    let state = parse_state(&body, &record.signature);

    if !state.linked {
        // Сервер говорит, что такого устройства не знает. Одиночный ответ может быть
        // и сбоем, а привязку человек восстанавливает руками — ждём подтверждения.
        record.misses += 1;
        record.last_poll = now_secs();
        if should_forget(record.misses) {
            record.code = String::new();
            record.secret = String::new();
            record.signature = String::new();
            record.last_poll = 0;
            record.misses = 0;
            save_record(&app, &record).await?;
            return Ok(Some(state));
        }
        save_record(&app, &record).await?;
        return Ok(None);
    }

    record.misses = 0;
    record.last_poll = now_secs();
    record.signature = signature_of(&state.subscription, state.is_trial);
    if let Some(interval) = body.get("poll_interval").and_then(|v| v.as_u64()) {
        record.poll_interval = interval;
    }
    save_record(&app, &record).await?;

    Ok(Some(state))
}

/// Что известно об устройстве без похода в сеть.
#[tauri::command]
pub async fn device_link_snapshot(app: AppHandle) -> Result<LinkSnapshot, String> {
    let record = load_record(&app).await?;
    let linked = !record.code.is_empty() && !record.secret.is_empty();
    Ok(LinkSnapshot {
        linked,
        code: if linked { Some(record.code) } else { None },
    })
}

/// Забыть привязку по просьбе человека. Отпечаток устройства остаётся.
#[tauri::command]
pub async fn device_link_forget(app: AppHandle) -> Result<(), String> {
    let mut record = load_record(&app).await?;
    record.code = String::new();
    record.secret = String::new();
    record.signature = String::new();
    record.last_poll = 0;
    save_record(&app, &record).await
}

#[cfg(test)]
#[path = "device_link.tests.rs"]
mod tests;
