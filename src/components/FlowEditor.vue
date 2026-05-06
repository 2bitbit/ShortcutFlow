<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, onActivated } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Flow } from '../types';
import { getCollapsedGroups, saveCollapsedGroups } from '../store';
import { useScrollCache } from '../composables/useScrollCache';
import FlowNode from './FlowNode.vue';

const scrollRef = ref<HTMLElement | null>(null);
useScrollCache(scrollRef);

const props = defineProps<{ flowData: Flow }>();
const emit = defineEmits<{ (e: 'back'): void }>();

const flow = ref<Flow>(JSON.parse(JSON.stringify(props.flowData)));

const availableTypes = ref<Record<string, { label: string, desc: string, reqIn: boolean, popOut: boolean, group: string, instructions?: string, ui_schema?: any, default_config?: any, is_builtin?: boolean }>>({});
const collapsedDropdownGroups = ref<Set<string>>(new Set());

const groupedTypes = computed(() => {
  const groups: Record<string, { key: string, label: string }[]> = {};
  for (const [key, obj] of Object.entries(availableTypes.value)) {
    const g = obj.group || '未分组';
    if (!groups[g]) groups[g] = [];
    groups[g].push({ key, label: obj.label });
  }
  return groups;
});

const allCollapsed = computed(() => flow.value.nodes?.every((n: any) => n._collapsed) ?? false);

function toggleAllCollapse() {
  const target = !allCollapsed.value;
  flow.value.nodes?.forEach((n: any) => n._collapsed = target);
}

async function toggleDropdownGroup(gName: string) {
  if (collapsedDropdownGroups.value.has(gName)) {
    collapsedDropdownGroups.value.delete(gName);
  } else {
    collapsedDropdownGroups.value.add(gName);
  }
  await saveCollapsedGroups('flow_editor__component_groups__dropdown', Array.from(collapsedDropdownGroups.value));
}

async function loadAndSyncComponents() {
  try {
    const saved = await getCollapsedGroups('flow_editor__component_groups__dropdown');
    collapsedDropdownGroups.value = new Set(saved);
    const list: any[] = await invoke('get_all_components');
    const newTypes: Record<string, any> = {};
    list.forEach(c => {
      newTypes[c.name] = {
        label: c.is_builtin ? c.name : `[扩展] ${c.name}`,
        desc: c.description || '没有提供说明...',
        reqIn: c.requires_input,
        popOut: c.produces_output,
        group: c.group || '未分组',
        instructions: c.instructions,
        ui_schema: c.ui_schema,
        default_config: c.default_config,
        is_builtin: c.is_builtin
      };
    });
    availableTypes.value = newTypes;

    flow.value.nodes?.forEach(comp => {
      if (!comp.config) {
        comp.config = {};
      }
      if (comp.delay_before_ms === null || comp.delay_before_ms === undefined) {
        comp.delay_before_ms = 0;
      }
    });

  } catch (e) { console.error(e); }
}

const openDropdownIdx = ref<number | null>(null);

// 修改点 1：基于 DOM 路径的智能关闭逻辑
const closeDropdown = (e: MouseEvent) => {
  if (openDropdownIdx.value === null) return;

  // 利用 composedPath 获取完整的事件冒泡路径
  const path = e.composedPath() as HTMLElement[];

  // 检查点击路径中是否包含带有 'ignore-click-outside' 类的元素
  // 这样如果点击在下拉菜单内部，就不会触发关闭
  const isInsideDropdown = path.some(el => el.classList && el.classList.contains('ignore-click-outside'));

  if (!isInsideDropdown) {
    openDropdownIdx.value = null;
  }
};

onMounted(() => {
  loadAndSyncComponents();
  document.addEventListener('click', closeDropdown);
});

onActivated(() => {
  loadAndSyncComponents();
});

onUnmounted(() => {
  document.removeEventListener('click', closeDropdown);
});

function toggleDropdown(idx: number, e: MouseEvent) {
  e.stopPropagation();
  if (openDropdownIdx.value === idx) openDropdownIdx.value = null;
  else openDropdownIdx.value = idx;
}

function selectType(comp: any, key: string) {
  comp.component_name = key;
  comp.config = {};
  openDropdownIdx.value = null;
}

async function save() {
  if (!flow.value.display_name || !flow.value.display_name.trim()) {
    alert("保存失败：工作流的名称不能为空！");
    return;
  }

  for (const comp of flow.value.nodes || []) {
    if (comp.component_name === 'SimulateMouse' && comp.config.sequence) {
      for (const step of comp.config.sequence) {
        if (step.button_type === 'Move' || step.button_type === '移动') {
          if (!/^([+-]?\d+),([+-]?\d+)$/.test(step.coords || '')) {
            alert(`【节点校验失败】模拟鼠标节点中的移动坐标 "${step.coords}" 格式错误，请使用 100,200 或 +10,-20 的格式，否则无法保存。`);
            return;
          }
        }
      }
    }
  }

  try {
    const payloadToSave = JSON.parse(JSON.stringify(flow.value));

    if (!payloadToSave.shortcut || payloadToSave.shortcut.trim() === '') {
      payloadToSave.shortcut = null;
    }

    if (payloadToSave.nodes) {
      payloadToSave.nodes.forEach((n: any) => {
        delete n._capturing;
        delete n._countdown;

        if (n.config) {
          for (const k of Object.keys(n.config)) {
            if (n.config[k] === "") {
              delete n.config[k];
            }
          }
        }
      });
    }

    await invoke('save_flow', { flow: payloadToSave });
    emit('back');
  } catch (err: any) {
    alert("保存失败: " + err);
  }
}


