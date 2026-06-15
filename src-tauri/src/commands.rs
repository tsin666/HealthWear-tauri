use crate::ble::SharedBackend;
use crate::health::{self, export::ExportError, HealthModuleId, HealthModuleInfo};
use crate::health::parse::{
    BloodOxygenRecord, BloodPressureRecord, BodyTempRecord, HealthAllRecord, HeartRateRecord,
    SleepRecord, SportRecord, TempHumidityRecord,
};
use crate::health::store::HealthStore;
use crate::wisdom::{build_mode_list, WisdomState};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

pub struct AppState {
    pub ble: SharedBackend,
    pub health: HealthStore,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshot {
    pub heart_rate: Vec<HeartRateRecord>,
    pub blood_oxygen: Vec<BloodOxygenRecord>,
    pub sport: Vec<SportRecord>,
    pub blood_pressure: Vec<BloodPressureRecord>,
    pub sleep: Vec<SleepRecord>,
    pub health_all: Vec<HealthAllRecord>,
    pub body_temp: Vec<BodyTempRecord>,
    pub temp_humidity: Vec<TempHumidityRecord>,
}

impl From<crate::ble::BleError> for ApiError {
    fn from(value: crate::ble::BleError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<health::db::DbError> for ApiError {
    fn from(value: health::db::DbError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<ExportError> for ApiError {
    fn from(value: ExportError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<tauri::Error> for ApiError {
    fn from(value: tauri::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

#[tauri::command]
pub fn get_connection(state: State<'_, AppState>) -> crate::ble::ConnectionInfo {
    state.ble.connection_info()
}

#[tauri::command]
pub fn scan_devices(state: State<'_, AppState>) -> Result<Vec<crate::ble::ScannedDevice>, ApiError> {
    state.ble.scan().map_err(Into::into)
}

#[tauri::command]
pub fn connect_device(state: State<'_, AppState>, device_id: String) -> Result<(), ApiError> {
    state.ble.connect(&device_id).map_err(Into::into)
}

#[tauri::command]
pub fn disconnect_device(state: State<'_, AppState>) -> Result<(), ApiError> {
    state.ble.disconnect().map_err(Into::into)
}

#[tauri::command]
pub fn list_wisdom_modes(state: State<'_, AppState>) -> Result<Vec<crate::wisdom::WisdomModeItem>, ApiError> {
    let wisdom = state.ble.get_wit_state().unwrap_or_default();
    Ok(build_mode_list(&wisdom))
}

#[tauri::command]
pub fn get_wisdom_state(state: State<'_, AppState>) -> WisdomState {
    state.ble.get_wit_state().unwrap_or_default()
}

#[tauri::command]
pub fn set_wisdom_mode(
    state: State<'_, AppState>,
    protocol_index: u8,
    enabled: bool,
) -> Result<WisdomState, ApiError> {
    state
        .ble
        .set_wit_mode(enabled, protocol_index)
        .map_err(ApiError::from)?;
    state.ble.get_wit_state().map_err(ApiError::from)
}

#[tauri::command]
pub fn list_health_modules(state: State<'_, AppState>) -> Vec<HealthModuleInfo> {
    health::list_modules(&state.health)
}

#[tauri::command]
pub fn get_health_snapshot(state: State<'_, AppState>) -> HealthSnapshot {
    HealthSnapshot {
        heart_rate: state.health.get_heart_rate(),
        blood_oxygen: state.health.get_blood_oxygen(),
        sport: state.health.get_sport(),
        blood_pressure: state.health.get_blood_pressure(),
        sleep: state.health.get_sleep(),
        health_all: state.health.get_health_all(),
        body_temp: state.health.get_body_temp(),
        temp_humidity: state.health.get_temp_humidity(),
    }
}

#[tauri::command]
pub fn sync_health_module(
    state: State<'_, AppState>,
    module_id: HealthModuleId,
    mock: bool,
) -> Result<HealthSnapshot, ApiError> {
    let ble = state.ble.as_ref();
    let use_mock = mock || ble.connection_info().mock;

    match module_id {
        HealthModuleId::HeartRate => {
            let records = if use_mock {
                health::sync::mock_heart_rate()
            } else {
                health::sync::sync_heart_rate(ble)?
            };
            state.health.set_heart_rate(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::BloodOxygen => {
            let records = if use_mock {
                health::sync::mock_blood_oxygen()
            } else {
                health::sync::sync_blood_oxygen(ble)?
            };
            state.health.set_blood_oxygen(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::Sport => {
            let records = if use_mock {
                health::sync::mock_sport()
            } else {
                health::sync::sync_sport(ble)?
            };
            state.health.set_sport(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::BloodPressure => {
            let records = if use_mock {
                health::sync::mock_blood_pressure()
            } else {
                health::sync::sync_blood_pressure(ble)?
            };
            state.health.set_blood_pressure(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::Sleep => {
            let records = if use_mock {
                health::sync::mock_sleep()
            } else {
                health::sync::sync_sleep(ble)?
            };
            state.health.set_sleep(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::HealthAll => {
            let records = if use_mock {
                health::sync::mock_health_all()
            } else {
                health::sync::sync_health_all(ble)?
            };
            state.health.set_health_all(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::BodyTemp => {
            let records = if use_mock {
                health::sync::mock_body_temp()
            } else {
                health::sync::sync_body_temp(ble)?
            };
            state.health.set_body_temp(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::TempHumidity => {
            let records = if use_mock {
                health::sync::mock_temp_humidity()
            } else {
                health::sync::sync_temp_humidity(ble)?
            };
            state.health.set_temp_humidity(records)?;
            Ok(snapshot(&state.health))
        }
        HealthModuleId::Ecg => Err(ApiError {
            message: "ECG 模块开发中，见 docs/HEALTH_MODULES.md".into(),
        }),
    }
}

#[tauri::command]
pub fn get_health_db_path(state: State<'_, AppState>) -> Option<String> {
    state
        .health
        .db_path_hint()
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn export_health_csv(
    app: AppHandle,
    state: State<'_, AppState>,
    module_id: HealthModuleId,
) -> Result<String, ApiError> {
    let exports_dir = app.path().app_data_dir()?.join("exports");
    let path = health::export::export_module_csv(&exports_dir, module_id, &state.health)?;
    Ok(path.to_string_lossy().into_owned())
}

fn snapshot(store: &HealthStore) -> HealthSnapshot {
    HealthSnapshot {
        heart_rate: store.get_heart_rate(),
        blood_oxygen: store.get_blood_oxygen(),
        sport: store.get_sport(),
        blood_pressure: store.get_blood_pressure(),
        sleep: store.get_sleep(),
        health_all: store.get_health_all(),
        body_temp: store.get_body_temp(),
        temp_humidity: store.get_temp_humidity(),
    }
}
