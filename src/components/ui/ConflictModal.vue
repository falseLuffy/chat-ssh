<script setup lang="ts">
import { ref } from 'vue';
import { useUIStore, ConflictAction } from '../../stores/ui';
import { AlertTriangle, HardDrive, SkipForward, RefreshCw, Copy } from 'lucide-vue-next';

const ui = useUIStore();
const applyToAll = ref(false);

const handleAction = (action: ConflictAction) => {
  ui.resolveConflict(action, applyToAll.value);
};
</script>

<template>
  <Teleport to="body">
    <div v-if="ui.conflictState.isOpen" class="fixed inset-0 z-[200] flex items-center justify-center p-4">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm animate-in fade-in duration-300"></div>
      
      <!-- Modal -->
      <div class="relative w-full max-w-md bg-[#1e293b] border border-slate-700 rounded-3xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200">
        <div class="p-6">
          <div class="flex items-center space-x-4 mb-6">
            <div class="p-3 bg-amber-500/10 rounded-2xl">
              <AlertTriangle class="text-amber-500" :size="28" />
            </div>
            <div>
              <h3 class="text-xl font-bold text-white">文件已存在</h3>
              <p class="text-slate-400 text-sm mt-1">
                远程服务器上已有名为 <span class="text-amber-400 font-mono">{{ ui.conflictState.options.fileName }}</span> 的项目。
              </p>
            </div>
          </div>

          <div class="space-y-3">
            <button 
              @click="handleAction('overwrite')"
              class="w-full flex items-center p-4 bg-slate-800/50 hover:bg-red-500/10 border border-slate-700 hover:border-red-500/50 rounded-2xl transition-all group"
            >
              <RefreshCw class="text-slate-400 group-hover:text-red-400 mr-4" :size="20" />
              <div class="text-left">
                <div class="text-sm font-bold text-slate-200 group-hover:text-red-400">覆盖</div>
                <div class="text-[10px] text-slate-500">用新文件替换服务器上的旧文件</div>
              </div>
            </button>

            <button 
              @click="handleAction('skip')"
              class="w-full flex items-center p-4 bg-slate-800/50 hover:bg-slate-700 border border-slate-700 rounded-2xl transition-all group"
            >
              <SkipForward class="text-slate-400 group-hover:text-white mr-4" :size="20" />
              <div class="text-left">
                <div class="text-sm font-bold text-slate-200 group-hover:text-white">跳过</div>
                <div class="text-[10px] text-slate-500">不上传此文件，保持服务器文件不变</div>
              </div>
            </button>

            <button 
              @click="handleAction('rename')"
              class="w-full flex items-center p-4 bg-slate-800/50 hover:bg-blue-500/10 border border-slate-700 hover:border-blue-500/50 rounded-2xl transition-all group"
            >
              <Copy class="text-slate-400 group-hover:text-blue-400 mr-4" :size="20" />
              <div class="text-left">
                <div class="text-sm font-bold text-slate-200 group-hover:text-blue-400">保留两者</div>
                <div class="text-[10px] text-slate-500">重命名新文件（例如：file (1).txt）</div>
              </div>
            </button>
          </div>

          <div class="mt-6 pt-4 border-t border-slate-800">
            <label class="flex items-center space-x-3 cursor-pointer group">
              <div class="relative flex items-center">
                <input 
                  type="checkbox" 
                  v-model="applyToAll" 
                  class="peer h-5 w-5 appearance-none rounded-md border border-slate-600 bg-slate-800 checked:bg-blue-600 checked:border-blue-500 transition-all cursor-pointer"
                />
                <svg class="absolute h-3.5 w-3.5 text-white pointer-events-none hidden peer-checked:block left-0.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
              </div>
              <span class="text-xs text-slate-400 group-hover:text-slate-200 transition-colors">对本次任务中所有冲突应用此操作</span>
            </label>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
