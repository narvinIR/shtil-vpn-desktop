use super::*;

#[test]
fn secure_traffic_still_reaches_the_proxy_over_plain_http() {
    // https:// здесь заставляло клиент начинать TLS-рукопожатие с открытым
    // портом ядра — обновление на Маке не приходило ни разу (07.08.2026).
    assert_eq!(
        proxy_env_value("127.0.0.1", 12080),
        "http://127.0.0.1:12080"
    );
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

/// Windows: свою запись в реестре снимаем, чужую — нет. Другой VPN или
/// корпоративная настройка на этом месте не наша забота: стереть её значит
/// оставить человека с мёртвым браузером в соседней программе.
#[test]
fn windows_touches_only_our_own_registry_record() {
    assert!(is_our_windows_proxy("127.0.0.1:12080", Some(12080)));
    assert!(is_our_windows_proxy(
        "http=127.0.0.1:12080;https=127.0.0.1:12080",
        Some(12080)
    ));

    // чужой клиент на петле — порт не наш
    assert!(!is_our_windows_proxy("127.0.0.1:2080", Some(12080)));
    // корпоративный прокси в сети — тем более
    assert!(!is_our_windows_proxy("proxy.corp.local:3128", Some(12080)));
    // своего порта не знаем — значит и снимать нечего
    assert!(!is_our_windows_proxy("127.0.0.1:12080", None));
    assert!(!is_our_windows_proxy("", Some(12080)));
}

/// Windows: чужую запись запоминаем ДО того, как ляжем поверх. Снятие там уже
/// умело не трогать чужое, а включение затирало без спроса — ровно половина,
/// на которой обожглись на Маке 06.08.2026. У владельца на боевой Windows
/// живёт Throne на 2080: без этого его настройка исчезала бы навсегда.
#[test]
fn foreign_windows_record_is_remembered_before_we_overwrite_it() {
    // строка реестра запоминается целиком: там бывает и составной вид
    assert_eq!(
        windows_foreign_value("127.0.0.1:2080", Some(12080)),
        Some("127.0.0.1:2080".to_string())
    );
    assert_eq!(
        windows_foreign_value("http=127.0.0.1:2080;https=127.0.0.1:2080", Some(12080)),
        Some("http=127.0.0.1:2080;https=127.0.0.1:2080".to_string())
    );
    assert_eq!(
        windows_foreign_value("proxy.corp.local:3128", Some(12080)),
        Some("proxy.corp.local:3128".to_string())
    );

    // своё запоминать нечего
    assert_eq!(windows_foreign_value("127.0.0.1:12080", Some(12080)), None);
    // пустое место — тоже
    assert_eq!(windows_foreign_value("", Some(12080)), None);
}

/// macOS: свой адрес пишем ТОЛЬКО в каналы, по которым машина реально ходит в
/// сеть. Раньше писали во все подряд — и след оставался в спящих каналах, где
/// его потом никто не искал (15.08.2026: Мак владельца полгода гнал трафик в
/// мёртвый порт, диктовка и браузер молчали, «Штиля» на машине уже не было).
#[cfg(target_os = "macos")]
#[test]
fn only_channels_with_a_device_are_written_to() {
    let order = "An asterisk (*) denotes that a network service is disabled.\n\
                 (1) Ethernet\n\
                 (Hardware Port: Ethernet, Device: en0)\n\
                 \n\
                 (2) Thunderbolt Bridge\n\
                 (Hardware Port: Thunderbolt Bridge, Device: bridge0)\n\
                 \n\
                 (3) Wi-Fi\n\
                 (Hardware Port: Wi-Fi, Device: en1)\n\
                 \n\
                 (4) Hiddify\n\
                 (Hardware Port: apple.hiddify.com, Device: )\n";

    assert_eq!(
        parse_service_devices(order),
        vec![
            ("Ethernet".to_string(), "en0".to_string()),
            ("Thunderbolt Bridge".to_string(), "bridge0".to_string()),
            ("Wi-Fi".to_string(), "en1".to_string()),
        ]
    );
}

/// Выключенную человеком службу (со звёздочкой) не трогаем вовсе.
#[cfg(target_os = "macos")]
#[test]
fn service_switched_off_by_the_person_is_skipped() {
    let order = "An asterisk (*) denotes that a network service is disabled.\n\
                 (1) Ethernet\n\
                 (Hardware Port: Ethernet, Device: en0)\n\
                 \n\
                 (*) Wi-Fi\n\
                 (Hardware Port: Wi-Fi, Device: en1)\n";

    assert_eq!(
        parse_service_devices(order),
        vec![("Ethernet".to_string(), "en0".to_string())]
    );
}

/// Кабель без линка пропускать НЕЛЬЗЯ: человек переставляет ноутбук с Wi-Fi на
/// кабель, и запись обязана уже стоять там — иначе трафик пойдёт мимо VPN при
/// зелёном экране «Подключено». Поэтому сетевые карты берём все, а отсекаем
/// только каналы без устройства.
#[cfg(target_os = "macos")]
#[test]
fn every_network_card_keeps_our_record() {
    let order = "An asterisk (*) denotes that a network service is disabled.\n\
                 (1) Ethernet\n\
                 (Hardware Port: Ethernet, Device: en0)\n\
                 \n\
                 (2) Wi-Fi\n\
                 (Hardware Port: Wi-Fi, Device: en1)\n";

    let services: Vec<String> = parse_service_devices(order)
        .into_iter()
        .map(|(service, _)| service)
        .collect();

    assert_eq!(services, vec!["Ethernet".to_string(), "Wi-Fi".to_string()]);
}
