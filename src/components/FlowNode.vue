<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import hljs from 'highlight.js/lib/core';
import xml from 'highlight.js/lib/languages/xml';
import 'highlight.js/styles/atom-one-dark.css';
hljs.registerLanguage('xml', xml);

const props = defineProps({
  comp: {
    type: Object,
    required: true
  },
  idx: {
    type: Number,
    required: true
  },
  availableTypes: {
    type: Object,
    required: true
  },
  groupedTypes: {
    type: Object,
    required: true
  },
  isOpen: {
    type: Boolean,
    default: false
  },
  hasWarning: {
    type: Boolean,
    default: false
  },
  isPreview: {
    type: Boolean,
    default: false
  },
  isFirst: {
    type: Boolean,
    default: false
  },
  isLast: {
    type: Boolean,
    default: false
  },
  collapsedDropdownGroups: {
    type: Set,
    default: () => new Set()
  }
});

const emit = defineEmits([
  'toggle-dropdown',
  'toggle-dropdown-group',
  'select-type',
  'move-up',
  'move-down',
  'remove'
]);

const hoveredCompKey = ref<string | null>(null);


const htmlPreview = ref<{ comp: any; key: string } | null>(null);

function openHtmlPreview(comp: any, key: string) {
  htmlPreview.value = { comp, key };
}

function closeHtmlPreview() {
  htmlPreview.value = null;
}

const highlightedHtml = computed(() => {
  const code = htmlPreview.value?.comp?.config?.[htmlPreview.value?.key] || '';
  if (!code) return '';
  return hljs.highlight(code, { language: 'xml' }).value;
});
// Keep the mouse capture grabber directly inside FlowNode since it just mutates local object properties via Tauri API
async function grabMouseCoords(comp: any) {
  if (props.isPreview) return; // Disable in preview mode
  if (comp._capturing) return;

  comp._capturing = true;
  comp._countdown = 3;

  let timer = setInterval(() => {
    comp._countdown--;
    if (comp._countdown <= 0) {
      clearInterval(timer);
    }
  }, 1000);

  try {
    const point: [number, number] = await invoke('capture_mouse_coords', { delayMs: 3000 });
    const coordStr = `${point[0]},${point[1]}`;
    try {
      await navigator.clipboard.writeText(coordStr);
    } catch (e) { }
    alert(`已获取当前坐标：${coordStr}\n\n已尝试自动将此坐标复制到剪贴板。您现在可以将其粘贴到对应“移动”操作的输入框内！`);
  } catch (e: any) {
    alert("获取坐标失败：" + e);
  } finally {
    clearInterval(timer);
    comp._capturing = false;
  }
}

function getUISchema(compName: string, key: string) {
  if (props.isPreview) return props.availableTypes?.ui_schema?.[key] || {};
  return props.availableTypes[compName]?.ui_schema?.[key] || {};
}

function getUISchemaObject(compName: string) {
  if (props.isPreview) return props.availableTypes?.ui_schema || {};
  return props.availableTypes[compName]?.ui_schema || {};
}

function getDisplayKeys(comp: any) {
  const schemaKeys = Object.keys(getUISchemaObject(comp.component_name));
  const configKeys = Object.keys(comp.config || {});
  return Array.from(new Set([...schemaKeys, ...configKeys]));
}
</script>

