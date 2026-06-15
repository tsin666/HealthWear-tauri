use super::constants::{
    DATATYPE_HISTORY_ALL, DATATYPE_HISTORY_BLOOD, DATATYPE_HISTORY_BLOOD_OXYGEN,
    DATATYPE_HISTORY_HEART, DATATYPE_HISTORY_SLEEP, DATATYPE_HISTORY_SPORT,
    DATATYPE_HISTORY_TEMP, DATATYPE_HISTORY_TEMP_AND_HUMIDITY,
};
use super::parse::{
    parse_blood_oxygen_history, parse_blood_pressure_history, parse_body_temp_history,
    parse_health_all_history, parse_heart_history, parse_sleep_history, parse_sport_history,
    parse_temp_humidity_history, BloodOxygenRecord, BloodPressureRecord, BodyTempRecord,
    HealthAllRecord, HeartRateRecord, SleepRecord, SportRecord, TempHumidityRecord,
};
use crate::ble::{BleError, RingBleBackend};

const HEALTH_SYNC_TIMEOUT_SECS: u64 = 15;

fn sync_raw(backend: &dyn RingBleBackend, data_type: u16) -> Result<Vec<u8>, BleError> {
    backend.send_raw_command(data_type, &[], HEALTH_SYNC_TIMEOUT_SECS)
}

pub fn sync_heart_rate(backend: &dyn RingBleBackend) -> Result<Vec<HeartRateRecord>, BleError> {
    Ok(parse_heart_history(&sync_raw(backend, DATATYPE_HISTORY_HEART)?))
}

pub fn sync_blood_oxygen(backend: &dyn RingBleBackend) -> Result<Vec<BloodOxygenRecord>, BleError> {
    Ok(parse_blood_oxygen_history(&sync_raw(
        backend,
        DATATYPE_HISTORY_BLOOD_OXYGEN,
    )?))
}

pub fn sync_sport(backend: &dyn RingBleBackend) -> Result<Vec<SportRecord>, BleError> {
    Ok(parse_sport_history(&sync_raw(backend, DATATYPE_HISTORY_SPORT)?))
}

pub fn sync_blood_pressure(
    backend: &dyn RingBleBackend,
) -> Result<Vec<BloodPressureRecord>, BleError> {
    Ok(parse_blood_pressure_history(&sync_raw(
        backend,
        DATATYPE_HISTORY_BLOOD,
    )?))
}

pub fn sync_sleep(backend: &dyn RingBleBackend) -> Result<Vec<SleepRecord>, BleError> {
    Ok(parse_sleep_history(&sync_raw(backend, DATATYPE_HISTORY_SLEEP)?))
}

pub fn sync_health_all(backend: &dyn RingBleBackend) -> Result<Vec<HealthAllRecord>, BleError> {
    Ok(parse_health_all_history(&sync_raw(backend, DATATYPE_HISTORY_ALL)?))
}

pub fn sync_body_temp(backend: &dyn RingBleBackend) -> Result<Vec<BodyTempRecord>, BleError> {
    Ok(parse_body_temp_history(&sync_raw(backend, DATATYPE_HISTORY_TEMP)?))
}

pub fn sync_temp_humidity(
    backend: &dyn RingBleBackend,
) -> Result<Vec<TempHumidityRecord>, BleError> {
    Ok(parse_temp_humidity_history(&sync_raw(
        backend,
        DATATYPE_HISTORY_TEMP_AND_HUMIDITY,
    )?))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn mock_heart_rate() -> Vec<HeartRateRecord> {
    let now = now_ms();
    vec![
        HeartRateRecord {
            timestamp_ms: now.saturating_sub(3600_000),
            mode: 1,
            bpm: 68,
        },
        HeartRateRecord {
            timestamp_ms: now.saturating_sub(300_000),
            mode: 2,
            bpm: 82,
        },
    ]
}

pub fn mock_blood_oxygen() -> Vec<BloodOxygenRecord> {
    let now = now_ms();
    vec![
        BloodOxygenRecord {
            timestamp_ms: now.saturating_sub(7200_000),
            record_type: 1,
            spo2: 97,
        },
        BloodOxygenRecord {
            timestamp_ms: now.saturating_sub(3600_000),
            record_type: 1,
            spo2: 98,
        },
    ]
}

pub fn mock_sport() -> Vec<SportRecord> {
    let now = now_ms();
    let start = now.saturating_sub(86_400_000);
    vec![SportRecord {
        start_ms: start,
        end_ms: start + 3_600_000,
        steps: 8432,
        distance: 5200,
        calories: 310,
    }]
}

pub fn mock_blood_pressure() -> Vec<BloodPressureRecord> {
    let now = now_ms();
    vec![BloodPressureRecord {
        timestamp_ms: now.saturating_sub(1800_000),
        is_inflated: 0,
        sbp: 118,
        dbp: 76,
        heart_rate: 72,
    }]
}

pub fn mock_sleep() -> Vec<SleepRecord> {
    let now = now_ms();
    let start = now.saturating_sub(28_800_000);
    let end = start + 28_800_000;
    vec![SleepRecord {
        start_ms: start,
        end_ms: end,
        deep_sleep_count: 4,
        light_sleep_count: 6,
        deep_sleep_total_secs: 7200,
        light_sleep_total_secs: 14_400,
        rem_total_secs: 5400,
        wake_count: 2,
        wake_duration_secs: 900,
        segments: vec![
            super::parse::SleepSegment {
                sleep_type: 241,
                start_ms: start,
                duration_secs: 5400,
            },
            super::parse::SleepSegment {
                sleep_type: 242,
                start_ms: start + 5_400_000,
                duration_secs: 7200,
            },
            super::parse::SleepSegment {
                sleep_type: 243,
                start_ms: start + 12_600_000,
                duration_secs: 5400,
            },
        ],
    }]
}

pub fn mock_health_all() -> Vec<HealthAllRecord> {
    let now = now_ms();
    vec![HealthAllRecord {
        timestamp_ms: now.saturating_sub(3600_000),
        steps: 6200,
        heart_rate: 74,
        sbp: 118,
        dbp: 76,
        spo2: 97,
        respiratory_rate: 16,
        hrv: 52,
        cvrr: 3,
        temperature: 36.5,
        body_fat: 18.2,
        blood_sugar: 95,
    }]
}

pub fn mock_body_temp() -> Vec<BodyTempRecord> {
    let now = now_ms();
    vec![
        BodyTempRecord {
            timestamp_ms: now.saturating_sub(7200_000),
            record_type: 1,
            temperature: 36.4,
        },
        BodyTempRecord {
            timestamp_ms: now.saturating_sub(3600_000),
            record_type: 1,
            temperature: 36.7,
        },
    ]
}

pub fn mock_temp_humidity() -> Vec<TempHumidityRecord> {
    let now = now_ms();
    vec![TempHumidityRecord {
        timestamp_ms: now.saturating_sub(1800_000),
        record_type: 1,
        temperature: 36.6,
        humidity: 48.0,
    }]
}
