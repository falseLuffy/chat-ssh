<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../stores/settings';
import { Sparkles, Play, ShieldAlert, Loader2, Copy, AlertTriangle, MessageSquare, Minimize2, Maximize2, User } from 'lucide-vue-next';

const settingsStore = useSettingsStore();

const prompt = ref('');
const isGenerating = ref(false);
const isReviewing = ref(false);
const isMinimized = ref(false);
const generatedCommand = ref('');
const riskAssessment = ref('');

onMounted(async () => {
  if (!settingsStore.isLoaded) {
    await settingsStore.loadSettings();
  }
});

const generateCommand = async () => {
  if (!prompt.value) return;
  isGenerating.value = true;
  generatedCommand.value = '';
  riskAssessment.value = '';
  
  try {
    const res = await invoke<string>('generate_ai_command', { 
      prompt: prompt.value, 
      apiKey: settingsStore.deepseekApiKey || '' 
    });
    generatedCommand.value = res;
  } catch (e) {
    console.error(e);
    generatedCommand.value = '生成失败，请检查设置中的 API Key 或网络。';
  } finally {
    isGenerating.value = false;
  }
};

const reviewCommand = async () => {
  if (!generatedCommand.value) return;
  isReviewing.value = true;
  
  try {
    const res = await invoke<string>('review_command_risk', { 
      command: generatedCommand.value, 
      apiKey: settingsStore.deepseekApiKey || '' 
    });
    riskAssessment.value = res;
  } catch (e) {
    console.error(e);
    riskAssessment.value = '审查失败，请检查设置中的 API Key。';
  } finally {
    isReviewing.value = false;
  }
};

const executeCommand = () => {
  // In a real app, this would emit an event to the Terminal component
  alert(`执行指令: ${generatedCommand.value}`);
};
</script>

<template>
  <div class="fixed bottom-6 right-6 z-40">
    <!-- Minimized FAB -->
    <button 
      v-if="isMinimized"
      @click="isMinimized = false"
      class="w-14 h-14 bg-blue-600 hover:bg-blue-500 text-white rounded-full flex items-center justify-center shadow-2xl shadow-blue-600/40 transition-all duration-300 transform hover:scale-110 active:scale-95 group animate-in zoom-in"
    >
      <div class="relative">
        <Sparkles :size="24" class="group-hover:animate-spin-slow" />
        <div class="absolute -top-1 -right-1 w-3 h-3 bg-emerald-500 rounded-full border-2 border-[#0f172a]"></div>
      </div>
    </button>

    <!-- Expanded Chatbox -->
    <div 
      v-else 
      class="glass w-[400px] rounded-3xl shadow-2xl overflow-hidden border border-slate-700/50 animate-in slide-in-from-bottom-6 duration-300"
    >
      <!-- Header -->
      <div class="px-5 py-3 border-b border-slate-800/50 flex items-center justify-between bg-white/5">
        <div class="flex items-center space-x-2">
          <div class="p-1.5 bg-blue-500/20 rounded-lg text-blue-400">
            <Sparkles :size="16" />
          </div>
          <span class="text-xs font-bold uppercase tracking-wider text-slate-400">DeepSeek AI 助手</span>
        </div>
        <button 
          @click="isMinimized = true"
          class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-500 hover:text-slate-300 transition-colors"
        >
          <Minimize2 :size="16" />
        </button>
      </div>
      <!-- Chatbox Body -->
      <div class="p-5">
        <!-- Risk Assessment Popup/Panel -->
        <div v-if="riskAssessment" class="mb-4 p-4 bg-amber-500/10 border border-amber-500/30 rounded-xl animate-in fade-in slide-in-from-bottom-2">
          <div class="flex items-center space-x-2 text-amber-500 mb-2 font-bold">
            <AlertTriangle :size="18" />
            <span>安全专家建议：</span>
          </div>
          <p class="text-[11px] text-slate-300 leading-relaxed whitespace-pre-wrap">{{ riskAssessment }}</p>
          <button @click="riskAssessment = ''" class="mt-2 text-xs text-slate-500 hover:text-slate-300 underline">忽略警告并关闭</button>
        </div>

        <!-- Main Input & Result -->
        <div class="flex flex-col space-y-4">
          <!-- Generated Command Display -->
          <div v-if="generatedCommand" class="flex flex-col p-3 bg-black/40 rounded-xl group border border-slate-800">
            <div class="flex items-center justify-between mb-2">
              <span class="text-[10px] text-slate-500 font-bold uppercase tracking-widest">建议指令</span>
              <div class="flex space-x-1">
                <button @click="reviewCommand" :disabled="isReviewing" class="p-1.5 hover:bg-amber-500/20 text-amber-500 rounded-lg transition-colors" title="审查风险">
                  <Loader2 v-if="isReviewing" class="animate-spin" :size="14" />
                  <ShieldAlert v-else :size="14" />
                </button>
                <button @click="executeCommand" class="p-1.5 hover:bg-emerald-500/20 text-emerald-500 rounded-lg transition-colors" title="立即执行">
                  <Play :size="14" />
                </button>
              </div>
            </div>
            <code class="text-emerald-400 font-mono text-xs overflow-x-auto whitespace-pre p-2 bg-black/20 rounded-md border border-white/5">{{ generatedCommand }}</code>
          </div>

          <!-- Input Bar -->
          <div class="relative flex items-center">
            <div class="absolute left-4 text-blue-500">
              <Sparkles :size="18" class="animate-pulse" />
            </div>
            <input 
              v-model="prompt"
              @keyup.enter="generateCommand"
              placeholder="描述任务... (例如：清理日志)"
              class="w-full bg-slate-800/50 border border-slate-700/50 rounded-2xl py-2.5 pl-11 pr-12 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-xs"
            />
            <div class="absolute right-2">
              <button 
                @click="generateCommand"
                :disabled="isGenerating"
                class="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white rounded-xl px-3 py-1.5 text-[10px] font-bold transition-all shadow-lg shadow-blue-600/20"
              >
                <Loader2 v-if="isGenerating" class="animate-spin" :size="14" />
                <span v-else>发送</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes spin-slow {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
.group-hover\:animate-spin-slow:hover {
  animation: spin-slow 8s linear infinite;
}
code::-webkit-scrollbar {
  display: none;
}
</style>
