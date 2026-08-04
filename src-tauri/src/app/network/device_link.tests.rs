use super::*;
use serde_json::json;

#[test]
fn first_poll_should_stay_silent() {
    // Подписка могла быть куплена до установки: «вы оплатили» на первом опросе
    // выглядит ошибкой приложения.
    assert_eq!(news_for("", "active"), None);
    assert_eq!(news_for("", "expired"), None);
}

#[test]
fn same_state_should_stay_silent() {
    assert_eq!(news_for("active", "active"), None);
    assert_eq!(news_for("none", "none"), None);
}

#[test]
fn news_should_follow_the_change() {
    assert_eq!(news_for("trial", "active"), Some("paid".to_string()));
    assert_eq!(news_for("none", "trial"), Some("trial".to_string()));
    assert_eq!(news_for("active", "expired"), Some("over".to_string()));
    assert_eq!(news_for("trial", "none"), Some("over".to_string()));
}

#[test]
fn signature_should_separate_trial_from_paid() {
    assert_eq!(signature_of("active", true), "trial");
    assert_eq!(signature_of("active", false), "active");
    assert_eq!(signature_of("expired", false), "expired");
    assert_eq!(signature_of("none", false), "none");
}

#[test]
fn interval_should_respect_server_but_keep_bounds() {
    assert_eq!(normalized_interval(0), DEFAULT_POLL_SEC);
    assert_eq!(normalized_interval(5), MIN_POLL_SEC);
    assert_eq!(normalized_interval(1800), 1800);
}

#[test]
fn poll_should_be_due_when_never_asked() {
    assert!(is_poll_due(0, 1800, 1_000_000));
}

#[test]
fn poll_should_wait_out_the_interval() {
    assert!(!is_poll_due(1_000_000, 1800, 1_000_100));
    assert!(is_poll_due(1_000_000, 1800, 1_001_800));
}

#[test]
fn linked_without_link_is_still_waiting() {
    // Сервер отвечает `linked` и до того, как появилась ссылка подписки.
    let parsed = parse_status(&json!({"status": "linked"}));
    assert_eq!(parsed.status, "pending");
    assert!(parsed.sub_url.is_none());
}

#[test]
fn linked_with_link_is_ready() {
    let parsed = parse_status(&json!({
        "status": "linked",
        "sub_url": "https://sub.ndvsdom54.ru/sub/token"
    }));
    assert_eq!(parsed.status, "linked");
    assert_eq!(
        parsed.sub_url.as_deref(),
        Some("https://sub.ndvsdom54.ru/sub/token")
    );
}

#[test]
fn absent_status_is_waiting_not_failure() {
    assert_eq!(parse_status(&json!({})).status, "pending");
}

#[test]
fn single_refusal_should_not_drop_the_link() {
    // Один ответ «не знаю такого устройства» бывает и от сбоя сервера, а привязку
    // человек восстанавливает только руками.
    assert!(!should_forget(1));
    assert!(should_forget(2));
    assert!(should_forget(5));
}

#[test]
fn unlinked_state_should_be_reported_as_not_linked() {
    let state = parse_state(&json!({"status": "unlinked"}), "active");
    assert!(!state.linked);
    assert!(state.news.is_none());
}

#[test]
fn state_should_carry_days_and_link() {
    let state = parse_state(
        &json!({
            "status": "linked",
            "subscription": "active",
            "is_trial": false,
            "days_left": 12,
            "expires_at": "2026-09-01T10:00:00+00:00",
            "sub_url": "https://sub.ndvsdom54.ru/sub/token",
            "poll_interval": 1800
        }),
        "trial",
    );

    assert!(state.linked);
    assert_eq!(state.subscription, "active");
    assert_eq!(state.days_left, 12);
    assert_eq!(state.news, Some("paid".to_string()));
    assert_eq!(
        state.sub_url.as_deref(),
        Some("https://sub.ndvsdom54.ru/sub/token")
    );
}

#[test]
fn expired_subscription_should_not_carry_link() {
    // Сервер отдаёт ссылку только действующей подписке — приложение не должно
    // додумывать её из прошлого ответа.
    let state = parse_state(
        &json!({
            "status": "linked",
            "subscription": "expired",
            "is_trial": false,
            "days_left": 0,
            "sub_url": null
        }),
        "active",
    );

    assert!(state.linked);
    assert!(state.sub_url.is_none());
    assert_eq!(state.news, Some("over".to_string()));
}
