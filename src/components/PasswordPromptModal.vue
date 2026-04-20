<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { X, Lock, Key, Server as ServerIcon, Network } from 'lucide-vue-next';
import { type Server } from '../stores/server';

const props = defineProps<{
  server: Server;
}>();

const emit = defineEmits(['close', 'submit']);

const password = ref('');
const passwordInput = ref<HTMLInputElement | null>(null);

onMounted(() => {
  passwordInput.value?.focus();
});

const handleSubmit = () => {
  if (!password.value) return;
  emit('submit', password.value);
};
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-[60] flex items-center justify-center bg-black/70 backdrop-blur-md animate-in fade-in duration-300">
      <div class="bg-[#1e293b] border border-slate-700 w-full max-w-sm rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200 ring-1 ring-white/10">
        <!-- Modal Header -->
        <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-900/50">
          <h3 class="text-sm font-bold text-white flex items-center space-x-2">
            <Lock :size="16" class="text-yellow-500" />
            <span>身份验证失败</span>
          </h3>
          <button @click="$emit('close')" class="p-1 hover:bg-slate-800 rounded-full text-slate-400 transition-colors">
            <X :size="16" />
          </button>
        </div>

        <!-- Modal Body -->
        <div class="p-6 space-y-4">
          <div class="flex items-center space-x-3 p-3 bg-red-500/10 border border-red-500/20 rounded-xl mb-2">
            <Network :size="20" class="text-red-400 shrink-0" />
            <div class="min-w-0">
              <div class="text-xs font-bold text-red-400 uppercase tracking-tighter">密码不正确</div>
              <div class="text-xs text-slate-400 truncate">{{ props.server.name }} ({{ props.server.host }})</div>
            </div>
          </div>

          <div class="space-y-1">
            <label class="text-[10px] font-semibold text-slate-500 uppercase tracking-widest px-1">请输入新密码</label>
            <div class="relative">
              <Key class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
              <input 
                ref="passwordInput"
                v-model="password" 
                type="password" 
                placeholder="键入新密码重新连接" 
                class="w-full bg-slate-800/80 border border-slate-700 rounded-lg py-2.5 pl-10 pr-4 focus:outline-none focus:ring-2 focus:ring-blue-500/40 focus:border-blue-500 transition-all text-sm text-slate-200 shadow-inner"
                @keyup.enter="handleSubmit"
              />
            </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end space-x-3">
          <button @click="$emit('close')" class="px-4 py-2 text-xs font-medium text-slate-400 hover:text-white transition-colors">
            稍后重试
          </button>
          <button 
            @click="handleSubmit" 
            :disabled="!password"
            class="px-5 py-2 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg text-xs font-bold flex items-center space-x-2 shadow-lg shadow-blue-600/20 transition-all active:scale-95"
          >
            <span>更新并连接</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
