use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tracing::{error, warn};

use crate::app::core::kernel_service::status::kernel_check_health;

const KERNEL_HEALTH_INTERVAL: Duration = Duration::from_secs(10 * 60); // 10min

pub async fn start_background_tasks(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_kernel_health_loop(app_handle.clone()).await {
            error!("后台内核健康检查任务结束，原因: {}", e);
        }
    });
}

async fn start_kernel_health_loop(app: AppHandle) -> Result<(), String> {
    loop {
        match kernel_check_health(None).await {
            Ok(payload) => {
                if let Err(e) = app.emit("kernel-health", &payload) {
                    error!("发送 kernel-health 事件失败: {}", e);
                }
            }
            Err(e) => warn!("后台内核健康检查失败: {}", e),
        }

        tokio::time::sleep(KERNEL_HEALTH_INTERVAL).await;
    }
}
