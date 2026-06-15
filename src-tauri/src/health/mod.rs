pub mod constants;
pub mod db;
pub mod export;
pub mod parse;
pub mod store;
pub mod sync;

use store::HealthStore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthModuleId {
    HeartRate,
    Sleep,
    BloodOxygen,
    BloodPressure,
    Sport,
    HealthAll,
    BodyTemp,
    TempHumidity,
    Ecg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthModuleStatus {
    Ready,
    Planned,
    JniRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthModuleInfo {
    pub id: HealthModuleId,
    pub title: String,
    pub description: String,
    pub data_type: Option<u16>,
    pub status: HealthModuleStatus,
    pub record_count: usize,
}

pub fn list_modules(store: &HealthStore) -> Vec<HealthModuleInfo> {
    vec![
        HealthModuleInfo {
            id: HealthModuleId::HeartRate,
            title: "心率".into(),
            description: "历史心率（Health_HistoryHeart）".into(),
            data_type: Some(constants::DATATYPE_HISTORY_HEART),
            status: HealthModuleStatus::Ready,
            record_count: store.get_heart_rate().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::BloodOxygen,
            title: "血氧".into(),
            description: "SpO2 历史（Health_HistoryBloodOxygen）".into(),
            data_type: Some(constants::DATATYPE_HISTORY_BLOOD_OXYGEN),
            status: HealthModuleStatus::Ready,
            record_count: store.get_blood_oxygen().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::Sport,
            title: "运动/步数".into(),
            description: "步数、距离、卡路里".into(),
            data_type: Some(constants::DATATYPE_HISTORY_SPORT),
            status: HealthModuleStatus::Ready,
            record_count: store.get_sport().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::BloodPressure,
            title: "血压".into(),
            description: "收缩压/舒张压".into(),
            data_type: Some(constants::DATATYPE_HISTORY_BLOOD),
            status: HealthModuleStatus::Ready,
            record_count: store.get_blood_pressure().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::Sleep,
            title: "睡眠".into(),
            description: "睡眠阶段（Health_HistorySleep）".into(),
            data_type: Some(constants::DATATYPE_HISTORY_SLEEP),
            status: HealthModuleStatus::Ready,
            record_count: store.get_sleep().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::HealthAll,
            title: "综合指标".into(),
            description: "步数/心率/HRV/体温等（Health_HistoryAll）".into(),
            data_type: Some(constants::DATATYPE_HISTORY_ALL),
            status: HealthModuleStatus::Ready,
            record_count: store.get_health_all().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::BodyTemp,
            title: "体温".into(),
            description: "体表温度（Health_HistoryTemp）".into(),
            data_type: Some(constants::DATATYPE_HISTORY_TEMP),
            status: HealthModuleStatus::Ready,
            record_count: store.get_body_temp().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::TempHumidity,
            title: "温湿度".into(),
            description: "温度 + 湿度（Health_HistoryTempAndHumidity）".into(),
            data_type: Some(constants::DATATYPE_HISTORY_TEMP_AND_HUMIDITY),
            status: HealthModuleStatus::Ready,
            record_count: store.get_temp_humidity().len(),
        },
        HealthModuleInfo {
            id: HealthModuleId::Ecg,
            title: "心电图".into(),
            description: "需 AIData JNI".into(),
            data_type: Some(constants::DATATYPE_REAL_UPLOAD_ECG),
            status: HealthModuleStatus::JniRequired,
            record_count: 0,
        },
    ]
}
