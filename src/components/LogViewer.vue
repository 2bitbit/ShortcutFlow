<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';

const emit = defineEmits(['close']);

const rawLogs = ref('');
const loading = ref(true);
const errorMsg = ref('');
const filterLevel = ref('ALL');
const logContainer = ref<HTMLElement | null>(null);

onMounted(async () => {
  await fetchLogs();
});

async function fetchLogs() {
  loading.value = true;
  errorMsg.value = '';
  try {
    rawLogs.value = await invoke('read_logs');
    await nextTick();
    scrollToBottom();
  } catch (e: any) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

function scrollToBottom() {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
}

interface LogLine {
  raw: string;
  timestamp?: string;
  level?: string;
  target?: string;
  message?: string;
}

const parsedLogs = computed<LogLine[]>(() => {
  if (!rawLogs.value) return [];
  const lines = rawLogs.value.split('\n');
  return lines.map(line => {
    if (!line.trim()) return { raw: line };
    // Try new format: [DATE][TIME][LEVEL][TARGET] msg
    const matchNew = line.match(/^\[(.*?)\]\[(.*?)\]\[(ERROR|WARN|INFO|DEBUG|TRACE)\]\[(.*?)\]\s+(.*)$/);
    if (matchNew) {
      return {
        raw: line,
        timestamp: `${matchNew[1]} ${matchNew[2]}`,
        level: matchNew[3],
        target: matchNew[4],
        message: matchNew[5]
      };
    }
    // Try old format: [DATE][TIME][TARGET][LEVEL] msg
    const matchOld = line.match(/^\[(.*?)\]\[(.*?)\]\[(.*?)\]\[(ERROR|WARN|INFO|DEBUG|TRACE)\]\s+(.*)$/);
    if (matchOld) {
      return {
        raw: line,
        timestamp: `${matchOld[1]} ${matchOld[2]}`,
        target: matchOld[3],
        level: matchOld[4],
        message: matchOld[5]
      };
    }
    return { raw: line, level: 'UNKNOWN' };
  });
});

const filteredLogs = computed(() => {
  if (filterLevel.value === 'ALL') return parsedLogs.value.filter(l => l.raw.trim() !== '');
  return parsedLogs.value.filter(l => l.level === filterLevel.value || (!l.level && l.raw.trim() !== ''));
});

function getLevelClass(level?: string) {
  if (!level || level === 'ALL') return 'text-default';
  switch (level.toUpperCase()) {
    case 'ERROR': return 'text-error';
    case 'WARN': return 'text-warn';
    case 'INFO': return 'text-info';
    case 'DEBUG': return 'text-debug';
    default: return '';
  }
}

async function exportLogs() {
  try {
    const path = await save({
      filters: [{ name: 'Log File', extensions: ['log', 'txt'] }],
      defaultPath: 'shortcutflow_export.log'
    });
    if (path) {
      await invoke('export_logs_to_file', { path, content: rawLogs.value });
      alert('导出成功！\n保存路径：' + path);
    }
  } catch (e) {
    alert('导出失败: ' + e);
  }
}
</script>

<template>
  <div class="log-viewer-modal backdrop-blur">
    <div class="log-window slide-up">
      <div class="window-header">
        <div class="header-left">
          <svg viewBox="0 0 24 24" class="icon" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" />
          </svg>
          <h2>系统诊断日志</h2>
        </div>
        <div class="header-actions">
          <select v-model="filterLevel" class="level-select" :class="getLevelClass(filterLevel)">
            <option value="ALL" class="text-default">全部级别 (ALL)</option>
            <option value="INFO" class="text-info">信息 (INFO)</option>
            <option value="WARN" class="text-warn">警告 (WARN)</option>
            <option value="ERROR" class="text-error">错误 (ERROR)</option>
          </select>
          <button class="btn btn-secondary" @click="fetchLogs">刷新</button>
          <button class="btn btn-primary" @click="exportLogs">导出保存</button>
          <button class="btn-close" @click="emit('close')">✕</button>
        </div>
      </div>

      <div class="log-content-area" ref="logContainer">
        <div v-if="loading" class="state-msg">加载中...</div>
        <div v-else-if="errorMsg" class="state-msg error">{{ errorMsg }}</div>
        <div v-else-if="filteredLogs.length === 0" class="state-msg">暂无日志记录</div>
        <div v-else class="log-lines">
          <div v-for="(log, idx) in filteredLogs" :key="idx" class="log-line">
            <template v-if="log.timestamp">
              <span class="log-time">{{ log.timestamp }}</span>
              <span class="log-level" :class="getLevelClass(log.level)">{{ log.level }}</span>
              <span class="log-target">[{{ log.target }}]</span>
              <span class="log-msg" :class="getLevelClass(log.level)">{{ log.message }}</span>
            </template>
            <template v-else>
              <span class="log-raw">{{ log.raw }}</span>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-viewer-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.backdrop-blur {
  backdrop-filter: blur(8px);
}

.log-window {
  width: 100%;
  max-width: 1000px;
  height: 85vh;
  background: var(--bg-card);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

.window-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--glass-border);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-left .icon {
  width: 20px;
  height: 20px;
  color: var(--accent);
}

.header-left h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-main);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.level-select {
  background: var(--bg-hover);
  color: var(--text-main);
  border: 1px solid var(--glass-border);
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.level-select option {
  background-color: #1e1e24;
  /* 深色背景 */
  color: #e5e7eb;
  /* 浅灰色文字 */
}

.level-select.text-error {
  color: #f87171;
  border-color: rgba(248, 113, 113, 0.3);
}

.level-select.text-warn {
  color: #fbbf24;
  border-color: rgba(251, 191, 36, 0.3);
}

.level-select.text-info {
  color: #60a5fa;
  border-color: rgba(96, 165, 250, 0.3);
}

.text-default {
  color: var(--text-main);
}

.btn {
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.btn-secondary {
  background: var(--bg-hover);
  color: var(--text-main);
  border-color: var(--glass-border);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.1);
}

.btn-primary {
  background: var(--accent);
  color: white;
}

.btn-primary:hover {
  filter: brightness(1.1);
}

.btn-close {
  background: transparent;
  color: var(--text-muted);
  border: none;
  font-size: 20px;
  cursor: pointer;
  padding: 0 8px;
  margin-left: 8px;
}

.btn-close:hover {
  color: var(--text-main);
}

.log-content-area {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  background: #111116;
  /* very dark for console feel */
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.5;
  user-select: text;
}

.state-msg {
  text-align: center;
  color: var(--text-muted);
  margin-top: 40px;
}

.state-msg.error {
  color: #f87171;
}

.log-lines {
  display: flex;
  flex-direction: column;
}

.log-line {
  padding: 2px 0;
  word-wrap: break-word;
  white-space: pre-wrap;
}

.log-line:hover {
  background: rgba(255, 255, 255, 0.02);
}

.log-time {
  color: #6b7280;
  margin-right: 12px;
}

.log-level {
  display: inline-block;
  width: 50px;
  font-weight: bold;
}

.log-target {
  color: #9ca3af;
  margin-right: 12px;
}

.log-raw {
  color: #d1d5db;
}

/* Level Colors */
.text-error {
  color: #f87171;
}

.text-warn {
  color: #fbbf24;
}

.text-info {
  color: #60a5fa;
}

.text-debug {
  color: #9ca3af;
}

.log-msg {
  color: #d1d5db;
}

.log-msg.text-error {
  color: #fca5a5;
  font-weight: 500;
}

.log-msg.text-warn {
  color: #fde68a;
}

.slide-up {
  animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.98);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
