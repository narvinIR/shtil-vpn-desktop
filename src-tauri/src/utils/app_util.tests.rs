use super::*;

#[test]
fn get_work_dir_sync_should_return_our_own_dir() {
    let work_dir = get_work_dir_sync();
    let work_dir_path = PathBuf::from(&work_dir);

    assert!(work_dir_path.exists());
    assert!(work_dir_path.ends_with(WORK_DIR_NAME));
}

#[tokio::test]
async fn get_work_dir_should_return_our_own_dir() {
    let work_dir = get_work_dir().await;
    let work_dir_path = PathBuf::from(&work_dir);

    assert!(work_dir_path.exists());
    assert!(work_dir_path.ends_with(WORK_DIR_NAME));
}

#[test]
fn get_service_path_should_point_to_expected_binary_name() {
    let service_path = get_service_path();

    #[cfg(target_os = "windows")]
    assert!(service_path.ends_with(r"src\config\sing-box-service.exe"));

    #[cfg(not(target_os = "windows"))]
    assert!(service_path.ends_with("src/config/sing-box-service"));
}

/// Свой каталог под наши файлы, чтобы не трогать настоящий.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("shtil-work-dir-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("временный каталог создаётся");
    dir
}

#[test]
fn folder_of_the_fork_should_be_adopted_once() {
    // Иначе после переименования человек теряет ядро и ключи: они остались в
    // старой папке, а приложение смотрит в новую.
    let base = scratch("adopt");
    let legacy = base.join(LEGACY_WORK_DIR_NAME);
    let ours = base.join(WORK_DIR_NAME);
    std::fs::create_dir_all(legacy.join("sing-box")).expect("старая папка создаётся");

    let chosen = adopt_legacy_dir(&ours, &legacy);

    assert_eq!(chosen, ours);
    assert!(ours.join("sing-box").exists());
    assert!(!legacy.exists());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn our_own_folder_wins_when_both_exist() {
    // Второй запуск и старый каталог рядом: содержимое нашей папки не трогаем.
    let base = scratch("both");
    let legacy = base.join(LEGACY_WORK_DIR_NAME);
    let ours = base.join(WORK_DIR_NAME);
    std::fs::create_dir_all(&legacy).expect("старая папка создаётся");
    std::fs::create_dir_all(&ours).expect("наша папка создаётся");

    let chosen = adopt_legacy_dir(&ours, &legacy);

    assert_eq!(chosen, ours);
    assert!(legacy.exists());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn clean_install_needs_no_adoption() {
    let base = scratch("clean");
    let legacy = base.join(LEGACY_WORK_DIR_NAME);
    let ours = base.join(WORK_DIR_NAME);

    let chosen = adopt_legacy_dir(&ours, &legacy);

    assert_eq!(chosen, ours);
    let _ = std::fs::remove_dir_all(&base);
}

/// Токен подписки — это и есть платный ключ клиента.
///
/// Журнал ядра лежит файлом на диске, попадает в резервную копию и приезжает
/// нам в поддержку скриншотом. Кто прочитал токен — забирает VPN без бота и
/// без пароля, поэтому в журнал уходит только адрес без опознавательной части.
#[test]
fn subscription_token_never_reaches_the_log() {
    let safe = safe_url("https://sub.ndvsdom54.ru/sub/a1b2c3d4e5?device=win");

    assert!(
        !safe.contains("a1b2c3d4e5"),
        "токен остался в записи: {safe}"
    );
    assert!(
        !safe.contains("device=win"),
        "хвост запроса остался: {safe}"
    );
    assert!(
        safe.starts_with("https://sub.ndvsdom54.ru/"),
        "по записи должно быть видно, куда ходили: {safe}"
    );
}

#[test]
fn a_broken_address_is_hidden_whole() {
    assert_eq!(safe_url("совсем не адрес"), "<адрес скрыт>");
}