<template>
  <div class="component-card glass-panel" :style="{
    zIndex: isOpen ? 50 : 1,
    opacity: comp.pass_through ? 0.6 : 1,
    border: comp.pass_through ? '1px dashed rgba(99, 102, 241, 0.4)' : ''
  }">
    <div class="comp-header">
      <div style="display: flex; flex-direction: column; align-items: center; gap: 4px;">
        <div class="comp-index" :style="{ opacity: comp.pass_through ? 0.6 : 1 }">{{ idx + 1 }}</div>
        <button v-if="!isPreview" class="btn ghost small"
          style="padding: 2px 4px; font-size: 10px; min-width: unset; height: auto;"
          :style="{ background: comp.pass_through ? 'rgba(99, 102, 241, 0.2)' : 'transparent', color: comp.pass_through ? '#a5b4fc' : 'var(--text-muted)' }"
          @click="comp.pass_through = !comp.pass_through" title="开启透传：该节点执行后，源数据将无视此节点的处理结果，直接传递给下一节点。">
          透传
        </button>
      </div>

      <div class="custom-select-wrapper" @click.stop>
        <div class="custom-select-trigger" @click="isPreview ? null : emit('toggle-dropdown', idx, $event)"
          :style="isPreview ? 'pointer-events:none' : ''">
          {{ availableTypes[comp.component_name]?.label || (isPreview ? comp.name : '选择组件...') }}
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </div>

        <div class="custom-options" v-if="isOpen">
          <template v-for="(items, gName) in groupedTypes" :key="gName">
            <div class="dropdown-group-label" @click.stop="emit('toggle-dropdown-group', String(gName))"
              style="cursor:pointer; display:flex; justify-content:space-between; align-items:center;">
              <span>{{ gName }}</span>
              <svg :style="{ transform: collapsedDropdownGroups.has(String(gName)) ? 'rotate(-90deg)' : 'rotate(0)' }"
                width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                style="transition: transform 0.2s" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </div>
            <template v-if="!collapsedDropdownGroups.has(String(gName))">
              <div v-for="item in items" :key="item.key" class="custom-option"
                @click.stop="emit('select-type', comp, item.key)" @mouseenter="hoveredCompKey = item.key"
                @mouseleave="hoveredCompKey = null">
                {{ item.label }}
              </div>
            </template>
          </template>
        </div>

        <div class="hover-tooltip" v-if="isOpen && hoveredCompKey && availableTypes[hoveredCompKey]">
          <div class="ht-title">{{ availableTypes[hoveredCompKey].label }}</div>
          <p class="ht-desc" style="white-space: pre-wrap;">{{ availableTypes[hoveredCompKey].desc }}</p>
          <div class="ht-meta">
            <span :class="availableTypes[hoveredCompKey].reqIn ? 'tag-deep-blue' : 'tag-light-blue'">数据流入: {{
              availableTypes[hoveredCompKey].reqIn ? '必须' : '可选或忽略' }}</span>
            <span :class="availableTypes[hoveredCompKey].popOut ? 'tag-deep-blue' : 'tag-light-blue'">数据流出: {{
              availableTypes[hoveredCompKey].popOut ? '有产出' : '无产出' }}</span>
          </div>
        </div>
      </div>

      <div class="actions" :style="isPreview ? 'opacity: 0.5; pointer-events: none;' : ''">
        <button class="btn ghost small" @click="emit('move-up', idx)" :disabled="isFirst || isPreview"
          title="上移">↑</button>
        <button class="btn ghost small" @click="emit('move-down', idx)" :disabled="isLast || isPreview"
          title="下移">↓</button>
        <button class="btn ghost small danger-icon" @click="emit('remove', idx)" :disabled="isPreview" title="删除组件">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round">
            <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"></path>
          </svg>
        </button>
        <button v-if="!isPreview" class="btn ghost small" @click="comp._collapsed = !comp._collapsed" :title="comp._collapsed ? '展开节点' : '折叠节点'" style="font-size:11px; padding:2px 5px;">{{ comp._collapsed ? '▶' : '▼' }}</button>
        <div class="delay-wrapper">
          <label>启动延时(ms):</label>
          <input class="input-base small-input" type="number" v-model="comp.delay_before_ms" :disabled="isPreview" />
        </div>
      </div>
    </div>

    <!-- Warnings & Information text bodies -->
    <div v-if="!comp._collapsed">
    <div class="comp-warning" v-if="hasWarning && !isPreview">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
        stroke-linecap="round">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
        <line x1="12" y1="9" x2="12" y2="13"></line>
        <line x1="12" y1="17" x2="12.01" y2="17"></line>
      </svg>
      此节点需求数据传入，但目前置位于无数据流出源的死区！
    </div>

    <div class="comp-desc-text" v-if="availableTypes[comp.component_name] || isPreview">
      <div class="desc-content" style="white-space: pre-wrap;">{{ isPreview ? (comp.description || '没有提供说明...') :
        availableTypes[comp.component_name]?.desc }}</div>
      <div class="ht-meta" style="margin-top: 10px;">
        <span
          :class="(isPreview ? comp.requires_input : availableTypes[comp.component_name]?.reqIn) ? 'tag-deep-blue' : 'tag-light-blue'">
          数据流入: {{ (isPreview ? comp.requires_input : availableTypes[comp.component_name]?.reqIn) ? '必须' : '可选或忽略'
          }}
        </span>
        <span v-if="comp.pass_through && !isPreview" class="tag-deep-blue"
          style="background: rgba(99, 102, 241, 0.2); border: 1px dashed #6366f1;">
          数据流出: [已开启透传] 原样放行
        </span>
        <span v-else
          :class="(isPreview ? comp.produces_output : availableTypes[comp.component_name]?.popOut) ? 'tag-deep-blue' : 'tag-light-blue'">
          数据流出: {{ (isPreview ? comp.produces_output : availableTypes[comp.component_name]?.popOut) ? '有产出' : '无产出' }}
        </span>
      </div>
    </div>

    <!-- Configuration Body -->
    <div class="comp-body"
      :style="isPreview ? 'padding: 20px; background: rgba(0, 0, 0, 0.15); border-radius: 12px; border: 1px dashed rgba(255, 255, 255, 0.05); margin-left: 0;' : ''">

      <div class="config-grid">
        <!-- Instructions -->
        <div v-if="(isPreview ? comp.instructions : availableTypes[comp.component_name]?.instructions)"
          class="form-group col-span-full"
          style="background: rgba(255,255,255,0.03); padding: 12px; border-radius: 6px; border-left: 3px solid #6366f1; margin-bottom: 4px;">
          <span style="font-size: 12px; color: var(--text-muted); line-height: 1.6;"
            v-html="(isPreview ? comp.instructions : availableTypes[comp.component_name]?.instructions)"></span>
        </div>

        <!-- Dynamic Form -->
        <template v-for="key in getDisplayKeys(comp)" :key="key">
          <div class="form-group"
            :class="{ 'col-span-full': getUISchema(comp.component_name, String(key)).full_width || (typeof comp.config[key] === 'string' && comp.config[key].length > 50) || ['sequence_key', 'sequence_mouse'].includes(getUISchema(comp.component_name, String(key)).type) }"
            style="margin-bottom: 0;">

            <label style="display: flex; flex-direction: column; gap: 4px;">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span>{{ getUISchema(comp.component_name, String(key)).label || key }}</span>

                <!-- Special add button for sequences in label -->
                <button v-if="getUISchema(comp.component_name, String(key)).type === 'sequence_key'"
                  class="btn ghost small" :disabled="isPreview"
                  @click="comp.config[key] = comp.config[key] || []; comp.config[key].push({ key: '', action: 'Click', tap_count: 1 })">+
                  添加按键步骤</button>
                <button v-if="getUISchema(comp.component_name, String(key)).type === 'sequence_mouse'"
                  class="btn ghost small" :disabled="isPreview"
                  @click="comp.config[key] = comp.config[key] || []; comp.config[key].push({ button_type: 'Left', action: 'Click', tap_count: 1, coords: '' })">+
                  添加鼠标步骤</button>
              </div>

              <span v-if="getUISchema(comp.component_name, String(key)).desc"
                style="font-size: 11px; color: var(--text-muted); font-weight: normal; line-height: 1.4;"
                v-html="getUISchema(comp.component_name, String(key)).desc">
              </span>
            </label>

            <!-- Grab Mouse Coords Button (special injection for sequence_mouse) -->
            <div v-if="getUISchema(comp.component_name, String(key)).type === 'sequence_mouse'"
              style="margin-bottom: 12px; margin-top: 8px;">
              <button class="btn" :disabled="isPreview"
                style="width: 100%; border: 1px dashed var(--accent); background: rgba(99, 102, 241, 0.1);"
                @click="grabMouseCoords(comp)">
                {{ comp._capturing ? `倒计时 ${comp._countdown} 秒后触发系统级鼠标监听...` : '📍 获取屏幕精确坐标 (点击后将有 3 秒瞄准准备时间)' }}
              </button>
            </div>

            <!-- Input Types based on schema 'type' (default fallback to val type inference) -->
            <template v-if="getUISchema(comp.component_name, String(key)).type === 'select'">
              <select class="input-base" :value="comp.config[key] ?? ''"
                @input="comp.config[key] = ($event.target as HTMLSelectElement).value || undefined"
                :disabled="isPreview"
                :style="isPreview ? 'opacity:0.6; cursor:not-allowed;' : ''">
                <option v-if="getUISchema(comp.component_name, String(key)).placeholder" value="" disabled hidden>
                  {{ getUISchema(comp.component_name, String(key)).placeholder }}
                </option>
                <option v-for="opt in getUISchema(comp.component_name, String(key)).options || []" :key="opt.value"
                  :value="opt.value">{{ opt.label }}</option>
              </select>
            </template>
            <template v-else-if="getUISchema(comp.component_name, String(key)).type === 'password'">
              <input class="input-base" type="password" v-model="comp.config[key]"
                :placeholder="getUISchema(comp.component_name, String(key)).placeholder" :disabled="isPreview"
                :style="isPreview ? 'opacity:0.6; cursor:not-allowed;' : ''" />
            </template>
            <template v-else-if="getUISchema(comp.component_name, String(key)).type === 'textarea'">
              <textarea class="input-base" v-model="comp.config[key]"
                :placeholder="getUISchema(comp.component_name, String(key)).placeholder" rows="3" :disabled="isPreview"
                :style="isPreview ? 'opacity:0.6; cursor:not-allowed;' : ''"></textarea>
              <button v-if="!isPreview && comp.component_name === 'HtmlWindow' && String(key) === 'html_content'" class="btn-html-preview" @click.stop="openHtmlPreview(comp, String(key))" title="可视化编辑 HTML">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
                预览编辑
              </button>
            </template>
            <template
              v-else-if="getUISchema(comp.component_name, String(key)).type === 'checkbox' || typeof comp.config[key] === 'boolean'">
              <label class="io-cb" style="margin-top: 8px;"
                :style="isPreview ? 'opacity:0.6; cursor:not-allowed;' : ''">
                <input type="checkbox" v-model="comp.config[key]" :disabled="isPreview" /> 启用 / 开启
              </label>
            </template>
            <template
              v-else-if="getUISchema(comp.component_name, String(key)).type === 'number' || typeof comp.config[key] === 'number'">
              <input class="input-base" type="number" v-model="comp.config[key]"
                :placeholder="getUISchema(comp.component_name, String(key)).placeholder" :disabled="isPreview"
                :style="isPreview ? 'opacity:0.6; cursor:not-allowed;' : ''" />
            </template>

            <!-- Sequence Key -->
            <template v-else-if="getUISchema(comp.component_name, String(key)).type === 'sequence_key'">
              <div v-for="(step, sIdx) in comp.config[key]" :key="sIdx" class="sequence-step">
                <div style="display: flex; gap: 8px; align-items: flex-end;">
                  <div class="form-group" style="flex: 2; margin-bottom: 0;">
                    <label v-if="sIdx === 0">按键/组合键</label>
                    <input class="input-base" v-model="step.key" placeholder="例如: ctrl+alt+f, a, enter"
                      :disabled="isPreview" />
                  </div>
                  <div class="form-group" style="flex: 1; margin-bottom: 0;">
                    <label v-if="sIdx === 0">动作类型</label>
                    <select class="input-base" v-model="step.action" :disabled="isPreview">
                      <option value="Click">点按</option>
                      <option value="Press">按住</option>
                      <option value="Release">释放</option>
                    </select>
                  </div>
                  <div class="form-group relative" style="flex: 1; margin-bottom: 0;"
                    v-if="step.action === 'Click' || step.action === '点按'">
                    <label v-if="sIdx === 0">点击次数</label>
                    <div class="tap-count-wrapper">
                      <input class="input-base" type="number" v-model="step.tap_count" style="padding-right: 20px;"
                        min="1" :disabled="isPreview" />
                      <span class="floating-label">次</span>
                    </div>
                  </div>
                  <button class="btn ghost small danger-icon" @click="comp.config[key].splice(sIdx, 1)"
                    :disabled="isPreview" style="margin-bottom: 2px;">×</button>
                </div>
              </div>
            </template>

            <!-- Sequence Mouse -->
            <template v-else-if="getUISchema(comp.component_name, String(key)).type === 'sequence_mouse'">
              <div v-for="(step, sIdx) in comp.config[key]" :key="sIdx" class="sequence-step">
                <div style="display: flex; gap: 8px; align-items: flex-end;">
                  <div class="form-group" style="flex: 1.5; margin-bottom: 0;">
                    <label v-if="sIdx === 0">目标指令</label>
                    <select class="input-base" v-model="step.button_type" :disabled="isPreview">
                      <option value="Left">左键</option>
                      <option value="Middle">中键</option>
                      <option value="Right">右键</option>
                      <option value="Move">移动</option>
                    </select>
                  </div>

                  <template v-if="step.button_type === 'Move' || step.button_type === '移动'">
                    <div class="form-group" style="flex: 2; margin-bottom: 0;">
                      <label v-if="sIdx === 0">送达坐标界限 (x,y)</label>
                      <input class="input-base" v-model="step.coords" placeholder="绝对(100,200) | 相对(+10,-20)"
                        :disabled="isPreview"
                        :class="{ 'input-error': step.coords && !/^([+-]?\d+),([+-]?\d+)$/.test(step.coords) }" />
                    </div>
                  </template>
                  <template v-else>
                    <div class="form-group" style="flex: 1.2; margin-bottom: 0;">
                      <label v-if="sIdx === 0">动作类型</label>
                      <select class="input-base" v-model="step.action" :disabled="isPreview">
                        <option value="Click">点按</option>
                        <option value="Press">按住</option>
                        <option value="Release">释放</option>
                      </select>
                    </div>
                    <div class="form-group relative" style="flex: 1; margin-bottom: 0;"
                      v-if="step.action === 'Click' || step.action === '点按'">
                      <label v-if="sIdx === 0">次数</label>
                      <div class="tap-count-wrapper">
                        <input class="input-base" type="number" v-model="step.tap_count" style="padding-right: 20px;"
                          min="1" :disabled="isPreview" />
                        <span class="floating-label">次</span>
                      </div>
                    </div>
                  </template>

                  <button class="btn ghost small danger-icon" @click="comp.config[key].splice(sIdx, 1)"
                    :disabled="isPreview" style="margin-bottom: 2px;">×</button>
                </div>
                <div
                  v-if="(step.button_type === 'Move' || step.button_type === '移动') && step.coords && !/^([+-]?\d+),([+-]?\d+)$/.test(step.coords)"
                  style="color: #f87171; font-size: 11px; margin-top: 4px;">
                  警告: 坐标不符合标准格式，本流将阻断保存操作！
                </div>
              </div>
            </template>

            <!-- Fallback to Text/String inference -->
            <template v-else>
              <textarea v-if="typeof comp.config[key] === 'string' && comp.config[key].length > 50" class="input-base"
                v-model="comp.config[key]" :placeholder="getUISchema(comp.component_name, String(key)).placeholder"
                rows="3" :disabled="isPreview" :style="isPreview ? 'opacity:0.6; cursor:not-allowed;' : ''"></textarea>
              <input v-else class="input-base" type="text" v-model="(comp.config as any)[key]"
                :placeholder="getUISchema(comp.component_name, String(key)).placeholder" :disabled="isPreview"
                :style="isPreview ? 'opacity:0.6; cursor:not-allowed;' : ''" />
            </template>
          </div>
        </template>

        <div v-if="isPreview && Object.keys(comp.config || {}).length === 0" class="col-span-full"
          style="color: rgba(255,255,255,0.3); font-size: 12px; text-align: center; padding: 20px 0; background: rgba(0,0,0,0.1); border-radius: 8px; border: 1px dashed rgba(255,255,255,0.05); margin-top: 8px;">
          （空配置，尚未录入任何动态参数）
        </div>
      </div>
    </div>
  </div>
