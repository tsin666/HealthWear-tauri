# Android BLE 接入说明

HealthWear Android 端与桌面端共用 Rust `btleplug` 后端（`src-tauri/src/ble/btleplug.rs`）。  
由于 btleplug 在 Android 上需要 **Rust + Java 混合构建**，需额外准备 Java 库。

## 1. 权限（已配置）

- `AndroidManifest.xml`：蓝牙扫描/连接权限
- `MainActivity.kt`：启动时请求运行时权限

## 2. 构建 btleplug Java 库

```bash
cd HealthWear-tauri
./scripts/setup-android-ble.sh
```

脚本会：

1. 克隆 `jni-utils-rs` 与 `btleplug`
2. 构建 jni-utils JAR
3. 构建 btleplug AAR
4. 复制到 `src-tauri/gen/android/app/libs/`

## 3. 运行 Android 应用

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
pnpm tauri android dev
```

## 4. 验证

- 设备面板显示 `backend: btleplug`（非 `mock`）
- 扫描可发现 Q520 等戒指
- 连接后可同步健康数据

## 5. Release 构建

`proguard-rules.pro` 已添加 btleplug keep 规则，避免 R8 剥离 JNI 类。

## 故障排查

| 现象 | 处理 |
|------|------|
| 面板显示 `mock` | 检查 libs 目录是否有 AAR，重新运行 setup 脚本 |
| 扫描为空 | 确认已授予蓝牙/定位权限 |
| 连接超时 | 关闭系统蓝牙中对戒指的已有连接 |
