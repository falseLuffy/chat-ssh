<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useServerStore } from '../stores/server';
import { useUIStore } from '../stores/ui';
import { 
  Activity, 
  Search, 
  RefreshCw, 
  XCircle, 
  Loader2,
  ShieldAlert,
  Cpu,
  Database,
  User as UserIcon,
  Terminal
} from 'lucide-vue-next';

const serverStore = useServerStore();
const ui = useUIStore();
const processes = ref<any[]>([]);
const isLoading = ref(true);
const searchQuery = ref('');
const error = ref('');
let pollTimer: any = null;

const fetchProcesses = async () => {
  if (!serverStore.activeServer || serverStore.activeServer.status !== 'online') {
    error.value = '服务器未连接';
    isLoading.value = false;
    return;
  }

  try {
    const info = await invoke<any>('get_server_sys_info', {
      serverName: serverStore.activeServer.name
    });
    processes.value = info.processes;
    error.value = '';
  } catch (e) {
    console.error('Failed to fetch processes:', e);
    error.value = String(e);
  } finally {
    isLoading.value = false;
  }
};

const handleKill = async (pid: string, name: string) => {
  const confirmed = await ui.showConfirm({
    title: '确认终止进程',
    message: `确定要强制终止进程 ${name} (PID: ${pid}) 吗？`,
    confirmText: '终止',
    type: 'danger'
  });

  if (!confirmed) return;

  try {
    ui.showToast(`正在终止进程 ${pid}...`, 'info');
    await invoke('kill_process', {
      serverName: serverStore.activeServer!.name,
      pid: pid
    });
    ui.showToast(`进程 ${pid} 已终止`, 'success');
    fetchProcesses();
  } catch (e) {
    ui.showToast(`终止失败: ${String(e)}`, 'error');
  }
};

onMounted(() => {
  // fetchProcesses is now handled by the immediate watch
  pollTimer = setInterval(fetchProcesses, 5000);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});

watch([() => serverStore.activeServerId, () => serverStore.activeServer?.status], ([newId, newStatus]) => {
  if (newId && newStatus === 'online') {
    fetchProcesses();
  } else {
    if (pollTimer) clearInterval(pollTimer);
    processes.value = [];
    error.value = '服务器未连接';
    isLoading.value = false;
  }
}, { immediate: true });

const filteredProcesses = ref<any[]>([]);
watch([processes, searchQuery], () => {
  if (!searchQuery.value) {
    filteredProcesses.value = processes.value;
  } else {
    const q = searchQuery.value.toLowerCase();
    filteredProcesses.value = processes.value.filter(p => 
      p.command.toLowerCase().includes(q) || 
      p.pid.toString().includes(q) ||
      p.user.toLowerCase().includes(q)
    );
  }
}, { immediate: true });
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-6 overflow-hidden bg-[#0f172a]/30">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <h2 class="text-xl font-bold text-white flex items-center space-x-2">
          <Activity class="text-blue-500" :size="24" />
          <span>实时进程管理</span>
        </h2>
        <p class="text-slate-400 text-xs mt-1">查看并管理系统高资源占用进程 (每5秒更新)</p>
      </div>
      
      <div class="flex items-center space-x-3">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
          <input 
            v-model="searchQuery"
            type="text" 
            placeholder="搜索进程名、PID、用户..."
            class="bg-slate-800/50 border border-slate-700 rounded-xl pl-10 pr-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 w-64 transition-all"
          />
        </div>
        <button @click="fetchProcesses" class="p-2 bg-slate-800 hover:bg-slate-700 rounded-xl transition-colors text-slate-300">
          <RefreshCw :class="{ 'animate-spin': isLoading }" :size="20" />
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div v-if="isLoading && processes.length === 0" class="flex-1 flex flex-col items-center justify-center space-y-4 opacity-50">
      <Loader2 class="animate-spin text-blue-500" :size="48" />
      <span class="text-sm font-medium">正在获取进程列表...</span>
    </div>

    <div v-else-if="error" class="flex-1 flex flex-col items-center justify-center text-center p-8">
      <div class="p-4 bg-red-500/10 rounded-full mb-4">
        <ShieldAlert class="text-red-500" :size="48" />
      </div>
      <h3 class="text-lg font-bold text-white">无法获取进程数据</h3>
      <p class="text-slate-400 mt-2 max-w-md">{{ error }}</p>
      <button @click="fetchProcesses" class="mt-6 px-4 py-2 bg-slate-800 hover:bg-slate-700 rounded-lg transition-colors text-sm">重试</button>
    </div>

    <div v-else class="flex-1 overflow-y-auto pr-2 custom-scrollbar">
      <div class="bg-[#1e293b]/30 rounded-2xl border border-slate-800/50 overflow-hidden">
        <table class="w-full text-left text-sm border-collapse">
          <thead>
            <tr class="border-b border-slate-800 bg-[#1e293b]/50">
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">PID</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">用户</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">CPU %</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">内存 %</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px]">指令</th>
              <th class="px-6 py-4 font-bold text-slate-500 uppercase tracking-widest text-[10px] text-right">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/50">
            <tr 
              v-for="proc in filteredProcesses" 
              :key="proc.pid"
              class="hover:bg-slate-800/30 transition-colors group"
            >
              <td class="px-6 py-4 font-mono text-blue-400">{{ proc.pid }}</td>
              <td class="px-6 py-4 flex items-center space-x-2">
                <UserIcon :size="12" class="text-slate-500" />
                <span class="text-slate-300">{{ proc.user }}</span>
              </td>
              <td class="px-6 py-4">
                <div class="flex items-center space-x-2">
                  <div class="flex-1 h-1.5 w-16 bg-slate-800 rounded-full overflow-hidden">
                    <div class="h-full bg-blue-500 rounded-full" :style="{ width: Math.min(proc.cpu, 100) + '%' }"></div>
                  </div>
                  <span class="text-xs font-bold" :class="proc.cpu > 50 ? 'text-amber-500' : 'text-slate-400'">{{ proc.cpu }}%</span>
                </div>
              </td>
              <td class="px-6 py-4 text-slate-400">{{ proc.mem }}%</td>
              <td class="px-6 py-4 max-w-xs truncate text-xs text-slate-300 font-mono" :title="proc.command">
                {{ proc.command }}
              </td>
              <td class="px-6 py-4 text-right">
                <button 
                  @click="handleKill(proc.pid, proc.command)"
                  class="p-1.5 bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white rounded-lg transition-all"
                  title="终止进程"
                >
                  <XCircle :size="14" />
                </button>
              </td>
            </tr>
          </tbody>
        </table>
        
        <div v-if="filteredProcesses.length === 0" class="py-20 text-center text-slate-500">
          <p>没有找到相关进程</p>
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
