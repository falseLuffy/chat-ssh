<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { X, Save, Key, Zap, Eye, EyeOff, Loader2, CheckCircle } from 'lucide-vue-next';
import { useSettingsStore } from '../stores/settings';

const emit = defineEmits(['close']);
const settingsStore = useSettingsStore();

const apiKey = ref('');
const model = ref('deepseek-chat');
const showKey = ref(false);
const isSaving = ref(false);
const saveSuccess = ref(false);

onMounted(async () => {
  await settingsStore.loadSettings();
  apiKey.value = settingsStore.deepseekApiKey;
  model.value = settingsStore.selectedModel;
});

const handleSave = async () => {
  isSaving.value = true;
  saveSuccess.value = false;
  try {
    await settingsStore.saveSettings(apiKey.value, model.value);
    saveSuccess.value = true;
    setTimeout(() => {
      emit('close');
    }, 1000);
  } catch (error) {
    console.error(error);
  } finally {
    isSaving.value = false;
  }
};
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
      <div class="bg-[#1e293b] border border-slate-700 w-full max-w-md rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200">
        <!-- Modal Header -->
        <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-900/50">
          <h3 class="text-lg font-bold text-white flex items-center space-x-2">
            <Zap :size="20" class="text-amber-500" />
            <span>AI 模型配置</span>
          </h3>
          <button @click="$emit('close')" class="p-1 hover:bg-slate-800 rounded-full text-slate-400 transition-colors">
            <X :size="20" />
          </button>
        </div>

        <!-- Modal Body -->
        <div class="p-6 space-y-6">
          <div class="space-y-2">
            <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider">DeepSeek API Key</label>
            <div class="relative">
              <Key class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
              <input 
                v-model="apiKey" 
                :type="showKey ? 'text' : 'password'" 
                placeholder="sk-..." 
                class="w-full bg-slate-800 border border-slate-700 rounded-lg py-2.5 pl-10 pr-12 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-sm text-slate-200" 
              />
              <button 
                @click="showKey = !showKey" 
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 transition-colors"
              >
                <Eye v-if="!showKey" :size="18" />
                <EyeOff v-else :size="18" />
              </button>
            </div>
            <p class="text-[10px] text-slate-500">建议在 DeepSeek 官网获取 API Key 以启用 AI 功能。</p>
          </div>

          <div class="space-y-2">
            <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider">首选模型</label>
            <select 
              v-model="model" 
              class="w-full bg-slate-800 border border-slate-700 rounded-lg py-2.5 px-3 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-sm text-slate-200 appearance-none"
            >
              <option value="deepseek-chat">DeepSeek-V3 (Chat)</option>
              <option value="deepseek-reasoner">DeepSeek-R1 (Reasoner)</option>
            </select>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end space-x-3">
          <button @click="$emit('close')" class="px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors">
            取消
          </button>
          <button 
            @click="handleSave" 
            :disabled="isSaving"
            class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-bold flex items-center space-x-2 shadow-lg shadow-blue-600/20 transition-all disabled:opacity-50"
          >
            <Loader2 v-if="isSaving" class="animate-spin" :size="16" />
            <CheckCircle v-else-if="saveSuccess" class="text-emerald-400" :size="16" />
            <Save v-else :size="16" />
            <span>{{ saveSuccess ? '已保存' : '保存设置' }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
