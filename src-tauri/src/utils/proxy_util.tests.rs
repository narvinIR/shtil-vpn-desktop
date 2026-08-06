use super::*;

#[test]
fn enabled_local_record_gives_its_port() {
    let record = "Enabled: Yes\nServer: 127.0.0.1\nPort: 12080\nAuthenticated Proxy Enabled: 0\n";

    assert_eq!(parse_local_proxy_port(record), Some(12080));
}

#[test]
fn disabled_record_is_nothing_to_remove() {
    let record = "Enabled: No\nServer: 127.0.0.1\nPort: 12080\n";

    assert_eq!(parse_local_proxy_port(record), None);
}

#[test]
fn proxy_outside_the_machine_is_not_ours() {
    let record = "Enabled: Yes\nServer: proxy.company.lan\nPort: 3128\n";

    assert_eq!(parse_local_proxy_port(record), None);
}

/// Главное правило. Слепое выключение всех каналов чинило нашу поломку чужими
/// руками: рядом на машине живёт другой VPN-клиент со своим портом, и после
/// выхода из «Штиля» без сети оставался уже он.
#[test]
fn foreign_client_on_the_same_machine_must_stay_untouched() {
    let alien = "Enabled: Yes\nServer: 127.0.0.1\nPort: 2080\n";

    assert!(!is_our_proxy_record(alien, Some(12080)));
}

#[test]
fn our_own_record_is_recognised() {
    let ours = "Enabled: Yes\nServer: 127.0.0.1\nPort: 12080\n";

    assert!(is_our_proxy_record(ours, Some(12080)));
}

/// Не знаем своего порта — не трогаем ничего: лучше оставить запись, чем снять
/// чужую.
#[test]
fn without_our_own_port_nothing_is_ours() {
    let ours = "Enabled: Yes\nServer: 127.0.0.1\nPort: 12080\n";

    assert!(!is_our_proxy_record(ours, None));
}

#[test]
fn remembered_port_is_the_one_we_clean_by() {
    remember_applied_proxy_port(12345);

    assert_eq!(applied_proxy_port(), Some(12345));
}
