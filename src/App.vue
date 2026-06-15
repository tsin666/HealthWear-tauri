<script setup lang="ts">
import { ref } from "vue";
import DevicePanel from "./components/DevicePanel.vue";
import HealthPanel from "./components/HealthPanel.vue";
import WisdomModeList from "./components/WisdomModeList.vue";
import { getConnection } from "./api/ring";
import type { ConnectionInfo } from "./types/ring";

const connection = ref<ConnectionInfo | null>(null);
const modeListRef = ref<InstanceType<typeof WisdomModeList> | null>(null);
const healthPanelRef = ref<InstanceType<typeof HealthPanel> | null>(null);

async function refreshAll() {
  connection.value = await getConnection();
  await modeListRef.value?.loadModes();
  await healthPanelRef.value?.load();
}

refreshAll();
</script>

<template>
  <div class="app">
    <header class="hero">
      <div>
        <p class="eyebrow">HealthWear · Tauri 跨平台版</p>
        <h1>智能戒指控制台</h1>
        <p class="desc">
          跨平台智能戒指控制台：BLE 连接、智慧触控与健康数据同步。
        </p>
      </div>
      <div class="badge">Vue 3 + Rust</div>
    </header>

    <main class="grid">
      <DevicePanel @refresh="refreshAll" />
      <WisdomModeList
        ref="modeListRef"
        :connected="!!connection?.connected"
      />
      <HealthPanel
        ref="healthPanelRef"
        class="full"
        :connected="!!connection?.connected"
        :mock="connection?.mock"
      />
    </main>

    <footer class="footer">
      <span>8 项健康数据 · SQLite 持久化</span>
      <span>详见 docs/ROADMAP.md</span>
    </footer>
  </div>
</template>

<style>
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family:
    "SF Pro Text",
    "PingFang SC",
    Inter,
    system-ui,
    sans-serif;
  background: #f5f6fa;
  color: #111827;
}

#app {
  min-height: 100vh;
}
</style>

<style scoped>
.app {
  max-width: 960px;
  margin: 0 auto;
  padding: 28px 20px 40px;
}

.hero {
  display: flex;
  justify-content: space-between;
  gap: 20px;
  align-items: flex-start;
  margin-bottom: 24px;
}

.eyebrow {
  margin: 0;
  color: #6b7280;
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h1 {
  margin: 8px 0;
  font-size: 28px;
}

.desc {
  margin: 0;
  color: #6b7280;
  line-height: 1.6;
  max-width: 560px;
}

.badge {
  padding: 8px 12px;
  border-radius: 999px;
  background: #111827;
  color: #fff;
  font-size: 12px;
  white-space: nowrap;
}

.grid {
  display: grid;
  gap: 16px;
}

.footer {
  margin-top: 20px;
  display: flex;
  justify-content: space-between;
  gap: 12px;
  color: #9ca3af;
  font-size: 12px;
  flex-wrap: wrap;
}

@media (min-width: 860px) {
  .grid {
    grid-template-columns: 1fr 1.2fr;
    align-items: start;
  }

  .full {
    grid-column: 1 / -1;
  }
}
</style>
