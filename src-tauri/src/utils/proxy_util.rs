use std::io;
use std::sync::atomic::{AtomicU16, Ordering};

#[cfg(target_os = "windows")]
use crate::app::constants::registry;
#[cfg(target_os = "windows")]
use tracing::warn;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// 默认的系统代理绕过列表
pub const DEFAULT_BYPASS_LIST: &str =
    "localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;\
172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*";

fn parse_bypass_entries(raw: Option<&str>) -> Vec<String> {
    let source = raw
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(DEFAULT_BYPASS_LIST);

    source
        .split([';', ',', '\n'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Значение для переменных окружения прокси (`http_proxy`, `https_proxy` и родня).
///
/// Схема здесь говорит, КАК соединяться с самим прокси, а не что через него везём.
/// Наш вход на петле — обычный HTTP, поэтому `https://` в `https_proxy` означало
/// TLS-рукопожатие в открытый порт: ядро отвечало `malformed HTTP request`, и всё,
/// что читает переменные окружения, теряло сеть — обновление приложения не
/// приходило вовсе, а проверка связи вечно показывала обрыв (07.08.2026).
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn proxy_env_value(host: &str, port: u16) -> String {
    format!("http://{}:{}", host, port)
}

/// Порт, который приложение прописало в системные настройки последним.
///
/// Нужен, чтобы снимать ТОЛЬКО СВОЮ запись. Рядом на машине живут чужие
/// прокси — другой VPN-клиент, рабочий прокси, — и слепое выключение всех
/// каналов чинит нашу поломку чужими руками: человек остаётся без сети уже
/// из-за нас. Ноль = мы ещё ничего не прописывали.
static APPLIED_PROXY_PORT: AtomicU16 = AtomicU16::new(0);

/// Запомнить порт, который стоит в системе от нашего имени. Зовётся при
/// включении прокси и при старте приложения: после падения память процесса
/// пуста, а запись в настройках сети осталась, и без порта из настроек мы
/// свою запись уже не узнаем.
pub fn remember_applied_proxy_port(port: u16) {
    APPLIED_PROXY_PORT.store(port, Ordering::Relaxed);
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn applied_proxy_port() -> Option<u16> {
    match APPLIED_PROXY_PORT.load(Ordering::Relaxed) {
        0 => None,
        port => Some(port),
    }
}

/// Запись прокси одного канала: то, что показывает `networksetup -getwebproxy`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ProxyRecord {
    server: String,
    port: u16,
    enabled: bool,
}

/// Чужие записи, поверх которых мы легли: ключ «канал|вид» → что там стояло.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
type ProxyBackup = std::collections::BTreeMap<String, ProxyRecord>;

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn backup_key(service: &str, kind: &str) -> String {
    format!("{}|{}", service, kind)
}

/// Что делать с записью на выходе.
#[cfg(any(target_os = "macos", test))]
#[derive(Debug, PartialEq, Eq)]
enum ProxyCleanup {
    /// Вернуть чужую запись — она стояла здесь до нас.
    Restore(ProxyRecord),
    /// До нас на этом месте ничего не было — просто выключить.
    TurnOff,
}

/// Разбор ответа `networksetup -getwebproxy <служба>`. Пустая запись (нет
/// адреса или порт 0) — это «здесь ничего не настроено».
#[cfg(any(target_os = "macos", test))]
fn parse_proxy_record(output: &str) -> Option<ProxyRecord> {
    let mut enabled = false;
    let mut server = String::new();
    let mut port: Option<u16> = None;

    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "Enabled" => enabled = value.eq_ignore_ascii_case("yes"),
            "Server" => server = value.to_string(),
            "Port" => port = value.parse().ok(),
            _ => {}
        }
    }

    let port = port.filter(|value| *value != 0)?;
    if server.is_empty() {
        return None;
    }

    Some(ProxyRecord {
        server,
        port,
        enabled,
    })
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn is_loopback(server: &str) -> bool {
    matches!(server, "127.0.0.1" | "::1" | "localhost")
}

/// Наша ли запись прокси в реестре Windows. Значение бывает голым
/// (`127.0.0.1:12080`) и списком по видам (`http=…;https=…`) — узнаём себя по
/// нашему порту на петле. Всё остальное принадлежит другой программе.
#[cfg(any(target_os = "windows", test))]
fn is_our_windows_proxy(value: &str, our_port: Option<u16>) -> bool {
    let Some(ours) = our_port else {
        return false;
    };

    value.split(';').any(|part| {
        let part = part.trim();
        let part = part.split_once('=').map_or(part, |(_, rest)| rest);
        match part.rsplit_once(':') {
            Some((server, port)) => {
                is_loopback(server.trim()) && port.trim().parse::<u16>().ok() == Some(ours)
            }
            None => false,
        }
    })
}

/// Порт включённой записи, которая ведёт на петлю. Выключенная и чужая по
/// адресу дают `None`.
#[cfg(any(target_os = "macos", test))]
fn parse_local_proxy_port(output: &str) -> Option<u16> {
    let record = parse_proxy_record(output)?;

    if record.enabled && is_loopback(&record.server) {
        Some(record.port)
    } else {
        None
    }
}

/// Чужая запись, которую мы сейчас затрём своей: её надо вернуть на выходе.
/// Своя (наш порт на петле) и пустая — не чужие, возвращать нечего.
#[cfg(any(target_os = "macos", test))]
fn foreign_proxy_record(output: &str, our_port: Option<u16>) -> Option<ProxyRecord> {
    let record = parse_proxy_record(output)?;

    if is_loopback(&record.server) && Some(record.port) == our_port {
        return None;
    }

    Some(record)
}

/// Запоминаем самую первую чужую запись: во втором запуске подряд на её месте
/// стоит уже наша, и перезапись потеряла бы настройку человека.
#[cfg(any(target_os = "macos", test))]
fn remember_foreign_record(
    backup: &mut ProxyBackup,
    service: &str,
    kind: &str,
    record: ProxyRecord,
) {
    backup.entry(backup_key(service, kind)).or_insert(record);
}

/// Что делать с записью канала на выходе. Чужую не трогаем вовсе.
#[cfg(any(target_os = "macos", test))]
fn cleanup_for(
    output: &str,
    our_port: Option<u16>,
    backup: &ProxyBackup,
    service: &str,
    kind: &str,
) -> Option<ProxyCleanup> {
    if !is_our_proxy_record(output, our_port) {
        return None;
    }

    match backup.get(&backup_key(service, kind)) {
        Some(previous) => Some(ProxyCleanup::Restore(previous.clone())),
        None => Some(ProxyCleanup::TurnOff),
    }
}

/// Наша ли это запись в настройках сети.
#[cfg(any(target_os = "macos", test))]
fn is_our_proxy_record(output: &str, our_port: Option<u16>) -> bool {
    match (parse_local_proxy_port(output), our_port) {
        (Some(port), Some(ours)) => port == ours,
        _ => false,
    }
}

/// 禁用系统代理 (跨平台实现)
pub fn disable_system_proxy() -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        disable_system_proxy_windows()
    }

    #[cfg(target_os = "linux")]
    {
        disable_system_proxy_linux()
    }

    #[cfg(target_os = "macos")]
    {
        disable_system_proxy_macos()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok(()) // 其他平台暂时不执行任何操作
    }
}

