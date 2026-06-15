use super::constants::YCBT_EPOCH_SECS;
use serde::{Deserialize, Serialize};

fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn ts_ms_from_raw(ts_raw: u32) -> u64 {
    (u64::from(ts_raw) + YCBT_EPOCH_SECS) * 1000
}

/// 对齐 Java `Float.parseFloat(int + "." + frac)`
fn decode_decimal_byte_pair(int_part: u8, frac_part: u8) -> f32 {
    format!("{int_part}.{frac_part}")
        .parse()
        .unwrap_or(f32::from(int_part))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartRateRecord {
    pub timestamp_ms: u64,
    pub mode: u8,
    pub bpm: u8,
}

/// YCBT 健康 payload case 6
pub fn parse_heart_history(payload: &[u8]) -> Vec<HeartRateRecord> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset + 6 <= payload.len() {
        let ts_raw = read_u32_le(payload, offset);
        records.push(HeartRateRecord {
            timestamp_ms: ts_ms_from_raw(ts_raw),
            mode: payload[offset + 4],
            bpm: payload[offset + 5],
        });
        offset += 6;
    }
    records
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloodOxygenRecord {
    pub timestamp_ms: u64,
    pub record_type: u8,
    pub spo2: u8,
}

/// YCBT 健康 payload case 26
pub fn parse_blood_oxygen_history(payload: &[u8]) -> Vec<BloodOxygenRecord> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset + 6 <= payload.len() {
        let ts_raw = read_u32_le(payload, offset);
        records.push(BloodOxygenRecord {
            timestamp_ms: ts_ms_from_raw(ts_raw),
            record_type: payload[offset + 4],
            spo2: payload[offset + 5],
        });
        offset += 6;
    }
    records
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SportRecord {
    pub start_ms: u64,
    pub end_ms: u64,
    pub steps: u16,
    pub distance: u16,
    pub calories: u16,
}

/// YCBT 健康 payload case 2
pub fn parse_sport_history(payload: &[u8]) -> Vec<SportRecord> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset + 14 <= payload.len() {
        let start_raw = read_u32_le(payload, offset);
        let end_raw = read_u32_le(payload, offset + 4);
        let steps = u16::from_le_bytes([payload[offset + 8], payload[offset + 9]]);
        let distance = u16::from_le_bytes([payload[offset + 10], payload[offset + 11]]);
        let calories = u16::from_le_bytes([payload[offset + 12], payload[offset + 13]]);
        records.push(SportRecord {
            start_ms: ts_ms_from_raw(start_raw),
            end_ms: ts_ms_from_raw(end_raw),
            steps,
            distance,
            calories,
        });
        offset += 14;
    }
    records
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloodPressureRecord {
    pub timestamp_ms: u64,
    pub is_inflated: u8,
    pub sbp: u8,
    pub dbp: u8,
    pub heart_rate: u8,
}

/// 睡眠阶段片段，case 4 内 sleepData 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepSegment {
    pub sleep_type: u8,
    pub start_ms: u64,
    pub duration_secs: u32,
}

/// 单次睡眠记录（含 20 字节头 + 变长阶段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepRecord {
    pub start_ms: u64,
    pub end_ms: u64,
    pub deep_sleep_count: u16,
    pub light_sleep_count: u16,
    pub deep_sleep_total_secs: u32,
    pub light_sleep_total_secs: u32,
    pub rem_total_secs: u32,
    pub wake_count: u32,
    pub wake_duration_secs: u32,
    pub segments: Vec<SleepSegment>,
}

