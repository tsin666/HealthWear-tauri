use serde::{Deserialize, Serialize};

/// 对应原 App `Constant.Wisdom` 与 `LifeFragment` 中的 protocolIndex
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WisdomMode {
    Video = 1,
    Music = 2,
    Read = 3,
    TakePhoto = 4,
    Sos = 5,
    Slides = 6,
}

impl WisdomMode {
    pub fn protocol_index(self) -> u8 {
        self as u8
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "短视频",
            Self::Music => "音乐",
            Self::Read => "阅读",
            Self::TakePhoto => "拍照/录像",
            Self::Sos => "SOS 求助",
            Self::Slides => "幻灯片",
        }
    }

    pub fn hint_primary(self) -> &'static str {
        match self {
            Self::Video => "上滑/下滑：切换视频",
            Self::Music => "双击：播放/暂停",
            Self::Read => "上滑/下滑：左右翻页",
            Self::TakePhoto => "双击：拍照/录像",
            Self::Sos => "双击 3 次 + 长按 5 秒",
            Self::Slides => "上滑/下滑：切换幻灯片",
        }
    }

    pub fn hint_secondary(self) -> Option<&'static str> {
        match self {
            Self::Video => Some("双击：点赞"),
            Self::Music => Some("上滑/下滑：切换歌曲"),
            _ => None,
        }
    }

    pub fn all() -> [Self; 6] {
        [
            Self::Video,
            Self::Music,
            Self::Read,
            Self::TakePhoto,
            Self::Sos,
            Self::Slides,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WisdomModeItem {
    pub id: String,
    pub protocol_index: u8,
    pub title: String,
    pub hint_primary: String,
    pub hint_secondary: Option<String>,
    pub enabled: bool,
    pub exclusive: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WisdomState {
    pub active_mode: Option<u8>,
    pub sos_enabled: bool,
}

impl WisdomState {
    pub fn apply_toggle(&mut self, protocol_index: u8, enabled: bool) {
        if protocol_index == WisdomMode::Sos as u8 {
            self.sos_enabled = enabled;
            return;
        }
        if enabled {
            self.active_mode = Some(protocol_index);
        } else if self.active_mode == Some(protocol_index) {
            self.active_mode = None;
        }
    }

    pub fn is_enabled(&self, protocol_index: u8) -> bool {
        if protocol_index == WisdomMode::Sos as u8 {
            return self.sos_enabled;
        }
        self.active_mode == Some(protocol_index)
    }
}

pub fn build_mode_list(state: &WisdomState) -> Vec<WisdomModeItem> {
    WisdomMode::all()
        .into_iter()
        .map(|mode| {
            let protocol_index = mode.protocol_index();
            WisdomModeItem {
                id: format!("mode-{protocol_index}"),
                protocol_index,
                title: mode.label().to_string(),
                hint_primary: mode.hint_primary().to_string(),
                hint_secondary: mode.hint_secondary().map(str::to_string),
                enabled: state.is_enabled(protocol_index),
                exclusive: protocol_index != WisdomMode::Sos as u8,
            }
        })
        .collect()
}
