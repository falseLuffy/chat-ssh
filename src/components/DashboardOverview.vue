<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import type { Server } from '../stores/server';
import { invoke } from '@tauri-apps/api/core';
import { Loader2, RefreshCw, Cpu, HardDrive, MemoryStick, Clock, Monitor, ArrowUp, ArrowDown, Activity, Server as ServerIcon } from 'lucide-vue-next';

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

const props = defineProps<{ server: Server }>();
const isLoading = ref(true);
const sysInfo = ref<any>(null);
const error = ref('');
let pollTimer: any = null;

const history = ref<{ cpu: number[], net_rx: number[], net_tx: number[], time: string[] }>({
  cpu: [],
  net_rx: [],
  net_tx: [],
  time: []
});

const fetchInfo = async () => {
  if (!props.server || props.server.status !== 'online') {
    error.value = '服务器未连接';
    isLoading.value = false;
    return;
  }

  try {
    const info = await invoke<any>('get_server_sys_info', {
      serverName: props.server.name
    });
    sysInfo.value = info;
    error.value = '';

    // Update history for line chart
    const now = new Date().toLocaleTimeString();
    history.value.time.push(now);
    history.value.cpu.push(info.cpu.usage * 100);
    history.value.net_rx.push(info.net.rx_speed);
    history.value.net_tx.push(info.net.tx_speed);
    if (history.value.time.length > 20) {
      history.value.time.shift();
      history.value.cpu.shift();
      history.value.net_rx.shift();
      history.value.net_tx.shift();
    }
  } catch (e) {
    console.error('Failed to fetch sys info:', e);
    error.value = String(e);
  } finally {
    isLoading.value = false;
  }
};

const startPolling = () => {
  if (pollTimer) return;
  pollTimer = setInterval(fetchInfo, 5000);
};

const stopPolling = () => {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
};

onMounted(() => {
  if (props.server.status === 'online') {
    isLoading.value = true;
    fetchInfo();
    startPolling();
  }
});

onUnmounted(() => {
  stopPolling();
});

watch(() => props.server.status, (newStatus) => {
  if (newStatus === 'online') {
    isLoading.value = true;
    fetchInfo();
    startPolling();
  } else {
    stopPolling();
    error.value = '服务器未连接';
    isLoading.value = false;
    sysInfo.value = null;
  }
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
  tooltip: {
    trigger: 'item',
    formatter: (params: any) => `${params.name}: ${formatBytes(params.value)} (${params.percent}%)`
  },
  legend: { bottom: '5%', left: 'center', textStyle: { color: '#94a3b8' } },
  series: [
    {
      name: '内存',
      type: 'pie',
      radius: ['50%', '70%'],
      avoidLabelOverlap: false,
      itemStyle: { borderRadius: 10, borderColor: '#0f172a', borderWidth: 2 },
      label: { show: false, position: 'center' },
      emphasis: { label: { show: true, fontSize: 16, fontWeight: 'bold', color: '#fff', formatter: '{d}%' } },
      labelLine: { show: false },
      data: []
    }
  ]
});

const diskOption = ref({
  tooltip: {
    trigger: 'item',
    formatter: (params: any) => `${params.name}: ${formatBytes(params.value)} (${params.percent}%)`
  },
  legend: { bottom: '5%', left: 'center', textStyle: { color: '#94a3b8' } },
  series: [
    {
      name: '磁盘',
      type: 'pie',
      radius: ['50%', '70%'],
      avoidLabelOverlap: false,
      itemStyle: { borderRadius: 10, borderColor: '#0f172a', borderWidth: 2 },
      label: { show: false, position: 'center' },
      emphasis: { label: { show: true, fontSize: 16, fontWeight: 'bold', color: '#fff', formatter: '{d}%' } },
      labelLine: { show: false },
      data: []
    }
  ]
});

const historyOption = ref({
  tooltip: {
    trigger: 'axis',
    formatter: (params: any) => {
      const p = params[0];
      return `<div class="bg-[#1e293b] border border-slate-800 p-2 rounded-lg shadow-xl">
                <div class="text-[10px] text-slate-400 mb-1">${p.name}</div>
                <div class="flex items-center justify-between space-x-4">
                  <span class="flex items-center text-xs text-slate-300">
                    <span class="w-2 h-2 rounded-full mr-2" style="background-color: #60a5fa"></span>
                    CPU 占用
                  </span>
                  <span class="text-xs font-mono font-bold text-blue-400">${p.value.toFixed(1)}%</span>
                </div>
              </div>`;
    },
    backgroundColor: 'transparent',
    borderWidth: 0,
    padding: 0
  },
  grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
  xAxis: { type: 'category', boundaryGap: false, data: [], axisLabel: { color: '#64748b' } },
  yAxis: { type: 'value', max: 100, axisLabel: { color: '#64748b' } },
  series: [{
    name: 'CPU',
    data: [],
    type: 'line',
    smooth: true,
    areaStyle: { color: 'rgba(96, 165, 250, 0.2)' },
    itemStyle: { color: '#60a5fa' }
  }]
});

