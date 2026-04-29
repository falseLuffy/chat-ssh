<script setup lang="ts">
import { useUIStore } from '../../stores/ui';
import { AlertTriangle, Info } from 'lucide-vue-next';

const ui = useUIStore();

const handleConfirm = () => ui.resolveConfirm(true);
const handleCancel = () => ui.resolveConfirm(false);
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity ease-out duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity ease-in duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="ui.confirmState.isOpen" class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 backdrop-blur-[2px]">
        <Transition
          enter-active-class="transition-transform ease-out duration-300"
          enter-from-class="scale-95 opacity-0"
          enter-to-class="scale-100 opacity-100"
          leave-active-class="transition-transform ease-in duration-200"
          leave-from-class="scale-100 opacity-100"
          leave-to-class="scale-95 opacity-0"
          appear
        >
          <div v-if="ui.confirmState.isOpen" class="bg-[#1e293b] border border-slate-700 w-full max-w-sm rounded-2xl shadow-2xl overflow-hidden p-6 relative">
            <div class="flex items-center space-x-3 mb-4">
              <div :class="['w-10 h-10 rounded-full flex items-center justify-center shrink-0', ui.confirmState.options.type === 'danger' ? 'bg-rose-500/10 text-rose-500' : 'bg-blue-500/10 text-blue-500']">
                <AlertTriangle v-if="ui.confirmState.options.type === 'danger' || ui.confirmState.options.type === 'warning'" :size="20" />
                <Info v-else :size="20" />
              </div>
              <h3 class="text-lg font-bold text-white">{{ ui.confirmState.options.title || '确认操作' }}</h3>
            </div>
            
            <p class="text-sm text-slate-300 mb-6 leading-relaxed">{{ ui.confirmState.options.message }}</p>
            
            <div class="flex space-x-3">
              <button @click="handleCancel" class="flex-1 py-2.5 rounded-xl text-slate-400 hover:bg-slate-800 transition-colors text-sm font-medium">
                {{ ui.confirmState.options.cancelText || '取消' }}
              </button>
              <button @click="handleConfirm" :class="['flex-1 py-2.5 rounded-xl text-white font-bold transition-all shadow-lg text-sm', ui.confirmState.options.type === 'danger' ? 'bg-rose-600 hover:bg-rose-500 shadow-rose-600/20' : 'bg-blue-600 hover:bg-blue-500 shadow-blue-600/20']">
                {{ ui.confirmState.options.confirmText || '确认' }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
