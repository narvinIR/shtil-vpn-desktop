use super::*;
use serde_json::json;

#[test]
fn refusal_should_speak_the_servers_code() {
    assert_eq!(
        reason_for(409, &json!({"error": "already_issued"})),
        "already_issued"
    );
    assert_eq!(reason_for(429, &json!({"error": "ip_limit"})), "ip_limit");
    assert_eq!(reason_for(503, &json!({"error": "disabled"})), "disabled");
}

#[test]
fn refusal_without_body_should_still_be_named() {
    // Сырой ответ сервера человеку не показываем, поэтому имя отказа нужно всегда.
    assert_eq!(reason_for(429, &json!({})), "rate_limited");
    assert_eq!(reason_for(500, &json!({})), "server");
}

#[test]
fn guest_should_carry_term_from_the_server() {
    // Срок берём из ответа, а не считаем своими часами: часы компьютера врут.
    let issued = parse_guest(&json!({
        "sub_url": "https://sub.ndvsdom54.ru/sub/token",
        "expires_at": "2026-08-05T02:00:00+00:00",
        "ttl_hours": 2,
        "traffic_gb": 1
    }))
    .expect("ответ с ключом разбирается");

    assert_eq!(issued.sub_url, "https://sub.ndvsdom54.ru/sub/token");
    assert_eq!(issued.expires_at, "2026-08-05T02:00:00+00:00");
    assert_eq!(issued.ttl_hours, 2);
    assert_eq!(issued.traffic_gb, 1);
}

#[test]
fn answer_without_link_is_not_a_guest() {
    assert!(parse_guest(&json!({"expires_at": "2026-08-05T02:00:00+00:00"})).is_none());
}
