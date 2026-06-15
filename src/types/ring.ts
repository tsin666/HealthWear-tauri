export interface ConnectionInfo {
  connected: boolean;
  deviceId: string | null;
  deviceName: string | null;
  mock: boolean;
}

export interface BlePlatformInfo {
  os: string;
  backend: string;
  bleAvailable: boolean;
  hint: string | null;
}

export interface ScannedDevice {
  id: string;
  name: string;
  rssi: number | null;
}

export interface WisdomModeItem {
  id: string;
  protocolIndex: number;
  title: string;
  hintPrimary: string;
  hintSecondary: string | null;
  enabled: boolean;
  exclusive: boolean;
}

export interface WisdomState {
  activeMode: number | null;
  sosEnabled: boolean;
}

export interface ApiError {
  message: string;
}