const netOption = ref({
  tooltip: {
    trigger: 'axis',
    formatter: (params: any) => {
      let res = `<div class="text-[10px] text-slate-400 mb-1">${params[0].name}</div>`;
      params.forEach((item: any) => {
        res += `<div class="flex items-center justify-between space-x-4">
                  <span class="flex items-center text-xs text-slate-300">
                    <span class="w-2 h-2 rounded-full mr-2" style="background-color: ${item.color}"></span>
                    ${item.seriesName}
                  </span>
                  <span class="text-xs font-mono font-bold" style="color: ${item.color}">${formatSpeed(item.value)}</span>
                </div>`;
      });
      return `<div class="bg-[#1e293b] border border-slate-800 p-2 rounded-lg shadow-xl">${res}</div>`;
    },
    backgroundColor: 'transparent',
    borderWidth: 0,
    padding: 0
  },
  legend: { data: ['下载', '上传'], textStyle: { color: '#94a3b8' } },
  grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
  xAxis: { type: 'category', boundaryGap: false, data: [], axisLabel: { color: '#64748b' } },
  yAxis: {
    type: 'value',
    axisLabel: {
      color: '#64748b',
      formatter: (val: number) => formatSpeed(val)
    }
  },
  series: [
    {
      name: '下载',
      data: [],
      type: 'line',
      smooth: true,
      itemStyle: { color: '#10b981' },
      areaStyle: { color: 'rgba(16, 185, 129, 0.1)' }
    },
    {
      name: '上传',
      data: [],
      type: 'line',
      smooth: true,
      itemStyle: { color: '#6366f1' },
      areaStyle: { color: 'rgba(99, 102, 241, 0.1)' }
    }
  ]
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

  netOption.value.xAxis.data = history.value.time as any;
  netOption.value.series[0].data = history.value.net_rx as any;
  netOption.value.series[1].data = history.value.net_tx as any;
}, { deep: true });

const formatSpeed = (bytesPerSec: number) => {
  if (bytesPerSec < 1024) return bytesPerSec.toFixed(0) + ' B/s';
  if (bytesPerSec < 1024 * 1024) return (bytesPerSec / 1024).toFixed(1) + ' KB/s';
  return (bytesPerSec / (1024 * 1024)).toFixed(1) + ' MB/s';
};

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};
</script>

<template>
  <div class="h-full flex flex-col p-6 space-y-6 overflow-y-auto bg-[#0f172a]/30 custom-scrollbar-main">
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
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-4 rounded-2xl flex items-center space-x-4">
          <div class="p-3 bg-indigo-500/10 rounded-xl text-indigo-500">
            <ServerIcon :size="24" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">操作系统</p>
            <p class="text-sm font-bold text-slate-200 truncate" :title="sysInfo.os_info">{{ sysInfo.os_info }}</p>
          </div>
        </div>

        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-4 rounded-2xl flex items-center space-x-4">
          <div class="p-3 bg-blue-500/10 rounded-xl text-blue-500">
            <Monitor :size="24" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">主机名</p>
            <p class="text-sm font-bold text-slate-200 truncate" :title="sysInfo.hostname">{{ sysInfo.hostname }}</p>
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
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">内存状态</p>
            <p class="text-sm font-bold text-slate-200">
              {{ formatBytes(sysInfo.memory.used) }} / {{ formatBytes(sysInfo.memory.total) }}
              <span class="text-blue-400 ml-1">({{ ((sysInfo.memory.used / sysInfo.memory.total) * 100).toFixed(1) }}%)</span>
            </p>
          </div>
        </div>

        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-4 rounded-2xl flex items-center space-x-4">
          <div class="p-3 bg-purple-500/10 rounded-xl text-purple-500">
            <HardDrive :size="24" />
          </div>
          <div>
            <p class="text-[10px] uppercase tracking-widest font-bold text-slate-500">磁盘状态</p>
            <p class="text-sm font-bold text-slate-200">
              {{ sysInfo.disks[0]?.mount || '/' }}
              <span :class="['ml-1', (sysInfo.disks[0]?.percent || 0) > 90 ? 'text-red-500' : 'text-amber-500']">
                ({{ sysInfo.disks[0]?.percent || 0 }}%)
              </span>
            </p>
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

      <!-- Charts Grid -->
      <div class="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <!-- CPU History -->
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-6 rounded-3xl flex flex-col min-h-[320px]">
          <h3 class="text-sm font-bold text-slate-400 mb-4 flex items-center">
            <Cpu class="mr-2" :size="16" /> CPU 负载趋势
          </h3>
          <div class="flex-1 min-h-0">
            <VChart :option="historyOption" autoresize />
          </div>
        </div>

        <!-- Network History -->
        <div class="bg-[#1e293b]/50 backdrop-blur-xl border border-slate-800 p-6 rounded-3xl flex flex-col min-h-[320px]">
          <h3 class="text-sm font-bold text-slate-400 mb-4 flex items-center justify-between">
            <div class="flex items-center">
              <Activity class="mr-2" :size="16" /> 实时网络流量
            </div>
            <div class="flex items-center space-x-3 text-[10px] font-mono">
              <span class="text-emerald-500 flex items-center"><ArrowDown :size="10" /> {{ formatSpeed(sysInfo?.net?.rx_speed || 0) }}</span>
              <span class="text-indigo-500 flex items-center"><ArrowUp :size="10" /> {{ formatSpeed(sysInfo?.net?.tx_speed || 0) }}</span>
            </div>
          </h3>
          <div class="flex-1 min-h-0">
            <VChart :option="netOption" autoresize />
          </div>
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
