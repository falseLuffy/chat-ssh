<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useServerStore } from '../stores/server';
import { Loader2, RefreshCw, Cpu, HardDrive, MemoryStick, Clock, Monitor } from 'lucide-vue-next';

// ECharts imports
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { PieChart, LineChart, GaugeChart } from 'echarts/charts';
import { 
  TitleComponent, 
  TooltipComponent, 
  LegendComponent, 
  GridComponent,
  VisualMapComponent
} from 'echarts/components';
import VChart, { THEME_KEY } from 'vue-echarts';

use([
  CanvasRenderer, 
  PieChart, 
  LineChart, 
  GaugeChart,
  TitleComponent, 
  TooltipComponent, 
  LegendComponent, 
  GridComponent,
  VisualMapComponent
]);

const serverStore = useServerStore();
const isLoading = ref(true);
const sysInfo = ref<any>(null);
const error = ref('');
let pollTimer: any = null;

const history = ref<{ cpu: number[], time: string[] }>({
  cpu: [],
  time: []
});

const fetchInfo = async () => {
  if (!serverStore.activeServer || serverStore.activeServer.status !== 'online') {
    error.value = '服务器未连接';
    isLoading.value = false;
    return;
  }

  try {
    const info = await invoke<any>('get_server_sys_info', {
      serverName: serverStore.activeServer.name
    });
    sysInfo.value = info;
    error.value = '';
    
    // Update history for line chart
    const now = new Date().toLocaleTimeString();
    history.value.time.push(now);
    history.value.cpu.push(info.cpu.usage * 100);
    if (history.value.time.length > 20) {
      history.value.time.shift();
      history.value.cpu.shift();
    }
  } catch (e) {
    console.error('Failed to fetch sys info:', e);
    error.value = String(e);
  } finally {
    isLoading.value = false;
  }
};

const startPolling = () => {
  stopPolling();
  fetchInfo();
  pollTimer = setInterval(fetchInfo, 5000);
};

const stopPolling = () => {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
};

onMounted(() => {
  if (serverStore.activeServer?.status === 'online') {
    startPolling();
  }
});

onUnmounted(() => {
  stopPolling();
});

watch(() => serverStore.activeServer?.id, () => {
  isLoading.value = true;
  history.value = { cpu: [], time: [] };
  startPolling();
});

// Chart Options
const cpuOption = ref({
  tooltip: { formatter: '{a} <br/>{b} : {c}%' },
  series: [{
    name: 'CPU',
    type: 'gauge',
    progress: { show: true },
    detail: { valueAnimation: true, formatter: '{value}%', color: '#60a5fa', fontSize: 16 },
    data: [{ value: 0, name: '负载' }],
    axisLabel: { fontSize: 10 },
    splitLine: { length: 10 },
    pointer: { width: 3 }
  }]
});

const memOption = ref({
  tooltip: { trigger: 'item' },
  legend: { bottom: '5%', left: 'center', textStyle: { color: '#94a3b8', fontSize: 10 } },
  series: [{
    name: '内存使用',
    type: 'pie',
    radius: ['40%', '70%'],
    avoidLabelOverlap: false,
    itemStyle: { borderRadius: 10, borderColor: '#1e293b', borderWidth: 2 },
    label: { show: false },
    data: [
      { value: 0, name: '已用', itemStyle: { color: '#60a5fa' } },
      { value: 0, name: '空闲', itemStyle: { color: '#334155' } }
    ]
  }]
});

const diskOption = ref({
  tooltip: { trigger: 'item' },
  legend: { bottom: '5%', left: 'center', textStyle: { color: '#94a3b8', fontSize: 10 } },
  series: [{
    name: '磁盘空间',
    type: 'pie',
    radius: ['40%', '70%'],
    itemStyle: { borderRadius: 10, borderColor: '#1e293b', borderWidth: 2 },
    label: { show: false },
    data: [
      { value: 0, name: '已用', itemStyle: { color: '#f59e0b' } },
      { value: 0, name: '可用', itemStyle: { color: '#334155' } }
    ]
  }]
});

const historyOption = ref({
  grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
  xAxis: { type: 'category', boundaryGap: false, data: [], axisLabel: { color: '#64748b' } },
  yAxis: { type: 'value', max: 100, axisLabel: { color: '#64748b' } },
  series: [{
    data: [],
    type: 'line',
    smooth: true,
    areaStyle: { color: 'rgba(96, 165, 250, 0.2)' },
    itemStyle: { color: '#60a5fa' }
  }]
});