/// 启用系统代理 (跨平台实现)
pub fn enable_system_proxy(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    // Своё имя в системе запоминаем до записи: снимать потом будем по нему.
    remember_applied_proxy_port(port);

    #[cfg(target_os = "windows")]
    {
        enable_system_proxy_windows(host, port, bypass)
    }

    #[cfg(target_os = "linux")]
    {
        enable_system_proxy_linux(host, port, bypass)
    }

    #[cfg(target_os = "macos")]
    {
        enable_system_proxy_macos(host, port, bypass)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok(()) // 其他平台暂时不执行任何操作
    }
}

#[cfg(target_os = "windows")]
fn disable_system_proxy_windows() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings =
        hkcu.open_subkey_with_flags(registry::INTERNET_SETTINGS, KEY_READ | KEY_WRITE)?;

    // Рядом живёт другой VPN-клиент или корпоративная настройка. Стереть чужую
    // запись — значит оставить человека с мёртвым браузером в соседней
    // программе, причём причины он не увидит. Снимаем только свою, узнавая её
    // по нашему порту на петле.
    let current: String = settings
        .get_value(registry::PROXY_SERVER)
        .unwrap_or_default();
    if !current.is_empty() && !is_our_windows_proxy(&current, applied_proxy_port()) {
        tracing::info!("системный прокси не наш — оставляем запись как есть");
        return Ok(());
    }

    // Уходя, возвращаем чужую запись, которую собой закрыли. Не было такой —
    // тогда просто выключаем свою.
    match read_foreign_backup().get(&backup_key("windows", "proxy")) {
        Some(previous) => {
            settings.set_value(registry::PROXY_SERVER, &previous.server)?;
            settings.set_value(registry::PROXY_ENABLE, &1u32)?;
            tracing::info!("чужая запись прокси возвращена: реестр → {}", previous.server);
        }
        None => {
            settings.set_value(registry::PROXY_ENABLE, &0u32)?;
            settings.set_value(registry::PROXY_SERVER, &"")?;
        }
    }

    clear_foreign_backup();

    // 通知 WinINet 重新读取设置，使基于 WinINet 的应用（Edge/Chrome/IE 等）立即生效。
    notify_wininet_change();

    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_system_proxy_linux() -> io::Result<()> {
    // Linux下的系统代理设置通常通过环境变量
    // 这里可以尝试使用gsettings或者直接设置环境变量
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    std::env::remove_var("HTTP_PROXY");
    std::env::remove_var("HTTPS_PROXY");
    std::env::remove_var("all_proxy");
    std::env::remove_var("ALL_PROXY");
    std::env::remove_var("no_proxy");
    std::env::remove_var("NO_PROXY");

    // 尝试使用gsettings重置代理设置 (GNOME/Unity/XFCE等)
    if std::process::Command::new("which")
        .arg("gsettings")
        .output()
        .is_ok()
    {
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.http", "host", "''"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.http", "port", "0"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.https", "host", "''"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.https", "port", "0"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy", "mode", "'none'"])
            .output();
    }

    // 尝试使用kwriteconfig5/6重置代理设置 (KDE Plasma)
    for kwriteconfig in &["kwriteconfig6", "kwriteconfig5"] {
        if std::process::Command::new("which")
            .arg(kwriteconfig)
            .output()
            .is_ok()
        {
            // 设置代理模式为无代理 (0)
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "ProxyType",
                    "0",
                ])
                .output();

            // 通知KDE配置已更改
            let _ = std::process::Command::new("dbus-send")
                .args([
                    "--type=signal",
                    "/KIO/Scheduler",
                    "org.kde.KIO.Scheduler.reparseSlaveConfiguration",
                    "string:''",
                ])
                .output();
            break;
        }
    }

    Ok(())
}

