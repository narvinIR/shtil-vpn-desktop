use super::*;

#[test]
fn secure_traffic_still_reaches_the_proxy_over_plain_http() {
    // https:// здесь заставляло клиент начинать TLS-рукопожатие с открытым
    // портом ядра — обновление на Маке не приходило ни разу (07.08.2026).
    assert_eq!(proxy_env_value("127.0.0.1", 12080), "http://127.0.0.1:12080");
}

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

/// Пока «Штиль» работает, системный прокси обязан быть наш — иначе трафик
/// пойдёт мимо VPN. Но чужую запись мы затираем своей, поэтому её надо
/// запомнить: уходя, вернём человеку ровно то, что стояло до нас.
#[test]
fn foreign_record_is_remembered_before_we_overwrite_it() {
    let alien = "Enabled: Yes\nServer: 127.0.0.1\nPort: 2080\n";

    assert_eq!(
        foreign_proxy_record(alien, Some(12080)),
        Some(ProxyRecord {
            server: "127.0.0.1".to_string(),
            port: 2080,
            enabled: true,
        })
    );
}

/// Своя же запись, оставшаяся выключенной с прошлого запуска, — не чужая.
/// Запомнить её значит вернуть человеку наш мёртвый прокси вместо его.
#[test]
fn our_own_leftover_is_not_a_foreign_record() {
    let ours = "Enabled: No\nServer: 127.0.0.1\nPort: 12080\n";

    assert_eq!(foreign_proxy_record(ours, Some(12080)), None);
}

#[test]
fn empty_record_gives_nothing_to_return() {
    let empty = "Enabled: No\nServer: \nPort: 0\n";

    assert_eq!(foreign_proxy_record(empty, Some(12080)), None);
}

/// Запусков бывает несколько подряд, и во втором на месте чужой записи стоит
/// уже наша. Запоминаем самую первую — она и есть настройка человека.
#[test]
fn first_foreign_record_wins() {
    let mut backup = ProxyBackup::new();
    let alien = ProxyRecord {
        server: "127.0.0.1".to_string(),
        port: 2080,
        enabled: true,
    };
    let later = ProxyRecord {
        server: "127.0.0.1".to_string(),
        port: 9999,
        enabled: true,
    };

    remember_foreign_record(&mut backup, "Ethernet", "web", alien.clone());
    remember_foreign_record(&mut backup, "Ethernet", "web", later);

    assert_eq!(backup.get(&backup_key("Ethernet", "web")), Some(&alien));
}

#[test]
fn our_record_gives_back_what_stood_before_us() {
    let ours = "Enabled: Yes\nServer: 127.0.0.1\nPort: 12080\n";
    let alien = ProxyRecord {
        server: "127.0.0.1".to_string(),
        port: 2080,
        enabled: true,
    };
    let mut backup = ProxyBackup::new();
    remember_foreign_record(&mut backup, "Ethernet", "web", alien.clone());

    assert_eq!(
        cleanup_for(ours, Some(12080), &backup, "Ethernet", "web"),
        Some(ProxyCleanup::Restore(alien))
    );
}

#[test]
fn our_record_without_predecessor_is_simply_turned_off() {
    let ours = "Enabled: Yes\nServer: 127.0.0.1\nPort: 12080\n";

    assert_eq!(
        cleanup_for(ours, Some(12080), &ProxyBackup::new(), "Wi-Fi", "web"),
        Some(ProxyCleanup::TurnOff)
    );
}

/// Главное: чужую запись мы не выключаем и не возвращаем — её не трогали.
#[test]
fn foreign_record_is_left_alone_on_exit() {
    let alien = "Enabled: Yes\nServer: 127.0.0.1\nPort: 2080\n";

    assert_eq!(
        cleanup_for(alien, Some(12080), &ProxyBackup::new(), "Ethernet", "web"),
        None
    );
}