/// YCBT 健康 payload case 4 — 20 字节头 + 8 字节/阶段
pub fn parse_sleep_history(payload: &[u8]) -> Vec<SleepRecord> {
    let mut records = Vec::new();
    let mut offset = 0;

    while offset + 20 <= payload.len() {
        let record_len = read_u16_le(payload, offset + 2) as usize;
        if record_len < 20 || offset + record_len > payload.len() {
            break;
        }

        let start_ms = ts_ms_from_raw(read_u32_le(payload, offset + 4));
        let end_ms = ts_ms_from_raw(read_u32_le(payload, offset + 8));
        let deep_sleep_count = read_u16_le(payload, offset + 12);

        let (light_sleep_count, deep_sleep_total_secs, light_sleep_total_secs, rem_total_secs) =
            if deep_sleep_count == 0xFFFF {
                (
                    0,
                    u32::from(read_u16_le(payload, offset + 16)),
                    u32::from(read_u16_le(payload, offset + 18)),
                    u32::from(read_u16_le(payload, offset + 14)),
                )
            } else {
                (
                    read_u16_le(payload, offset + 14),
                    u32::from(read_u16_le(payload, offset + 16)) * 60,
                    u32::from(read_u16_le(payload, offset + 18)) * 60,
                    0,
                )
            };

        let mut segments = Vec::new();
        let mut wake_count = 0;
        let mut wake_duration_secs = 0;
        let header_end = offset + 20;
        let mut seg_offset = header_end;

        while seg_offset + 8 <= offset + record_len
            && (seg_offset - header_end) + 8 <= record_len - 20
        {
            let sleep_type = payload[seg_offset];
            let seg_start_ms = ts_ms_from_raw(read_u32_le(payload, seg_offset + 1));
            let duration_secs = u32::from(read_u16_le(payload, seg_offset + 5))
                | (u32::from(payload[seg_offset + 7]) << 16);

            if sleep_type == 244 {
                wake_count += 1;
                wake_duration_secs += duration_secs;
            }

            if !segments
                .iter()
                .any(|s: &SleepSegment| s.start_ms == seg_start_ms)
            {
                segments.push(SleepSegment {
                    sleep_type,
                    start_ms: seg_start_ms,
                    duration_secs,
                });
            }

            seg_offset += 8;
        }

        records.push(SleepRecord {
            start_ms,
            end_ms,
            deep_sleep_count,
            light_sleep_count,
            deep_sleep_total_secs,
            light_sleep_total_secs,
            rem_total_secs,
            wake_count,
            wake_duration_secs,
            segments,
        });

        offset = seg_offset;
    }

    records
}

/// YCBT 健康 payload case 8
pub fn parse_blood_pressure_history(payload: &[u8]) -> Vec<BloodPressureRecord> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset + 8 <= payload.len() {
        let ts_raw = read_u32_le(payload, offset);
        records.push(BloodPressureRecord {
            timestamp_ms: ts_ms_from_raw(ts_raw),
            is_inflated: payload[offset + 4],
            sbp: payload[offset + 5],
            dbp: payload[offset + 6],
            heart_rate: payload[offset + 7],
        });
        offset += 8;
    }
    records
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthAllRecord {
    pub timestamp_ms: u64,
    pub steps: u16,
    pub heart_rate: u8,
    pub sbp: u8,
    pub dbp: u8,
    pub spo2: u8,
    pub respiratory_rate: u8,
    pub hrv: u8,
    pub cvrr: u8,
    pub temperature: f32,
    pub body_fat: f32,
    pub blood_sugar: u8,
}

/// YCBT 健康 payload case 9 — 20 字节/条
pub fn parse_health_all_history(payload: &[u8]) -> Vec<HealthAllRecord> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset + 20 <= payload.len() {
        let ts_raw = read_u32_le(payload, offset);
        records.push(HealthAllRecord {
            timestamp_ms: ts_ms_from_raw(ts_raw),
            steps: read_u16_le(payload, offset + 4),
            heart_rate: payload[offset + 6],
            sbp: payload[offset + 7],
            dbp: payload[offset + 8],
            spo2: payload[offset + 9],
            respiratory_rate: payload[offset + 10],
            hrv: payload[offset + 11],
            cvrr: payload[offset + 12],
            temperature: decode_decimal_byte_pair(payload[offset + 13], payload[offset + 14]),
            body_fat: decode_decimal_byte_pair(payload[offset + 15], payload[offset + 16]),
            blood_sugar: payload[offset + 17],
        });
        offset += 20;
    }
    records
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempHumidityRecord {
    pub timestamp_ms: u64,
    pub record_type: u8,
    pub temperature: f32,
    pub humidity: f32,
}

