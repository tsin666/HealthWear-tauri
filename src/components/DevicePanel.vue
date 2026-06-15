<script setup lang="ts">
import { ref } from "vue";
import type { BlePlatformInfo, ConnectionInfo, ScannedDevice } from "../types/ring";
import {
  connectDevice,
  disconnectDevice,
  getBlePlatform,
  getConnection,
  scanDevices,
} from "../api/ring";

const emit = defineEmits<{
  refresh: [];
}>();

const connection = ref<ConnectionInfo | null>(null);
const platform = ref<BlePlatformInfo | null>(null);
const devices = ref<ScannedDevice[]>([]);
const scanning = ref(false);
const busy = ref(false);
const message = ref("");

async function refreshConnection() {
  connection.value = await getConnection();
  platform.value = await getBlePlatform();
}

async function handleScan() {
  scanning.value = true;
  message.value = "";
  try {
    devices.value = await scanDevices();
    if (devices.value.length === 0) {
      message.value = "未发现戒指设备，可使用模拟设备测试";
    }
  } catch (e) {
    message.value = (e as { message: string }).message;
  } finally {
    scanning.value = false;
  }
}

async function handleConnect(id: string) {
  busy.value = true;
  message.value = "";
  try {
    await connectDevice(id);
    await refreshConnection();
    emit("refresh");
  } catch (e) {
    message.value = (e as { message: string }).message;
  } finally {
    busy.value = false;
  }
}

async function handleDisconnect() {
  busy.value = true;
  try {
    await disconnectDevice();
    await refreshConnection();
    emit("refresh");
  } catch (e) {
    message.value = (e as { message: string }).message;
  } finally {
    busy.value = false;
  }
}

refreshConnection();
</script>

<template>
  <section class="panel">
    <div class="panel-head">
      <div>
        <h2>设备连接</h2>
        <p class="sub">
          {{
            connection?.connected
              ? `已连接：${connection.deviceName}`
              : "未连接戒指"
          }}
          <span v-if="connection?.mock" class="tag mock">模拟</span>
          <span v-else-if="connection?.connected" class="tag ble">真实 BLE</span>
          <span v-if="platform" class="tag platform">{{ platform.backend }}</span>
        </p>
        <p v-if="platform?.hint" class="platform-hint">{{ platform.hint }}</p>
      </div>
      <div class="actions">
        <button class="ghost" :disabled="scanning" @click="handleScan">
          {{ scanning ? "扫描中..." : "扫描设备" }}
        </button>
        <button
          v-if="connection?.connected"
          class="danger"
          :disabled="busy"
          @click="handleDisconnect"
        >
          断开
        </button>
      </div>
    </div>

    <ul v-if="devices.length" class="device-list">
      <li v-for="device in devices" :key="device.id">
        <div>
          <strong>{{ device.name }}</strong>
          <span class="meta">{{ device.rssi ?? "--" }} dBm</span>
        </div>
        <button
          class="primary"
          :disabled="busy || connection?.connected"
          @click="handleConnect(device.id)"
        >
          连接
        </button>
      </li>
    </ul>

    <p v-if="message" class="hint">{{ message }}</p>
    <p v-else class="hint tip">
      <template v-if="platform?.os === 'android'">
        Android 需授予蓝牙权限。真机 BLE 请先运行 scripts/setup-android-ble.sh 构建 Java 库。
      </template>
      <template v-else>
        戒指在 macOS 可能显示为「Apple 无线键盘」（如 Q520 2A90）。若连接失败，请先在系统蓝牙里点「断开连接」，再在本应用扫描连接。
      </template>
    </p>
  </section>
</template>

<style scoped>
.panel {
  background: #fff;
  border: 1px solid #ececf3;
  border-radius: 16px;
  padding: 20px;
  box-shadow: 0 8px 24px rgba(24, 28, 50, 0.04);
}

.panel-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

h2 {
  margin: 0;
  font-size: 18px;
}

.sub {
  margin: 6px 0 0;
  color: #6b7280;
  font-size: 13px;
}

.tag {
  margin-left: 8px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 12px;
}

.tag.mock {
  background: #eef2ff;
  color: #4f46e5;
}

.tag.ble {
  background: #ecfdf5;
  color: #047857;
}

.tag.platform {
  background: #f3f4f6;
  color: #374151;
}

.platform-hint {
  margin: 6px 0 0;
  color: #b45309;
  font-size: 12px;
  line-height: 1.5;
}

.actions {
  display: flex;
  gap: 8px;
}

button {
  border: none;
  border-radius: 10px;
  padding: 8px 14px;
  font-size: 13px;
  cursor: pointer;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ghost {
  background: #f3f4f6;
  color: #111827;
}

.primary {
  background: #111827;
  color: #fff;
}

.danger {
  background: #fee2e2;
  color: #b91c1c;
}

.device-list {
  list-style: none;
  margin: 16px 0 0;
  padding: 0;
  display: grid;
  gap: 10px;
}

.device-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px;
  border-radius: 12px;
  background: #f9fafb;
}

.meta {
  display: block;
  color: #9ca3af;
  font-size: 12px;
  margin-top: 2px;
}

.hint {
  margin: 12px 0 0;
  color: #6b7280;
  font-size: 13px;
}

.hint.tip {
  color: #9ca3af;
  font-size: 12px;
  line-height: 1.5;
}
</style>
