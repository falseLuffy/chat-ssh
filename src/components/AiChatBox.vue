<script setup lang="ts">
  import { ref, onMounted, watch } from 'vue';
  import { invoke } from '@tauri-apps/api/core';
  import { useSettingsStore } from '../stores/settings';
  import { Sparkles, Play, ShieldAlert, Loader2, Copy, AlertTriangle, MessageSquare, Minimize2, Maximize2, User, CheckCircle } from 'lucide-vue-next';

  const settingsStore = useSettingsStore();

  const prompt = ref('');
  const isGenerating = ref(false);
  const isReviewing = ref(false);
  const riskAssessment = ref('');
  const isExecuted = ref(false);
  const viewState = ref<'minimized' | 'compact' | 'full'>((localStorage.getItem('ssh_ai_view_state') as 'minimized' | 'compact' | 'full') || 'minimized');

  // Persist view state
  watch(viewState, (newState) => {
    localStorage.setItem('ssh_ai_view_state', newState);
  });

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

  const executeCommand = async () => {
    if (!generatedCommand.value) return;
    try {
      const { emit } = await import('@tauri-apps/api/event');
      await emit('terminal-input', generatedCommand.value + '\n');

      isExecuted.value = true;
      setTimeout(() => {
        isExecuted.value = false;
      }, 2000);
    } catch (e) {
      console.error('Failed to emit terminal input:', e);
    }
  };
</script>

<template>
  <div class="fixed bottom-6 right-6 z-40">
    <!-- Minimized FAB -->
    <button v-if="viewState === 'minimized'" @click="viewState = 'compact'"
      class="w-14 h-14 bg-blue-600 hover:bg-blue-500 text-white rounded-full flex items-center justify-center shadow-2xl shadow-blue-600/40 transition-all duration-300 transform hover:scale-110 active:scale-95 group animate-in zoom-in">
      <div class="relative">
        <Sparkles :size="24" class="group-hover:animate-spin-slow" />
        <div class="absolute -top-1 -right-1 w-3 h-3 bg-emerald-500 rounded-full border-2 border-[#0f172a]"></div>
      </div>
    </button>

    <!-- Expanded Chatbox -->
    <div v-else :class="[
      'glass w-[400px] flex flex-col rounded-3xl shadow-2xl overflow-hidden border border-slate-700/50 animate-in slide-in-from-bottom-6 transition-all duration-500 ease-out',
      viewState === 'full' ? 'h-[calc(100vh-48px)] bottom-0' : 'max-h-[85vh]'
    ]">
      <!-- Header -->
      <div class="px-5 py-3 border-b border-slate-800/50 flex items-center justify-between bg-white/5">
        <div class="flex items-center space-x-2">
          <div class="p-1.5 bg-blue-500/20 rounded-lg text-blue-400">
            <Sparkles :size="16" />
          </div>
          <span class="text-xs font-bold uppercase tracking-wider text-slate-400">DeepSeek AI 助手</span>
        </div>
        <div class="flex items-center space-x-1">
          <button @click="viewState = viewState === 'full' ? 'compact' : 'full'"
            class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-500 hover:text-slate-300 transition-colors"
            :title="viewState === 'full' ? '还原窗口' : '全屏显示'">
            <Maximize2 v-if="viewState === 'compact'" :size="16" />
            <Minimize2 v-else :size="16" />
          </button>
          <button @click="viewState = 'minimized'"
            class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-500 hover:text-slate-300 transition-colors"
            title="收起">
            <MessageSquare :size="16" />
          </button>
        </div>
      </div>
      <!-- Chatbox Body -->
      <div class="p-5 overflow-y-auto flex-1 custom-scrollbar-main">
        <!-- Risk Assessment Popup/Panel -->
        <div v-if="riskAssessment"
          class="mb-4 p-4 bg-amber-500/10 border border-amber-500/30 rounded-xl animate-in fade-in slide-in-from-bottom-2 max-h-60 overflow-y-auto custom-scrollbar">
          <div class="flex items-center space-x-2 text-amber-500 mb-2 font-bold">
            <AlertTriangle :size="18" />
            <span>安全专家建议：</span>
          </div>
          <p class="text-[11px] text-slate-300 leading-relaxed whitespace-pre-wrap">{{ riskAssessment }}</p>
          <button @click="riskAssessment = ''"
            class="mt-2 text-xs text-slate-500 hover:text-slate-300 underline">忽略警告并关闭</button>
        </div>

        <!-- Main Input & Result -->
        <div class="flex flex-col space-y-4">
          <!-- Generated Command Display -->
          <div v-if="generatedCommand" class="flex flex-col p-3 bg-black/40 rounded-xl group border border-slate-800">
            <div class="flex items-center justify-between mb-2">
              <span class="text-[10px] text-slate-500 font-bold uppercase tracking-widest">建议指令</span>
              <div class="flex space-x-1">
                <button @click="reviewCommand" :disabled="isReviewing"
                  class="p-1.5 hover:bg-amber-500/20 text-amber-500 rounded-lg transition-colors" title="审查风险">
                  <Loader2 v-if="isReviewing" class="animate-spin" :size="14" />
                  <ShieldAlert v-else :size="14" />
                </button>
                <button @click="executeCommand"
                  class="p-1.5 hover:bg-emerald-500/20 text-emerald-500 rounded-lg transition-colors"
                  :title="isExecuted ? '指令已发送' : '立即执行'">
                  <CheckCircle v-if="isExecuted" :size="14" />
                  <Play v-else :size="14" />
                </button>
              </div>
            </div>
            <code
              class="text-emerald-400 font-mono text-xs overflow-x-auto whitespace-pre p-2 bg-black/20 rounded-md border border-white/5">{{ generatedCommand }}</code>
          </div>

          <!-- Input Bar -->
          <div class="relative flex items-center">
            <div class="absolute left-4 text-blue-500">
              <Sparkles :size="18" class="animate-pulse" />
            </div>
            <input v-model="prompt" @keyup.enter="generateCommand" placeholder="描述任务... (例如：清理日志)"
              class="w-full bg-slate-800/50 border border-slate-700/50 rounded-2xl py-2.5 pl-11 pr-12 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-xs" />
            <div class="absolute right-2">
              <button @click="generateCommand" :disabled="isGenerating"
                class="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white rounded-xl px-3 py-1.5 text-[10px] font-bold transition-all shadow-lg shadow-blue-600/20">
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
    from {
      transform: rotate(0deg);
    }

    to {
      transform: rotate(360deg);
    }
  }

  .group-hover\:animate-spin-slow:hover {
    animation: spin-slow 8s linear infinite;
  }

  code::-webkit-scrollbar {
    display: none;
  }

  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }

  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(245, 158, 11, 0.2);
    border-radius: 10px;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(245, 158, 11, 0.4);
  }

  .custom-scrollbar-main::-webkit-scrollbar {
    width: 4px;
  }

  .custom-scrollbar-main::-webkit-scrollbar-track {
    background: transparent;
  }

  .custom-scrollbar-main::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
  }

  .custom-scrollbar-main::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .transition-all {
    transition-property: all;
    transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
    transition-duration: 300ms;
  }
</style>
