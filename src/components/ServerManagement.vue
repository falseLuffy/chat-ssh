<script setup lang="ts">
import { ref } from 'vue';
import { LayoutDashboard, Box, Settings, Activity } from 'lucide-vue-next';
import DashboardOverview from './DashboardOverview.vue';
import DockerView from './DockerView.vue';
import ServicesView from './ServicesView.vue';

const activeSubTab = ref('overview');

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
      
      <div class="ml-auto flex items-center space-x-2 text-[10px] uppercase tracking-widest font-bold text-slate-500">
        <Activity :size="12" class="text-emerald-500 animate-pulse" />
        <span>实时监控中</span>
      </div>
    </div>

    <!-- Sub-tab Content -->
    <div class="flex-1 min-h-0 overflow-hidden">
      <transition 
        name="fade" 
        mode="out-in"
      >
        <DashboardOverview v-if="activeSubTab === 'overview'" />
        <DockerView v-else-if="activeSubTab === 'docker'" />
        <ServicesView v-else-if="activeSubTab === 'services'" />
      </transition>
    </div>
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
