<script setup lang="ts">
import { ref } from 'vue';
import { X, Loader2, ShieldCheck, AlertTriangle, Sparkles, Copy, Check } from 'lucide-vue-next';
import MarkdownIt from 'markdown-it';

const props = defineProps<{
  isOpen: boolean;
  isGenerating: boolean;
  result: string;
  error: string;
}>();

const emit = defineEmits(['close']);

const md = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true
});

const isCopied = ref(false);

const copyResult = () => {
  navigator.clipboard.writeText(props.result);
  isCopied.value = true;
  setTimeout(() => isCopied.value = false, 2000);
};

const renderedResult = ref('');
import { watch } from 'vue';
watch(() => props.result, (newVal) => {
  if (newVal) {
    renderedResult.value = md.render(newVal);
  }
}, { immediate: true });

</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
    <div class="bg-[#1e293b] border border-slate-700 w-full max-w-2xl rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[80vh] animate-in zoom-in duration-200">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-white/5">
        <div class="flex items-center space-x-2">
          <div class="p-2 bg-blue-500/20 rounded-xl text-blue-400">
            <Sparkles :size="20" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-white">AI 智能诊断报告</h3>
            <p class="text-[10px] text-slate-500 uppercase tracking-widest">DeepSeek Engine</p>
          </div>
        </div>
        <button @click="emit('close')" class="p-2 hover:bg-slate-800 rounded-xl text-slate-500 transition-colors">
          <X :size="20" />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6 custom-scrollbar">
        <div v-if="isGenerating" class="h-64 flex flex-col items-center justify-center space-y-4">
          <div class="relative">
            <Loader2 class="animate-spin text-blue-500" :size="48" />
            <Sparkles class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-blue-400/50" :size="20" />
          </div>
          <p class="text-sm text-slate-400 animate-pulse">正在深度分析服务器状态...</p>
        </div>

        <div v-else-if="error" class="bg-red-500/10 border border-red-500/30 p-6 rounded-2xl text-center">
          <AlertTriangle class="text-red-500 mx-auto mb-3" :size="32" />
          <p class="text-sm text-slate-300">{{ error }}</p>
        </div>

        <div v-else class="space-y-4">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center space-x-2 text-emerald-500">
              <ShieldCheck :size="18" />
              <span class="text-xs font-bold uppercase">诊断完成</span>
            </div>
            <button @click="copyResult" class="flex items-center space-x-1 text-[10px] text-slate-500 hover:text-slate-300 transition-colors">
              <Check v-if="isCopied" :size="12" class="text-emerald-500" />
              <Copy v-else :size="12" />
              <span>{{ isCopied ? '已复制' : '复制报告' }}</span>
            </button>
          </div>
          
          <div class="markdown-content text-sm text-slate-300 leading-relaxed" v-html="renderedResult"></div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end">
        <button @click="emit('close')" class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-bold rounded-xl transition-all shadow-lg shadow-blue-600/20">
          了解并关闭
        </button>
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
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}

.markdown-content :deep(h1), .markdown-content :deep(h2), .markdown-content :deep(h3) {
  color: white;
  font-weight: 700;
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
}
.markdown-content :deep(h1) { font-size: 1.25rem; }
.markdown-content :deep(h2) { font-size: 1.1rem; border-left: 3px solid #3b82f6; padding-left: 0.75rem; }
.markdown-content :deep(p) { margin-bottom: 1rem; }
.markdown-content :deep(strong) { color: #f59e0b; }
.markdown-content :deep(ul) { list-style-type: disc; margin-left: 1.5rem; margin-bottom: 1rem; }
.markdown-content :deep(code) { background: rgba(0,0,0,0.3); padding: 0.1rem 0.3rem; border-radius: 0.25rem; font-family: monospace; color: #10b981; }
</style>
