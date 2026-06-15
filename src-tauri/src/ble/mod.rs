pub mod protocol;

use crate::wisdom::WisdomState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BleError {
    #[error("蓝牙未初始化")]
    NotInitialized,
    #[error("未连接设备")]
    NotConnected,
    #[error("平台暂不支持 BLE: {0}")]
    Unsupported(String),
    #[error("蓝牙错误: {0}")]
    Inner(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedDevice {
    pub id: String,
    pub name: String,
    pub rssi: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub connected: bool,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub mock: bool,
}

pub trait RingBleBackend: Send + Sync {
    fn scan(&self) -> Result<Vec<ScannedDevice>, BleError>;
    fn connect(&self, device_id: &str) -> Result<(), BleError>;
    fn disconnect(&self) -> Result<(), BleError>;
    fn connection_info(&self) -> ConnectionInfo;
    fn set_wit_mode(&self, on: bool, protocol_index: u8) -> Result<(), BleError>;
    fn get_wit_state(&self) -> Result<WisdomState, BleError>;
    /// 通用 YCBT 指令（健康同步等）
    fn send_raw_command(
        &self,
        data_type: u16,
        payload: &[u8],
        timeout_secs: u64,
    ) -> Result<Vec<u8>, BleError>;
}

pub struct MockBleBackend {
    connection: parking_lot::Mutex<ConnectionInfo>,
    wisdom: parking_lot::Mutex<WisdomState>,
}

impl Default for MockBleBackend {
    fn default() -> Self {
        Self {
            connection: parking_lot::Mutex::new(ConnectionInfo {
                connected: false,
                device_id: None,
                device_name: None,
                mock: true,
            }),
            wisdom: parking_lot::Mutex::new(WisdomState::default()),
        }
    }
}

impl RingBleBackend for MockBleBackend {
    fn scan(&self) -> Result<Vec<ScannedDevice>, BleError> {
        Ok(vec![
            ScannedDevice {
                id: "mock-ring-01".into(),
                name: "Health Ring (模拟)".into(),
                rssi: Some(-48),
            },
            ScannedDevice {
                id: "mock-ring-02".into(),
                name: "YCBT Ring".into(),
                rssi: Some(-62),
            },
        ])
    }

    fn connect(&self, device_id: &str) -> Result<(), BleError> {
        let mut conn = self.connection.lock();
        conn.connected = true;
        conn.device_id = Some(device_id.to_string());
        conn.device_name = Some(if device_id.contains("mock") {
            "Health Ring (模拟)".into()
        } else {
            device_id.to_string()
        });
        conn.mock = true;
        Ok(())
    }

    fn disconnect(&self) -> Result<(), BleError> {
        let mut conn = self.connection.lock();
        *conn = ConnectionInfo {
            connected: false,
            device_id: None,
            device_name: None,
            mock: true,
        };
        *self.wisdom.lock() = WisdomState::default();
        Ok(())
    }

    fn connection_info(&self) -> ConnectionInfo {
        self.connection.lock().clone()
    }

    fn set_wit_mode(&self, on: bool, protocol_index: u8) -> Result<(), BleError> {
        if !self.connection.lock().connected {
            return Err(BleError::NotConnected);
        }
        let payload = protocol::build_set_wit_payload(on, protocol_index);
        let _ = payload;
        self.wisdom.lock().apply_toggle(protocol_index, on);
        Ok(())
    }

    fn get_wit_state(&self) -> Result<WisdomState, BleError> {
        if !self.connection.lock().connected {
            return Err(BleError::NotConnected);
        }
        Ok(self.wisdom.lock().clone())
    }

    fn send_raw_command(
        &self,
        data_type: u16,
        payload: &[u8],
        _timeout_secs: u64,
    ) -> Result<Vec<u8>, BleError> {
        if !self.connection.lock().connected {
            return Err(BleError::NotConnected);
        }
        let _ = (data_type, payload);
        Err(BleError::Inner("模拟设备不支持原始协议请求".into()))
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod desktop;

pub type SharedBackend = Arc<dyn RingBleBackend>;

pub fn create_backend() -> SharedBackend {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Ok(backend) = desktop::DesktopBleBackend::try_new() {
            return Arc::new(backend);
        }
    }
    Arc::new(MockBleBackend::default())
}
