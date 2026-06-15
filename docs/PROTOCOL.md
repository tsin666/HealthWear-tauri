# 协议说明

> 完整路线图见 **[ROADMAP.md](./ROADMAP.md)**

## 架构概览

1. **前端**：Vue 3 负责 UI 与交互
2. **后端**：Rust 负责 BLE、协议解析与本地存储
3. **协议**：YCBT GATT 帧 + DATATYPE 常量

## 功能对照表

> 状态以 `ROADMAP.md` 第五节为准。

| 功能 | Tauri 位置 | 状态 |
|------|------------|------|
| 智慧触控 UI | `src/components/WisdomModeList.vue` | ✅ |
| 触控协议 | `src-tauri/src/wisdom.rs` | ✅ |
| GATT 传输 | `ble/desktop.rs` | ✅ |
| 健康同步 | `health/sync.rs` | ✅ |
| 睡眠 / 综合 / 体温 | `health/parse.rs` | ✅ |
| ECG | — | ⬜ P8 |
| 媒体控制扩展 | 待实现 | ⬜ |

## BLE 传输层

GATT 连接、封包、CRC、通知订阅已在 P2 完成。  
后续：MTU 分包、命令队列（P2 待办）、Android BLE（P7）。

## 短视频 / HID 控制

戒指固件通过 **HID** 发送按键事件；App 侧只需开启对应智慧模式（`setWitOnOff`）。

桌面版无法直接控制手机应用；Android 版可探索 AccessibilityService 或 MediaSession。

## 推荐路线

按 `ROADMAP.md`：**P7 Android BLE → P8 ECG**。