</div>

  <!-- HTML Preview Editor Modal -->
  <Teleport to="body">
    <div v-if="htmlPreview" class="html-editor-overlay" @click.self="closeHtmlPreview">
      <div class="html-editor-modal">
        <div class="html-editor-header">
          <span>HTML 可视化编辑器</span>
          <button class="btn-close" @click="closeHtmlPreview" title="关闭 (Esc)">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div class="html-editor-body">
          <div class="html-editor-preview">
            <div class="panel-label">预览</div>
            <iframe :srcdoc="htmlPreview.comp.config[htmlPreview.key] || ''" sandbox="allow-scripts" class="preview-frame"></iframe>
          </div>
          <div class="html-editor-source">
            <div class="panel-label">源码</div>
            <div class="editor-wrapper">
              <textarea
                v-model="htmlPreview.comp.config[htmlPreview.key]"
                class="editor-textarea"
                spellcheck="false"
                @keydown.escape="closeHtmlPreview"
                @scroll="(e: Event) => { const pre = (e.target as HTMLElement).nextElementSibling as HTMLElement; if (pre) pre.scrollTop = (e.target as HTMLElement).scrollTop; pre.scrollLeft = (e.target as HTMLElement).scrollLeft; }"
              ></textarea>
              <pre class="editor-highlight"><code class="language-xml" v-html="highlightedHtml"></code></pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

