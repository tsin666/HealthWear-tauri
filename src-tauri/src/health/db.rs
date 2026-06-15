use super::parse::{
    BloodOxygenRecord, BloodPressureRecord, BodyTempRecord, HealthAllRecord, HeartRateRecord,
    SleepRecord, SportRecord, TempHumidityRecord,
};
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("数据库错误: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);
INSERT OR IGNORE INTO schema_version (version) VALUES (1);

CREATE TABLE IF NOT EXISTS heart_rate (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_ms INTEGER NOT NULL,
    mode INTEGER NOT NULL,
    bpm INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS blood_oxygen (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_ms INTEGER NOT NULL,
    record_type INTEGER NOT NULL,
    spo2 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sport (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL,
    steps INTEGER NOT NULL,
    distance INTEGER NOT NULL,
    calories INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS blood_pressure (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_ms INTEGER NOT NULL,
    is_inflated INTEGER NOT NULL,
    sbp INTEGER NOT NULL,
    dbp INTEGER NOT NULL,
    heart_rate INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sleep (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL,
    deep_sleep_count INTEGER NOT NULL,
    light_sleep_count INTEGER NOT NULL,
    deep_sleep_total_secs INTEGER NOT NULL,
    light_sleep_total_secs INTEGER NOT NULL,
    rem_total_secs INTEGER NOT NULL,
    wake_count INTEGER NOT NULL,
    wake_duration_secs INTEGER NOT NULL,
    segments_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS health_all (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_ms INTEGER NOT NULL,
    steps INTEGER NOT NULL,
    heart_rate INTEGER NOT NULL,
    sbp INTEGER NOT NULL,
    dbp INTEGER NOT NULL,
    spo2 INTEGER NOT NULL,
    respiratory_rate INTEGER NOT NULL,
    hrv INTEGER NOT NULL,
    cvrr INTEGER NOT NULL,
    temperature REAL NOT NULL,
    body_fat REAL NOT NULL,
    blood_sugar INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS body_temp (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_ms INTEGER NOT NULL,
    record_type INTEGER NOT NULL,
    temperature REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS temp_humidity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_ms INTEGER NOT NULL,
    record_type INTEGER NOT NULL,
    temperature REAL NOT NULL,
    humidity REAL NOT NULL
);
"#;

pub fn open(path: &Path) -> Result<Connection, DbError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

pub fn load_heart_rate(conn: &Connection) -> Result<Vec<HeartRateRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, mode, bpm FROM heart_rate ORDER BY timestamp_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(HeartRateRecord {
            timestamp_ms: row.get::<_, i64>(0)? as u64,
            mode: row.get::<_, i64>(1)? as u8,
            bpm: row.get::<_, i64>(2)? as u8,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn replace_heart_rate(conn: &Connection, records: &[HeartRateRecord]) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM heart_rate", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO heart_rate (timestamp_ms, mode, bpm) VALUES (?1, ?2, ?3)",
            params![record.timestamp_ms, record.mode, record.bpm],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_blood_oxygen(conn: &Connection) -> Result<Vec<BloodOxygenRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, record_type, spo2 FROM blood_oxygen ORDER BY timestamp_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(BloodOxygenRecord {
            timestamp_ms: row.get::<_, i64>(0)? as u64,
            record_type: row.get::<_, i64>(1)? as u8,
            spo2: row.get::<_, i64>(2)? as u8,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn replace_blood_oxygen(
    conn: &Connection,
    records: &[BloodOxygenRecord],
) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM blood_oxygen", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO blood_oxygen (timestamp_ms, record_type, spo2) VALUES (?1, ?2, ?3)",
            params![record.timestamp_ms, record.record_type, record.spo2],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_sport(conn: &Connection) -> Result<Vec<SportRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT start_ms, end_ms, steps, distance, calories FROM sport ORDER BY start_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SportRecord {
            start_ms: row.get::<_, i64>(0)? as u64,
            end_ms: row.get::<_, i64>(1)? as u64,
            steps: row.get::<_, i64>(2)? as u16,
            distance: row.get::<_, i64>(3)? as u16,
            calories: row.get::<_, i64>(4)? as u16,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn replace_sport(conn: &Connection, records: &[SportRecord]) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM sport", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO sport (start_ms, end_ms, steps, distance, calories) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                record.start_ms,
                record.end_ms,
                record.steps,
                record.distance,
                record.calories
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_blood_pressure(conn: &Connection) -> Result<Vec<BloodPressureRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, is_inflated, sbp, dbp, heart_rate FROM blood_pressure ORDER BY timestamp_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(BloodPressureRecord {
            timestamp_ms: row.get::<_, i64>(0)? as u64,
            is_inflated: row.get::<_, i64>(1)? as u8,
            sbp: row.get::<_, i64>(2)? as u8,
            dbp: row.get::<_, i64>(3)? as u8,
            heart_rate: row.get::<_, i64>(4)? as u8,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn replace_blood_pressure(
    conn: &Connection,
    records: &[BloodPressureRecord],
) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM blood_pressure", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO blood_pressure (timestamp_ms, is_inflated, sbp, dbp, heart_rate) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                record.timestamp_ms,
                record.is_inflated,
                record.sbp,
                record.dbp,
                record.heart_rate
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_sleep(conn: &Connection) -> Result<Vec<SleepRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT start_ms, end_ms, deep_sleep_count, light_sleep_count, deep_sleep_total_secs, light_sleep_total_secs, rem_total_secs, wake_count, wake_duration_secs, segments_json FROM sleep ORDER BY start_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)? as u64,
            row.get::<_, i64>(1)? as u64,
            row.get::<_, i64>(2)? as u16,
            row.get::<_, i64>(3)? as u16,
            row.get::<_, i64>(4)? as u32,
            row.get::<_, i64>(5)? as u32,
            row.get::<_, i64>(6)? as u32,
            row.get::<_, i64>(7)? as u32,
            row.get::<_, i64>(8)? as u32,
            row.get::<_, String>(9)?,
        ))
    })?;

    let mut records = Vec::new();
    for row in rows {
        let (
            start_ms,
            end_ms,
            deep_sleep_count,
            light_sleep_count,
            deep_sleep_total_secs,
            light_sleep_total_secs,
            rem_total_secs,
            wake_count,
            wake_duration_secs,
            segments_json,
        ) = row?;
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
            segments: serde_json::from_str(&segments_json)?,
        });
    }
    Ok(records)
}

pub fn replace_sleep(conn: &Connection, records: &[SleepRecord]) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM sleep", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO sleep (start_ms, end_ms, deep_sleep_count, light_sleep_count, deep_sleep_total_secs, light_sleep_total_secs, rem_total_secs, wake_count, wake_duration_secs, segments_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.start_ms,
                record.end_ms,
                record.deep_sleep_count,
                record.light_sleep_count,
                record.deep_sleep_total_secs,
                record.light_sleep_total_secs,
                record.rem_total_secs,
                record.wake_count,
                record.wake_duration_secs,
                serde_json::to_string(&record.segments)?,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_health_all(conn: &Connection) -> Result<Vec<HealthAllRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, steps, heart_rate, sbp, dbp, spo2, respiratory_rate, hrv, cvrr, temperature, body_fat, blood_sugar FROM health_all ORDER BY timestamp_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(HealthAllRecord {
            timestamp_ms: row.get::<_, i64>(0)? as u64,
            steps: row.get::<_, i64>(1)? as u16,
            heart_rate: row.get::<_, i64>(2)? as u8,
            sbp: row.get::<_, i64>(3)? as u8,
            dbp: row.get::<_, i64>(4)? as u8,
            spo2: row.get::<_, i64>(5)? as u8,
            respiratory_rate: row.get::<_, i64>(6)? as u8,
            hrv: row.get::<_, i64>(7)? as u8,
            cvrr: row.get::<_, i64>(8)? as u8,
            temperature: row.get(9)?,
            body_fat: row.get(10)?,
            blood_sugar: row.get::<_, i64>(11)? as u8,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn replace_health_all(conn: &Connection, records: &[HealthAllRecord]) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM health_all", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO health_all (timestamp_ms, steps, heart_rate, sbp, dbp, spo2, respiratory_rate, hrv, cvrr, temperature, body_fat, blood_sugar) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.timestamp_ms,
                record.steps,
                record.heart_rate,
                record.sbp,
                record.dbp,
                record.spo2,
                record.respiratory_rate,
                record.hrv,
                record.cvrr,
                record.temperature,
                record.body_fat,
                record.blood_sugar,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_body_temp(conn: &Connection) -> Result<Vec<BodyTempRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, record_type, temperature FROM body_temp ORDER BY timestamp_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(BodyTempRecord {
            timestamp_ms: row.get::<_, i64>(0)? as u64,
            record_type: row.get::<_, i64>(1)? as u8,
            temperature: row.get(2)?,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn replace_body_temp(conn: &Connection, records: &[BodyTempRecord]) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM body_temp", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO body_temp (timestamp_ms, record_type, temperature) VALUES (?1, ?2, ?3)",
            params![record.timestamp_ms, record.record_type, record.temperature],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_temp_humidity(conn: &Connection) -> Result<Vec<TempHumidityRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, record_type, temperature, humidity FROM temp_humidity ORDER BY timestamp_ms DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TempHumidityRecord {
            timestamp_ms: row.get::<_, i64>(0)? as u64,
            record_type: row.get::<_, i64>(1)? as u8,
            temperature: row.get(2)?,
            humidity: row.get(3)?,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn replace_temp_humidity(
    conn: &Connection,
    records: &[TempHumidityRecord],
) -> Result<(), DbError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM temp_humidity", [])?;
    for record in records {
        tx.execute(
            "INSERT INTO temp_humidity (timestamp_ms, record_type, temperature, humidity) VALUES (?1, ?2, ?3, ?4)",
            params![
                record.timestamp_ms,
                record.record_type,
                record.temperature,
                record.humidity
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::parse::SleepSegment;

    #[test]
    fn persist_and_load_heart_rate() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        let records = vec![HeartRateRecord {
            timestamp_ms: 1_700_000_000_000,
            mode: 1,
            bpm: 72,
        }];
        replace_heart_rate(&conn, &records).unwrap();
        let loaded = load_heart_rate(&conn).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].bpm, 72);
    }

    #[test]
    fn persist_and_load_sleep_with_segments() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        let records = vec![SleepRecord {
            start_ms: 1000,
            end_ms: 2000,
            deep_sleep_count: 1,
            light_sleep_count: 2,
            deep_sleep_total_secs: 100,
            light_sleep_total_secs: 200,
            rem_total_secs: 50,
            wake_count: 1,
            wake_duration_secs: 10,
            segments: vec![SleepSegment {
                sleep_type: 241,
                start_ms: 1000,
                duration_secs: 100,
            }],
        }];
        replace_sleep(&conn, &records).unwrap();
        let loaded = load_sleep(&conn).unwrap();
        assert_eq!(loaded[0].segments.len(), 1);
    }
}
