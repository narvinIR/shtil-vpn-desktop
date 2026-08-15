use super::*;

/// Тёмная тема живёт на HUD-материале: он выбран замером 06.08.2026 (32 % света).
#[test]
fn dark_theme_keeps_hud_material() {
    assert_eq!(material_for(true), WindowMaterial::Hud);
}

/// Светлая тема обязана брать ДРУГОЙ материал: HUD у системы всегда тёмный, и
/// светлая тема поверх него выходит серой (замер 15.08.2026: рабочая область
/// 173, а карточки и колонка 224 — «светлая» тема темнее собственных карточек).
#[test]
fn light_theme_takes_light_material() {
    assert_eq!(material_for(false), WindowMaterial::Content);
    assert_ne!(material_for(false), material_for(true));
}
