use super::model::TrayProxyMode;
use tauri::image::Image;

/// Знак «Штиля» для строки меню: белый силуэт на прозрачном, без плашки.
/// Собирается `scripts/brand/make_desktop_icons.py` из общего описания знака —
/// руками его не рисуют, иначе разъедется с телефоном.
const GLYPH: &[u8] = include_bytes!("../../../icons/tray.png");

type Rgb = [u8; 3];

/// Цвета — из палитры продукта (`src/assets/tokens.css`). Красного в «Штиле»
/// нет нигде, кроме настоящей ошибки: прежний значок красил знак в `#dc2638`,
/// и в трее это читалось как «сломалось», а не «работает».
const OFF: Rgb = [155, 170, 196]; // ink-400 — защита выключена
const SYSTEM: Rgb = [79, 146, 255]; // azure-500 — системный прокси
const TUN: Rgb = [70, 209, 137]; // green-400 — весь трафик в туннеле

/// Выключенное состояние приглушаем силой знака, а не цветом: на macOS значок
/// шаблонный (система красит его сама под светлую и тёмную строку меню), и до
/// неё доходит только прозрачность.
const OFF_STRENGTH: f32 = 0.45;

pub fn tray_icon_for_mode(mode: TrayProxyMode) -> Option<Image<'static>> {
    let base = Image::from_bytes(GLYPH).ok()?;
    Some(paint(&base, tint(mode), strength(mode)))
}

fn tint(mode: TrayProxyMode) -> Rgb {
    match mode {
        TrayProxyMode::Manual => OFF,
        TrayProxyMode::System => SYSTEM,
        TrayProxyMode::Tun => TUN,
    }
}

fn strength(mode: TrayProxyMode) -> f32 {
    match mode {
        TrayProxyMode::Manual => OFF_STRENGTH,
        _ => 1.0,
    }
}

/// Красит силуэт: цвет ставим свой, форму берём из прозрачности исходника.
fn paint(base: &Image<'_>, color: Rgb, strength: f32) -> Image<'static> {
    let mut painted = Vec::with_capacity(base.rgba().len());
    for pixel in base.rgba().chunks_exact(4) {
        let alpha = (f32::from(pixel[3]) * strength).round().clamp(0.0, 255.0) as u8;
        painted.extend_from_slice(&color);
        painted.push(alpha);
    }

    Image::new_owned(painted, base.width(), base.height())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn max_alpha(icon: &Image<'_>) -> u8 {
        icon.rgba()
            .chunks_exact(4)
            .map(|pixel| pixel[3])
            .max()
            .unwrap_or(0)
    }

    fn icon(mode: TrayProxyMode) -> Image<'static> {
        tray_icon_for_mode(mode).expect("знак трея должен собираться")
    }

    #[test]
    fn glyph_decodes_and_keeps_size() {
        let base = Image::from_bytes(GLYPH).expect("знак должен читаться");
        let painted = icon(TrayProxyMode::System);

        assert!(base.width() > 0 && base.height() > 0);
        assert_eq!(painted.width(), base.width());
        assert_eq!(painted.height(), base.height());
        assert_eq!(painted.rgba().len(), base.rgba().len());
    }

    #[test]
    fn every_mode_paints_its_own_colour() {
        let system = icon(TrayProxyMode::System);
        let tun = icon(TrayProxyMode::Tun);
        let manual = icon(TrayProxyMode::Manual);

        assert_ne!(system.rgba(), tun.rgba());
        assert_ne!(system.rgba(), manual.rgba());
        assert_eq!(&system.rgba()[0..3], &SYSTEM);
        assert_eq!(&tun.rgba()[0..3], &TUN);
        assert_eq!(&manual.rgba()[0..3], &OFF);
    }

    #[test]
    fn disabled_mode_is_dimmer() {
        let manual = max_alpha(&icon(TrayProxyMode::Manual));
        let system = max_alpha(&icon(TrayProxyMode::System));

        assert!(manual > 0, "выключенный знак всё равно виден");
        assert!(
            manual < system,
            "выключенный знак бледнее включённого: {manual} против {system}"
        );
    }

    #[test]
    fn transparent_pixels_stay_transparent() {
        let base = Image::from_bytes(GLYPH).expect("знак должен читаться");
        let painted = icon(TrayProxyMode::System);

        for (from, to) in base
            .rgba()
            .chunks_exact(4)
            .zip(painted.rgba().chunks_exact(4))
        {
            if from[3] == 0 {
                assert_eq!(to[3], 0, "пустое место не должно закраситься");
            }
        }
    }
}
