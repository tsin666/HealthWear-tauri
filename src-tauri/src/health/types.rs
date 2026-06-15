use serde::{Deserialize, Serialize};

/// 原 App `Constants.DATATYPE`
pub const HEALTH_HISTORY_HEART: u16 = 1286; // 0x0506
pub const HEALTH_HISTORY_SPORT: u16 = 1282;
pub const HEALTH_HISTORY_SLEEP: u16 = 1284;
pub const HEALTH_HISTORY_BLOCK_ACK: u16 = 1408; // 0x0580
pub const HEALTH_DELETE_HEART: u16 = 1346;

/// `unpackHealthData` 中心率分支的 health_kind
pub const HEALTH_KIND_HEART: u8 = 6;

/// YCBT 时间戳基准：秒级偏移 + 946684800 → Unix 秒
pub const YCBT_EPOCH_OFFSET_SEC: u32 = 946_684_800;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartRateRecord {
    pub id: Option<i64>,
    pub start_time_ms: i64,
    pub mode: u8,
    pub value: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartRateSummary {
    pub count: usize,
    pub latest: Option<u8>,
    pub avg: Option<u8>,
    pub records: Vec<HeartRateRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthMetricKind {
    HeartRate,
    Sleep,
    Sport,
    BloodOxygen,
    Ecg,
}

impl HealthMetricKind {
    pub fn history_datatype(self) -> Option<u16> {
        match self {
            Self::HeartRate => Some(HEALTH_HISTORY_HEART),
            Self::Sport => Some(HEALTH_HISTORY_SPORT),
            Self::Sleep => Some(HEALTH_HISTORY_SLEEP),
            Self::BloodOxygen | Self::Ecg => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::HeartRate => "心率",
            Self::Sleep => "睡眠",
            Self::Sport => "运动",
            Self::BloodOxygen => "血氧",
            Self::Ecg => "心电图",
        }
    }
}
