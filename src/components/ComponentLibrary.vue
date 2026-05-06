<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import FlowNode from './FlowNode.vue';
import { ask } from '@tauri-apps/plugin-dialog';
import { useScrollCache } from '../composables/useScrollCache';

const scrollRef = ref<HTMLElement | null>(null);
useScrollCache(scrollRef);
import { getCollapsedGroups, saveCollapsedGroups } from '../store';

import type { Component } from '../types';

const allComponents = ref<Component[]>([]);

const groupedComponents = computed(() => {
  const sorted = [...allComponents.value].sort((a, b) => {
    const gA = a.group || '未分组';
    const gB = b.group || '未分组';

    // Sort logic for group weights
    const weight = (g: string) => {
      if (g.includes('内置')) return -1;
      if (g === '未分组') return 1;
      return 0;
    };

    const wDiff = weight(gA) - weight(gB);
    if (wDiff !== 0) return wDiff;

    if (gA !== gB) return gA.localeCompare(gB, 'zh-Hans-CN');

    return a.name.localeCompare(b.name, 'zh-Hans-CN');
  });

  const groups: Record<string, Component[]> = {};
  for (const c of sorted) {
    const g = c.group || '未分组';
    if (!groups[g]) groups[g] = [];
    groups[g].push(c);
  }
  return groups;
});

const showModal = ref(false);
const collapsedGroups = ref<Set<string>>(new Set());

const editingComp = ref<Component>({
  name: '',
  description: '',
  default_config: {},
  requires_input: false,
  produces_output: true,
  is_builtin: false,
  group: ''
});

const defaultConfigStr = ref('{}');
const uiSchemaStr = ref('');

async function loadComponents() {
  allComponents.value = await invoke('get_all_components');
  const saved = await getCollapsedGroups('component_groups');
  collapsedGroups.value = new Set(saved);
}

async function toggleGroup(groupName: string) {
  if (collapsedGroups.value.has(groupName)) {
    collapsedGroups.value.delete(groupName);
  } else {
    collapsedGroups.value.add(groupName);
  }
  await saveCollapsedGroups('library_groups', Array.from(collapsedGroups.value));
}

onMounted(loadComponents);

function openCreateModal() {
  editingComp.value = {
    name: '新建Shell组件',
    description: '',
    default_config: { command: '' },
    requires_input: true,
    produces_output: true,
    is_builtin: false,
    group: '',
    instructions: '',
    ui_schema: null
  };
  defaultConfigStr.value = '{\n  "command": ""\n}';
  uiSchemaStr.value = '';
  showModal.value = true;
}

function reorderSchemaKeys(schema: any): any {
  if (!schema || typeof schema !== 'object') return schema;
  const ordered: any = {};
  for (const [field, mapping] of Object.entries(schema)) {
    if (mapping && typeof mapping === 'object' && !Array.isArray(mapping)) {
      const m = mapping as Record<string, any>;
      const reordered: Record<string, any> = {};
      // label 优先，desc 其次，其余保持原序
      if ('label' in m) reordered.label = m.label;
      if ('desc' in m) reordered.desc = m.desc;
      for (const [k, v] of Object.entries(m)) {
        if (k !== 'label' && k !== 'desc') reordered[k] = v;
      }
      ordered[field] = reordered;
    } else {
      ordered[field] = mapping;
    }
  }
  return ordered;
}

const previewConfig = computed(() => {
  try {
    return JSON.parse(defaultConfigStr.value || '{}');
  } catch (e) {
    return null;
  }
});

const previewSchema = computed(() => {
  try {
    if (!uiSchemaStr.value.trim()) return {};
    return reorderSchemaKeys(JSON.parse(uiSchemaStr.value));
  } catch (e) {
    return null;
  }
});

function editComp(c: Component) {
  editingComp.value = JSON.parse(JSON.stringify(c));
  defaultConfigStr.value = c.default_config ? JSON.stringify(c.default_config, null, 2) : '{}';
  uiSchemaStr.value = c.ui_schema ? JSON.stringify(reorderSchemaKeys(c.ui_schema), null, 2) : '';
  showModal.value = true;
}

async function saveComp() {
  try {
    editingComp.value.default_config = JSON.parse(defaultConfigStr.value || '{}');
  } catch (e) {
    alert("默认配置 JSON 格式错误！仔细检查是否遗漏了逗号或双引号。");
    return;
  }

  if (uiSchemaStr.value.trim()) {
    try {
      editingComp.value.ui_schema = JSON.parse(uiSchemaStr.value);
    } catch (e) {
      alert("UI Schema JSON 格式错误！");
      return;
    }
  } else {
    editingComp.value.ui_schema = null;
  }

  await invoke('save_custom_component', { comp: editingComp.value });
  showModal.value = false;
  loadComponents();
}

async function confirmDeleteComp(name: string) {
  const confirmed = await ask(`确定要删除组件 "${name}" 吗？此操作不可恢复。`, { title: '确认删除', kind: 'warning' });
  if (confirmed) {
    await invoke('delete_custom_component', { compName: name });
    loadComponents();
  }
}
</script>

