use super::protocol;
use super::{BleError, ConnectionInfo, RingBleBackend, ScannedDevice};
use crate::wisdom::WisdomState;
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use uuid::Uuid;

const SCAN_SECONDS: u64 = 4;
const CMD_TIMEOUT: Duration = Duration::from_secs(8);

struct Session {
    peripheral: Peripheral,
    write_char: Characteristic,
    notify_char: Characteristic,
    recv_buffer: Mutex<Vec<u8>>,
    pending: Mutex<Option<PendingCmd>>,
}

struct PendingCmd {
    data_type: u16,
    tx: oneshot::Sender<Result<Vec<u8>, String>>,
}

pub struct DesktopBleBackend {
    runtime: Runtime,
    manager: Manager,
    connection: Mutex<ConnectionInfo>,
    wisdom: Mutex<WisdomState>,
    session: Mutex<Option<Arc<Session>>>,
}

impl DesktopBleBackend {
    pub fn try_new() -> Result<Self, BleError> {
        let runtime = Runtime::new().map_err(|e| BleError::Inner(e.to_string()))?;
        let manager = runtime
            .block_on(Manager::new())
            .map_err(|e| BleError::Inner(e.to_string()))?;
        Ok(Self {
            runtime,
            manager,
            connection: Mutex::new(ConnectionInfo {
                connected: false,
                device_id: None,
                device_name: None,
                mock: false,
            }),
            wisdom: Mutex::new(WisdomState::default()),
            session: Mutex::new(None),
        })
    }

    fn block_on<F, T>(&self, fut: F) -> Result<T, BleError>
    where
        F: std::future::Future<Output = Result<T, BleError>>,
    {
        self.runtime.block_on(fut)
    }

    async fn adapter(&self) -> Result<Adapter, BleError> {
        let adapters = self
            .manager
            .adapters()
            .await
            .map_err(|e| BleError::Inner(e.to_string()))?;
        adapters
            .into_iter()
            .next()
            .ok_or_else(|| BleError::Inner("未找到蓝牙适配器".into()))
    }

