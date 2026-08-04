use super::*;
use crate::app::core::tun_profile::default_tun_route_exclude_addresses;
use crate::app::storage::state_model::AppConfig;
use serde_json::Value;

fn assert_inbounds_do_not_contain_legacy_fields(config: &Value) {
    let inbounds = config
        .get("inbounds")
        .and_then(|v| v.as_array())
        .expect("inbounds 应存在");

    for inbound in inbounds {
        for legacy_field in [
            "sniff",
            "sniff_override_destination",
            "sniff_timeout",
            "domain_strategy",
            "udp_disable_domain_unmapping",
        ] {
            assert!(
                inbound.get(legacy_field).is_none(),
                "inbound 不应包含 legacy 字段 {}: {:?}",
                legacy_field,
                inbound
            );
        }
    }
}

fn assert_route_rules_keep_sniff_action(config: &Value) {
    let rules = config
        .get("route")
        .and_then(|v| v.get("rules"))
        .and_then(|v| v.as_array())
        .expect("route.rules 应存在");

    assert!(
        rules
            .iter()
            .any(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("sniff")),
        "route.rules 应保留 sniff action: {:?}",
        rules
    );
}

#[test]
fn generated_dns_servers_should_use_new_format() {
    let config = generate_base_config(&AppConfig::default());
    let servers = config
        .get("dns")
        .and_then(|v| v.get("servers"))
        .and_then(|v| v.as_array())
        .expect("dns.servers 应存在");

    for server in servers {
        assert!(
            server.get("type").and_then(|v| v.as_str()).is_some(),
            "dns server 应包含 type 字段: {:?}",
            server
        );
        assert!(
            server.get("address").is_none(),
            "dns server 不应再输出 legacy address 字段: {:?}",
            server
        );
        assert!(
            server.get("address_resolver").is_none(),
            "dns server 不应再输出 legacy address_resolver 字段: {:?}",
            server
        );
        assert!(
            server.get("strategy").is_none(),
            "dns server 不应包含 strategy 字段（该字段属于 dns 根配置而非 server）: {:?}",
            server
        );
        assert!(
            server.get("domain_strategy").is_none(),
            "dns server 不应包含已弃用的 domain_strategy 字段: {:?}",
            server
        );
        assert!(
            server.get("detour").and_then(|v| v.as_str()) != Some("direct"),
            "dns server 不应显式设置 detour=direct: {:?}",
            server
        );
    }

    let route_default_resolver = config
        .get("route")
        .and_then(|v| v.get("default_domain_resolver"))
        .expect("route.default_domain_resolver 应存在");
    assert_eq!(
        route_default_resolver
            .get("server")
            .and_then(|v| v.as_str()),
        Some(DNS_RESOLVER)
    );
    assert!(route_default_resolver.get("strategy").is_some());
}