<template>
  <div class="library-container">
    <div class="header">
      <h2>组件库资源</h2>
      <div style="display: flex; gap: 12px; align-items: center;">
        <button class="btn ghost" @click="loadComponents" title="从本地存储刷新列表">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"></polyline>
            <polyline points="1 20 1 14 7 14"></polyline>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
          </svg>
          刷新
        </button>
        <button class="btn" @click="openCreateModal">+ 添加自定义组件</button>
      </div>
    </div>

    <div class="components-scroll" ref="scrollRef">
      <section v-for="(comps, groupName) in groupedComponents" :key="groupName" class="section-block">
        <h3 class="section-title" @click="toggleGroup(String(groupName))">
          <span>{{ groupName }}</span>
          <svg :class="{ 'is-collapsed': collapsedGroups.has(String(groupName)) }" width="16" height="16"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
            stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </h3>
        <div class="grid" v-show="!collapsedGroups.has(String(groupName))">
          <div v-for="comp in comps" :key="comp.name" class="card glass-panel"
            :class="{ 'built-in': comp.is_builtin, 'custom': !comp.is_builtin }"
            @click="!comp.is_builtin && editComp(comp)">
            <h4>{{ comp.name }}</h4>
            <p class="desc" style="white-space: pre-wrap;" :title="comp.description">{{ comp.description || '暂无描述' }}
            </p>
            <div class="meta io-meta ht-meta" style="margin-top: 12px; margin-bottom: 0;">
              <span :class="comp.requires_input ? 'tag-deep-blue' : 'tag-light-blue'">数据流入: {{ comp.requires_input
                ?
                '必须' : '可选或忽略' }}</span>
              <span :class="comp.produces_output ? 'tag-deep-blue' : 'tag-light-blue'">数据流出: {{ comp.produces_output ?
                '有产出' : '无产出' }}</span>
            </div>
            <button v-if="!comp.is_builtin" class="btn danger small del-btn"
              @click.stop="confirmDeleteComp(comp.name)">删除</button>
          </div>
        </div>
      </section>
    </div>

    <div v-if="showModal" class="modal-overlay">
      <div class="modal glass-panel" style="display: flex; flex-direction: column; overflow: hidden; height: 95vh;">
        <h3 style="flex-shrink: 0; margin-bottom: 0px;">定制你的自定义组件</h3>

        <div
          style="display: flex; flex-direction: row; align-items: stretch; gap: 4px; flex: 1; overflow: hidden; width: 100%;">
          <!-- Left side: Preview -->
          <div class="preview-side"
            style="flex: 1.55; border-right: 1px dashed rgba(255,255,255,0.1); padding-right: 24px; display: flex; flex-direction: column; overflow-y: auto;">
            <label style="color: #a5b4fc; margin-bottom: 16px; font-weight: 600; font-size: 14px;">流编辑页面渲染预览 (实时)
              (以实际为准)</label>

            <div v-if="previewConfig !== null && previewSchema !== null">
              <!-- 为了让预览节点看起来完全一致，包裹在左边距负值抵消父容器的地方 -->
              <div style="margin-left: -6px; margin-top: -20px; pointer-events: none;">
                <FlowNode :comp="{ ...editingComp, config: previewConfig, component_name: 'preview_placeholder' }"
                  :idx="0"
                  :availableTypes="{ 'preview_placeholder': { label: `[扩展] ${editingComp.name || '未命名组件'}`, desc: editingComp.description, reqIn: editingComp.requires_input, popOut: editingComp.produces_output, ui_schema: previewSchema } }"
                  :groupedTypes="{}" :isOpen="false" :hasWarning="false" :isPreview="true" :isFirst="true"
                  :isLast="true" />
              </div>
            </div>

            <div v-else class="form-group mt-3"
              style="color: #f87171; font-size: 12px; padding: 16px; background: rgba(239,68,68,0.1); border-radius: 8px; border: 1px solid rgba(239,68,68,0.2);">
              ⚠️ 配置或 Schema 的 JSON 格式破损，无法渲染预览，<br />请检查右侧右下角的代码！
            </div>
          </div>

          <!-- Right side: Edit Form -->
          <div class="form-side"
            style="flex: 1.5; overflow-y: auto; padding-right: 12px; display: flex; flex-direction: column;">

            <!-- 组件名称 / 所属分组 / 数据连通性 —— 同一行 -->
            <div class="top-row">
              <div class="form-group top-row-item" style="flex: 1;">
                <label>组件名称</label>
                <input class="input-base" v-model="editingComp.name" placeholder="例如：翻译助手" />
              </div>
              <div class="form-group top-row-item" style="flex: 1;">
                <label>所属分组</label>
                <input class="input-base" v-model="editingComp.group" placeholder="留空则为未分组" />
              </div>
              <div class="form-group top-row-item connectivity-box">
                <label>数据连通性约定</label>
                <div class="connectivity-checks">
                  <label class="check-label">
                    <input type="checkbox" v-model="editingComp.requires_input" />
                    需要前置节点流入数据
                  </label>
                  <label class="check-label">
                    <input type="checkbox" v-model="editingComp.produces_output" />
                    会向后流出新数据
                  </label>
                </div>
              </div>
            </div>

            <div class="form-group">
              <label>描述</label>
              <textarea class="input-base" v-model="editingComp.description" placeholder="它能做什么？" rows="3"></textarea>
            </div>
            <div class="form-group">
              <label>节点专属高亮指南 (支持 HTML，效果同官方黄点说明)</label>
              <textarea class="input-base" v-model="editingComp.instructions"
                placeholder="<strong>💡 执行指南：</strong><br/>这是一个自定义组件说明..." rows="3"></textarea>
            </div>



            <div class="form-group highlight-config mt-3">
              <label>默认配置变量 (必须为严格 JSON 格式)</label>
              <textarea class="input-base" v-model="defaultConfigStr"
                placeholder='{"command": "python script.py", "retries": 1}' rows="4"
                style="font-family: monospace;"></textarea>
              <span style="font-size: 11px; color: var(--text-muted); margin-top: 4px;">
                在此处键入的所有配置变量，将在工作流中自适应渲染为独立的输入框供自定义修改。<br>
                运行节点时，引擎会将这些配置写入 <code>DataEnvelope</code> 的 <code>metadata</code> 字段。
              </span>
            </div>

            <div class="form-group highlight-config mt-3">
              <label>自定义字段界面映射 - UI Schema (JSON格式，可选)</label>
              <textarea class="input-base" v-model="uiSchemaStr"
                placeholder='{"command": {"label": "执行命令", "desc": "终端调用的核心命令配置"}}' rows="4"
                style="font-family: monospace;"></textarea>
              <span style="font-size: 11px; color: var(--text-muted); margin-top: 4px;">通过配置 Schema 可以将英文变量名赋予直观的中文
                Label 和 说明小字。</span>
            </div>

          </div> <!-- End of .form-side -->
        </div> <!-- End of 2 column flex container -->

        <div class="modal-actions mt-4"
          style="flex-shrink: 0; padding-top: 16px; border-top: 1px solid rgba(255,255,255,0.05); margin-top: 24px;">
          <button class="btn ghost" @click="showModal = false">取消</button>
          <button class="btn" @click="saveComp">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.library-container {
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

.header h2 {
  font-size: 22px;
  font-weight: 600;
}

.components-scroll {
  flex: 1;
  overflow-y: auto;
  padding-right: 12px;
}

.section-block {
  margin-bottom: 32px;
}

.section-title {
  margin-bottom: 16px;
  font-size: 16px;
  color: var(--text-muted);
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, 220px);
  gap: clamp(10px, 1.2vw, 18px);
}