    fn scan_blocking(&self) -> Result<Vec<ScannedDevice>, BleError> {
        self.block_on(async {
            let central = self.adapter().await?;
            central
                .start_scan(ScanFilter::default())
                .await
                .map_err(|e| BleError::Inner(e.to_string()))?;

            tokio::time::sleep(Duration::from_secs(SCAN_SECONDS)).await;

            let peripherals = central
                .peripherals()
                .await
                .map_err(|e| BleError::Inner(e.to_string()))?;

            let ycbt_service = Uuid::parse_str(protocol::YCBT_SERVICE_UUID)
                .map_err(|e| BleError::Inner(e.to_string()))?;

            let mut devices = Vec::new();
            let mut fallback = Vec::new();

            for peripheral in peripherals {
                let props = peripheral.properties().await.ok().flatten();
                let name = props
                    .as_ref()
                    .and_then(|p| p.local_name.clone())
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| "未知设备".into());

                let name_match = protocol::is_ring_candidate(&name);

                let service_match = props
                    .as_ref()
                    .map(|p| p.services.contains(&ycbt_service))
                    .unwrap_or(false);

                let entry = ScannedDevice {
                    id: peripheral.id().to_string(),
                    name: name.clone(),
                    rssi: props.and_then(|p| p.rssi),
                };

                if name_match || service_match {
                    devices.push(entry);
                } else if name != "未知设备" {
                    fallback.push(entry);
                }
            }

            let _ = central.stop_scan().await;

            if devices.is_empty() {
                devices = fallback;
            }

            devices.sort_by(|a, b| {
                let a_pri = protocol::is_ring_candidate(&a.name) as i8;
                let b_pri = protocol::is_ring_candidate(&b.name) as i8;
                b_pri.cmp(&a_pri).then_with(|| {
                    b.rssi
                        .unwrap_or(i16::MIN)
                        .cmp(&a.rssi.unwrap_or(i16::MIN))
                })
            });

            Ok(devices)
        })
    }

    async fn find_peripheral(&self, device_id: &str) -> Result<Peripheral, BleError> {
        let central = self.adapter().await?;

        async fn lookup(central: &Adapter, device_id: &str) -> Result<Peripheral, BleError> {
            let peripherals = central
                .peripherals()
                .await
                .map_err(|e| BleError::Inner(e.to_string()))?;
            for peripheral in peripherals {
                if peripheral.id().to_string() == device_id {
                    return Ok(peripheral);
                }
            }
            Err(BleError::Inner(format!("未找到设备: {device_id}")))
        }

        if let Ok(peripheral) = lookup(&central, device_id).await {
            return Ok(peripheral);
        }

        central
            .start_scan(ScanFilter::default())
            .await
            .map_err(|e| BleError::Inner(e.to_string()))?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let peripheral = lookup(&central, device_id).await?;
        let _ = central.stop_scan().await;
        Ok(peripheral)
    }

    fn find_ycbt_chars(
        peripheral: &Peripheral,
    ) -> Result<(Characteristic, Characteristic), BleError> {
        let service_uuid =
            Uuid::parse_str(protocol::YCBT_SERVICE_UUID).map_err(|e| BleError::Inner(e.to_string()))?;
        let write_uuid = Uuid::parse_str(protocol::YCBT_WRITE_CHAR_UUID)
            .map_err(|e| BleError::Inner(e.to_string()))?;
        let notify_uuid = Uuid::parse_str(protocol::YCBT_NOTIFY_CHAR_UUID)
            .map_err(|e| BleError::Inner(e.to_string()))?;

        let chars = peripheral.characteristics();
        let in_service = |c: &Characteristic| c.service_uuid == service_uuid;

        let write_char = chars
            .iter()
            .find(|c| in_service(c) && c.uuid == write_uuid)
            .cloned()
            .ok_or_else(|| BleError::Inner("未找到 YCBT 写特征 be940001".into()))?;

        let notify_char = chars
            .iter()
            .find(|c| in_service(c) && c.uuid == notify_uuid)
            .cloned()
            .ok_or_else(|| BleError::Inner("未找到 YCBT 通知特征 be940003".into()))?;

        Ok((write_char, notify_char))
    }

    async fn connect_gatt(&self, device_id: &str) -> Result<(), BleError> {
        let peripheral = self.find_peripheral(device_id).await?;

        peripheral
            .connect()
            .await
            .map_err(|e| BleError::Inner(format!("GATT 连接失败: {e}")))?;

        peripheral
            .discover_services()
            .await
            .map_err(|e| BleError::Inner(format!("服务发现失败: {e}")))?;

        let device_name = peripheral
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|p| p.local_name)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| device_id.to_string());

        let (write_char, notify_char) = Self::find_ycbt_chars(&peripheral)?;

        peripheral
            .subscribe(&notify_char)
            .await
            .map_err(|e| BleError::Inner(format!("订阅通知失败: {e}")))?;

        let session = Arc::new(Session {
            peripheral,
            write_char,
            notify_char,
            recv_buffer: Mutex::new(Vec::new()),
            pending: Mutex::new(None),
        });

        Self::spawn_notify_listener(session.clone());

        // 连接后查询当前智慧触控状态
        if let Ok(state) = self
            .send_command_on_session(
                &session,
                protocol::DATATYPE_CUSTOMIZE_INTELLIGENT_FUNCTIONS,
                &protocol::build_get_wit_payload(),
                CMD_TIMEOUT,
            )
            .await
        {
            if let Some(parsed) = protocol::parse_get_wit_response(&state) {
                *self.wisdom.lock() = parsed;
            }
        }

        *self.session.lock() = Some(session);
        let mut conn = self.connection.lock();
        conn.connected = true;
        conn.device_id = Some(device_id.to_string());
        conn.device_name = Some(device_name);
        conn.mock = false;

        Ok(())
    }

    fn spawn_notify_listener(session: Arc<Session>) {
        let peripheral = session.peripheral.clone();
        let session_clone = session.clone();

        tokio::spawn(async move {
            let Ok(mut stream) = peripheral.notifications().await else {
                return;
            };

            use futures_util::StreamExt;
            while let Some(notification) = stream.next().await {
                Self::handle_notification(&session_clone, &notification.value);
            }
        });
    }

    fn handle_notification(session: &Session, chunk: &[u8]) {
        let mut buffer = session.recv_buffer.lock();
        let frame = match protocol::try_reassemble(&mut buffer, chunk) {
            Some(f) => f,
            None => return,
        };

        let Ok(parsed) = protocol::parse_frame(&frame) else {
            return;
        };

        let mut pending = session.pending.lock();
        if let Some(cmd) = pending.take() {
            if cmd.data_type == parsed.data_type {
                let _ = cmd.tx.send(Ok(parsed.payload));
            } else {
                *pending = Some(cmd);
            }
        }
    }

    async fn send_command_on_session(
        &self,
        session: &Session,
        data_type: u16,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, BleError> {
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = session.pending.lock();
            *pending = Some(PendingCmd { data_type, tx });
        }

        let frame = protocol::build_frame(data_type, payload);
        session
            .peripheral
            .write(&session.write_char, &frame, WriteType::WithResponse)
            .await
            .map_err(|e| {
                session.pending.lock().take();
                BleError::Inner(format!("写入特征失败: {e}"))
            })?;

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(Ok(payload))) => Ok(payload),
            Ok(Ok(Err(msg))) => Err(BleError::Inner(msg)),
            Ok(Err(_)) => {
                session.pending.lock().take();
                Err(BleError::Inner("通知通道已关闭".into()))
            }
            Err(_) => {
                session.pending.lock().take();
                Err(BleError::Inner("等待戒指响应超时".into()))
            }
        }
    }

    async fn send_command(
        &self,
        data_type: u16,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, BleError> {
        let session = self
            .session
            .lock()
            .clone()
            .ok_or(BleError::NotConnected)?;
        self.send_command_on_session(&session, data_type, payload, timeout)
            .await
    }

    async fn disconnect_gatt(&self) -> Result<(), BleError> {
        if let Some(session) = self.session.lock().take() {
            let _ = session
                .peripheral
                .unsubscribe(&session.notify_char)
                .await;
            let _ = session.peripheral.disconnect().await;
        }
        Ok(())
    }
}

