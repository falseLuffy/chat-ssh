<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import type { Server } from '../stores/server';
import { invoke } from '@tauri-apps/api/core';
import { useUIStore } from '../stores/ui';
import {
  Settings,
  Play,
  Square,
  RefreshCw,
  Search,
  Loader2,
  ShieldAlert,
  Activity,
  CheckCircle2,
  XCircle,
  AlertCircle
} from 'lucide-vue-next';

const props = defineProps<{ server: Server }>();
const ui = useUIStore();
const services = ref<any[]>([]);
const isLoading = ref(true);
const searchQuery = ref('');
const error = ref('');

const fetchServices = async () => {
  if (!props.server || props.server.status !== 'online') {
    error.value = '服务器未连接';
    isLoading.value = false;
    return;
  }

  try {
    isLoading.value = true;
    const result = await invoke<any[]>('get_system_services', {
      serverName: props.server.name
    });
    services.value = result;
    error.value = '';
  } catch (e) {
    console.error('Failed to fetch services:', e);
    error.value = String(e);
  } finally {
    isLoading.value = false;
  }
};

const handleAction = async (name: string, action: string) => {
  if (!props.server) return;

  try {
    ui.showToast(`正在执行 ${action}: ${name}...`, 'info');
    await invoke('manage_system_service', {
      serverName: props.server.name,
      serviceName: name,
      action: action
    });
    ui.showToast(`${action} 成功: ${name}`, 'success');
    await fetchServices();
  } catch (e) {
    ui.showToast(`操作失败: ${String(e)}`, 'error');
  }
};

onMounted(() => {
  if (props.server.status === 'online') {
    fetchServices();
  }
});

watch(() => props.server.status, (newStatus) => {
  if (newStatus === 'online') {
    fetchServices();
  } else {
    error.value = '服务器未连接';
    isLoading.value = false;
  }
});

const filteredServices = ref<any[]>([]);
watch([services, searchQuery], () => {
  if (!searchQuery.value) {
    filteredServices.value = services.value;
  } else {
    const q = searchQuery.value.toLowerCase();
    filteredServices.value = services.value.filter(s =>
      s.name.toLowerCase().includes(q) ||
      s.description.toLowerCase().includes(q)
    );
  }
}, { immediate: true });

const getStatusIcon = (active: string) => {
  switch (active.toLowerCase()) {
    case 'active': return CheckCircle2;
    case 'inactive': return XCircle;
    case 'failed': return AlertCircle;
    default: return Activity;
  }
};

const getStatusColor = (active: string) => {
  switch (active.toLowerCase()) {
    case 'active': return 'text-emerald-500';
    case 'inactive': return 'text-slate-500';
    case 'failed': return 'text-red-500';
    default: return 'text-blue-500';
  }
};
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-6 overflow-hidden bg-[#0f172a]/30">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <h2 class="text-xl font-bold text-white flex items-center space-x-2">
          <Settings class="text-blue-500" :size="24" />
          <span>系统服务管理 (systemd)</span>
        </h2>
        <p class="text-slate-400 text-xs mt-1">管理远程服务器上的系统单元与后台服务</p>
      </div>

      <div class="flex items-center space-x-3">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索服务名称、描述..."
            class="bg-slate-800/50 border border-slate-700 rounded-xl pl-10 pr-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 w-64 transition-all"
          />
        </div>
        <button @click="fetchServices" class="p-2 bg-slate-800 hover:bg-slate-700 rounded-xl transition-colors text-slate-300">
          <RefreshCw :class="{ 'animate-spin': isLoading }" :size="20" />
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div v-if="isLoading && services.length === 0" class="flex-1 flex flex-col items-center justify-center space-y-4 opacity-50">
      <Loader2 class="animate-spin text-blue-500" :size="48" />
      <span class="text-sm font-medium">正在获取服务列表...</span>
    </div>

    <div v-else-if="error" class="flex-1 flex flex-col items-center justify-center text-center p-8">
      <div class="p-4 bg-red-500/10 rounded-full mb-4">
        <ShieldAlert class="text-red-500" :size="48" />
      </div>
      <h3 class="text-lg font-bold text-white">无法获取服务列表</h3>
      <p class="text-slate-400 mt-2 max-w-md">请检查 SSH 连接是否正常，以及是否有权限执行 systemctl 命令。错误: {{ error }}</p>
      <button @click="fetchServices" class="mt-6 px-4 py-2 bg-slate-800 hover:bg-slate-700 rounded-lg transition-colors text-sm text-white">
        重试
      </button>
    </div>

    <div v-else class="flex-1 overflow-y-auto pr-2 custom-scrollbar">
      <div class="bg-[#1e293b]/30 rounded-2xl border border-slate-800/50 overflow-hidden">
        <table class="w-full text-left text-sm border-collapse">
          <thead>
            <tr class="border-b border-slate-800 bg-[#1e293b]/50">
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">服务名称</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">状态</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">描述</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px] text-right">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/50">
            <tr
              v-for="service in filteredServices"
              :key="service.name"
              class="hover:bg-slate-800/30 transition-colors group"
            >
              <td class="px-6 py-4">
                <div class="flex items-center space-x-3">
                  <Settings :size="14" class="text-slate-500" />
                  <span class="font-medium text-slate-200">{{ service.name }}</span>
                </div>
              </td>
              <td class="px-6 py-4">
                <div class="flex items-center space-x-2" :class="getStatusColor(service.active)">
                  <component :is="getStatusIcon(service.active)" :size="14" />
                  <span class="capitalize">{{ service.active }}</span>
                  <span class="text-[10px] opacity-60">({{ service.sub }})</span>
                </div>
              </td>
              <td class="px-6 py-4 text-slate-400 text-xs max-w-md truncate">
                {{ service.description }}
              </td>
              <td class="px-6 py-4 text-right">
                <div class="flex items-center justify-end space-x-2">
                  <button
                    v-if="service.active !== 'active'"
                    @click="handleAction(service.name, 'start')"
                    class="p-1.5 bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500 hover:text-white rounded-lg transition-all"
                    title="启动"
                  >
                    <Play :size="14" />
                  </button>
                  <button
                    v-if="service.active === 'active'"
                    @click="handleAction(service.name, 'stop')"
                    class="p-1.5 bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white rounded-lg transition-all"
                    title="停止"
                  >
                    <Square :size="14" />
                  </button>
                  <button
                    @click="handleAction(service.name, 'restart')"
                    class="p-1.5 bg-blue-500/10 text-blue-400 hover:bg-blue-500 hover:text-white rounded-lg transition-all"
                    title="重启"
                  >
                    <RefreshCw :size="14" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>

        <div v-if="filteredServices.length === 0" class="py-20 text-center text-slate-500">
          <p>没有找到相关服务</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #475569;
}
</style>
