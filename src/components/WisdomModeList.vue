<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { WisdomModeItem } from "../types/ring";
import { listWisdomModes, setWisdomMode } from "../api/ring";

const props = defineProps<{ connected: boolean }>();

const modes = ref<WisdomModeItem[]>([]);
const loading = ref(false);
const toast = ref("");

async function loadModes() {
  modes.value = await listWisdomModes();
}

async function toggleMode(mode: WisdomModeItem) {
  if (!props.connected) {
    toast.value = "请先连接戒指";
    return;
  }
  loading.value = true;
  toast.value = "";
  const next = !mode.enabled;
  try {
    await setWisdomMode(mode.protocolIndex, next);
    await loadModes();
    toast.value = next ? `已开启：${mode.title}` : `已关闭：${mode.title}`;
  } catch (e) {
    toast.value = (e as { message: string }).message;
  } finally {
    loading.value = false;
  }
}

onMounted(loadModes);

defineExpose({ loadModes });
</script>

<template>
  <section class="panel">
    <div class="panel-head">
      <div>
        <h2>智能生活</h2>
        <p class="sub">开启智能触控后，可在戒指触控区通过手势控制手机/电脑</p>
      </div>
    </div>

    <ul class="mode-list">
      <li v-for="mode in modes" :key="mode.id" class="mode-item">
        <div class="mode-icon">{{ mode.title.slice(0, 1) }}</div>
        <div class="mode-body">
          <div class="mode-title-row">
            <strong>{{ mode.title }}</strong>
            <label class="switch">
              <input
                type="checkbox"
                :checked="mode.enabled"
                :disabled="loading || !connected"
                @change="toggleMode(mode)"
              />
              <span />
            </label>
          </div>
          <p class="hint">{{ mode.hintPrimary }}</p>
          <p v-if="mode.hintSecondary" class="hint secondary">
            {{ mode.hintSecondary }}
          </p>
        </div>
      </li>
    </ul>

    <p v-if="toast" class="toast">{{ toast }}</p>
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
  line-height: 1.5;
}

.mode-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 12px;
}

.mode-item {
  display: flex;
  gap: 14px;
  padding: 14px;
  border-radius: 14px;
  background: #f9fafb;
}

.mode-icon {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  background: linear-gradient(135deg, #111827, #374151);
  color: #fff;
  display: grid;
  place-items: center;
  font-weight: 700;
  flex-shrink: 0;
}

.mode-body {
  flex: 1;
  min-width: 0;
}

.mode-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.hint {
  margin: 6px 0 0;
  color: #6b7280;
  font-size: 12px;
}

.hint.secondary {
  color: #9ca3af;
}

.switch {
  position: relative;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.switch span {
  position: absolute;
  inset: 0;
  background: #d1d5db;
  border-radius: 999px;
  transition: 0.2s;
}

.switch span::before {
  content: "";
  position: absolute;
  width: 18px;
  height: 18px;
  left: 3px;
  top: 3px;
  background: #fff;
  border-radius: 50%;
  transition: 0.2s;
}

.switch input:checked + span {
  background: #111827;
}

.switch input:checked + span::before {
  transform: translateX(20px);
}

.toast {
  margin: 14px 0 0;
  font-size: 13px;
  color: #4b5563;
}
</style>
