use super::*;
use crate::app::core::tun_profile::default_tun_route_exclude_addresses;
use crate::app::storage::state_model::AppConfig;
use serde_json::{json, Value};

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

#[test]
fn apply_app_settings_should_remove_legacy_inbound_fields_from_existing_config() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 7890,
                "sniff": true,
                "sniff_override_destination": true,
                "sniff_timeout": "1s",
                "domain_strategy": "prefer_ipv4",
                "udp_disable_domain_unmapping": true
            }
        ],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    assert_inbounds_do_not_contain_legacy_fields(&config);
}

#[test]
fn apply_app_settings_should_add_kernel_log_output_to_existing_config() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [],
            "final": "direct",
            "auto_detect_interface": true
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    let log = config
        .get("log")
        .and_then(|v| v.as_object())
        .expect("应补充 log 配置");
    assert_eq!(log.get("disabled").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(log.get("level").and_then(|v| v.as_str()), Some("info"));
    assert_eq!(log.get("timestamp").and_then(|v| v.as_bool()), Some(true));

    let output = log
        .get("output")
        .and_then(|v| v.as_str())
        .expect("应补充 log.output");
    assert!(
        output.ends_with("sing-box.log"),
        "log.output 应指向 sing-box.log: {output}"
    );
}

#[test]
fn apply_app_settings_should_not_reintroduce_legacy_fields_when_tun_enabled() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        tun_enable_ipv6: true,
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

    let inbounds = config
        .get("inbounds")
        .and_then(|v| v.as_array())
        .expect("inbounds 应存在");
    assert_eq!(inbounds.len(), 2, "启用 TUN 时应生成 mixed + tun 两个入站");
    assert_inbounds_do_not_contain_legacy_fields(&config);
}

#[test]
fn apply_app_settings_should_insert_sniff_route_for_legacy_configs() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 7890,
                "sniff": true,
                "sniff_override_destination": true
            }
        ],
        "route": {
            "rule_set": [],
            "rules": [],
            "final": "direct",
            "auto_detect_interface": true
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    let rules = config
        .get("route")
        .and_then(|v| v.get("rules"))
        .and_then(|v| v.as_array())
        .expect("route.rules 应存在");
    assert!(
        rules
            .iter()
            .any(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("sniff")),
        "旧配置迁移后应补充 sniff 规则: {:?}",
        rules
    );
}

#[test]
fn apply_app_settings_should_use_explicit_tun_route_exclude_address_override() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        tun_route_exclude_address: Some(vec!["203.0.113.0/24".to_string()]),
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

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
        Some(&json!(["203.0.113.0/24"]))
    );
}

#[test]
fn apply_app_settings_should_preserve_existing_tun_route_exclude_address_when_override_absent() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [
            {
                "type": "tun",
                "tag": "tun-in",
                "route_exclude_address": ["198.51.100.0/24"]
            }
        ],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

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
        Some(&json!(["198.51.100.0/24"]))
    );
}

#[test]
fn apply_app_settings_should_fallback_to_canonical_tun_route_exclude_address_default() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

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
        Some(&json!(default_tun_route_exclude_addresses()))
    );
}

/// Чужой конфиг со скачиваемыми наборами приходит к нам обеззараженным.
///
/// Настройки накладываются на ЛЮБОЙ конфиг — и на нашу подписку, и на ключ,
/// принесённый человеком со стороны. Если оставить в нём набор с чужого
/// адреса, ядро полезет за ним при старте и не поднимется вовсе, когда адрес
/// не отвечает. Приложение на телефоне поступает так же (`RuRouting.apply`).
#[test]
fn foreign_config_loses_its_downloadable_rule_sets() {
    let mut config = json!({
        "route": {
            "rule_set": [
                {
                    "tag": "geosite-cn",
                    "type": "remote",
                    "format": "binary",
                    "url": "https://gh-proxy.com/https://raw.githubusercontent.com/x/geosite-cn.srs",
                    "download_detour": "direct"
                },
                { "tag": "мой-список", "type": "local", "format": "binary", "path": "a.srs" }
            ],
            "rules": [
                { "rule_set": "geosite-cn", "outbound": "direct" },
                { "rule_set": ["geosite-cn", "мой-список"], "outbound": "direct" },
                { "rule_set": "мой-список", "outbound": "direct" }
            ],
            "final": "manual"
        },
        "dns": {
            "servers": [],
            "rules": [{ "rule_set": "geosite-cn", "server": "dns-direct" }]
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    let text = serde_json::to_string(&config).expect("конфиг должен сериализоваться");
    assert!(
        !text.contains("geosite-cn") && !text.contains("gh-proxy"),
        "скачиваемый набор остался в конфиге: {text}"
    );
    assert!(
        text.contains("мой-список"),
        "выбросили лишнее: местные наборы трогать нельзя"
    );
}

/// Правило, которому оставили только чужой набор, уходит вместе с ним.
#[test]
fn rules_that_lose_their_only_rule_set_are_dropped() {
    let mut config = json!({
        "route": {
            "rule_set": [{
                "tag": "geosite-ads",
                "type": "remote",
                "format": "binary",
                "url": "https://example.invalid/ads.srs",
                "download_detour": "direct"
            }],
            "rules": [{ "rule_set": "geosite-ads", "action": "reject" }],
            "final": "manual"
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    let rules = config
        .get("route")
        .and_then(|route| route.get("rules"))
        .and_then(|value| value.as_array())
        .expect("route.rules 应存在");
    assert!(
        !rules.iter().any(|rule| rule.get("rule_set").is_some()),
        "осталось правило с висячим набором: {rules:?}"
    );
}
