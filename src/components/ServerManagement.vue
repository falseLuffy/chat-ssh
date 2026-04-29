<script setup lang="ts">
import { ref } from 'vue';
import { LayoutDashboard, Box, Settings, Activity, Sparkles } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import { useServerStore } from '../stores/server';
import { useSettingsStore } from '../stores/settings';
import DashboardOverview from './DashboardOverview.vue';
import DockerView from './DockerView.vue';
import ServicesView from './ServicesView.vue';
import AiDiagnosisModal from './AiDiagnosisModal.vue';

const serverStore = useServerStore();
const settingsStore = useSettingsStore();

const activeSubTab = ref('overview');
const isDiagnosisOpen = ref(false);
const isGenerating = ref(false);
const diagnosisResult = ref('');
const diagnosisError = ref('');

const runDiagnosis = async () => {
  if (!serverStore.activeServer) return;
  
  isDiagnosisOpen.value = true;
  isGenerating.value = true;
  diagnosisError.value = '';
  diagnosisResult.value = '';

  try {
    // 1. Collect Context
    const [sysInfo, dockerInfo, servicesInfo] = await Promise.all([
      invoke<any>('get_server_sys_info', { serverName: serverStore.activeServer.name }),
      invoke<any[]>('get_docker_containers', { serverName: serverStore.activeServer.name }),
      invoke<any[]>('get_system_services', { serverName: serverStore.activeServer.name })
    ]);

    const context = `
      [系统状态]
      主机名: ${sysInfo.hostname}
      运行时间: ${sysInfo.uptime}
      CPU负载: ${sysInfo.cpu.usage}
      内存: 已用 ${Math.round(sysInfo.memory.used/1024/1024)}MB / 总计 ${Math.round(sysInfo.memory.total/1024/1024)}MB
      磁盘: ${sysInfo.disks.map((d: any) => `${d.mount} ${d.percent}%`).join(', ')}

      [Docker容器]
      ${dockerInfo.map(c => `- ${c.Names}: ${c.State} (${c.Status})`).join('\n')}

      [关键服务状态]
      ${servicesInfo.filter(s => s.active === 'failed' || s.name.includes('nginx') || s.name.includes('mysql') || s.name.includes('docker')).map(s => `- ${s.name}: ${s.active}`).join('\n')}
    `;

    // 2. Call AI
    const result = await invoke<string>('diagnose_server_issue', {
      serverName: serverStore.activeServer.name,
      context,
      apiKey: settingsStore.deepseekApiKey || ''
    });

    diagnosisResult.value = result;
  } catch (e) {
    console.error('Diagnosis failed:', e);
    diagnosisError.value = String(e);
  } finally {
    isGenerating.value = false;
  }
};

const subTabs = [
  { id: 'overview', name: '概览', icon: LayoutDashboard },
  { id: 'docker', name: 'Docker', icon: Box },
  { id: 'services', name: '服务', icon: Settings },
];
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-[#0f172a]/20">
    <!-- Sub-navigation Header -->
    <div class="flex items-center px-6 py-3 border-b border-slate-800/50 bg-[#1e293b]/30 backdrop-blur-md">
      <div class="flex items-center space-x-1 bg-slate-900/50 p-1 rounded-xl border border-slate-800/50">
        <button 
          v-for="tab in subTabs" 
          :key="tab.id"
          @click="activeSubTab = tab.id"
          :class="[
            'flex items-center space-x-2 px-4 py-1.5 rounded-lg text-xs font-medium transition-all duration-200',
            activeSubTab === tab.id 
              ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/20' 
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          ]"
        >
          <component :is="tab.icon" :size="14" />
          <span>{{ tab.name }}</span>
        </button>
      </div>
      
      <div class="ml-auto flex items-center space-x-4">
        <button 
          @click="runDiagnosis"
          class="flex items-center space-x-2 px-4 py-1.5 bg-blue-600/10 hover:bg-blue-600/20 text-blue-400 border border-blue-500/30 rounded-xl text-xs font-bold transition-all group"
        >
          <Sparkles :size="14" class="group-hover:animate-spin-slow" />
          <span>智能诊断</span>
        </button>
        <div class="flex items-center space-x-2 text-[10px] uppercase tracking-widest font-bold text-slate-500">
          <Activity :size="12" class="text-emerald-500 animate-pulse" />
          <span>实时监控中</span>
        </div>
      </div>
    </div>

    <!-- Sub-tab Content -->
    <div class="flex-1 min-h-0 overflow-hidden flex flex-col">
      <transition 
        name="fade" 
        mode="out-in"
      >
        <DashboardOverview v-if="activeSubTab === 'overview'" />
        <DockerView v-else-if="activeSubTab === 'docker'" />
        <ServicesView v-else-if="activeSubTab === 'services'" />
      </transition>
    </div>

    <!-- AI Diagnosis Modal -->
    <AiDiagnosisModal 
      :is-open="isDiagnosisOpen"
      :is-generating="isGenerating"
      :result="diagnosisResult"
      :error="diagnosisError"
      @close="isDiagnosisOpen = false"
    />
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