/// YCBT 健康 payload case 28 — 9 字节/条
pub fn parse_temp_humidity_history(payload: &[u8]) -> Vec<TempHumidityRecord> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset + 9 <= payload.len() {
        let ts_raw = read_u32_le(payload, offset);
        records.push(TempHumidityRecord {
            timestamp_ms: ts_ms_from_raw(ts_raw),
            record_type: payload[offset + 4],
            temperature: decode_decimal_byte_pair(payload[offset + 5], payload[offset + 6]),
            humidity: decode_decimal_byte_pair(payload[offset + 7], payload[offset + 8]),
        });
        offset += 9;
    }
    records
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyTempRecord {
    pub timestamp_ms: u64,
    pub record_type: u8,
    pub temperature: f32,
}

/// YCBT 健康 payload case 30 — 7 字节/条
pub fn parse_body_temp_history(payload: &[u8]) -> Vec<BodyTempRecord> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset + 7 <= payload.len() {
        let ts_raw = read_u32_le(payload, offset);
        records.push(BodyTempRecord {
            timestamp_ms: ts_ms_from_raw(ts_raw),
            record_type: payload[offset + 4],
            temperature: decode_decimal_byte_pair(payload[offset + 5], payload[offset + 6]),
        });
        offset += 7;
    }
    records
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_heart_record() {
        let payload = [0u8, 0, 0, 0, 1, 72];
        let records = parse_heart_history(&payload);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].bpm, 72);
    }

    #[test]
    fn parse_blood_oxygen_record() {
        let payload = [0u8, 0, 0, 0, 1, 98];
        let records = parse_blood_oxygen_history(&payload);
        assert_eq!(records[0].spo2, 98);
    }

    #[test]
    fn parse_sport_record() {
        let payload = [
            0, 0, 0, 0, 10, 0, 0, 0, 100, 0, 50, 0, 20, 0,
        ];
        let records = parse_sport_history(&payload);
        assert_eq!(records[0].steps, 100);
        assert_eq!(records[0].distance, 50);
    }

    #[test]
    fn parse_sleep_record_with_segment() {
        // 28 字节：20 字节头 + 1 个 8 字节阶段（深睡 30 分钟）
        let payload = [
            0, 0, 28, 0, 0, 0, 0, 0, 0x10, 0x0E, 0, 0, 2, 0, 3, 0, 60, 0, 120, 0, 241, 0, 0, 0,
            0, 0x08, 0x07, 0,
        ];
        let records = parse_sleep_history(&payload);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].deep_sleep_count, 2);
        assert_eq!(records[0].light_sleep_count, 3);
        assert_eq!(records[0].deep_sleep_total_secs, 3600);
        assert_eq!(records[0].light_sleep_total_secs, 7200);
        assert_eq!(records[0].segments.len(), 1);
        assert_eq!(records[0].segments[0].sleep_type, 241);
        assert_eq!(records[0].segments[0].duration_secs, 1800);
    }

    #[test]
    fn parse_sleep_record_with_rem_totals() {
        // deepSleepCount = 0xFFFF 时使用 REM/深/浅 直接数值（不乘 60）
        let payload = [
            0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 90, 0, 120, 0, 180, 0,
        ];
        let records = parse_sleep_history(&payload);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].deep_sleep_count, 0xFFFF);
        assert_eq!(records[0].rem_total_secs, 90);
        assert_eq!(records[0].deep_sleep_total_secs, 120);
        assert_eq!(records[0].light_sleep_total_secs, 180);
    }

    #[test]
    fn parse_health_all_record() {
        let payload = [
            0, 0, 0, 0, 100, 0, 72, 120, 80, 98, 16, 55, 1, 36, 5, 18, 2, 95, 0, 0,
        ];
        let records = parse_health_all_history(&payload);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].steps, 100);
        assert_eq!(records[0].heart_rate, 72);
        assert_eq!(records[0].hrv, 55);
        assert!((records[0].temperature - 36.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_body_temp_record() {
        let payload = [0, 0, 0, 0, 1, 36, 8];
        let records = parse_body_temp_history(&payload);
        assert_eq!(records.len(), 1);
        assert!((records[0].temperature - 36.8).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_temp_humidity_record() {
        let payload = [0, 0, 0, 0, 1, 36, 5, 55, 0];
        let records = parse_temp_humidity_history(&payload);
        assert_eq!(records.len(), 1);
        assert!((records[0].temperature - 36.5).abs() < f32::EPSILON);
        assert!((records[0].humidity - 55.0).abs() < f32::EPSILON);
    }
}
