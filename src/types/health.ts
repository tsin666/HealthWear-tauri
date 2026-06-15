export type HealthModuleStatus = "ready" | "planned" | "jni_required";

export type HealthModuleId =
  | "heart_rate"
  | "sleep"
  | "blood_oxygen"
  | "blood_pressure"
  | "sport"
  | "health_all"
  | "body_temp"
  | "temp_humidity"
  | "ecg";

export interface HealthModuleInfo {
  id: HealthModuleId;
  title: string;
  description: string;
  dataType: number | null;
  status: HealthModuleStatus;
  recordCount: number;
}

export interface HeartRateRecord {
  timestampMs: number;
  mode: number;
  bpm: number;
}

export interface BloodOxygenRecord {
  timestampMs: number;
  recordType: number;
  spo2: number;
}

export interface SportRecord {
  startMs: number;
  endMs: number;
  steps: number;
  distance: number;
  calories: number;
}

export interface BloodPressureRecord {
  timestampMs: number;
  isInflated: number;
  sbp: number;
  dbp: number;
  heartRate: number;
}

export interface SleepSegment {
  sleepType: number;
  startMs: number;
  durationSecs: number;
}

export interface SleepRecord {
  startMs: number;
  endMs: number;
  deepSleepCount: number;
  lightSleepCount: number;
  deepSleepTotalSecs: number;
  lightSleepTotalSecs: number;
  remTotalSecs: number;
  wakeCount: number;
  wakeDurationSecs: number;
  segments: SleepSegment[];
}

export interface HealthAllRecord {
  timestampMs: number;
  steps: number;
  heartRate: number;
  sbp: number;
  dbp: number;
  spo2: number;
  respiratoryRate: number;
  hrv: number;
  cvrr: number;
  temperature: number;
  bodyFat: number;
  bloodSugar: number;
}

export interface BodyTempRecord {
  timestampMs: number;
  recordType: number;
  temperature: number;
}

export interface TempHumidityRecord {
  timestampMs: number;
  recordType: number;
  temperature: number;
  humidity: number;
}

export interface HealthSnapshot {
  heartRate: HeartRateRecord[];
  bloodOxygen: BloodOxygenRecord[];
  sport: SportRecord[];
  bloodPressure: BloodPressureRecord[];
  sleep: SleepRecord[];
  healthAll: HealthAllRecord[];
  bodyTemp: BodyTempRecord[];
  tempHumidity: TempHumidityRecord[];
}