#[test]
fn generated_log_should_write_to_kernel_work_dir_file() {
    let config = generate_base_config(&AppConfig::default());
    let log = config
        .get("log")
        .and_then(|v| v.as_object())
        .expect("log 配置应存在");

    assert_eq!(log.get("disabled").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(log.get("level").and_then(|v| v.as_str()), Some("info"));
    assert_eq!(log.get("timestamp").and_then(|v| v.as_bool()), Some(true));

    let output = log
        .get("output")
        .and_then(|v| v.as_str())
        .expect("log.output 应存在");
    assert!(
        output.ends_with("sing-box.log"),
        "log.output 应指向 sing-box.log: {output}"
    );
}

#[test]
fn fake_dns_should_append_fakeip_server_and_enable_reverse_mapping() {
    let app_config = AppConfig {
        singbox_fake_dns_enabled: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let servers = config
        .get("dns")
        .and_then(|v| v.get("servers"))
        .and_then(|v| v.as_array())
        .expect("dns.servers 应存在");

    let fakeip_server = servers
        .iter()
        .find(|server| server.get("tag").and_then(|v| v.as_str()) == Some(DNS_FAKEIP))
        .expect("启用 fake dns 后应包含 fakeip dns server");

    assert_eq!(
        fakeip_server.get("type").and_then(|v| v.as_str()),
        Some("fakeip")
    );
    assert_eq!(
        fakeip_server.get("inet4_range").and_then(|v| v.as_str()),
        Some("198.18.0.0/15")
    );
    assert_eq!(
        fakeip_server.get("inet6_range").and_then(|v| v.as_str()),
        Some("fc00::/18")
    );

    assert_eq!(
        config
            .get("dns")
            .and_then(|v| v.get("reverse_mapping"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        config
            .get("experimental")
            .and_then(|v| v.get("cache_file"))
            .and_then(|v| v.get("store_rdrc"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

/// Подменные адреса выдаются всему, что идёт в туннель.
///
/// Раньше зарубежное отличалось от российского по скачиваемому списку; теперь
/// российские домены отсеиваются правилами выше и решаются настоящим адресом.
#[test]
fn fake_dns_adds_a_catch_all_query_rule() {
    let app_config = AppConfig {
        singbox_fake_dns_enabled: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let rules = config
        .get("dns")
        .and_then(|v| v.get("rules"))
        .and_then(|v| v.as_array())
        .expect("dns.rules 应存在");

    let catch_all = rules.iter().find(|rule| {
        rule.get("server").and_then(|v| v.as_str()) == Some(DNS_FAKEIP)
            && rule.get("rule_set").is_none()
            && rule.get("query_type").is_some()
    });
    assert!(catch_all.is_some(), "нет правила подменных адресов");
}

#[test]
fn generated_inbounds_should_not_use_legacy_fields() {
    let config = generate_base_config(&AppConfig::default());

    assert_inbounds_do_not_contain_legacy_fields(&config);
    assert_route_rules_keep_sniff_action(&config);
}

#[test]
fn generated_tun_inbounds_should_not_use_legacy_fields() {
    let app_config = AppConfig {
        tun_enabled: true,
        tun_enable_ipv6: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let inbounds = config
        .get("inbounds")
        .and_then(|v| v.as_array())
        .expect("inbounds 应存在");

    assert_eq!(inbounds.len(), 2, "启用 TUN 时应生成 mixed + tun 两个入站");
    assert_inbounds_do_not_contain_legacy_fields(&config);
    assert_route_rules_keep_sniff_action(&config);
}

#[test]
fn generated_tun_inbound_should_use_canonical_route_exclude_address_default() {
    let app_config = AppConfig {
        tun_enabled: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let tun_in = config
        .get("inbounds")
        .and_then(|value| value.as_array())
        .and_then(|inbounds| {
            inbounds.iter().find(|inbound| {
                inbound.get("tag").and_then(|value| value.as_str()) == Some("tun-in")
            })
        })
        .expect("tun-in 应存在");

    assert_eq!(
        tun_in.get("route_exclude_address"),
        Some(&serde_json::json!(default_tun_route_exclude_addresses()))
    );
}

#[test]
fn generated_tun_inbound_should_use_explicit_route_exclude_address_override() {
    let app_config = AppConfig {
        tun_enabled: true,
        tun_route_exclude_address: Some(vec!["203.0.113.0/24".to_string()]),
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let tun_in = config
        .get("inbounds")
        .and_then(|value| value.as_array())
        .and_then(|inbounds| {
            inbounds.iter().find(|inbound| {
                inbound.get("tag").and_then(|value| value.as_str()) == Some("tun-in")
            })
        })
        .expect("tun-in 应存在");

    assert_eq!(
        tun_in.get("route_exclude_address"),
        Some(&serde_json::json!(["203.0.113.0/24"]))
    );
}

/// Списки правил не качаются: недоступный адрес роняет старт ядра целиком.
///
/// Форк тянул одиннадцать наборов `.srs` через китайское зеркало
/// `gh-proxy.com`. Из России первоисточник не отвечает с мая 2026, а зеркало
/// вдобавок видит реальный адрес человека — то есть ровно то, от чего он
/// включает VPN. Списки везём с собой, как в боте и на телефоне (правило
/// `no-remote-rulesets-in-config`).
#[test]
fn generated_config_should_not_download_anything_at_startup() {
    let config = generate_base_config(&AppConfig::default());
    let text = serde_json::to_string(&config).expect("конфиг должен сериализоваться");

    for forbidden in [
        "gh-proxy",
        "raw.githubusercontent",
        "\"type\":\"remote\"",
        ".srs",
    ] {
        assert!(
            !text.contains(forbidden),
            "конфиг всё ещё качает списки при старте ({forbidden})"
        );
    }
}

/// Ни одно правило не ссылается на набор, которого нет.
///
/// Висячий тег ядро отвергает вместе со всем конфигом — VPN не включится
/// вовсе. Тест страхует от половинчатой правки: наборы убрали, а правила,
/// которые на них ссылались, забыли.
#[test]
fn no_rule_may_point_to_a_missing_rule_set() {
    let config = generate_base_config(&AppConfig::default());
    let declared: Vec<String> = config
        .get("route")
        .and_then(|route| route.get("rule_set"))
        .and_then(|value| value.as_array())
        .map(|sets| {
            sets.iter()
                .filter_map(|set| set.get("tag").and_then(|tag| tag.as_str()))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let mut used: Vec<String> = Vec::new();
    for section in [
        config.get("route").and_then(|route| route.get("rules")),
        config.get("dns").and_then(|dns| dns.get("rules")),
    ]
    .into_iter()
    .flatten()
    {
        for rule in section.as_array().into_iter().flatten() {
            match rule.get("rule_set") {
                Some(Value::String(tag)) => used.push(tag.clone()),
                Some(Value::Array(tags)) => used.extend(
                    tags.iter()
                        .filter_map(|tag| tag.as_str())
                        .map(str::to_string),
                ),
                _ => {}
            }
        }
    }

    for tag in used {
        assert!(
            declared.contains(&tag),
            "правило ссылается на набор «{tag}», которого нет в route.rule_set"
        );
    }
}

/// Чужой резолвер по умолчанию — это утечка запросов клиента.
///
/// Форк по умолчанию слал прямые запросы в Alibaba (`dns.alidns.com`) и в
/// China Telecom (`114.114.114.114`) — мимо туннеля, с реального адреса.
/// Наш профиль совпадает с ботом: зарубежное имя спрашиваем через туннель,
/// российское — у российского резолвера напрямую.
#[test]
fn default_dns_must_not_leak_to_foreign_resolvers() {
    let config = generate_base_config(&AppConfig::default());
    let text = serde_json::to_string(&config).expect("конфиг должен сериализоваться");

    for forbidden in ["alidns", "114.114.114.114", "223.5.5.5", "119.29.29.29"] {
        assert!(
            !text.contains(forbidden),
            "запросы клиента по умолчанию уходят в чужой резолвер ({forbidden})"
        );
    }
}

/// Заблокированные сайты в зоне `.ru` обязаны идти ЧЕРЕЗ туннель.
///
/// Их 35 — «Новая газета», «Дождь», The Insider и подобные. Правило «всё
/// российское напрямую» отправило бы их мимо туннеля, и человек не открыл бы
/// ровно то, ради чего платит. У бота такое правило стоит первым, здесь тоже.
#[test]
fn blocked_ru_sites_must_go_through_the_tunnel() {
    let config = generate_base_config(&AppConfig::default());
    let rules = config
        .get("route")
        .and_then(|route| route.get("rules"))
        .and_then(|value| value.as_array())
        .expect("route.rules 应存在");

    let position = |needle: &str| {
        rules.iter().position(|rule| {
            rule.get("domain_suffix")
                .and_then(|value| value.as_array())
                .map(|list| list.iter().any(|d| d.as_str() == Some(needle)))
                .unwrap_or(false)
        })
    };

    let blocked = position("novayagazeta.ru").expect("нет правила для заблокированных сайтов");
    let zones = position(".ru").expect("нет правила для российских зон");

    assert!(
        blocked < zones,
        "правило «{}» стоит после правила зон — сайт уйдёт напрямую",
        "novayagazeta.ru"
    );
    assert_eq!(
        rules[blocked].get("outbound").and_then(|v| v.as_str()),
        Some(TAG_MANUAL),
        "заблокированный сайт отправлен не в туннель"
    );
}
