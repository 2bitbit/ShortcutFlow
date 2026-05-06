<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import './style.css';
import FlowList from './components/FlowList.vue';
import FlowEditor from './components/FlowEditor.vue';
import ComponentLibrary from './components/ComponentLibrary.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import type { Flow } from './types';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { getRunInBackground, runInBackgroundCache, getLightweightAuto, getLightweightDelay, lightweightAutoCache, lightweightDelayCache } from './store';

const currentView = ref<'list' | 'editor' | 'components' | 'settings'>('list');
const isSidebarOpen = ref(false);
const editingFlow = ref<Flow | null>(null);

import { computed } from 'vue';
const activeComponent = computed(() => {
  switch (currentView.value) {
    case 'list': return FlowList;
    case 'editor': return FlowEditor;
    case 'components': return ComponentLibrary;
    case 'settings': return SettingsPanel;
    default: return FlowList;
  }
});

const currentProps = computed(() => {
  if (currentView.value === 'editor') return { flowData: editingFlow.value };
  return {};
});

const currentEvents = computed(() => {
  if (currentView.value === 'list') return { edit: openEditor };
  if (currentView.value === 'editor') return { back: goBack };
  return {};
});

let unlistenClose: (() => void) | null = null;

onMounted(async () => {
  // 必须先在主页面挂载时完成一次初始化拉取
  await getRunInBackground();
  await getLightweightAuto();
  await getLightweightDelay();

  let lightweightTimer: ReturnType<typeof setTimeout> | null = null;

  unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();

    if (runInBackgroundCache.value !== true) {
      // 不允许后台运行 → 直接退出
      await invoke('exit_app');
      return;
    }

    // 先隐藏窗口
    await getCurrentWindow().hide();

    // 自动轻量模式：延迟后关闭 WebView 释放内存
    if (lightweightAutoCache.value) {
      if (lightweightTimer) clearTimeout(lightweightTimer);
      lightweightTimer = setTimeout(async () => {
        try {
          await invoke('enter_lightweight_mode');
        } catch (e) {
          console.error('自动轻量模式失败:', e);
        }
      }, lightweightDelayCache.value * 1000);
    }
  });
});

onUnmounted(() => {
  if (unlistenClose) {
    unlistenClose();
  }
});

function openEditor(flow?: Flow) {
  if (flow) {
    editingFlow.value = JSON.parse(JSON.stringify(flow));
  } else {
    editingFlow.value = {
      id: crypto.randomUUID(),
      display_name: "新连接流",
      description: "",
      group: "",
      shortcut: "",
      startup_delay_ms: 0,
      nodes: []
    };
  }
  currentView.value = 'editor';
}

function goBack() {
  currentView.value = 'list';
  editingFlow.value = null;
}
</script>

<template>
  <div class="app-wrapper">
    
    <div class="sidebar-toggle" @click="isSidebarOpen = true" v-if="!isSidebarOpen">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="3" y1="12" x2="21" y2="12"></line><line x1="3" y1="6" x2="21" y2="6"></line><line x1="3" y1="18" x2="21" y2="18"></line>
      </svg>
    </div>

    <!-- Backdrop for mobile/drawer feeling -->
    <div class="sidebar-backdrop" v-if="isSidebarOpen" @click="isSidebarOpen = false"></div>

    <aside class="sidebar glass-panel" :class="{ 'is-open': isSidebarOpen }">
      <div class="sidebar-top">
        <div class="brand">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"></path>
          </svg>
          <div class="title">ShortcutFlow</div>
        </div>
        <button class="nav-item btn ghost close-sidebar" @click="isSidebarOpen = false" style="padding:4px; max-width:32px;">
           <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>
      
      <nav class="nav-menu">
        <button class="nav-item" :class="{ active: currentView === 'list' || currentView === 'editor' }" 
                @click="currentView = 'list'; isSidebarOpen = false">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
          工作流管理
        </button>
        <button class="nav-item" :class="{ active: currentView === 'components' }" 
                @click="currentView = 'components'; isSidebarOpen = false">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
          高级组件库
        </button>
        <button class="nav-item" :class="{ active: currentView === 'settings' }" 
                @click="currentView = 'settings'; isSidebarOpen = false">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
          应用设置
        </button>
      </nav>
    </aside>
    
    <main class="app-main" :class="{ 'with-sidebar': isSidebarOpen }">
      <div class="page-container">
        <transition name="fade" mode="out-in">
          <KeepAlive include="FlowList,ComponentLibrary,SettingsPanel" :max="10">
            <component 
              :is="activeComponent" 
              v-bind="currentProps" 
              v-on="currentEvents" 
              :key="currentView === 'editor' ? 'editor-' + editingFlow?.id : currentView"
            />
          </KeepAlive>
        </transition>
      </div>
    </main>
  </div>
</template>

<style scoped>
.app-wrapper {
  display: flex;
  height: 100vh;
  padding: clamp(12px, 2vw, 32px) clamp(16px, 3vw, 48px);
  gap: clamp(8px, 1.5vw, 24px);
  background-image: 
    radial-gradient(circle at 15% 50%, rgba(99, 102, 241, 0.1) 0%, transparent 50%),
    radial-gradient(circle at 85% 30%, rgba(192, 132, 252, 0.1) 0%, transparent 50%);
}

.sidebar-toggle {
  position: absolute;
  top: 20px;
  left: 20px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 50;
  transition: all 0.2s ease;
  color: var(--text-muted);
  border-radius: 8px;
}
.sidebar-toggle:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-main);
}
.sidebar-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.4);
  z-index: 90;
}
.sidebar {
  position: fixed;
  left: -280px;
  top: 16px;
  bottom: 16px;
  width: 250px;
  display: flex;
  flex-direction: column;
  padding: 24px 16px;
  z-index: 100;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow: 4px 0 24px rgba(0,0,0,0.2);
}
.sidebar.is-open {
  transform: translateX(296px);
}
.sidebar-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 40px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  color: #a5b4fc;
}

.title {
  font-size: 22px;
  font-weight: 700;
  background: linear-gradient(to right, #818cf8, #c084fc);
  -webkit-background-clip: text;
  color: transparent;
  letter-spacing: -0.5px;
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 12px 16px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-main);
}

.nav-item.active {
  background: rgba(99, 102, 241, 0.15);
  color: #a5b4fc;
  font-weight: 600;
}

.app-main {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.page-container {
  width: 100%;
  margin: 0 auto;
  height: 100%;
  padding: clamp(16px, 2vw, 40px) clamp(8px, 1.5vw, 24px) 0;
}
</style>