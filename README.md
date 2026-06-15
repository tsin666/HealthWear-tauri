# HealthWear Tauri 跨平台版

仓库：[https://github.com/tsin666/HealthWear-tauri](https://github.com/tsin666/HealthWear-tauri)

用 **Tauri 2 + Vue 3 + Rust** 编写的跨平台智能戒指控制台。

## 支持平台

| 平台 | 状态 | 说明 |
|------|------|------|
| macOS | ✅ | 真实 BLE（`btleplug`）+ Q520 戒指识别 |
| Windows | ✅ | 同上 |
| Linux | ✅ | 需 BlueZ |
| Android | 🚧 | 已 `tauri android init`，待稳定跑模拟器/真机 |
| iOS | 🚧 | `pnpm tauri ios init` |

## 已实现

- 6 种智能触控模式（`setWitOnOff` / `getWitState`）
- 桌面端真实 GATT 连接（`be940000` 服务）
- 健康模块：心率 / 血氧 / 运动 / 血压 / 睡眠 / 综合 / 体温 / 温湿度
- Vue 3 中文 UI：设备连接 / 触控模式 / 健康数据
- **SQLite 本地持久化** + CSV 导出

## 进行中 / 未实现

- ECG（见 `docs/HEALTH_MODULES.md`）
- 账号登录 / 云同步 / OTA
- Android 端真实 BLE（当前回退 Mock）

## 快速开始

```bash
git clone https://github.com/tsin666/HealthWear-tauri.git
cd HealthWear-tauri
pnpm install
pnpm tauri dev
```

## 项目结构

```
src/                      # Vue 3 前端
src-tauri/src/
  wisdom.rs               # 6 种触控模式
  ble/                    # GATT + YCBT 封包
  health/                 # 健康协议、解析、SQLite
  commands.rs             # Tauri API
docs/
  ROADMAP.md              # 开发路线图（主文档）
  HEALTH_MODULES.md       # 健康模块说明
  PROTOCOL.md             # 协议速查
```

## Android 开发

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
pnpm tauri android dev
```

需本机安装 Android SDK、NDK 与模拟器/真机。

## 文档

| 文档 | 说明 |
|------|------|
| [docs/ROADMAP.md](docs/ROADMAP.md) | **开发路线图**（阶段 P0–P9） |
| [docs/HEALTH_MODULES.md](docs/HEALTH_MODULES.md) | 健康模块协议与实现 |
| [docs/PROTOCOL.md](docs/PROTOCOL.md) | BLE / 触控协议速查 |