</template>
<style scoped>
.component-card {
  position: relative;
  z-index: 1;
  padding: 16px 16px 8px 10px;
  margin: 4px 6px 24px 6px;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  overflow: visible !important;
}

.component-card:hover {
  transform: translateY(-2px) scale(1.01);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.2);
  border-color: rgba(99, 102, 241, 0.5);
}

.comp-index {
  width: 32px;
  height: 32px;
  border-radius: 16px;
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.9), rgba(168, 85, 247, 0.9));
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 800;
  font-size: 15px;
  flex-shrink: 0;
  border: 1px solid rgba(255, 255, 255, 0.2);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
}

.comp-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: nowrap;
}

.type-selector-wrapper {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
}

.info-icon {
  cursor: help;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  margin-right: 4px;
}

.info-icon:hover {
  color: var(--text-main);
}

.actions {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-wrap: nowrap;
  flex-shrink: 0;
}

.delay-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 8px;
  flex-wrap: nowrap;
}

.delay-wrapper label {
  font-size: 13px;
  color: var(--text-muted);
  white-space: nowrap;
}

.small-input {
  width: 70px;
  padding: 6px 8px;
}

.comp-body {
  padding-left: 44px;
}

.comp-warning {
  margin-left: 44px;
  margin-bottom: 12px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  color: #fca5a5;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.comp-desc-text {
  margin-left: 44px;
  margin-bottom: 16px;
  padding: 10px 14px;
  background: rgba(255, 255, 255, 0.03);
  border-left: 3px solid var(--accent);
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 13px;
  line-height: 1.5;
}

