<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useServerStore } from '../stores/server';
import { useUIStore } from '../stores/ui';
import { 
  Box, 
  Play, 
  Square, 
  RefreshCw, 
  Trash2, 
  Search, 
  Loader2, 
  ExternalLink,
  ShieldAlert,
  Activity
} from 'lucide-vue-next';

const serverStore = useServerStore();
const ui = useUIStore();
const containers = ref<any[]>([]);
const isLoading = ref(true);
const searchQuery = ref('');
const error = ref('');

const fetchContainers = async () => {
  if (!serverStore.activeServer || serverStore.activeServer.status !== 'online') {
    error.value = '服务器未连接';
    isLoading.value = false;
    return;
  }

  try {
    isLoading.value = true;
    const result = await invoke<any[]>('get_docker_containers', {
      serverName: serverStore.activeServer.name
    });
    containers.value = result;
    error.value = '';
  } catch (e) {
    console.error('Failed to fetch containers:', e);
    error.value = String(e);
  } finally {
    isLoading.value = false;
  }
};

const handleAction = async (id: string, action: string, name: string) => {
  if (!serverStore.activeServer) return;

  try {
    ui.showToast(`正在执行 ${action}: ${name}...`, 'info');
    await invoke('manage_docker_container', {
      serverName: serverStore.activeServer.name,
      containerId: id,
      action: action
    });
    ui.showToast(`${action} 成功: ${name}`, 'success');
    await fetchContainers();
  } catch (e) {
    ui.showToast(`操作失败: ${String(e)}`, 'error');
  }
};

onMounted(() => {
  if (serverStore.activeServer?.status === 'online') {
    fetchContainers();
  }
});

watch(() => serverStore.activeServer?.id, () => {
  fetchContainers();
});

const filteredContainers = ref<any[]>([]);
watch([containers, searchQuery], () => {
  if (!searchQuery.value) {
    filteredContainers.value = containers.value;
  } else {
    const q = searchQuery.value.toLowerCase();
    filteredContainers.value = containers.value.filter(c => 
      (c.names && c.names.toLowerCase().includes(q)) || 
      (c.image && c.image.toLowerCase().includes(q)) ||
      (c.id && c.id.toLowerCase().includes(q))
    );
  }
}, { immediate: true });

const getStatusColor = (state: string) => {
  if (!state) return 'bg-slate-500';
  switch (state.toLowerCase()) {
    case 'running': return 'bg-emerald-500';
    case 'exited': return 'bg-red-500';
    case 'paused': return 'bg-amber-500';
    default: return 'bg-slate-500';
  }
};
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-6 overflow-hidden bg-[#0f172a]/30">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <h2 class="text-xl font-bold text-white flex items-center space-x-2">
          <Box class="text-blue-500" :size="24" />
          <span>Docker 容器管理</span>
        </h2>
        <p class="text-slate-400 text-xs mt-1">可视化管理远程服务器上的容器镜像</p>
      </div>
      
      <div class="flex items-center space-x-3">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
          <input 
            v-model="searchQuery"
            type="text" 
            placeholder="搜索容器名称、镜像..."
            class="bg-slate-800/50 border border-slate-700 rounded-xl pl-10 pr-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 w-64 transition-all"
          />
        </div>
        <button @click="fetchContainers" class="p-2 bg-slate-800 hover:bg-slate-700 rounded-xl transition-colors text-slate-300">
          <RefreshCw :class="{ 'animate-spin': isLoading }" :size="20" />
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div v-if="isLoading && containers.length === 0" class="flex-1 flex flex-col items-center justify-center space-y-4 opacity-50">
      <Loader2 class="animate-spin text-blue-500" :size="48" />
      <span class="text-sm font-medium">正在拉取容器列表...</span>
    </div>

    <div v-else-if="error" class="flex-1 flex flex-col items-center justify-center text-center p-8">
      <div class="p-4 bg-red-500/10 rounded-full mb-4">
        <ShieldAlert class="text-red-500" :size="48" />
      </div>
      <h3 class="text-lg font-bold text-white">无法连接 Docker 服务</h3>
      <p class="text-slate-400 mt-2 max-w-md">请检查服务器是否已安装 Docker 且当前用户有执行权限。详细错误: {{ error }}</p>
      <button @click="fetchContainers" class="mt-6 px-4 py-2 bg-slate-800 hover:bg-slate-700 rounded-lg transition-colors text-sm text-white">
        重试连接
      </button>
    </div>

    <div v-else class="flex-1 overflow-y-auto pr-2 custom-scrollbar">
      <div v-if="filteredContainers.length === 0" class="h-full flex flex-col items-center justify-center text-slate-500">
        <Box :size="48" class="mb-4 opacity-20" />
        <p>未找到符合条件的容器</p>
      </div>
      
      <div v-else class="grid grid-cols-1 xl:grid-cols-2 2xl:grid-cols-3 gap-4">
          <div v-for="container in filteredContainers" 
            :key="container.id"
            class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 hover:border-slate-700 p-5 rounded-2xl transition-all group"
          >
            <!-- Card Header -->
            <div class="flex items-start justify-between mb-4">
              <div class="flex items-center space-x-3 min-w-0">
                <div class="p-2.5 rounded-xl bg-slate-800 group-hover:bg-blue-500/10 transition-colors">
                  <Box :size="20" class="text-slate-400 group-hover:text-blue-400" />
                </div>
                <div class="min-w-0">
                  <h3 class="font-bold text-slate-200 truncate">{{ container.names }}</h3>
                  <p class="text-[10px] font-mono text-slate-500 truncate">{{ container.id ? container.id.substring(0, 12) : '' }}</p>
                </div>
              </div>
              <div class="flex items-center space-x-2">
                <span 
                  :class="['px-2 py-0.5 rounded text-[10px] font-bold text-white uppercase', getStatusColor(container.state)]"
                >
                  {{ container.state }}
                </span>
              </div>
            </div>

          <!-- Info List -->
          <div class="space-y-2 mb-5">
            <div class="flex items-center justify-between text-xs">
              <span class="text-slate-500">镜像:</span>
              <span class="text-slate-300 font-medium truncate ml-4">{{ container.image }}</span>
            </div>
            <div class="flex items-center justify-between text-xs">
              <span class="text-slate-500">状态:</span>
              <span class="text-slate-300">{{ container.status }}</span>
            </div>
            <div class="flex items-center justify-between text-xs">
              <span class="text-slate-500">端口:</span>
              <span class="text-slate-300 truncate ml-4">{{ container.ports || '无' }}</span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center justify-between pt-4 border-t border-slate-800">
            <div class="flex space-x-2">
              <button 
                v-if="container.state !== 'running'"
                @click="handleAction(container.id, 'start', container.names)"
                class="p-2 bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500 hover:text-white rounded-lg transition-all"
                title="启动"
              >
                <Play :size="14" />
              </button>
              <button 
                v-if="container.state === 'running'"
                @click="handleAction(container.id, 'stop', container.names)"
                class="p-2 bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white rounded-lg transition-all"
                title="停止"
              >
                <Square :size="14" />
              </button>
              <button 
                @click="handleAction(container.id, 'restart', container.names)"
                class="p-2 bg-blue-500/10 text-blue-400 hover:bg-blue-500 hover:text-white rounded-lg transition-all"
                title="重启"
              >
                <RefreshCw :size="14" />
              </button>
            </div>
            
            <div class="flex space-x-2">
              <button 
                @click="handleAction(container.id, 'rm', container.names)"
                class="p-2 bg-slate-800 text-slate-500 hover:bg-red-600 hover:text-white rounded-lg transition-all"
                title="删除"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
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
