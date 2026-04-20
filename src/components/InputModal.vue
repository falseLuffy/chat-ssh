<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { X, Check } from 'lucide-vue-next';

const props = defineProps<{
  show: boolean;
  title: string;
  defaultValue: string;
  placeholder?: string;
}>();

const emit = defineEmits(['close', 'confirm']);
const inputValue = ref(props.defaultValue);
const inputRef = ref<HTMLInputElement | null>(null);

onMounted(() => {
  if (inputRef.value) {
    inputRef.value.focus();
    inputRef.value.select();
  }
});

const handleConfirm = () => {
  if (inputValue.value.trim()) {
    emit('confirm', inputValue.value.trim());
  }
};
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="fixed inset-0 z-[200] flex items-center justify-center p-4">
      <!-- Backdrop -->
      <div 
        class="absolute inset-0 bg-slate-950/60 backdrop-blur-sm animate-in fade-in duration-300"
        @click="emit('close')"
      ></div>
      
      <!-- Modal Content -->
      <div class="relative w-full max-w-sm bg-[#1e293b] border border-slate-700 rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 slide-in-from-bottom-4 duration-300">
        <div class="flex items-center justify-between p-4 border-b border-slate-800">
          <h3 class="text-sm font-bold text-white">{{ title }}</h3>
          <button @click="emit('close')" class="p-1 hover:bg-slate-800 rounded-lg text-slate-500 transition-colors">
            <X :size="16" />
          </button>
        </div>
        
        <div class="p-6">
          <input 
            ref="inputRef"
            v-model="inputValue"
            type="text"
            :placeholder="placeholder"
            class="w-full bg-slate-900 border border-slate-700 rounded-xl px-4 py-3 text-sm text-white placeholder:text-slate-600 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all"
            @keyup.enter="handleConfirm"
            @keyup.esc="emit('close')"
          />
        </div>
        
        <div class="flex items-center justify-end space-x-3 p-4 bg-slate-900/50">
          <button 
            @click="emit('close')"
            class="px-4 py-2 text-xs font-bold text-slate-400 hover:text-white transition-colors"
          >
            取消
          </button>
          <button 
            @click="handleConfirm"
            class="flex items-center space-x-2 px-5 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-bold transition-all shadow-lg shadow-blue-600/20 active:scale-95"
          >
            <Check :size="14" />
            <span>确认</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
