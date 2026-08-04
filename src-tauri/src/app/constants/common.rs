//! Общие константы: сообщения и журнал.
//!
//! Эти строки доезжают до человека в редких ветках — окном об ошибке. Языков в
//! продукте пять, поэтому здесь нейтральный английский, а не язык форка:
//! человеческие фразы живут в словарях `src/locales/*.ts`.

/// Сообщения, которые уходят наверх вместе с ошибкой.
pub mod messages {
    // Ошибки
    pub const ERR_KERNEL_NOT_FOUND: &str = "Kernel file not found";
    pub const ERR_VERSION_CHECK_FAILED: &str = "Version check failed";
    pub const ERR_GET_VERSION_FAILED: &str = "Could not read version";
    pub const ERR_CONFIG_READ_FAILED: &str = "Could not read the config file";
    pub const ERR_DOWNLOAD_FAILED: &str = "Download failed";
    pub const ERR_SUBSCRIPTION_FAILED: &str = "Could not download the subscription";
    pub const ERR_PROCESS_SUBSCRIPTION_FAILED: &str = "Could not process the subscription";
    pub const ERR_GET_EXE_PATH_FAILED: &str = "Could not get the app path";
    pub const ERR_RESTART_FAILED: &str = "Restart failed";
    pub const ERR_INVALID_CONFIG: &str = "The config file is invalid";
    pub const ERR_PROCESS_ALREADY_RUNNING: &str = "The process is already running";
    pub const ERR_PROCESS_NOT_RUNNING: &str = "The process is not running";
    pub const ERR_PROCESS_START_FAILED: &str = "Could not start the process";
    pub const ERR_PROCESS_STOP_FAILED: &str = "Could not stop the process";
    pub const ERR_HTTP_CLIENT_FAILED: &str = "Could not create the HTTP client";
    pub const ERR_REQUEST_FAILED: &str = "Request failed";
    pub const ERR_SERVER_ERROR: &str = "The server returned an error status";
    pub const ERR_FILE_SIZE_UNKNOWN: &str = "Could not get the file size";
    pub const ERR_CREATE_DIR_FAILED: &str = "Could not create the folder";
    pub const ERR_CREATE_FILE_FAILED: &str = "Could not create the file";
    pub const ERR_OPEN_FILE_FAILED: &str = "Could not open the file";
    pub const ERR_READ_ARCHIVE_FAILED: &str = "Could not read the archive";
    pub const ERR_EXTRACT_FILE_FAILED: &str = "Could not extract the file";
    pub const ERR_INVALID_FILENAME: &str = "Invalid file name";
    pub const ERR_WRITE_FILE_FAILED: &str = "Could not write the file";
    pub const ERR_READ_FILE_FAILED: &str = "Could not read the file";
    pub const ERR_KEY_NOT_FOUND: &str = "Key not found";

    // Сообщения
    pub const INFO_PROCESS_STARTED: &str = "Process started";
    pub const INFO_PROCESS_STOPPED: &str = "Process stopped";
    pub const INFO_SYSTEM_PROXY_DISABLED: &str = "System proxy turned off";
    pub const INFO_CONFIG_CHECK_PASSED: &str = "Config check passed";
    pub const INFO_PROXY_MODE_ENABLED: &str = "Proxy mode enabled";
    pub const INFO_DOWNLOAD_STARTED: &str = "Download started";
    pub const INFO_UNZIP_STARTED: &str = "Unpacking started";
    pub const INFO_EXTRACTING_FILE: &str = "Extracting";
}

/// Константы журнала.
pub mod log {
    /// Уровень журнала
    pub const DEFAULT_LEVEL: &str = "debug";

    /// Папка журнала
    pub const DEFAULT_DIR: &str = "logs";

    /// Начало имени файла журнала
    pub const DEFAULT_FILE_PREFIX: &str = "app";

    /// Как часто заводится новый файл
    pub mod rotation {
        pub const HOURLY: &str = "hourly";
        pub const DAILY: &str = "daily";
        pub const NEVER: &str = "never";
        pub const DEFAULT: &str = "daily";
    }

    /// Сколько файлов храним — по одному на день, последние 5 дней
    pub const DEFAULT_MAX_FILES: u32 = 5;
}
