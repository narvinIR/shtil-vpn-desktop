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

/// Материал стекла под окном. Свой перечень, а не системный: так выбор
/// проверяется тестом на любой системе, а не только на Маке.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMaterial {
    /// HUD — материал системы для тёмных панелей. Всегда тёмный по своей природе.
    Hud,
    /// Фон содержимого окна. Светлеет и темнеет вместе с темой окна.
    Content,
}

/// Материал под тему.
///
/// Тёмная остаётся на HUD: он выбран замером 06.08.2026 (32 % света против 14 %
/// у соседей). Светлой он не годится вовсе — HUD у системы тёмный всегда, и
/// светлая тема поверх него выходила серой: замер 15.08.2026 дал рабочую
/// область 173 при карточках и колонке 224, то есть «светлая» тема была темнее
/// собственных карточек.
pub fn material_for(dark: bool) -> WindowMaterial {
    if dark {
        WindowMaterial::Hud
    } else {
        WindowMaterial::Content
    }
}

/// Навести стекло под окном. Тема задаётся отдельно: при создании окна её ещё
/// никто не знает, поэтому берём тёмную — она в продукте по умолчанию.
#[cfg(target_os = "macos")]
pub fn apply(window: &WebviewWindow) {
    apply_theme(window, true);
}

/// Стекло и светлота окна под выбранную тему.
///
/// Материалу мало быть светлым по имени: систему он спрашивает у самого окна
/// (`effectiveAppearance`). Пока окну не сказано, что оно светлое, любой
/// материал остаётся тёмным — поэтому светлота ставится здесь же.
#[cfg(target_os = "macos")]
pub fn apply_theme(window: &WebviewWindow, dark: bool) {
    use tauri::window::{Effect, EffectState, EffectsBuilder};
    use tauri::Theme;

    let theme = if dark { Theme::Dark } else { Theme::Light };
    if let Err(err) = window.set_theme(Some(theme)) {
        tracing::warn!("не удалось задать светлоту окна: {}", err);
    }

    let effect = match material_for(dark) {
        WindowMaterial::Hud => Effect::HudWindow,
        WindowMaterial::Content => Effect::ContentBackground,
    };

    let effects = EffectsBuilder::new()
        .effect(effect)
        .state(EffectState::FollowsWindowActiveState)
        .radius(MACOS_CORNER_RADIUS)
        .build();

    if let Err(err) = window.set_effects(effects) {
        tracing::warn!("не удалось включить стекло окна: {}", err);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply(_window: &WebviewWindow) {}

#[cfg(not(target_os = "macos"))]
pub fn apply_theme(_window: &WebviewWindow, _dark: bool) {}

/// Экраны сообщают сюда свою тему: окно на Маке светлеет вместе с ними.
#[tauri::command]
pub fn set_window_theme(window: WebviewWindow, dark: bool) {
    apply_theme(&window, dark);
}

#[cfg(test)]
#[path = "window_appearance.tests.rs"]
mod tests;