impl RingBleBackend for DesktopBleBackend {
    fn scan(&self) -> Result<Vec<ScannedDevice>, BleError> {
        self.scan_blocking()
    }

    fn connect(&self, device_id: &str) -> Result<(), BleError> {
        if device_id.starts_with("mock-ring") {
            let mut conn = self.connection.lock();
            conn.connected = true;
            conn.device_id = Some(device_id.to_string());
            conn.device_name = Some("Health Ring (模拟测试)".into());
            conn.mock = true;
            return Ok(());
        }

        self.block_on(async {
            self.disconnect_gatt().await.ok();
            self.connect_gatt(device_id).await
        })
    }

    fn disconnect(&self) -> Result<(), BleError> {
        self.block_on(async {
            self.disconnect_gatt().await?;
            *self.connection.lock() = ConnectionInfo {
                connected: false,
                device_id: None,
                device_name: None,
                mock: false,
            };
            *self.wisdom.lock() = WisdomState::default();
            Ok(())
        })
    }

    fn connection_info(&self) -> ConnectionInfo {
        self.connection.lock().clone()
    }

    fn set_wit_mode(&self, on: bool, protocol_index: u8) -> Result<(), BleError> {
        if !self.connection.lock().connected {
            return Err(BleError::NotConnected);
        }

        if self.connection.lock().mock {
            self.wisdom.lock().apply_toggle(protocol_index, on);
            return Ok(());
        }

        let payload = protocol::build_set_wit_payload(on, protocol_index);
        self.block_on(async {
            let response = self
                .send_command(
                    protocol::DATATYPE_CUSTOMIZE_INTELLIGENT_FUNCTIONS,
                    &payload,
                    CMD_TIMEOUT,
                )
                .await?;
            protocol::parse_set_wit_response(&response)
                .map_err(|e| BleError::Inner(e))?;
            self.wisdom.lock().apply_toggle(protocol_index, on);
            Ok(())
        })
    }

    fn get_wit_state(&self) -> Result<WisdomState, BleError> {
        if !self.connection.lock().connected {
            return Err(BleError::NotConnected);
        }

        if self.connection.lock().mock {
            return Ok(self.wisdom.lock().clone());
        }

        self.block_on(async {
            let response = self
                .send_command(
                    protocol::DATATYPE_CUSTOMIZE_INTELLIGENT_FUNCTIONS,
                    &protocol::build_get_wit_payload(),
                    CMD_TIMEOUT,
                )
                .await?;
            let state = protocol::parse_get_wit_response(&response)
                .ok_or_else(|| BleError::Inner("解析智慧触控状态失败".into()))?;
            *self.wisdom.lock() = state.clone();
            Ok(state)
        })
    }

    fn send_raw_command(
        &self,
        data_type: u16,
        payload: &[u8],
        timeout_secs: u64,
    ) -> Result<Vec<u8>, BleError> {
        if !self.connection.lock().connected {
            return Err(BleError::NotConnected);
        }
        if self.connection.lock().mock {
            return Err(BleError::Inner("模拟设备请使用健康同步的演示数据".into()));
        }
        let timeout = Duration::from_secs(timeout_secs.max(3));
        self.block_on(async { self.send_command(data_type, payload, timeout).await })
    }
}