function insertComponent(atIndex: number) {
  if (!flow.value.nodes) flow.value.nodes = [];
  flow.value.nodes.splice(atIndex, 0, {
    id: crypto.randomUUID(),
    component_name: 'Popup',
    delay_before_ms: 0,
    config: {}
  });
}
function addComponent() {
  if (!flow.value.nodes) flow.value.nodes = [];
  flow.value.nodes.push({
    id: crypto.randomUUID(),
    component_name: 'Popup',
    delay_before_ms: 0,
    config: {}
  });
}

function removeComp(idx: number) {
  flow.value.nodes?.splice(idx, 1);
}

function moveUp(idx: number) {
  if (idx > 0 && flow.value.nodes) {
    const temp = flow.value.nodes[idx];
    flow.value.nodes[idx] = flow.value.nodes[idx - 1];
    flow.value.nodes[idx - 1] = temp;
  }
}

function moveDown(idx: number) {
  if (flow.value.nodes && idx < flow.value.nodes.length - 1) {
    const temp = flow.value.nodes[idx];
    flow.value.nodes[idx] = flow.value.nodes[idx + 1];
    flow.value.nodes[idx + 1] = temp;
  }
}

function getEffectivePrevOutput(idx: number): boolean {
  if (idx <= 0) return false;
  for (let i = idx - 1; i >= 0; i--) {
    const prevComp = flow.value.nodes?.[i];
    if (!prevComp || prevComp.pass_through) continue;
    const prevTypeInfo = availableTypes.value[prevComp.component_name];
    return prevTypeInfo?.popOut ?? false;
  }
  return false;
}

function hasWarning(idx: number): boolean {
  if (!flow.value.nodes || idx >= flow.value.nodes.length) return false;
  const comp = flow.value.nodes[idx];
  const typeInfo = availableTypes.value[comp.component_name];
  if (!typeInfo?.reqIn) return false;
  if (idx === 0) return true;

  return !getEffectivePrevOutput(idx);
}

function onShortcutKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    flow.value.shortcut = null;
    return;
  }
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return;

  const modifiers = [];
  if (e.ctrlKey || e.metaKey) modifiers.push('Control');
  if (e.shiftKey) modifiers.push('Shift');
  if (e.altKey) modifiers.push('Alt');

  const keyName = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  flow.value.shortcut = [...modifiers, keyName].join('+');
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
  <div class="flow-editor">
    <div class="editor-header">
      <button class="btn ghost" @click="emit('back')">
        ← 返回
      </button>
      <button class="btn" @click="save">保存流</button>
    </div>

    <div class="editor-content" ref="scrollRef">
      <div class="settings-panel glass-panel">
        <h3 class="section-title">
          <span>全局设置</span>
          <div class="status-indicator">
            <span class="pulse-dot"></span> 编辑中
          </div>
        </h3>
        <div class="form-row">
          <div class="form-group flex-1">
            <label>流名称 (Display Name)</label>
            <input class="input-base" v-model="flow.display_name" placeholder="起个名字..." />
          </div>
          <div class="form-group flex-1">
            <label>分组归属 (Group)</label>
            <input class="input-base" v-model="flow.group" placeholder="例如：内置流、工作流..." />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group flex-1">
            <label>触发快捷键 (Shortcut) - <span style="color:var(--accent);">按 Esc 取消</span></label>
            <input class="input-base shortcut-input" :value="formatShortcut(flow.shortcut)"
              @keydown.prevent="onShortcutKeydown" placeholder="点击后按键盘录制..." />
          </div>
          <div class="form-group flex-1">
            <label>启动延时 (Delay MS)</label>
            <input class="input-base" type="number" v-model="flow.startup_delay_ms" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group flex-1">
            <label>流级执行时的工作目录强制覆盖 (CWD - 必须是绝对路径)</label>
            <input class="input-base" v-model="flow.cwd" placeholder="例如：D:\Workspace" />
          </div>
        </div>
        <div class="form-group" style="margin-bottom: 0;">
          <label>流说明 (Description)</label>
          <textarea class="input-base" v-model="flow.description" placeholder="在这记录下这个流的用途..." rows="2"></textarea>
        </div>
      </div>
      <button v-if="flow.nodes && flow.nodes.length" class="btn ghost small" @click="toggleAllCollapse"
        style="font-size:11px; padding:4px 8px;">
        {{ allCollapsed ? '展开全部' : '折叠全部' }}
      </button>
      <div class="timeline">

        <div class="timeline-line"></div>
        <div v-if="flow.nodes && flow.nodes.length" class="insert-between" style="margin-left:6px;">
          <button class="btn ghost small add-btn" @click="insertComponent(0)" title="在开头插入节点"
            style="font-size:13px; padding:2px 8px; border-radius:12px;">+</button>
        </div>
        <template v-for="(comp, idx) in flow.nodes" :key="comp.id">
          <FlowNode :comp="comp" :idx="idx" :availableTypes="availableTypes" :groupedTypes="groupedTypes"
            :isOpen="openDropdownIdx === idx" :hasWarning="hasWarning(idx)" :isFirst="idx === 0"
            :isLast="idx === (flow.nodes?.length || 0) - 1" :collapsedDropdownGroups="collapsedDropdownGroups"
            @toggle-dropdown="toggleDropdown" @toggle-dropdown-group="toggleDropdownGroup" @select-type="selectType"
            @move-up="moveUp" @move-down="moveDown" @remove="removeComp" />
          <div v-if="idx < (flow.nodes?.length || 0) - 1" class="insert-between" style="margin-left:6px;">
            <button class="btn ghost small add-btn" @click="insertComponent(idx + 1)" title="在此插入节点"
              style="font-size:13px; padding:2px 8px; border-radius:12px;">+</button>
          </div>
        </template>

        <div class="add-node-wrapper" style="display:flex; gap:8px; align-items:center;">
          <button class="btn ghost add-btn" @click="addComponent">
            + 添加组件节点
          </button>

        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.flow-editor {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

/* 修改点 2：增加 padding-bottom 以防止底层溢出时被截断 */
.editor-content {
  flex: 1;
  overflow-y: auto;
  padding-right: 12px;
  padding-bottom: 300px;
  /* 增加视口补偿，确保最后的下拉框能完全展示 */
}

.settings-panel {
  padding: 24px;
  margin-bottom: 40px;
  border-left: 4px solid var(--accent);
}

.section-title {
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-indicator {
  font-size: 12px;
  color: var(--success);
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
  background: rgba(16, 185, 129, 0.1);
  padding: 4px 10px;
  border-radius: 20px;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--success);
  box-shadow: 0 0 8px var(--success);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% {
    transform: scale(0.95);
    opacity: 0.8;
  }

  50% {
    transform: scale(1.1);
    opacity: 1;
  }

  100% {
    transform: scale(0.95);
    opacity: 0.8;
  }
}

.shortcut-input {
  font-family: monospace;
  font-weight: 600;
  color: var(--secondary);
  letter-spacing: 0.5px;
}

.shortcut-input:focus {
  border-color: var(--secondary);
  box-shadow: 0 0 0 3px rgba(192, 132, 252, 0.25), inset 0 2px 4px rgba(0, 0, 0, 0.2);
}

/* 修改点 3：优化 Timeline 层级，确保下拉框不被其他组件遮挡 */
.timeline {
  position: relative;
  padding-left: 28px;
  padding-bottom: 60px;
  z-index: 10;
  /* 提升 timeline 的默认层级 */
}

.timeline-line {
  position: absolute;
  left: 42px;
  top: 16px;
  bottom: 0;
  width: 2px;
  background: linear-gradient(to bottom, var(--accent), var(--secondary), transparent);
  opacity: 0.7;
  box-shadow: 0 0 15px var(--accent-glow);
  z-index: 0;
  border-radius: 2px;
}

.add-node-wrapper {
  position: relative;
  z-index: 2;
  margin-left: 20px;
  margin-top: 16px;
}

.btn.ghost.small.add-btn {
  max-width: 1.5rem;
}

.add-btn {
  background: rgba(99, 102, 241, 0.1);
  border: 3px dashed rgba(99, 102, 241, 0.4);
  color: #a5b4fc;
  padding: 12px 24px;
  width: 100%;
  border-radius: 12px;
  font-size: 14px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  max-width: 12rem;
}

.add-btn:hover {
  background: rgba(99, 102, 241, 0.2);
  border-color: var(--accent);
  color: white;
  transform: translateY(-2px);
  box-shadow: 0 8px 24px var(--accent-glow);
}

.btn.danger-icon:hover {
  color: #f87171;
}

/* --- 新增：修复子组件中下拉框箭头平铺的动画 Bug --- */
:deep(select),
:deep(.input-base select),
:deep(select.input-base) {
  /* 强制阻止背景箭头平铺 */
  background-repeat: no-repeat !important;
  /* 确保箭头锚定在右侧垂直居中 */
  background-position: right 12px center !important;
}

:deep(select:focus),
:deep(select:active),
:deep(.input-base select:focus),
:deep(select.input-base:focus) {
  /* 保证在 focus/active 动画期间也绝不平铺 */
  background-repeat: no-repeat !important;
}
</style>