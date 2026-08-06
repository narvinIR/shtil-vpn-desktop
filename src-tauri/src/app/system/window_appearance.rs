//! Внешний вид окна под систему.
//!
//! Окно у нас без украшений системы (`decorations: false`), поэтому на macOS
//! скруглять углы и наводить стекло приходится самим. Под содержимым включаем
//! родной материал системы (NSVisualEffectView), а сам webview делаем
//! прозрачным — иначе он закрасил бы стекло и углы остались бы квадратными.
//!
//! Одного материала мало: страница сверху тоже обязана быть прозрачной.
//! Фон `<body>` кладёт naive-ui собственной строкой стиля прямо в элементе, и
//! её не перебить ни одним правилом таблицы — значение приходит из палитры
//! (`--body-bg` в `tokens.css`).
//!
//! На Windows и Linux ничего не делаем: там окно рисуется как раньше.

use tauri::WebviewWindow;

/// Скругление окна macOS в точках. Тем же числом скруглён корень интерфейса
/// (`--window-radius` в `tokens.css`) — иначе содержимое вылезет из-под угла.
pub const MACOS_CORNER_RADIUS: f64 = 12.0;

#[cfg(target_os = "macos")]
pub fn apply(window: &WebviewWindow) {
    use tauri::window::{Effect, EffectState, EffectsBuilder};

    // Материал выбран замером на живой машине (06.08.2026): одно и то же окно
    // над светлым и над тёмным местом рабочего стола. Света сквозь окно
    // проходит: `Sidebar` — 14 %, `HudWindow` — 32 %. Берём второй: рабочий
    // стол за окном виден, а текст читается и в тёмной теме, и в светлой.
    let effects = EffectsBuilder::new()
        .effect(Effect::HudWindow)
        .state(EffectState::FollowsWindowActiveState)
        .radius(MACOS_CORNER_RADIUS)
        .build();

    if let Err(err) = window.set_effects(effects) {
        tracing::warn!("не удалось включить стекло окна: {}", err);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply(_window: &WebviewWindow) {}
