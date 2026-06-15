# 健康模块开发说明

## 思路

按功能模块在 Rust 层实现 YCBT 健康协议：传输 → 解析 → 同步 → 存储 → UI。

```
传输层   ble/desktop.rs       GATT 连接、收发、通知
解析层   health/parse.rs      二进制 payload 解析
同步层   health/sync.rs       历史数据拉取
存储层   health/store.rs      SQLite 持久化
接口层   commands.rs          Tauri API
UI 层    HealthPanel.vue      展示与操作
```

## 分层

| 层 | 目录 | 职责 |
|----|------|------|
| 传输 | `ble/` | GATT、封包、CRC |
| 协议 | `health/constants.rs` | DATATYPE 常量 |
| 解析 | `health/parse.rs` | 各类型二进制布局 |
| 业务 | `health/sync.rs` | 同步流程 |
| 存储 | `health/store.rs` + `db.rs` | SQLite |
| API | `commands.rs` | invoke 接口 |
| UI | `HealthPanel.vue` | 模块列表与数据 |

## 实现顺序

| 阶段 | 功能 | DATATYPE | 状态 |
|------|------|----------|------|
| 0 | 智慧触控 | `0x0D02` | ✅ |
| 1 | 心率 | 1286 | ✅ |
| 2 | 血氧 | 1306 | ✅ |
| 3 | 运动/步数 | 1282 | ✅ |
| 4 | 血压 | 1288 | ✅ |
| 5 | 睡眠 | 1284 | ✅ |
| 6 | 综合指标 | 1289 | ✅ |
| 7 | 体温 | 1310 | ✅ |
| 8 | 温湿度 | 1308 | ✅ |
| 9 | ECG | 1541 | ⬜ |
| 10 | 云同步/账号 | HTTP | 可选 |

## 单功能开发 checklist

以心率为模板：

1. 确认 DATATYPE 数值
2. 分析 payload 字节布局（case 编号见 parse.rs 注释）
3. 写入 `health/parse.rs` + 单元测试
4. `health/sync.rs` 调用 `ble.send_raw_command(data_type, &[])`
5. 更新 `store.rs` / `db.rs`
6. 注册 Tauri command + Vue 展示
7. 真机连戒指验证

## 同步 API 示例

```rust
ble.send_raw_command(DATATYPE_HISTORY_HEART, &[], 15)?;
parse_heart_history(&payload)?;
```

## ECG 说明

ECG 涉及实时波形与可选的分析接口，计划分阶段：

- **阶段 1**：原始波形拉取 + 简单展示
- **阶段 2**：分析能力（平台相关，待评估）
