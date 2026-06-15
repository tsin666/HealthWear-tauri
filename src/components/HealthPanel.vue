<script setup lang="ts">
import { computed, ref } from "vue";
import {
  exportHealthCsv,
  getHealthSnapshot,
  listHealthModules,
  syncHealthModule,
} from "../api/health";
import type { HealthModuleInfo, HealthSnapshot } from "../types/health";

defineProps<{
  connected: boolean;
  mock?: boolean;
}>();

const modules = ref<HealthModuleInfo[]>([]);
const snapshot = ref<HealthSnapshot>({
  heartRate: [],
  bloodOxygen: [],
  sport: [],
  bloodPressure: [],
  sleep: [],
  healthAll: [],
  bodyTemp: [],
  tempHumidity: [],
});
const busy = ref(false);
const message = ref("");

const statusLabel: Record<string, string> = {
  ready: "可同步",
  planned: "待开发",
  jni_required: "需 JNI",
};

async function load() {
  modules.value = await listHealthModules();
  snapshot.value = await getHealthSnapshot();
}

async function handleSync(module: HealthModuleInfo, useMock: boolean) {
  busy.value = true;
  message.value = "";
  try {
    snapshot.value = await syncHealthModule(module.id, useMock);
    message.value = `${module.title}同步完成（已写入本地数据库）`;
    await load();
  } catch (e) {
    message.value = (e as { message: string }).message;
  } finally {
    busy.value = false;
  }
}

async function handleExport(module: HealthModuleInfo) {
  busy.value = true;
  message.value = "";
  try {
    const path = await exportHealthCsv(module.id);
    message.value = `${module.title}已导出 CSV：${path}`;
  } catch (e) {
    message.value = (e as { message: string }).message;
  } finally {
    busy.value = false;
  }
}

const latestHeart = computed(() => {
  const list = snapshot.value.heartRate;
  if (!list.length) return null;
  return [...list].sort((a, b) => b.timestampMs - a.timestampMs)[0];
});

