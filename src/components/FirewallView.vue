<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useServerStore } from '../stores/server';
import { useUIStore } from '../stores/ui';
import { 
  ShieldCheck, 
  Search, 
  RefreshCw, 
  Plus, 
  Trash2, 
  Loader2, 
  ShieldAlert,
  Globe,
  Lock,
  Unlock,
  AlertTriangle
} from 'lucide-vue-next';

const serverStore = useServerStore();
const ui = useUIStore();
const rules = ref<any[]>([]);
const isLoading = ref(true);
const searchQuery = ref('');
const error = ref('');

const fetchRules = async () => {
  if (!serverStore.activeServer || serverStore.activeServer.status !== 'online') {
    error.value = '服务器未连接';
    isLoading.value = false;
    return;
  }

  try {
    isLoading.value = true;
    const result = await invoke<any[]>('get_firewall_rules', {
      serverName: serverStore.activeServer.name
    });
    rules.value = result;
    error.value = '';
  } catch (e) {
    console.error('Failed to fetch firewall rules:', e);
    error.value = String(e);
  } finally {
    isLoading.value = false;
  }
};

const handleAddRule = async () => {
  // Simplistic for now, just a prompt
  const port = prompt('请输入要放行的端口或服务名 (例如: 80, 443, mysql):');
  if (!port) return;

  try {
    ui.showToast(`正在放行端口 ${port}...`, 'info');
    await invoke('manage_firewall_rule', {
      serverName: serverStore.activeServer!.name,
      port: port,
      action: 'allow'
    });
    ui.showToast(`端口 ${port} 已放行`, 'success');
    fetchRules();
  } catch (e) {
    ui.showToast(`操作失败: ${String(e)}`, 'error');
  }
};

const handleDeleteRule = async (to: string) => {
  try {
    ui.showToast(`正在删除规则 ${to}...`, 'info');
    await invoke('manage_firewall_rule', {
      serverName: serverStore.activeServer!.name,
      port: to,
      action: 'delete allow'
    });
    ui.showToast(`规则已删除`, 'success');
    fetchRules();
  } catch (e) {
    ui.showToast(`删除失败: ${String(e)}`, 'error');
  }
};

onMounted(() => {
  fetchRules();
});

watch(() => serverStore.activeServer?.id, () => {
  fetchRules();
});