.card {
  padding: clamp(6px, 0.7vw, 12px) clamp(8px, 1vw, 16px);
  border-radius: 14px;
  position: relative;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  display: flex;
  flex-direction: column;
}

.card.built-in {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.card.custom {
  cursor: pointer;
}

.card.custom:hover {
  transform: translateY(-4px) scale(1.02);
  border-color: var(--accent);
  box-shadow: 0 15px 25px -5px rgba(0, 0, 0, 0.3), 0 0 15px var(--accent-glow);
}

.card h4 {
  font-size: clamp(13px, 1vw, 16px);
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--text-main);
}

.card p {
  font-size: clamp(10px, 0.75vw, 12px);
  min-height: 0;
  color: var(--text-muted);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 5;
  line-clamp: 5;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1;
}

.ht-meta {
  display: flex;
  gap: 8px;
  font-size: 11px;
}

.ht-meta span {
  padding: 4px 8px;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.2);
  font-weight: 500;
}

.ht-meta span.tag-light-blue {
  color: #a5b4fc;
}

.ht-meta span.tag-deep-blue {
  color: #6366f1;
  font-weight: 600;
}

.badge {
  display: block;
  margin-top: 12px;
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  font-size: 11px;
  width: 75%;
}

.del-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  opacity: 0;
  transition: 0.2s;
  padding: 4px 8px;
  font-size: 12px;
}

.card.custom:hover .del-btn {
  opacity: 1;
}

.mt-3 {
  margin-top: 12px;
}

.mt-4 {
  margin-top: 24px;
}

.mb-2 {
  margin-bottom: 8px;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    backdrop-filter: blur(0px);
  }

  to {
    opacity: 1;
    backdrop-filter: blur(12px);
  }
}

.modal {
  width: 95vw;
  max-width: 1200px;
  padding: 24px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
}

.modal h3 {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 24px;
}

.form-group {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.form-group label {
  font-size: 12px;
  color: var(--text-muted);
}

.highlight-config {
  background: rgba(0, 0, 0, 0.2);
  padding: 0px;
  border-radius: 8px;
  margin: 0px 0px 2vh;
}

.top-row {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.top-row-item {
  margin-bottom: 0 !important;
}

.connectivity-box {
  background: rgba(0, 0, 0, 0.15);
  padding: 0px 4px;
  border-radius: 8px;
}

.connectivity-checks {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.check-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px !important;
  color: var(--text-main) !important;
  cursor: pointer;
  white-space: nowrap;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.col-span-full {
  grid-column: 1 / -1;
}

.io-cb {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-muted);
}
</style>
