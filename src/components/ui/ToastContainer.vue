<script setup lang="ts">
import { useUIStore } from '../../stores/ui';
import { CheckCircle, AlertCircle, Info, AlertTriangle, X } from 'lucide-vue-next';

const ui = useUIStore();

const getIcon = (type: string) => {
  switch (type) {
    case 'success': return CheckCircle;
    case 'error': return AlertCircle;
    case 'warning': return AlertTriangle;
    case 'info':
    default: return Info;
  }
};

const getColors = (type: string) => {
  switch (type) {
    case 'success': return 'border-emerald-500/30 text-emerald-400 shadow-emerald-500/10';
    case 'error': return 'border-rose-500/30 text-rose-400 shadow-rose-500/10';
    case 'warning': return 'border-amber-500/30 text-amber-400 shadow-amber-500/10';
    case 'info':
    default: return 'border-blue-500/30 text-blue-400 shadow-blue-500/10';
  }
};
</script>

<template>
  <div class="fixed top-4 right-4 z-[200] flex flex-col space-y-3 pointer-events-none w-80">
    <TransitionGroup 
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0 translate-x-8 scale-95"
      enter-to-class="opacity-100 translate-x-0 scale-100"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 translate-x-0 scale-100"
      leave-to-class="opacity-0 translate-x-8 scale-95"
    >
      <div v-for="toast in ui.toasts" :key="toast.id"
        :class="['pointer-events-auto flex items-start space-x-3 p-4 rounded-xl border backdrop-blur-xl shadow-lg bg-[#1e293b]/95', getColors(toast.type)]">
        <component :is="getIcon(toast.type)" class="shrink-0 mt-0.5" :size="18" />
        <div class="flex-1 text-sm font-medium leading-relaxed">{{ toast.message }}</div>
        <button @click="ui.removeToast(toast.id)" class="shrink-0 opacity-50 hover:opacity-100 transition-opacity">
          <X :size="16" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