function formatTime(ms: number) {
  return new Date(ms).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatDuration(secs: number) {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}小时${m}分`;
  return `${m}分钟`;
}

function sleepTypeLabel(type: number) {
  switch (type) {
    case 241:
      return "深睡";
    case 242:
      return "浅睡";
    case 243:
      return "REM";
    case 244:
      return "清醒";
    default:
      return `阶段 ${type}`;
  }
}

function formatTemp(value: number) {
  return `${value.toFixed(1)}°C`;
}

load();
defineExpose({ load });
</script>

<template>
  <section class="panel">
    <div class="panel-head">
      <div>
        <h2>健康数据</h2>
        <p class="sub">8 项健康数据 · SQLite 本地持久化 · 支持 CSV 导出</p>
      </div>
      <div v-if="latestHeart" class="highlight">
        <span class="bpm">{{ latestHeart.bpm }}</span>
        <span class="unit">bpm</span>
      </div>
    </div>

    <ul class="module-list">
      <li v-for="module in modules" :key="module.id">
        <div class="module-main">
          <strong>{{ module.title }}</strong>
          <span class="desc">{{ module.description }}</span>
          <span class="status" :data-status="module.status">
            {{ statusLabel[module.status] }}
            <template v-if="module.recordCount">
              · {{ module.recordCount }} 条
            </template>
          </span>
        </div>
        <div class="actions">
          <button
            class="primary"
            :disabled="busy || module.status !== 'ready' || !connected"
            @click="handleSync(module, !!mock)"
          >
            同步
          </button>
          <button
            class="ghost"
            :disabled="
              busy || module.status !== 'ready' || !module.recordCount
            "
            @click="handleExport(module)"
          >
            导出
          </button>
        </div>
      </li>
    </ul>

    <div v-if="snapshot.healthAll.length" class="records">
      <h3>综合指标</h3>
      <ul>
        <li v-for="(item, idx) in snapshot.healthAll.slice(0, 5)" :key="idx">
          <span>{{ formatTime(item.timestampMs) }}</span>
          <strong>
            {{ item.steps }} 步 · HRV {{ item.hrv }} ·
            {{ formatTemp(item.temperature) }}
          </strong>
          <span class="meta">
            {{ item.heartRate }} bpm · SpO2 {{ item.spo2 }}%
          </span>
        </li>
      </ul>
    </div>

    <div v-if="snapshot.bodyTemp.length" class="records">
      <h3>体温</h3>
      <ul>
        <li v-for="(item, idx) in snapshot.bodyTemp.slice(0, 5)" :key="idx">
          <span>{{ formatTime(item.timestampMs) }}</span>
          <strong>{{ formatTemp(item.temperature) }}</strong>
        </li>
      </ul>
    </div>

    <div v-if="snapshot.tempHumidity.length" class="records">
      <h3>温湿度</h3>
      <ul>
        <li v-for="(item, idx) in snapshot.tempHumidity.slice(0, 5)" :key="idx">
          <span>{{ formatTime(item.timestampMs) }}</span>
          <strong>{{ formatTemp(item.temperature) }}</strong>
          <span class="meta">湿度 {{ item.humidity.toFixed(0) }}%</span>
        </li>
      </ul>
    </div>

    <div v-if="snapshot.sleep.length" class="records">
      <h3>睡眠</h3>
      <ul>
        <li v-for="(item, idx) in snapshot.sleep.slice(0, 3)" :key="idx">
          <div class="sleep-row">
            <span>{{ formatTime(item.startMs) }} – {{ formatTime(item.endMs) }}</span>
            <strong>
              深 {{ formatDuration(item.deepSleepTotalSecs) }} · 浅
              {{ formatDuration(item.lightSleepTotalSecs) }}
              <template v-if="item.remTotalSecs">
                · REM {{ formatDuration(item.remTotalSecs) }}
              </template>
            </strong>
            <span v-if="item.wakeCount" class="meta">
              清醒 {{ item.wakeCount }} 次
            </span>
          </div>
          <ul v-if="item.segments.length" class="segment-list">
            <li v-for="(seg, sidx) in item.segments.slice(0, 6)" :key="sidx">
              {{ sleepTypeLabel(seg.sleepType) }}
              {{ formatDuration(seg.durationSecs) }}
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <div v-if="snapshot.bloodOxygen.length" class="records">
      <h3>血氧</h3>
      <ul>
        <li v-for="(item, idx) in snapshot.bloodOxygen.slice(0, 5)" :key="idx">
          <span>{{ formatTime(item.timestampMs) }}</span>
          <strong>{{ item.spo2 }}%</strong>
        </li>
      </ul>
    </div>

    <div v-if="snapshot.sport.length" class="records">
      <h3>运动</h3>
      <ul>
        <li v-for="(item, idx) in snapshot.sport.slice(0, 5)" :key="idx">
          <span>{{ formatTime(item.startMs) }}</span>
          <strong>{{ item.steps }} 步</strong>
          <span class="meta">{{ item.calories }} kcal</span>
        </li>
      </ul>
    </div>

    <div v-if="snapshot.bloodPressure.length" class="records">
      <h3>血压</h3>
      <ul>
        <li
          v-for="(item, idx) in snapshot.bloodPressure.slice(0, 5)"
          :key="idx"
        >
          <span>{{ formatTime(item.timestampMs) }}</span>
          <strong>{{ item.sbp }}/{{ item.dbp }} mmHg</strong>
        </li>
      </ul>
    </div>

    <div v-if="snapshot.heartRate.length" class="records">
      <h3>心率</h3>
      <ul>
        <li v-for="(item, idx) in snapshot.heartRate.slice(0, 5)" :key="idx">
          <span>{{ formatTime(item.timestampMs) }}</span>
          <strong>{{ item.bpm }} bpm</strong>
        </li>
      </ul>
    </div>

    <p v-if="message" class="hint">{{ message }}</p>
    <p v-else-if="!connected" class="hint">连接戒指后可同步真实健康数据</p>
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
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 16px;
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

.highlight {
  text-align: right;
}

.bpm {
  font-size: 32px;
  font-weight: 700;
  color: #dc2626;
}

.unit {
  display: block;
  color: #9ca3af;
  font-size: 12px;
}

.module-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 10px;
}

.module-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 12px;
  background: #f9fafb;
}

.actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.module-main {
  display: grid;
  gap: 2px;
}

.desc {
  color: #9ca3af;
  font-size: 12px;
}

.status {
  font-size: 12px;
  color: #6b7280;
}

.status[data-status="ready"] {
  color: #047857;
}

.status[data-status="jni_required"] {
  color: #b45309;
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

.primary {
  background: #111827;
  color: #fff;
}

.ghost {
  background: #fff;
  color: #374151;
  border: 1px solid #e5e7eb;
}

.records {
  margin-top: 16px;
}

.records h3 {
  margin: 0 0 8px;
  font-size: 14px;
}

.records ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 6px;
}

.records li {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  font-size: 13px;
  color: #6b7280;
}

.sleep-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.records > ul > li:has(.segment-list) {
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.segment-list {
  list-style: none;
  margin: 0;
  padding: 0 0 0 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.segment-list li {
  font-size: 12px;
  color: #9ca3af;
  background: #f3f4f6;
  padding: 2px 8px;
  border-radius: 999px;
}

.records strong {
  color: #111827;
}

.meta {
  margin-left: auto;
  font-size: 12px;
}

.hint {
  margin: 12px 0 0;
  color: #6b7280;
  font-size: 13px;
}
</style>