const filteredRules = ref<any[]>([]);
watch([rules, searchQuery], () => {
  if (!searchQuery.value) {
    filteredRules.value = rules.value;
  } else {
    const q = searchQuery.value.toLowerCase();
    filteredRules.value = rules.value.filter(r => 
      r.to.toLowerCase().includes(q) || 
      r.from.toLowerCase().includes(q) ||
      r.action.toLowerCase().includes(q)
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
          <ShieldCheck class="text-blue-500" :size="24" />
          <span>防火墙安全管理 (UFW)</span>
        </h2>
        <p class="text-slate-400 text-xs mt-1">管理远程服务器的网络访问规则与端口开放状态</p>
      </div>
      
      <div class="flex items-center space-x-3">
        <button @click="handleAddRule" class="flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-xl text-sm font-bold transition-all shadow-lg shadow-blue-600/20">
          <Plus :size="16" />
          <span>放行端口</span>
        </button>
        <button @click="fetchRules" class="p-2 bg-slate-800 hover:bg-slate-700 rounded-xl transition-colors text-slate-300">
          <RefreshCw :class="{ 'animate-spin': isLoading }" :size="20" />
        </button>
      </div>
    </div>

    <!-- Warning for UFW / Firewalld -->
    <div class="bg-amber-500/10 border border-amber-500/30 p-4 rounded-2xl flex items-start space-x-3">
      <AlertTriangle class="text-amber-500 mt-0.5" :size="18" />
      <div>
        <p class="text-xs text-amber-200 font-bold">多系统适配提示</p>
        <p class="text-[10px] text-amber-200/70 mt-1">当前支持 Ubuntu/Debian (UFW) 和 CentOS/RHEL (Firewalld)。系统会自动识别并执行对应的防火墙指令，请确保当前用户具有 `sudo` 权限。</p>
      </div>
    </div>

    <!-- Main Content -->
    <div v-if="isLoading && rules.length === 0" class="flex-1 flex flex-col items-center justify-center space-y-4 opacity-50">
      <Loader2 class="animate-spin text-blue-500" :size="48" />
      <span class="text-sm font-medium">正在获取防火墙规则...</span>
    </div>

    <div v-else-if="error" class="flex-1 flex flex-col items-center justify-center text-center p-8">
      <div class="p-4 bg-red-500/10 rounded-full mb-4">
        <ShieldAlert class="text-red-500" :size="48" />
      </div>
      <h3 class="text-lg font-bold text-white">无法读取防火墙状态</h3>
      <p class="text-slate-400 mt-2 max-w-md">可能原因：未安装 UFW、权限不足或服务未启动。错误: {{ error }}</p>
      <button @click="fetchRules" class="mt-6 px-4 py-2 bg-slate-800 hover:bg-slate-700 rounded-lg transition-colors text-sm">重试</button>
    </div>

    <div v-else class="flex-1 overflow-y-auto pr-2 custom-scrollbar">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div 
          v-for="rule in filteredRules" 
          :key="rule.to + rule.from"
          class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-5 rounded-2xl flex items-center justify-between group hover:border-blue-500/30 transition-all"
        >
          <div class="flex items-center space-x-4">
            <div :class="['p-3 rounded-xl', rule.action.includes('ALLOW') ? 'bg-emerald-500/10 text-emerald-500' : 'bg-red-500/10 text-red-500']">
              <Unlock v-if="rule.action.includes('ALLOW')" :size="20" />
              <Lock v-else :size="20" />
            </div>
            <div>
              <p class="text-sm font-bold text-slate-200">{{ rule.to }}</p>
              <div class="flex items-center space-x-2 mt-1">
                <span class="text-[10px] uppercase font-bold tracking-widest text-slate-500">{{ rule.action }}</span>
                <span class="text-[10px] text-slate-600">from</span>
                <span class="text-[10px] font-mono text-blue-500/70">{{ rule.from }}</span>
              </div>
            </div>
          </div>
          
          <button 
            @click="handleDeleteRule(rule.to)"
            class="p-2 opacity-0 group-hover:opacity-100 hover:bg-red-500/20 text-slate-500 hover:text-red-500 rounded-lg transition-all"
            title="删除规则"
          >
            <Trash2 :size="16" />
          </button>
        </div>

        <div v-if="filteredRules.length === 0" class="col-span-full py-16 px-8 text-center flex flex-col items-center">
          <div class="p-6 bg-slate-800/50 rounded-full mb-6">
            <ShieldAlert :size="48" class="text-slate-500 opacity-50" />
          </div>
          <h3 class="text-lg font-bold text-white mb-2">未检测到活跃的防火墙规则</h3>
          <p class="text-sm text-slate-400 max-w-lg leading-relaxed">
            这通常意味着您的服务器尚未安装或启用本地防火墙工具（如 UFW 或 Firewalld）。<br/>
            在这种状态下，服务器系统层面不会拦截任何入站流量。
          </p>
          
          <div class="mt-8 grid grid-cols-1 md:grid-cols-2 gap-4 max-w-2xl w-full">
            <div class="bg-blue-500/5 border border-blue-500/20 p-4 rounded-2xl text-left">
              <p class="text-xs font-bold text-blue-400 flex items-center">
                <Globe :size="14" class="mr-2" /> 云服务商安全组
              </p>
              <p class="text-[10px] text-slate-500 mt-2">
                即使本地没有防火墙，您的云服务商（如阿里云、腾讯云）通常在外部设有一道“安全组”墙。请务必前往云控制台确认相关端口已放行。
              </p>
            </div>
            <div class="bg-emerald-500/5 border border-emerald-500/20 p-4 rounded-2xl text-left">
              <p class="text-xs font-bold text-emerald-400 flex items-center">
                <Lock :size="14" class="mr-2" /> 服务监听状态
              </p>
              <p class="text-[10px] text-slate-500 mt-2">
                如果您的程序无法访问，除了检查防火墙，还需确认程序是否已正确启动并正在监听对应的端口。
              </p>
            </div>
          </div>
          
          <div class="mt-8 flex space-x-4">
            <button @click="fetchRules" class="text-xs text-blue-500 hover:underline">重新扫描</button>
            <span class="text-slate-700">|</span>
            <p class="text-xs text-slate-600">建议通过终端安装 ufw 或 firewalld 以增强安全性</p>
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
</style>