watch(sysInfo, (newVal) => {
  if (!newVal) return;
  
  cpuOption.value.series[0].data[0].value = Math.round(newVal.cpu.usage * 100);
  
  memOption.value.series[0].data = [
    { value: newVal.memory.used, name: '已用', itemStyle: { color: '#60a5fa' } },
    { value: newVal.memory.free, name: '空闲', itemStyle: { color: '#334155' } }
  ];

  if (newVal.disks.length > 0) {
    diskOption.value.series[0].data = [
      { value: newVal.disks[0].used, name: '已用', itemStyle: { color: '#f59e0b' } },
      { value: newVal.disks[0].total - newVal.disks[0].used, name: '可用', itemStyle: { color: '#334155' } }
    ];
  }

  historyOption.value.xAxis.data = history.value.time as any;
  historyOption.value.series[0].data = history.value.cpu as any;
}, { deep: true });

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-6 overflow-y-auto bg-[#0f172a]/30">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-bold text-white flex items-center space-x-2">
          <Monitor class="text-blue-500" :size="24" />
          <span>服务器概览</span>
        </h2>
        <p class="text-slate-400 text-xs mt-1">实时监控系统核心资源占用情况</p>
      </div>
      <button @click="fetchInfo" class="p-2 hover:bg-slate-800 rounded-lg transition-colors text-slate-400 hover:text-white">
        <RefreshCw :class="{ 'animate-spin': isLoading }" :size="20" />
      </button>
    </div>

    <div v-if="isLoading && !sysInfo" class="flex-1 flex flex-col items-center justify-center space-y-4 opacity-50">
      <Loader2 class="animate-spin text-blue-500" :size="48" />
      <span class="text-sm font-medium">正在获取系统状态...</span>
    </div>

    <div v-else-if="error" class="flex-1 flex flex-col items-center justify-center text-center p-8">
      <div class="p-4 bg-red-500/10 rounded-full mb-4">
        <Cpu class="text-red-500" :size="48" />
      </div>
      <h3 class="text-lg font-bold text-white">无法获取数据</h3>
      <p class="text-slate-400 mt-2 max-w-md">{{ error }}</p>
      <button @click="fetchInfo" class="mt-6 px-4 py-2 bg-slate-800 hover:bg-slate-700 rounded-lg transition-colors text-sm">
        重试连接
      </button>
    </div>

    <template v-else-if="sysInfo">
      <!-- Quick Stats -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-4 rounded-2xl flex items-center space-x-4">
          <div class="p-3 bg-blue-500/10 rounded-xl text-blue-500">
            <Monitor :size="24" />
          </div>
          <div>
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">主机名</p>
            <p class="text-sm font-bold text-slate-200 truncate">{{ sysInfo.hostname }}</p>
          </div>
        </div>
        
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-4 rounded-2xl flex items-center space-x-4">
          <div class="p-3 bg-emerald-500/10 rounded-xl text-emerald-500">
            <Clock :size="24" />
          </div>
          <div>
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">运行时间</p>
            <p class="text-sm font-bold text-slate-200">{{ sysInfo.uptime }}</p>
          </div>
        </div>

        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-4 rounded-2xl flex items-center space-x-4">
          <div class="p-3 bg-amber-500/10 rounded-xl text-amber-500">
            <MemoryStick :size="24" />
          </div>
          <div>
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">总内存</p>
            <p class="text-sm font-bold text-slate-200">{{ formatBytes(sysInfo.memory.total) }}</p>
          </div>
        </div>

        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-4 rounded-2xl flex items-center space-x-4">
          <div class="p-3 bg-purple-500/10 rounded-xl text-purple-500">
            <HardDrive :size="24" />
          </div>
          <div>
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">系统磁盘</p>
            <p class="text-sm font-bold text-slate-200">{{ sysInfo.disks[0]?.mount || '/' }} ({{ sysInfo.disks[0]?.percent }}%)</p>
          </div>
        </div>
      </div>

      <!-- Main Charts -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- CPU Load Gauge -->
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-6 rounded-3xl flex flex-col h-80">
          <h3 class="text-sm font-bold text-slate-400 mb-4 flex items-center">
            <Cpu class="mr-2" :size="16" /> CPU 实时负载
          </h3>
          <div class="flex-1 min-h-0">
            <VChart :option="cpuOption" autoresize />
          </div>
        </div>

        <!-- Memory Pie -->
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-6 rounded-3xl flex flex-col h-80">
          <h3 class="text-sm font-bold text-slate-400 mb-4 flex items-center">
            <MemoryStick class="mr-2" :size="16" /> 内存占用详情
          </h3>
          <div class="flex-1 min-h-0">
            <VChart :option="memOption" autoresize />
          </div>
        </div>

        <!-- Disk Pie -->
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-6 rounded-3xl flex flex-col h-80">
          <h3 class="text-sm font-bold text-slate-400 mb-4 flex items-center">
            <HardDrive class="mr-2" :size="16" /> 磁盘空间分布
          </h3>
          <div class="flex-1 min-h-0">
            <VChart :option="diskOption" autoresize />
          </div>
        </div>
      </div>

      <!-- History Line Chart -->
      <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-6 rounded-3xl flex flex-col h-80">
        <h3 class="text-sm font-bold text-slate-400 mb-4">负载趋势 (最近 20 次采样)</h3>
        <div class="flex-1 min-h-0">
          <VChart :option="historyOption" autoresize />
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.v-chart {
  width: 100%;
  height: 100%;
}
</style>
