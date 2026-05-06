<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import { invoke } from '@tauri-apps/api/core';
import {
  getRunInBackground, setRunInBackground,
  getLightweightAuto, setLightweightAuto,
  getLightweightDelay, setLightweightDelay
} from '../store';
import LogViewer from './LogViewer.vue';

const autoStartEnabled = ref(false);
const runInBackground = ref(true);
const lightweightAuto = ref(false);
const lightweightDelay = ref(30);
const showLogViewer = ref(false);
const enteringLightweight = ref(false);

// --- 检查更新 ---
const checkingUpdate = ref(false);
const updateStatus = ref<'idle' | 'checking' | 'no-update' | 'has-update' | 'error'>('idle');
const updateMessage = ref('');
const updateUrl = ref('');

async function checkUpdate() {
  checkingUpdate.value = true;
  updateStatus.value = 'checking';
  try {
    const [hasUpdate, latestVersion, url] = await invoke<[boolean, string, string]>('check_for_update');
    if (hasUpdate) {
      updateStatus.value = 'has-update';
      updateMessage.value = `发现新版本 v${latestVersion}！`;
      updateUrl.value = url;
    } else {
      updateStatus.value = 'no-update';
      updateMessage.value = '已是最新版本';
    }
  } catch (e: any) {
    updateStatus.value = 'error';
    updateMessage.value = `检查失败: ${e}`;
  } finally {
    checkingUpdate.value = false;
  }
}

function openUpdatePage() {
  if (updateUrl.value) {
    window.open(updateUrl.value, '_blank');
  }
}

onMounted(async () => {
  try {
    autoStartEnabled.value = await isEnabled();
  } catch (error) {
    console.error("Failed to check autostart status:", error);
  }
  runInBackground.value = await getRunInBackground();
  lightweightAuto.value = await getLightweightAuto();
  lightweightDelay.value = await getLightweightDelay();
});

async function toggleAutoStart() {
  try {
    if (autoStartEnabled.value) {
      await enable();
    } else {
      await disable();
    }
  } catch (error) {
    console.error("Failed to toggle autostart:", error);
    autoStartEnabled.value = !autoStartEnabled.value;
  }
}

async function toggleRunInBackground() {
  await setRunInBackground(runInBackground.value);
}

async function toggleLightweightAuto() {
  await setLightweightAuto(lightweightAuto.value);
}

async function saveLightweightDelay() {
  const val = Math.max(1, Math.min(3600, lightweightDelay.value));
  lightweightDelay.value = val;
  await setLightweightDelay(val);
}

async function enterLightweight() {
  enteringLightweight.value = true;
  try {
    await invoke('enter_lightweight_mode');
  } catch (e) {
    console.error("进入轻量模式失败:", e);
    enteringLightweight.value = false;
  }
}
</script>

