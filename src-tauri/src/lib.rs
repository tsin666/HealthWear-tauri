mod ble;
mod commands;
mod health;
mod wisdom;

use commands::AppState;
use health::store::HealthStore;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("health.db");
            let health = HealthStore::open(db_path).map_err(|e| {
                Box::<dyn std::error::Error>::from(std::io::Error::other(e.to_string()))
            })?;
            let bundle = ble::create_backend_bundle();
            app.manage(AppState {
                ble: bundle.backend,
                ble_platform: bundle.platform,
                health,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_ble_platform,
            commands::get_connection,
            commands::scan_devices,
            commands::connect_device,
            commands::disconnect_device,
            commands::list_wisdom_modes,
            commands::get_wisdom_state,
            commands::set_wisdom_mode,
            commands::list_health_modules,
            commands::get_health_snapshot,
            commands::sync_health_module,
            commands::export_health_csv,
            commands::get_health_db_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
