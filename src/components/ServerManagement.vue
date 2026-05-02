<script setup lang="ts">
import { ref } from 'vue';
import type { Server } from '../stores/server';
import { LayoutDashboard, Box, Settings, Activity, ShieldCheck, SearchCheck } from 'lucide-vue-next';
import DashboardOverview from './DashboardOverview.vue';
import DockerView from './DockerView.vue';
import ServicesView from './ServicesView.vue';
import ProcessView from './ProcessView.vue';
import FirewallView from './FirewallView.vue';
import InspectionRunner from './InspectionRunner.vue';
import InspectionHistory from './InspectionHistory.vue';
import InspectionRuleEditor from './InspectionRuleEditor.vue';
import InspectionScheduler from './InspectionScheduler.vue';

const props = defineProps<{ server: Server }>();

const activeSubTab = ref('overview');
const inspectionView = ref<'runner' | 'history'>('runner');
const showRuleEditor = ref(false);
const showScheduler = ref(false);

const subTabs = [
  { id: 'overview', name: '概览', icon: LayoutDashboard },
  { id: 'processes', name: '进程', icon: Activity },
  { id: 'docker', name: 'Docker', icon: Box },
  { id: 'services', name: '服务', icon: Settings },
  { id: 'firewall', name: '防火墙', icon: ShieldCheck },
  { id: 'inspection', name: '巡检', icon: SearchCheck },
];

const onViewHistory = () => {
  inspectionView.value = 'history';
};

const onBackToRunner = () => {
  inspectionView.value = 'runner';
};
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
        <!-- Inspection tab buttons -->
        <div v-if="activeSubTab === 'inspection'" class="flex items-center space-x-2">
          <button
            @click="inspectionView = 'runner'"
            :class="[
              'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
              inspectionView === 'runner'
                ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/20'
                : 'text-slate-400 hover:text-slate-200 bg-slate-800/50'
            ]"
          >
            立即巡检
          </button>
          <button
            @click="inspectionView = 'history'"
            :class="[
              'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
              inspectionView === 'history'
                ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/20'
                : 'text-slate-400 hover:text-slate-200 bg-slate-800/50'
            ]"
          >
            历史记录
          </button>
          <div class="w-px h-5 bg-slate-700/50"></div>
          <button
            @click="showRuleEditor = true"
            class="text-[10px] px-2 py-1 rounded-lg text-slate-500 hover:text-slate-200 hover:bg-slate-800 transition-colors"
          >
            编辑规则
          </button>
          <button
            @click="showScheduler = true"
            class="text-[10px] px-2 py-1 rounded-lg text-slate-500 hover:text-slate-200 hover:bg-slate-800 transition-colors"
          >
            定时设置
          </button>
        </div>
        <!-- Non-inspection tabs show monitoring indicator -->
        <div v-else class="flex items-center space-x-2 text-[10px] uppercase tracking-widest font-bold text-slate-500">
          <Activity :size="12" class="text-emerald-500 animate-pulse" />
          <span>实时监控中</span>
        </div>
      </div>
    </div>

    <!-- Sub-tab Content -->
    <div class="flex-1 min-h-0 overflow-hidden flex flex-col">
      <DashboardOverview v-show="activeSubTab === 'overview'" :server="server" />
      <ProcessView v-show="activeSubTab === 'processes'" :server="server" />
      <DockerView v-show="activeSubTab === 'docker'" :server="server" />
      <ServicesView v-show="activeSubTab === 'services'" :server="server" />
      <FirewallView v-show="activeSubTab === 'firewall'" :server="server" />
      <div v-show="activeSubTab === 'inspection'" class="flex-1 min-h-0 overflow-hidden flex flex-col">
        <InspectionRunner
          v-if="inspectionView === 'runner'"
          :server="server"
          @view-history="onViewHistory"
        />
        <InspectionHistory
          v-else
          :server="server"
          @back="onBackToRunner"
        />
      </div>
    </div>

    <!-- Rule Editor Modal -->
    <InspectionRuleEditor
      v-if="showRuleEditor && server"
      :server="server"
      @close="showRuleEditor = false"
    />

    <!-- Scheduler Modal -->
    <InspectionScheduler
      v-if="showScheduler"
      @close="showScheduler = false"
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