/// Виды записей прокси: как прочитать, как записать адрес, как переключить.
#[cfg(target_os = "macos")]
const PROXY_KINDS: [(&str, &str, &str, &str); 3] = [
    ("web", "-getwebproxy", "-setwebproxy", "-setwebproxystate"),
    (
        "secure",
        "-getsecurewebproxy",
        "-setsecurewebproxy",
        "-setsecurewebproxystate",
    ),
    (
        "socks",
        "-getsocksfirewallproxy",
        "-setsocksfirewallproxy",
        "-setsocksfirewallproxystate",
    ),
];

/// Что стояло в реестре Windows до нас. Значение берём целиком: там бывает и
/// простой вид `host:port`, и составной `http=…;https=…`, а вернуть человеку
/// надо ровно его настройку, а не наш пересказ.
#[cfg(any(target_os = "windows", test))]
fn windows_foreign_value(current: &str, our_port: Option<u16>) -> Option<String> {
    if current.is_empty() || is_our_windows_proxy(current, our_port) {
        return None;
    }

    Some(current.to_string())
}

/// Пары «служба — устройство» из ответа `networksetup -listnetworkserviceorder`.
///
/// Служба без устройства (виртуальный канал чужого VPN-клиента) и выключенная
/// человеком (в ответе помечена звёздочкой) сюда не попадают.
#[cfg(target_os = "macos")]
fn parse_service_devices(order_output: &str) -> Vec<(String, String)> {
    let mut services = Vec::new();
    let mut pending: Option<String> = None;

    for line in order_output.lines() {
        let line = line.trim();

        if line.starts_with("(Hardware Port:") {
            let device = line
                .rsplit_once("Device:")
                .map(|(_, tail)| tail.trim_end_matches(')').trim())
                .unwrap_or_default();
            if let (Some(service), false) = (pending.take(), device.is_empty()) {
                services.push((service, device.to_string()));
            }
            continue;
        }

        if let Some((marker, name)) = line.strip_prefix('(').and_then(|r| r.split_once(") ")) {
            pending = (marker != "*").then(|| name.trim().to_string());
        }
    }

    services
}

