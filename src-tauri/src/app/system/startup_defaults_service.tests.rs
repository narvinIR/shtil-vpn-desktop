use super::*;

#[test]
fn version_should_be_parsed_by_three_numbers() {
    assert_eq!(parse_version("1.1.1"), Some((1, 1, 1)));
    assert_eq!(parse_version("1.2.0"), Some((1, 2, 0)));
    assert_eq!(parse_version("10.0.3"), Some((10, 0, 3)));
    assert_eq!(parse_version("не версия"), None);
    assert_eq!(parse_version("1.2"), None);
}

#[test]
fn old_installation_without_vpn_should_be_moved_to_system_mode() {
    // До 1.2.0 значением по умолчанию был ручной режим: приложение показывало
    // «Подключено», а трафик системы шёл мимо VPN.
    assert!(needs_system_mode(None, false, false));
    assert!(needs_system_mode(Some("1.1.1"), false, false));
    assert!(needs_system_mode(Some("1.0.0"), false, false));
}

#[test]
fn working_setup_should_be_left_alone() {
    assert!(!needs_system_mode(Some("1.1.1"), true, false));
    assert!(!needs_system_mode(Some("1.1.1"), false, true));
    assert!(!needs_system_mode(None, true, false));
}

#[test]
fn choice_made_after_the_switch_should_survive() {
    // Человек сам вернулся в ручной режим уже на 1.2.0 — второй раз не переводим.
    assert!(!needs_system_mode(Some("1.2.0"), false, false));
    assert!(!needs_system_mode(Some("1.3.7"), false, false));
}

#[test]
fn unreadable_version_should_not_repeat_the_move() {
    // Строка испорчена: считаем, что перевод уже был, иначе он повторялся бы
    // при каждом запуске и перебивал выбор человека.
    assert!(!needs_system_mode(Some("мусор"), false, false));
}

// Перевод путей со старой папки на нашу проверяется там, где он теперь живёт, —
// `utils/app_util.tests.rs`: его зовёт и старт, и каждое чтение пути ключа.

#[test]
fn leftover_proxy_should_be_cleared_only_when_it_is_ours() {
    // Ядро на старте уже снято, поэтому наш системный прокси осиротел.
    assert!(should_clear_leftover_proxy(true));
    // В ручном режиме прокси в системе не наш — чужую настройку не трогаем.
    assert!(!should_clear_leftover_proxy(false));
}
