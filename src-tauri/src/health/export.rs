use super::{HealthModuleId, HealthStore};
use super::db::DbError;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("该模块暂不支持导出")]
    Unsupported,
    #[error("暂无数据可导出")]
    Empty,
    #[error("{0}")]
    Store(#[from] DbError),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

pub fn export_module_csv(
    exports_dir: &Path,
    module_id: HealthModuleId,
    store: &HealthStore,
) -> Result<PathBuf, ExportError> {
    fs::create_dir_all(exports_dir)?;
    let filename = format!("health_{}_{}.csv", module_slug(module_id), now_stamp());
    let path = exports_dir.join(filename);
    let content = build_csv(module_id, store)?;
    if content.lines().count() <= 1 {
        return Err(ExportError::Empty);
    }
    let mut file = fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(path)
}

fn module_slug(module_id: HealthModuleId) -> &'static str {
    match module_id {
        HealthModuleId::HeartRate => "heart_rate",
        HealthModuleId::BloodOxygen => "blood_oxygen",
        HealthModuleId::Sport => "sport",
        HealthModuleId::BloodPressure => "blood_pressure",
        HealthModuleId::Sleep => "sleep",
        HealthModuleId::HealthAll => "health_all",
        HealthModuleId::BodyTemp => "body_temp",
        HealthModuleId::TempHumidity => "temp_humidity",
        HealthModuleId::Ecg => "ecg",
    }
}

fn now_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn build_csv(module_id: HealthModuleId, store: &HealthStore) -> Result<String, ExportError> {
    match module_id {
        HealthModuleId::HeartRate => {
            let mut csv = String::from("timestamp_ms,mode,bpm\n");
            for r in store.get_heart_rate() {
                csv.push_str(&format!("{},{},{}\n", r.timestamp_ms, r.mode, r.bpm));
            }
            Ok(csv)
        }
        HealthModuleId::BloodOxygen => {
            let mut csv = String::from("timestamp_ms,record_type,spo2\n");
            for r in store.get_blood_oxygen() {
                csv.push_str(&format!(
                    "{},{},{}\n",
                    r.timestamp_ms, r.record_type, r.spo2
                ));
            }
            Ok(csv)
        }
        HealthModuleId::Sport => {
            let mut csv = String::from("start_ms,end_ms,steps,distance,calories\n");
            for r in store.get_sport() {
                csv.push_str(&format!(
                    "{},{},{},{},{}\n",
                    r.start_ms, r.end_ms, r.steps, r.distance, r.calories
                ));
            }
            Ok(csv)
        }
        HealthModuleId::BloodPressure => {
            let mut csv = String::from("timestamp_ms,is_inflated,sbp,dbp,heart_rate\n");
            for r in store.get_blood_pressure() {
                csv.push_str(&format!(
                    "{},{},{},{},{}\n",
                    r.timestamp_ms, r.is_inflated, r.sbp, r.dbp, r.heart_rate
                ));
            }
            Ok(csv)
        }
        HealthModuleId::Sleep => {
            let mut csv = String::from(
                "start_ms,end_ms,deep_sleep_total_secs,light_sleep_total_secs,rem_total_secs,wake_count\n",
            );
            for r in store.get_sleep() {
                csv.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    r.start_ms,
                    r.end_ms,
                    r.deep_sleep_total_secs,
                    r.light_sleep_total_secs,
                    r.rem_total_secs,
                    r.wake_count
                ));
            }
            Ok(csv)
        }
        HealthModuleId::HealthAll => {
            let mut csv = String::from(
                "timestamp_ms,steps,heart_rate,sbp,dbp,spo2,hrv,temperature,body_fat,blood_sugar\n",
            );
            for r in store.get_health_all() {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{:.1},{:.1},{}\n",
                    r.timestamp_ms,
                    r.steps,
                    r.heart_rate,
                    r.sbp,
                    r.dbp,
                    r.spo2,
                    r.hrv,
                    r.temperature,
                    r.body_fat,
                    r.blood_sugar
                ));
            }
            Ok(csv)
        }
        HealthModuleId::BodyTemp => {
            let mut csv = String::from("timestamp_ms,record_type,temperature\n");
            for r in store.get_body_temp() {
                csv.push_str(&format!(
                    "{},{},{:.1}\n",
                    r.timestamp_ms, r.record_type, r.temperature
                ));
            }
            Ok(csv)
        }
        HealthModuleId::TempHumidity => {
            let mut csv = String::from("timestamp_ms,record_type,temperature,humidity\n");
            for r in store.get_temp_humidity() {
                csv.push_str(&format!(
                    "{},{},{:.1},{:.1}\n",
                    r.timestamp_ms, r.record_type, r.temperature, r.humidity
                ));
            }
            Ok(csv)
        }
        HealthModuleId::Ecg => Err(ExportError::Unsupported),
    }
}
