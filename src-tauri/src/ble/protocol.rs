use crate::wisdom::WisdomState;

/// `Constants.DATATYPE.Customize_Intelligent_Functions`
pub const DATATYPE_CUSTOMIZE_INTELLIGENT_FUNCTIONS: u16 = 3330; // 0x0D02

pub const YCBT_SERVICE_UUID: &str = "be940000-7333-be46-b7ae-689e71722bd5";
pub const YCBT_WRITE_CHAR_UUID: &str = "be940001-7333-be46-b7ae-689e71722bd5";
pub const YCBT_WRITE_CHAR2_UUID: &str = "be940002-7333-be46-b7ae-689e71722bd5";
pub const YCBT_NOTIFY_CHAR_UUID: &str = "be940003-7333-be46-b7ae-689e71722bd5";

pub const UART_SERVICE_UUID: &str = "6e400001-b5a3-f393-e0a9-e50e24dcca9e";
pub const UART_RX_CHARACTERISTIC: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";
pub const UART_TX_CHARACTERISTIC: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";

#[derive(Debug, Clone)]
pub struct ParsedFrame {
    pub data_type: u16,
    pub payload: Vec<u8>,
}

/// 原 App `ByteUtil.crc16_compute`（按 Java signed short 语义移植）
pub fn crc16_compute(data: &[u8]) -> u16 {
    let mut s: i16 = -1;
    for &byte in data {
        let inner = (((s as u16) << 8) & 0xFF00) | (((s as u16) >> 8) & 0x00FF);
        let inner2 = inner ^ u16::from(byte);
        let inner2s = inner2 as i16;
        let s2 = inner2s ^ (((inner2 & 0x00FF) >> 4) as i16);
        let s3 = s2 ^ (s2 << 12);
        s = s3 ^ ((((s3 as u16) & 0x00FF) << 5) as i16);
    }
    s as u16
}

/// 原 App `YCBTClientImpl.sendData2Device`
pub fn build_frame(data_type: u16, payload: &[u8]) -> Vec<u8> {
    let total_len = payload.len() + 6;
    let mut frame = vec![0u8; total_len];
    frame[0] = ((data_type >> 8) & 0xFF) as u8;
    frame[1] = (data_type & 0xFF) as u8;
    frame[2] = (total_len & 0xFF) as u8;
    frame[3] = ((total_len >> 8) & 0xFF) as u8;
    frame[4..4 + payload.len()].copy_from_slice(payload);
    let crc = crc16_compute(&frame[..4 + payload.len()]);
    frame[4 + payload.len()] = (crc & 0xFF) as u8;
    frame[5 + payload.len()] = ((crc >> 8) & 0xFF) as u8;
    frame
}

/// 解析完整单包（`bleDataResponse` 在 `array.len() == cmdlen` 时）
pub fn parse_frame(data: &[u8]) -> Result<ParsedFrame, String> {
    if data.len() < 6 {
        return Err("帧长度不足".into());
    }
    let data_type = (u16::from(data[0]) << 8) | u16::from(data[1]);
    let cmd_len = usize::from(data[2]) | (usize::from(data[3]) << 8);
    if data.len() != cmd_len {
        return Err(format!("帧长度不匹配: 期望 {cmd_len}, 实际 {}", data.len()));
    }
    let recv_crc =
        u16::from(data[cmd_len - 2]) | (u16::from(data[cmd_len - 1]) << 8);
    let calc_crc = crc16_compute(&data[..cmd_len - 2]);
    if recv_crc != calc_crc {
        return Err(format!("CRC 校验失败: {recv_crc} != {calc_crc}"));
    }
    let payload = data[4..cmd_len - 2].to_vec();
    Ok(ParsedFrame { data_type, payload })
}

/// 多包重组：首包带 4 字节头，后续包可不带（原 App `isFlag` 逻辑）
pub fn try_reassemble(buffer: &mut Vec<u8>, chunk: &[u8]) -> Option<Vec<u8>> {
    buffer.extend_from_slice(chunk);
    if buffer.len() < 4 {
        return None;
    }
    let cmd_len = usize::from(buffer[2]) | (usize::from(buffer[3]) << 8);
    if buffer.len() < cmd_len {
        return None;
    }
    if buffer.len() > cmd_len {
        buffer.drain(..cmd_len);
        return try_reassemble(buffer, &[]);
    }
    let frame = buffer.clone();
    buffer.clear();
    Some(frame)
}

pub fn build_set_wit_payload(on: bool, protocol_index: u8) -> Vec<u8> {
    vec![1, u8::from(on), protocol_index]
}

pub fn build_get_wit_payload() -> Vec<u8> {
    vec![2]
}

/// 判断广播名是否像 YCBT 智能戒指（含 Q520 2A90 等型号）
pub fn is_ring_candidate(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.contains("ring")
        || lower.contains("ycbt")
        || lower.contains("health")
        || lower.contains("wear")
        || lower.contains("smart")
        || lower.contains("r11")
        || lower.contains("r12")
        || lower.contains("q520")
        || lower.contains("q521")
    {
        return true;
    }
    // Q520 2A90、Q521 xxxx 等常见命名：Q + 数字开头
    if let Some(rest) = lower.strip_prefix('q') {
        return rest.chars().next().is_some_and(|c| c.is_ascii_digit());
    }
    false
}

/// 解析 getWit 响应
pub fn parse_get_wit_response(payload: &[u8]) -> Option<WisdomState> {
    if payload.is_empty() {
        return None;
    }
    let opcode = payload[0];
    if opcode != 2 {
        return None;
    }
    let flags = payload.get(1).copied().unwrap_or(0);
    let mut state = WisdomState::default();
    if flags & 1 != 0 {
        state.active_mode = Some(1);
    } else if (flags >> 1) & 1 != 0 {
        state.active_mode = Some(2);
    } else if (flags >> 2) & 1 != 0 {
        state.active_mode = Some(3);
    } else if (flags >> 3) & 1 != 0 {
        state.active_mode = Some(4);
    } else if (flags >> 5) & 1 != 0 {
        state.active_mode = Some(6);
    }
    state.sos_enabled = (flags >> 4) & 1 != 0;
    Some(state)
}

/// 解析 setWit 响应，code==0 为成功
pub fn parse_set_wit_response(payload: &[u8]) -> Result<(), String> {
    if payload.is_empty() {
        return Err("空响应".into());
    }
    if payload[0] != 1 {
        return Err(format!("未知 opcode: {}", payload[0]));
    }
    let code = payload.get(1).copied().unwrap_or(0xFF) as u16;
    if code == 0 {
        Ok(())
    } else {
        Err(format!("戒指返回错误码: {code}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q520_name_is_ring_candidate() {
        assert!(is_ring_candidate("Q520 2A90"));
        assert!(is_ring_candidate("q521_abcd"));
        assert!(!is_ring_candidate("AirPods Pro"));
    }

    #[test]
    fn build_set_wit_frame_matches_java_layout() {
        let frame = build_frame(
            DATATYPE_CUSTOMIZE_INTELLIGENT_FUNCTIONS,
            &build_set_wit_payload(true, 1),
        );
        assert_eq!(frame.len(), 9);
        assert_eq!(frame[0], 0x0D);
        assert_eq!(frame[1], 0x02);
        assert_eq!(frame[2], 9);
        assert_eq!(&frame[4..7], &[1, 1, 1]);
        assert_eq!(parse_frame(&frame).unwrap().data_type, 3330);
    }
}
