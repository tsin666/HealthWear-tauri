use super::db::{self, DbError};
use super::parse::{
    BloodOxygenRecord, BloodPressureRecord, BodyTempRecord, HealthAllRecord, HeartRateRecord,
    SleepRecord, SportRecord, TempHumidityRecord,
};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct HealthStore {
    conn: Mutex<Connection>,
    heart_rate: Mutex<Vec<HeartRateRecord>>,
    blood_oxygen: Mutex<Vec<BloodOxygenRecord>>,
    sport: Mutex<Vec<SportRecord>>,
    blood_pressure: Mutex<Vec<BloodPressureRecord>>,
    sleep: Mutex<Vec<SleepRecord>>,
    health_all: Mutex<Vec<HealthAllRecord>>,
    body_temp: Mutex<Vec<BodyTempRecord>>,
    temp_humidity: Mutex<Vec<TempHumidityRecord>>,
}

impl HealthStore {
    pub fn open(db_path: PathBuf) -> Result<Self, DbError> {
        let conn = db::open(&db_path)?;
        let store = Self {
            conn: Mutex::new(conn),
            heart_rate: Mutex::new(Vec::new()),
            blood_oxygen: Mutex::new(Vec::new()),
            sport: Mutex::new(Vec::new()),
            blood_pressure: Mutex::new(Vec::new()),
            sleep: Mutex::new(Vec::new()),
            health_all: Mutex::new(Vec::new()),
            body_temp: Mutex::new(Vec::new()),
            temp_humidity: Mutex::new(Vec::new()),
        };
        store.reload_all()?;
        Ok(store)
    }

    fn conn(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    fn reload_all(&self) -> Result<(), DbError> {
        let conn = self.conn();
        *self.heart_rate.lock() = db::load_heart_rate(&conn)?;
        *self.blood_oxygen.lock() = db::load_blood_oxygen(&conn)?;
        *self.sport.lock() = db::load_sport(&conn)?;
        *self.blood_pressure.lock() = db::load_blood_pressure(&conn)?;
        *self.sleep.lock() = db::load_sleep(&conn)?;
        *self.health_all.lock() = db::load_health_all(&conn)?;
        *self.body_temp.lock() = db::load_body_temp(&conn)?;
        *self.temp_humidity.lock() = db::load_temp_humidity(&conn)?;
        Ok(())
    }

    pub fn db_path_hint(&self) -> Option<PathBuf> {
        self.conn().path().map(PathBuf::from)
    }

    pub fn set_heart_rate(&self, records: Vec<HeartRateRecord>) -> Result<(), DbError> {
        db::replace_heart_rate(&self.conn(), &records)?;
        *self.heart_rate.lock() = records;
        Ok(())
    }

    pub fn get_heart_rate(&self) -> Vec<HeartRateRecord> {
        self.heart_rate.lock().clone()
    }

    pub fn set_blood_oxygen(&self, records: Vec<BloodOxygenRecord>) -> Result<(), DbError> {
        db::replace_blood_oxygen(&self.conn(), &records)?;
        *self.blood_oxygen.lock() = records;
        Ok(())
    }

    pub fn get_blood_oxygen(&self) -> Vec<BloodOxygenRecord> {
        self.blood_oxygen.lock().clone()
    }

    pub fn set_sport(&self, records: Vec<SportRecord>) -> Result<(), DbError> {
        db::replace_sport(&self.conn(), &records)?;
        *self.sport.lock() = records;
        Ok(())
    }

    pub fn get_sport(&self) -> Vec<SportRecord> {
        self.sport.lock().clone()
    }

    pub fn set_blood_pressure(&self, records: Vec<BloodPressureRecord>) -> Result<(), DbError> {
        db::replace_blood_pressure(&self.conn(), &records)?;
        *self.blood_pressure.lock() = records;
        Ok(())
    }

    pub fn get_blood_pressure(&self) -> Vec<BloodPressureRecord> {
        self.blood_pressure.lock().clone()
    }

    pub fn set_sleep(&self, records: Vec<SleepRecord>) -> Result<(), DbError> {
        db::replace_sleep(&self.conn(), &records)?;
        *self.sleep.lock() = records;
        Ok(())
    }

    pub fn get_sleep(&self) -> Vec<SleepRecord> {
        self.sleep.lock().clone()
    }

    pub fn set_health_all(&self, records: Vec<HealthAllRecord>) -> Result<(), DbError> {
        db::replace_health_all(&self.conn(), &records)?;
        *self.health_all.lock() = records;
        Ok(())
    }

    pub fn get_health_all(&self) -> Vec<HealthAllRecord> {
        self.health_all.lock().clone()
    }

    pub fn set_body_temp(&self, records: Vec<BodyTempRecord>) -> Result<(), DbError> {
        db::replace_body_temp(&self.conn(), &records)?;
        *self.body_temp.lock() = records;
        Ok(())
    }

    pub fn get_body_temp(&self) -> Vec<BodyTempRecord> {
        self.body_temp.lock().clone()
    }

    pub fn set_temp_humidity(&self, records: Vec<TempHumidityRecord>) -> Result<(), DbError> {
        db::replace_temp_humidity(&self.conn(), &records)?;
        *self.temp_humidity.lock() = records;
        Ok(())
    }

    pub fn get_temp_humidity(&self) -> Vec<TempHumidityRecord> {
        self.temp_humidity.lock().clone()
    }
}