.io-checkboxes {
  display: flex;
  gap: 20px;
  margin-bottom: 16px;
}

.io-cb {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-muted);
  cursor: pointer;
}

.io-cb input {
  cursor: pointer;
}

.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.col-span-full {
  grid-column: 1 / -1;
}

textarea.input-base {
  resize: vertical;
}

/* Custom Select styles */
.custom-select-wrapper {
  position: relative;
  width: 100%;
}

.custom-select-trigger {
  width: 100%;
  padding: 8px 12px;
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  color: var(--text-main);
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: all 0.2s ease;
}

.custom-select-trigger:hover {
  background: rgba(255, 255, 255, 0.08);
}

.custom-options {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  width: 100%;
  background: rgba(20, 20, 25, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 12px;
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  z-index: 100;
  max-height: 300px;
  overflow-y: auto;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.7);
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dropdown-group-label {
  padding: 6px 14px 4px;
  font-size: 11px;
  font-weight: 700;
  color: rgba(165, 180, 252, 0.8);
  text-transform: uppercase;
  letter-spacing: 1px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  margin-bottom: 2px;
}

.custom-option {
  padding: 10px 14px;
  cursor: pointer;
  border-radius: 8px;
  transition: all 0.2s ease;
  position: relative;
  font-weight: 500;
  color: #e2e8f0;
}

.custom-option:last-child {
  border-bottom: none;
}

.custom-option:hover {
  background: rgba(99, 102, 241, 0.2);
  color: #fff;
  transform: translateX(4px);
}

.hover-tooltip {
  position: absolute;
  left: 105%;
  top: -10px;
  width: 240px;
  background: rgba(30, 30, 35, 0.85);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  padding: 16px;
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
  z-index: 200;
  cursor: default;
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.ht-title {
  font-weight: 700;
  font-size: 15px;
  margin-bottom: 8px;
  color: #fff;
  letter-spacing: 0.5px;
}

.ht-desc {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
  line-height: 1.5;
  margin-bottom: 12px;
}

.ht-meta {
  display: flex;
  gap: 8px;
  font-size: 11px;
}

.ht-meta span {
  padding: 4px 8px;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.4);
  font-weight: 500;
}

.ht-meta span.tag-light-blue {
  color: #a5b4fc;
}

/* 浅靛蓝 原色 */
.ht-meta span.tag-deep-blue {
  color: #6366f1;
  font-weight: 600;
}

.btn.danger-icon:hover {
  color: #f87171;
}

/* Scrollbar styling for custom options */
.custom-options::-webkit-scrollbar {
  width: 6px;
}

.custom-options::-webkit-scrollbar-track {
  background: transparent;
}

.custom-options::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.custom-options::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

.sequence-step {
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  margin-top: 8px;
  transition: border-color 0.2s;
}

.sequence-step:hover {
  border-color: rgba(99, 102, 241, 0.3);
}

.tap-count-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.floating-label {
  position: absolute;
  right: 8px;
  font-size: 11px;
  color: var(--text-muted);
  pointer-events: none;
}

.input-error {
  border-color: #f87171 !important;
  box-shadow: 0 0 0 1px #f87171 !important;
}

/* HTML Preview Editor */
.html-editor-overlay {
  position: fixed; inset: 0; z-index: 9999;
  background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center;
}
.html-editor-modal {
  background: #1a1a2e; border-radius: 12px; width: 90vw; max-width: 1200px; height: 80vh;
  display: flex; flex-direction: column; overflow: hidden;
  border: 1px solid rgba(255,255,255,0.1);
}
.html-editor-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 12px 16px; border-bottom: 1px solid rgba(255,255,255,0.08);
  color: #e0e0e0; font-weight: bold;
}
.btn-close {
  background: none; border: none; color: #888; cursor: pointer; padding: 4px;
}
.btn-close:hover { color: #e94560; }
.html-editor-body {
  display: flex; flex: 1; overflow: hidden;
}
.html-editor-preview, .html-editor-source {
  flex: 1; display: flex; flex-direction: column; padding: 12px;
}
.html-editor-preview { border-right: 1px solid rgba(255,255,255,0.08); }
.panel-label {
  color: #888; font-size: 12px; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 1px;
}
.preview-frame {
  flex: 1; border: none; border-radius: 8px; background: #111122;
}
.editor-wrapper {
  flex: 1; position: relative; overflow: hidden; border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.08);
}
.editor-wrapper:focus-within {
  border-color: #6366f1;
}
.editor-textarea, .editor-highlight {
  position: absolute; inset: 0; margin: 0; padding: 12px;
  font-family: "Consolas", "Cascadia Code", monospace;
  font-size: 13px; line-height: 1.5; tab-size: 2;
  white-space: pre; overflow: auto;
  border: none; border-radius: 0;
}
.editor-textarea {
  background: transparent; color: transparent; caret-color: #e0e0e0;
  resize: none; outline: none; z-index: 2;
}
.editor-highlight {
  background: #0d0d1a; z-index: 1; pointer-events: none;
}
.editor-highlight code {
  font-family: inherit; font-size: inherit; line-height: inherit;
}
.btn-html-preview {
  display: inline-flex; align-items: center; gap: 4px;
  margin-top: 6px; padding: 4px 10px; font-size: 12px;
  background: rgba(99, 102, 241, 0.15); color: #a5b4fc;
  border: 1px solid rgba(99, 102, 241, 0.3); border-radius: 6px; cursor: pointer;
}
.btn-html-preview:hover {
  background: rgba(99, 102, 241, 0.25);
}
</style>
