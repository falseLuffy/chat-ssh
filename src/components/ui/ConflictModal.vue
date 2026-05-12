<script setup lang="ts">
import { ref, computed } from 'vue';
import { useUIStore, ConflictAction, ConflictScope } from '../../stores/ui';
import { AlertTriangle, SkipForward, RefreshCw, Copy, ChevronDown } from 'lucide-vue-next';

const ui = useUIStore();
const selectedScope = ref<ConflictScope>(ui.conflictState.options.persistentAction ? 'persistent' : 'once');

const scopeOptions = [
  { value: 'once' as ConflictScope, label: '仅本次' },
  { value: 'batch' as ConflictScope, label: '本次任务全部' },
  { value: 'persistent' as ConflictScope, label: '以后全部上传' },
];

const hasPersistent = computed(() => !!ui.conflictState.options.persistentAction);

const handleAction = (action: ConflictAction) => {
  ui.resolveConflict(action, selectedScope.value);
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

          <div class="mt-5 pt-4 border-t border-slate-800">
            <div class="relative w-1/3 min-w-[120px]">
              <select
                v-model="selectedScope"
                class="w-full appearance-none bg-slate-800/80 border border-slate-700 rounded-lg px-2.5 py-1.5 pr-7 text-xs text-slate-300 outline-none cursor-pointer transition-all hover:border-slate-500 focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/30"
              >
                <option v-for="opt in scopeOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <ChevronDown class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none" :size="12" />
            </div>
            <p class="text-[10px] text-slate-500 mt-1.5 pl-1">
              <template v-if="hasPersistent">
                当前默认：以后全部上传 → "{{ { overwrite: '覆盖', skip: '跳过', rename: '保留两者' }[ui.conflictState.options.persistentAction!] }}"
              </template>
              <template v-else-if="selectedScope === 'batch'">
                本次任务中的后续冲突自动应用相同操作
              </template>
              <template v-else-if="selectedScope === 'persistent'">
                保存为此服务器的默认冲突处理方式
              </template>
              <template v-else>
                仅处理当前文件冲突
              </template>
            </p>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