<template>
  <div class="settings-panel fade-in">
    <div class="header">
      <div class="title-bar">
        <h2>应用设置</h2>
      </div>
    </div>

    <div class="settings-content">
      <!-- 基础设置 -->
      <div class="card glass-panel" style="padding: 24px; margin-bottom: 24px;">
        <h3 class="section-title" style="margin-top: 0; margin-bottom: 24px;">基础运行设置</h3>

        <div class="setting-item">
          <div class="setting-info">
            <h4>开机自启动</h4>
            <p>建议开启，以在系统启动时自动运行本软件，从而开启快捷键服务。</p>
          </div>
          <div class="setting-control">
            <label class="switch">
              <input type="checkbox" v-model="autoStartEnabled" @change="toggleAutoStart">
              <span class="slider round"></span>
            </label>
          </div>
        </div>

        <div class="setting-item highlight-config" style="border-radius: 8px;">
          <div class="setting-info">
            <h4>允许后台运行</h4>
            <p>开启后，点击主窗口右上角的 ❌ 号不会退出程序，而是最小化至系统托盘。<br>(要彻底退出请在右下角托盘图标处右键选择"退出")</p>
          </div>
          <div class="setting-control">
            <label class="switch">
              <input type="checkbox" v-model="runInBackground" @change="toggleRunInBackground">
              <span class="slider round"></span>
            </label>
          </div>
        </div>
      </div>

      <!-- 轻量模式 -->
      <div class="card glass-panel" style="padding: 24px; margin-bottom: 24px;">
        <h3 class="section-title" style="margin-top: 0; margin-bottom: 24px;">轻量模式</h3>
        <p style="color: var(--text-muted); font-size: 13px; margin-bottom: 20px;">
          关闭主窗口并销毁 WebView，大幅降低内存占用（~100MB+）。后台托盘图标和全局快捷键保持运行。
          点击托盘图标可随时恢复窗口。
        </p>

        <div class="setting-item">
          <div class="setting-info">
            <h4>立即进入轻量模式</h4>
            <p>关闭当前窗口并释放 WebView 内存。</p>
          </div>
          <div class="setting-control">
            <button class="btn btn-primary" style="padding: 8px 16px;"
              :disabled="enteringLightweight"
              @click="enterLightweight">
              {{ enteringLightweight ? '正在进入...' : '立即进入轻量模式' }}
            </button>
          </div>
        </div>

        <div class="setting-item highlight-config" style="border-radius: 8px;">
          <div class="setting-info">
            <h4>自动轻量模式</h4>
            <p>关闭窗口后，等待指定秒数自动进入轻量模式。</p>
          </div>
          <div class="setting-control">
            <label class="switch">
              <input type="checkbox" v-model="lightweightAuto" @change="toggleLightweightAuto">
              <span class="slider round"></span>
            </label>
          </div>
        </div>

        <div class="setting-item" v-if="lightweightAuto">
          <div class="setting-info">
            <h4>延迟时间</h4>
            <p>关闭窗口后等待多少秒进入轻量模式（1-3600秒）。</p>
          </div>
          <div class="setting-control" style="display: flex; align-items: center; gap: 8px;">
            <input class="input-base" type="number" v-model.number="lightweightDelay"
              min="1" max="3600" style="width: 80px;"
              @change="saveLightweightDelay" />
            <span style="color: var(--text-muted); font-size: 13px;">秒</span>
          </div>
        </div>
      </div>

      <!-- 检查更新 -->
      <div class="card glass-panel" style="padding: 24px; margin-bottom: 24px;">
        <h3 class="section-title" style="margin-top: 0; margin-bottom: 24px;">检查更新</h3>
        <p style="color: var(--text-muted); font-size: 13px; margin-bottom: 20px;">
          从 GitHub Releases 获取最新版本信息。有更新时请自行下载压缩包安装。
        </p>

        <div class="setting-item">
          <div class="setting-info">
            <h4>版本检查</h4>
            <p v-if="updateStatus === 'idle'">点击按钮检查是否有新版本可用。</p>
            <p v-else-if="updateStatus === 'checking'" style="color: #fbbf24;">⏳ 正在检查...</p>
            <p v-else-if="updateStatus === 'no-update'" style="color: #34d399;">✅ {{ updateMessage }}</p>
            <p v-else-if="updateStatus === 'has-update'" style="color: #f87171;">🔔 {{ updateMessage }}</p>
            <p v-else style="color: #f87171;">❌ {{ updateMessage }}</p>
          </div>
          <div class="setting-control" style="display: flex; gap: 8px;">
            <button class="btn btn-primary" style="padding: 8px 16px;"
              :disabled="checkingUpdate" @click="checkUpdate">
              {{ checkingUpdate ? '检查中...' : '检查更新' }}
            </button>
            <button v-if="updateStatus === 'has-update'" class="btn" style="padding: 8px 16px; background: rgba(239,68,68,0.15); border-color: rgba(239,68,68,0.4); color: #fca5a5;"
              @click="openUpdatePage">
              前往下载
            </button>
          </div>
        </div>
      </div>

      <!-- 系统诊断 -->
      <div class="card glass-panel" style="padding: 24px; margin-bottom: 24px;">
        <h3 class="section-title" style="margin-top: 0; margin-bottom: 24px;">系统诊断</h3>
        <div class="setting-item">
          <div class="setting-info">
            <h4>运行日志</h4>
            <p>查看后台核心引擎以及全局快捷键触发的流执行报错日志，支持分级过滤与导出功能，方便问题排查与反馈分享。</p>
          </div>
          <div class="setting-control">
            <button class="btn btn-primary" style="padding: 8px 16px;" @click="showLogViewer = true">读取日志</button>
          </div>
        </div>
      </div>
    </div>

    <LogViewer v-if="showLogViewer" @close="showLogViewer = false" />
  </div>
</template>

<style scoped>
.settings-panel {
  padding: 0 0 40px 0;
  height: 100%;
  box-sizing: border-box;
  overflow-y: auto;
}

.header {
  margin-bottom: 24px;
}

.title-bar h2 {
  font-size: clamp(18px, 2.2vw, 28px);
  font-weight: 700;
  background: linear-gradient(to right, #818cf8, #c084fc);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  margin: 0;
  letter-spacing: -0.5px;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: clamp(600px, 80vw, 1000px);
}

.section-title {
  padding-bottom: 4px;
  border-bottom: 2px solid var(--glass-border);

}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
}

.setting-item:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.setting-item:first-of-type {
  padding-top: 0;
}

.setting-info h4 {
  margin: 0 0 8px 0;
  font-size: 16px;
  color: var(--text-main);
  font-weight: 600;
}

.setting-info p {
  margin: 0;
  font-size: 14px;
  color: var(--text-muted);
  max-width: 100%;
  line-height: 1.6;
}

/* Switch Styles (Adapted for Dark Theme) */
.switch {
  position: relative;
  display: inline-block;
  width: 50px;
  height: 28px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.1);
  border: 1px solid var(--glass-border);
  transition: .4s cubic-bezier(0.4, 0.0, 0.2, 1);
}

.slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background-color: var(--text-muted);
  transition: .4s cubic-bezier(0.4, 0.0, 0.2, 1);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

input:checked+.slider {
  background-color: var(--accent);
  border-color: var(--accent);
}

input:checked+.slider:before {
  transform: translateX(22px);
  background-color: #fff;
}

.slider.round {
  border-radius: 34px;
}

.slider.round:before {
  border-radius: 50%;
}
</style>
