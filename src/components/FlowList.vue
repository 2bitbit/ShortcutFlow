<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ask } from '@tauri-apps/plugin-dialog';
import type { Flow } from '../types';
import { getCollapsedGroups, saveCollapsedGroups } from '../store';
import { useScrollCache } from '../composables/useScrollCache';

const scrollRef = ref<HTMLElement | null>(null);
useScrollCache(scrollRef);

const emit = defineEmits<{
  (e: 'edit', flow?: Flow): void
}>();

const flows = ref<Flow[]>([]);
const collapsedGroups = ref<Set<string>>(new Set());

async function refresh() {
  try {
    await invoke('refreash_all_flow_shortcut');
  } catch (e) {
    console.warn("刷新快捷键失败", e);
  }
  flows.value = await invoke('get_all_flows');
  const saved = await getCollapsedGroups('flow_groups');
  collapsedGroups.value = new Set(saved);
}

async function toggleGroup(groupName: string) {
  if (collapsedGroups.value.has(groupName)) {
    collapsedGroups.value.delete(groupName);
  } else {
    collapsedGroups.value.add(groupName);
  }
  await saveCollapsedGroups('flow_groups', Array.from(collapsedGroups.value));
}

import { onActivated } from 'vue';
onActivated(refresh);

const groupedFlows = computed(() => {
  const sorted = [...flows.value].sort((a, b) => {
    const gA = (a as any).group || '未分组';
    const gB = (b as any).group || '未分组';

    const weight = (g: string) => {
      if (g.includes('内置')) return -1;
      if (g === '未分组') return 1;
      return 0;
    };

    const wDiff = weight(gA) - weight(gB);
    if (wDiff !== 0) return wDiff;
    if (gA !== gB) return gA.localeCompare(gB, 'zh-Hans-CN');

    return a.display_name.localeCompare(b.display_name, 'zh-Hans-CN');
  });

  const groups: Record<string, Flow[]> = {};
  for (const f of sorted) {
    const g = (f as any).group || '未分组';
    if (!groups[g]) groups[g] = [];
    groups[g].push(f);
  }
  return groups;
});

async function deleteFlow(id: string) {
  await invoke('delete_flow', { flowId: id });
  await refresh();
}

async function confirmDelete(id: string, display_name: string) {
  const confirmed = await ask(`确定要删除工作流 "${display_name}" 吗？`, { title: '确认删除', kind: 'warning' });
  if (confirmed) {
    await deleteFlow(id);
  }
}

async function executeFlow(id: string) {
  try {
    await invoke('run_flow_by_id', { flowId: id, initialData: { metadata: null, payload: null, } });
  } catch (e) {
    console.error("Failed to run flow manually: ", e);
  }
}
function formatShortcut(sc: string | null | undefined): string {
  if (!sc) return '';
  return sc
    .replace(/Key([A-Z])/g, '$1')
    .replace(/Digit(\d)/g, '$1')
    .replace(/Control/gi, 'CTRL')
    .replace(/alt/gi, 'ALT')
    .replace(/shift/gi, 'SHIFT')
    .split('+')
    .map(part => part.trim())
    .join(' + ');
}
</script>

<template>
  <div class="flow-list">
    <div class="header">
      <h2>我的工作流</h2>
      <div style="display: flex; gap: 12px; align-items: center;">
        <button class="btn ghost" @click="refresh" title="从本地存储刷新列表">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"></polyline>
            <polyline points="1 20 1 14 7 14"></polyline>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
          </svg>
          刷新
        </button>
        <button class="btn" @click="emit('edit')">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 5v14M5 12h14" />
          </svg>
          新建连接流
        </button>
      </div>
    </div>

    <div v-if="flows.length === 0" class="empty-state">
      <div class="empty-icon">✨</div>
      <p>还没有任何自动化流。点击右上角新建。</p>
    </div>

    <div class="components-scroll" v-else ref="scrollRef">
      <section v-for="(groupFlows, groupName) in groupedFlows" :key="groupName" class="section-block">
        <h3 class="section-title" @click="toggleGroup(String(groupName))">
          <span>{{ groupName }}</span>
          <svg :class="{ 'is-collapsed': collapsedGroups.has(String(groupName)) }" width="16" height="16"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
            stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </h3>
        <div class="grid" v-show="!collapsedGroups.has(String(groupName))">
          <div v-for="flow in groupFlows" :key="flow.id" class="card glass-panel" @click="emit('edit', flow)">
            <div class="card-row title-row">
              <h3>{{ flow.display_name }}</h3>
              <div style="display:flex; gap: 4px;">
                <button class="btn ghost small run-btn" @click.stop="executeFlow(flow.id)" title="立即执行">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                    stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3" fill="currentColor"></polygon>
                  </svg>
                </button>
                <button class="btn ghost small del-btn" @click.stop="confirmDelete(flow.id, flow.display_name)"
                  title="删除">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                    stroke-linecap="round">
                    <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"></path>
                  </svg>
                </button>
              </div>
            </div>
            <p class="flow-desc" v-if="flow.description">{{ flow.description }}</p>
            <div class="card-row meta-row">
              <div class="shortcut-wrapper">
                <div class="shortcut-badge" v-if="flow.shortcut && flow.shortcut !== ''">⚡ {{
                  formatShortcut(flow.shortcut) }}</div>
              </div>
              <p class="components-desc">共 {{ flow.nodes?.length || 0 }} 个节点</p>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.flow-list {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

h2 {
  font-weight: 600;
  color: var(--text-main);
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 16px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.8;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, 220px);
  gap: 14px;
  padding: 0 4px;
}

.card {
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

.card:hover {
  transform: translateY(-4px) scale(1.02);
  border-color: rgba(99, 102, 241, 0.4);
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.card-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.title-row h3 {
  font-size: 14px;
  margin: 0;
  font-weight: 600;
  color: white;
  line-height: 1.3;
}

.flow-desc {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.35;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.meta-row {
  align-items: center;
}

.shortcut-wrapper {
  flex: 1;
}

.components-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
}

.shortcut-badge {
  display: inline-flex;
  background: rgba(99, 102, 241, 0.2);
  color: #c7d2fe;
  padding: 3px 8px;
  border-radius: 5px;
  font-size: 11px;
  font-weight: 600;
  border: 1px solid rgba(99, 102, 241, 0.3);
}

.del-btn,
.run-btn {
  padding: 6px;
  opacity: 0;
  transition: opacity 0.2s ease, background 0.2s ease;
}

.card:hover .del-btn,
.card:hover .run-btn {
  opacity: 1;
}

.del-btn:hover {
  color: var(--danger);
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
}

.run-btn:hover {
  color: #10b981;
  background: rgba(16, 185, 129, 0.15);
  border-color: rgba(16, 185, 129, 0.3);
}

.components-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.section-block {
  margin-bottom: 28px;
}

.section-title {
  font-size: 14px;
  font-weight: 700;
  color: rgba(165, 180, 252, 0.9);
  text-transform: uppercase;
  letter-spacing: 1.5px;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}
</style>