/// Каналы, в которые мы имеем право прописать свой адрес: сетевые карты
/// машины.
///
/// Только сюда — но во ВСЕ, а не в один активный: человек переставляет ноутбук
/// с Wi-Fi на кабель, и запись обязана уже стоять там, иначе трафик молча
/// пойдёт мимо VPN при зелёном экране. А вот канал без устройства — это
/// виртуальная служба чужого VPN-клиента: там наша запись бессмысленна с
/// первой секунды и переживает нас насовсем.
#[cfg(target_os = "macos")]
fn services_to_apply() -> Vec<String> {
    let Ok(output) = std::process::Command::new("networksetup")
        .args(["-listnetworkserviceorder"])
        .output()
    else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    parse_service_devices(&String::from_utf8_lossy(&output.stdout))
        .into_iter()
        .map(|(service, _)| service)
        .collect()
}

/// Где лежат запомненные чужие записи. На диске, а не в памяти: приложение
/// может упасть, а вернуть человеку его настройку надо в любом случае.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn foreign_backup_path() -> std::path::PathBuf {
    std::path::PathBuf::from(crate::utils::app_util::get_work_dir_sync()).join("foreign_proxy.json")
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn read_foreign_backup() -> ProxyBackup {
    std::fs::read_to_string(foreign_backup_path())
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn write_foreign_backup(backup: &ProxyBackup) {
    match serde_json::to_string_pretty(backup) {
        Ok(raw) => {
            if let Err(e) = std::fs::write(foreign_backup_path(), raw) {
                tracing::warn!("не удалось запомнить чужие записи прокси: {}", e);
            }
        }
        Err(e) => tracing::warn!("не удалось собрать список чужих записей прокси: {}", e),
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn clear_foreign_backup() {
    let _ = std::fs::remove_file(foreign_backup_path());
}

#[cfg(target_os = "macos")]
fn disable_system_proxy_macos() -> io::Result<()> {
    // Прокси прописывается на все каналы разом, поэтому и снимать надо со всех:
    // забытая запись на неактивном канале потом немеет браузером. Но трогаем
    // ТОЛЬКО свою, а на её место возвращаем чужую, которую собой закрыли:
    // рядом живёт другой VPN-клиент, и без возврата человек уходит с мёртвым
    // браузером, не видя причины.
    let our_port = applied_proxy_port();
    let backup = read_foreign_backup();

    let output = std::process::Command::new("networksetup")
        .args(["-listallnetworkservices"])
        .output()?;

    if output.status.success() {
        let services = String::from_utf8_lossy(&output.stdout);

        // 跳过第一行（标题行），处理每个网络服务
        for line in services.lines().skip(1) {
            let service = line.trim();
            if service.is_empty() || service == "*" {
                continue;
            }

            for (kind, read, set_value, set_state) in PROXY_KINDS {
                let Ok(current) = std::process::Command::new("networksetup")
                    .args([read, service])
                    .output()
                else {
                    continue;
                };

                let record = String::from_utf8_lossy(&current.stdout);
                match cleanup_for(&record, our_port, &backup, service, kind) {
                    Some(ProxyCleanup::Restore(previous)) => {
                        let _ = std::process::Command::new("networksetup")
                            .args([
                                set_value,
                                service,
                                &previous.server,
                                &previous.port.to_string(),
                            ])
                            .output();
                        let _ = std::process::Command::new("networksetup")
                            .args([
                                set_state,
                                service,
                                if previous.enabled { "on" } else { "off" },
                            ])
                            .output();
                        tracing::info!(
                            "чужая запись прокси возвращена: {} / {} → {}:{}",
                            service,
                            kind,
                            previous.server,
                            previous.port
                        );
                    }
                    Some(ProxyCleanup::TurnOff) => {
                        let _ = std::process::Command::new("networksetup")
                            .args([set_state, service, "off"])
                            .output();
                    }
                    None => {}
                }
            }
        }
    }

    clear_foreign_backup();

    // 同时清除环境变量
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    std::env::remove_var("HTTP_PROXY");
    std::env::remove_var("HTTPS_PROXY");
    std::env::remove_var("all_proxy");
    std::env::remove_var("ALL_PROXY");

    Ok(())
}

#[cfg(target_os = "windows")]
fn enable_system_proxy_windows(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings =
        hkcu.open_subkey_with_flags(registry::INTERNET_SETTINGS, KEY_READ | KEY_WRITE)?;

    // На этом месте могла стоять чужая запись — другой VPN-клиент или рабочий
    // прокси. Запоминаем ДО того, как ляжем поверх: на выходе вернём ровно её,
    // иначе человек остаётся с мёртвым браузером в соседней программе.
    let current: String = settings
        .get_value(registry::PROXY_SERVER)
        .unwrap_or_default();
    if let Some(foreign) = windows_foreign_value(&current, applied_proxy_port()) {
        let mut backup = read_foreign_backup();
        backup
            .entry(backup_key("windows", "proxy"))
            .or_insert(ProxyRecord {
                server: foreign.clone(),
                port: 0,
                enabled: true,
            });
        write_foreign_backup(&backup);
        tracing::info!("чужая запись прокси запомнена: реестр → {}", foreign);
    }

    // 设置代理服务器地址
    let proxy_server = format!("{}:{}", host, port);
    settings.set_value(registry::PROXY_SERVER, &proxy_server)?;

    // 启用代理
    settings.set_value(registry::PROXY_ENABLE, &1u32)?;

    // 设置绕过本地地址
    let entries = parse_bypass_entries(bypass);
    let override_value = if entries.is_empty() {
        DEFAULT_BYPASS_LIST.to_string()
    } else {
        entries.join(";")
    };
    settings.set_value(registry::PROXY_OVERRIDE, &override_value)?;

    // 通知 WinINet 重新读取设置，使基于 WinINet 的应用（Edge/Chrome/IE 等）立即生效。
    notify_wininet_change();

    Ok(())
}

/// 通知 WinINet 配置已变更并刷新，使系统代理设置即时生效。
///
/// 仅写注册表而不调用本函数时，基于 WinINet 的应用（Edge/Chrome/IE/资源管理器等）
/// 不会主动重新读取代理设置，表现为“代理已写入但浏览器仍连不上”，需要手动重启才能恢复。
///
/// 这里调用两个选项：`INTERNET_OPTION_SETTINGS_CHANGED`（通知设置变更）+
/// `INTERNET_OPTION_REFRESH`（强制重新读取）。任一失败仅记录警告，不阻断代理写入。
///
/// 实现说明：直接用 FFI 声明 + `#[link(name = "wininet")]` 链接系统库，
/// 避免引入 `windows` crate 的庞大 Win32 feature（会拖慢甚至拖崩编译器）。
#[cfg(target_os = "windows")]
fn notify_wininet_change() {
    // Win32 常量（来自 wininet.h）。
    const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
    const INTERNET_OPTION_REFRESH: u32 = 37;

    #[link(name = "wininet")]
    extern "system" {
        fn InternetSetOptionW(
            hinternet: *mut std::ffi::c_void,
            option: u32,
            buffer: *mut std::ffi::c_void,
            buffer_length: u32,
        ) -> i32;
    }

    // SAFETY: INTERNET_OPTION_SETTINGS_CHANGED / REFRESH 不需要 buffer
    // （lpBuffer=NULL，dwBufferLength=0），是 Win32 API 文档允许的用法；
    // hInternet 对这两个选项也必须为 NULL。函数无不可控副作用。
    unsafe {
        let hinternet = std::ptr::null_mut::<std::ffi::c_void>();
        let buffer = std::ptr::null_mut::<std::ffi::c_void>();

        if InternetSetOptionW(hinternet, INTERNET_OPTION_SETTINGS_CHANGED, buffer, 0) == 0 {
            warn!("InternetSetOptionW(SETTINGS_CHANGED) 返回 0");
        }

        if InternetSetOptionW(hinternet, INTERNET_OPTION_REFRESH, buffer, 0) == 0 {
            warn!("InternetSetOptionW(REFRESH) 返回 0");
        }
    }
}

#[cfg(target_os = "linux")]
fn enable_system_proxy_linux(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    let proxy_url = proxy_env_value(host, port);
    let entries = parse_bypass_entries(bypass);
    let no_proxy = if entries.is_empty() {
        DEFAULT_BYPASS_LIST.replace(';', ",")
    } else {
        entries.join(",")
    };

    // 设置环境变量
    std::env::set_var("http_proxy", &proxy_url);
    std::env::set_var("https_proxy", &proxy_url);
    std::env::set_var("HTTP_PROXY", &proxy_url);
    std::env::set_var("HTTPS_PROXY", &proxy_url);
    std::env::set_var("all_proxy", &proxy_url);
    std::env::set_var("ALL_PROXY", &proxy_url);
    std::env::set_var("no_proxy", &no_proxy);
    std::env::set_var("NO_PROXY", &no_proxy);

    // 尝试使用gsettings设置代理 (GNOME/Unity/XFCE等)
    if std::process::Command::new("which")
        .arg("gsettings")
        .output()
        .is_ok()
    {
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.http", "host", host])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args([
                "set",
                "org.gnome.system.proxy.http",
                "port",
                &port.to_string(),
            ])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.https", "host", host])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args([
                "set",
                "org.gnome.system.proxy.https",
                "port",
                &port.to_string(),
            ])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy", "mode", "'manual'"])
            .output();
    }

    // 尝试使用kwriteconfig5/6设置代理 (KDE Plasma)
    for kwriteconfig in &["kwriteconfig6", "kwriteconfig5"] {
        if std::process::Command::new("which")
            .arg(kwriteconfig)
            .output()
            .is_ok()
        {
            let proxy_url = format!("http://{}:{}", host, port);

            // 设置HTTP代理
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "httpProxy",
                    &proxy_url,
                ])
                .output();

            // 设置HTTPS代理
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "httpsProxy",
                    &proxy_url,
                ])
                .output();

            // 设置代理模式为手动 (1)
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "ProxyType",
                    "1",
                ])
                .output();

            // 通知KDE配置已更改
            let _ = std::process::Command::new("dbus-send")
                .args([
                    "--type=signal",
                    "/KIO/Scheduler",
                    "org.kde.KIO.Scheduler.reparseSlaveConfiguration",
                    "string:''",
                ])
                .output();
            break;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn enable_system_proxy_macos(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    // Пока «Штиль» работает, системный прокси обязан быть наш — иначе трафик
    // пойдёт мимо VPN. Но на этом месте могла стоять чужая запись (другой
    // VPN-клиент, рабочий прокси), и мы её собой закрываем. Запоминаем ДО
    // записи: на выходе вернём ровно то, что стояло.
    let mut backup = read_foreign_backup();

    // Только сетевые карты машины: виртуальный канал чужого клиента не наш.
    let entries = parse_bypass_entries(bypass);

    for service in services_to_apply() {
        let service = service.as_str();

        // HTTP и HTTPS: запоминаем чужое, потом кладём своё
        for (kind, read, set_value, set_state) in PROXY_KINDS.into_iter().take(2) {
            if let Ok(current) = std::process::Command::new("networksetup")
                .args([read, service])
                .output()
            {
                let record = String::from_utf8_lossy(&current.stdout);
                if let Some(foreign) = foreign_proxy_record(&record, Some(port)) {
                    tracing::info!(
                        "чужая запись прокси запомнена: {} / {} → {}:{}",
                        service,
                        kind,
                        foreign.server,
                        foreign.port
                    );
                    remember_foreign_record(&mut backup, service, kind, foreign);
                }
            }

            let _ = std::process::Command::new("networksetup")
                .args([set_value, service, host, &port.to_string()])
                .output();

            let _ = std::process::Command::new("networksetup")
                .args([set_state, service, "on"])
                .output();
        }

        // 设置代理绕过列表
        if !entries.is_empty() {
            let mut cmd = std::process::Command::new("networksetup");
            cmd.args(["-setproxybypassdomains", service]);
            for entry in &entries {
                cmd.arg(entry);
            }
            let _ = cmd.output();
        }
    }

    write_foreign_backup(&backup);

    // 同时设置环境变量
    let proxy_url = proxy_env_value(host, port);

    std::env::set_var("http_proxy", &proxy_url);
    std::env::set_var("https_proxy", &proxy_url);
    std::env::set_var("HTTP_PROXY", &proxy_url);
    std::env::set_var("HTTPS_PROXY", &proxy_url);
    std::env::set_var("all_proxy", &proxy_url);
    std::env::set_var("ALL_PROXY", &proxy_url);

    Ok(())
}

#[cfg(test)]
#[path = "proxy_util.tests.rs"]
mod tests;
