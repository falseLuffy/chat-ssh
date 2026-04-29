<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useScriptsStore, type Script } from '../stores/scripts';
import { useServerStore } from '../stores/server';
import { useSettingsStore } from '../stores/settings';
import { useUIStore } from '../stores/ui';
import { Plus, Play, Trash2, Edit2, ShieldAlert, Cpu, Sparkles, Terminal, AlertTriangle, Key, X, Layers, MonitorPlay, ChevronDown } from 'lucide-vue-next';

const scriptsStore = useScriptsStore();
const serverStore = useServerStore();
const settingsStore = useSettingsStore();
const ui = useUIStore();

const activeScript = ref<Script | null>(null);
const isEditing = ref(false);
const showPasswordModal = ref(false);
const passwordInput = ref('');
const passwordError = ref('');
const pendingAction = ref<(() => void) | null>(null);

const editingName = ref('');
const editingDescription = ref('');
const editingContent = ref('');
const editingSkipWarning = ref(false);

const isAnalyzing = ref(false);

const executionLogs = ref<Array<{ server: string, output: string, error?: boolean }>>([]);
const isExecuting = ref(false);

// Run confirmation modal
const showRunConfirm = ref(false);
const hasPasswordLeak = computed(() => {
  return activeScript.value?.description.includes('[AI 警告: 检测到可能存在的明文密码或 Token]') || false;
});

const selectedTargetIds = ref<number[]>([]);
const isMultiExecMode = ref(false);
const isTargetListExpanded = ref(true);

const toggleTarget = (id: number) => {
  const index = selectedTargetIds.value.indexOf(id);
  if (index === -1) {
    selectedTargetIds.value.push(id);
  } else {
    selectedTargetIds.value.splice(index, 1);
  }
};

// Context Menu State
const showContextMenu = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });
const contextMenuScript = ref<Script | null>(null);

const handleContextMenu = (e: MouseEvent, script: Script) => {
  contextMenuPos.value = { x: e.clientX, y: e.clientY };
  contextMenuScript.value = script;
  showContextMenu.value = true;
  
  const closeMenu = () => {
    showContextMenu.value = false;
    window.removeEventListener('click', closeMenu);
  };
  window.addEventListener('click', closeMenu);
};

const handleContextEdit = () => {
  if (contextMenuScript.value) {
    selectScript(contextMenuScript.value, false);
    editScript();
  }
};

const handleContextSingleRun = () => {
  if (contextMenuScript.value) {
    selectScript(contextMenuScript.value, false);
  }
};

const handleContextMultiRun = () => {
  if (contextMenuScript.value) {
    selectScript(contextMenuScript.value, true);
  }
};

const handleContextDelete = () => {
  if (contextMenuScript.value) {
    deleteScript(contextMenuScript.value.id);
  }
};

const selectScript = async (script: Script, multiMode = false) => {
  if (isEditing.value) {
    if (!(await ui.showConfirm({ title: '放弃修改', message: '尚未保存修改，确定要切换吗？', type: 'warning' }))) return;
  }
  activeScript.value = script;
  isMultiExecMode.value = multiMode;
  if (multiMode) {
    isTargetListExpanded.value = true;
  }
  isEditing.value = false;
};

const createNewScript = () => {
  activeScript.value = null;
  editingName.value = '新建脚本';
  editingDescription.value = '';
  editingContent.value = '#!/bin/bash\n\n';
  editingSkipWarning.value = false;
  isEditing.value = true;
};

const editScript = () => {
  if (!activeScript.value) return;
  editingName.value = activeScript.value.name;
  editingDescription.value = activeScript.value.description;
  editingContent.value = activeScript.value.content;
  editingSkipWarning.value = activeScript.value.skip_warning;
  
  // Require password
  requirePassword(() => {
    isEditing.value = true;
  });
};

const requirePassword = (action: () => void) => {
  if (!scriptsStore.masterPassword) {
    ui.showToast('请先在系统设置中配置“安全保护密码”才能编辑或执行脚本。', 'warning');
    return;
  }
  pendingAction.value = action;
  passwordInput.value = '';
  passwordError.value = '';
  showPasswordModal.value = true;
};

const confirmPassword = () => {
  if (scriptsStore.verifyPassword(passwordInput.value)) {
    showPasswordModal.value = false;
    if (pendingAction.value) {
      pendingAction.value();
      pendingAction.value = null;
    }
  } else {
    passwordError.value = '密码错误';
  }
};

const saveScript = async () => {
  if (!editingName.value.trim()) {
    ui.showToast('脚本名称不能为空', 'error');
    return;
  }

  try {
    if (activeScript.value) {
      await scriptsStore.updateScript(activeScript.value.id, {
        name: editingName.value,
        description: editingDescription.value,
        content: editingContent.value,
        skip_warning: editingSkipWarning.value
      });
      activeScript.value = scriptsStore.scripts.find(s => s.id === activeScript.value!.id) || null;
    } else {
      const newScript = await scriptsStore.addScript({
        name: editingName.value,
        description: editingDescription.value,
        content: editingContent.value,
        skip_warning: editingSkipWarning.value
      });
      activeScript.value = newScript;
    }
    isEditing.value = false;
  } catch (e) {
    ui.showToast('保存失败: ' + e, 'error');
  }
};

const cancelEdit = () => {
  isEditing.value = false;
  activeScript.value = null;
};

const deleteScript = async (id: number) => {
  requirePassword(async () => {
    if (await ui.showConfirm({ title: '删除脚本', message: '确定要删除此脚本吗？', type: 'danger' })) {
      await scriptsStore.deleteScript(id);
      if (activeScript.value?.id === id) {
        activeScript.value = null;
      }
    }
  });
};

const analyzeScript = async () => {
  if (!settingsStore.deepseekApiKey) {
    ui.showToast('请先配置 DeepSeek API Key', 'warning');
    return;
  }

  isAnalyzing.value = true;
  try {
    const result: any = await invoke('analyze_script_with_ai', {
      scriptContent: editingContent.value,
      apiKey: settingsStore.deepseekApiKey
    });
    
    let desc = result.description;
    if (result.has_password_leak) {
      desc = '[AI 警告: 检测到可能存在的明文密码或 Token]\n' + desc;
    }
    editingDescription.value = desc;
  } catch (error) {
    ui.showToast('AI 分析失败: ' + error, 'error');
  } finally {
    isAnalyzing.value = false;
  }
};

const onlineServers = computed(() => {
  return serverStore.servers.filter(s => s.status === 'online');
});

watch([activeScript, isEditing, () => serverStore.activeServerId, isMultiExecMode], ([newScript, newEditing, newServerId, multiMode]) => {
  executionLogs.value = [];
  if (newScript || newEditing) {
    if (multiMode) {
      if (newServerId && !selectedTargetIds.value.includes(newServerId)) {
        selectedTargetIds.value.push(newServerId);
      }
    } else {
      if (newServerId) {
        selectedTargetIds.value = [newServerId];
      } else {
        selectedTargetIds.value = [];
      }
    }
  }
});

const handleRunClick = async () => {
  if (!activeScript.value) return;

  let offlineTargets: any[] = [];

  if (isMultiExecMode.value) {
    if (selectedTargetIds.value.length === 0) {
      ui.showToast('请至少选择一个执行目标', 'warning');
      return;
    }
    offlineTargets = serverStore.servers.filter(s => selectedTargetIds.value.includes(s.id) && s.status !== 'online');
  } else {
    const activeServer = serverStore.activeServer;
    if (!activeServer) {
      ui.showToast('没有选中任何服务器', 'warning');
      return;
    }
    if (activeServer.status !== 'online') {
      offlineTargets = [activeServer];
    }
  }

  if (offlineTargets.length > 0) {
    const confirm = await ui.showConfirm({
      title: '服务器未连接',
      message: isMultiExecMode.value 
        ? `有 ${offlineTargets.length} 台目标服务器尚未连接，是否立即连接并执行脚本？`
        : `目标服务器 [${offlineTargets[0].name}] 尚未连接，是否立即连接并执行脚本？`,
      type: 'info'
    });
    
    if (!confirm) return;
    
    isExecuting.value = true;
    executionLogs.value = [];
    
    let hasFailedConnection = false;
    for (const server of offlineTargets) {
      executionLogs.value.push({ server: '系统提示', output: `正在连接到 ${server.name}...` });
      const connectResult = await serverStore.connectServer(server);
      if (!connectResult.success) {
        executionLogs.value.push({ server: '系统提示', output: `[${server.name}] 连接失败: ${connectResult.error}`, error: true });
        hasFailedConnection = true;
      }
    }
    
    if (hasFailedConnection) {
      isExecuting.value = false;
      ui.showToast('部分或全部服务器连接失败，已终止执行。', 'error');
      return;
    }
    
    executionLogs.value = [];
    isExecuting.value = false;
  }

  // Auto-collapse target list before execution
  if (isMultiExecMode.value) {
    isTargetListExpanded.value = false;
  }

  if (activeScript.value.skip_warning) {
    // Direct run
    executeScript();
  } else {
    showRunConfirm.value = true;
  }
};

const executeScript = async () => {
  showRunConfirm.value = false;
  if (!activeScript.value) return;
  
  isExecuting.value = true;
  executionLogs.value = [];

  const scriptContent = activeScript.value.content;
  const targets = isMultiExecMode.value 
    ? serverStore.servers.filter(s => selectedTargetIds.value.includes(s.id))
    : serverStore.servers.filter(s => s.id === serverStore.activeServerId);

  if (targets.length === 0) {
    executionLogs.value.push({ server: '系统提示', output: '没有匹配的目标服务器执行此脚本。', error: true });
    isExecuting.value = false;
    return;
  }

  for (const server of targets) {
    try {
      const output: string = await invoke('execute_script', {
        serverName: server.name,
        scriptContent: scriptContent
      });
      executionLogs.value.push({ server: server.name, output });
    } catch (e) {
      executionLogs.value.push({ server: server.name, output: String(e), error: true });
    }
  }

  isExecuting.value = false;
};

</script>

<template>
  <div class="h-full flex text-slate-200 relative overflow-hidden bg-slate-900/50">
    
    <!-- Main Script List Area (Takes full width) -->
    <div class="flex-1 flex flex-col h-full z-10">
      <div class="p-6 border-b border-slate-800 flex justify-between items-center bg-[#1e293b]/40 shadow-sm backdrop-blur-sm z-10">
        <div>
          <h2 class="text-xl font-bold text-white flex items-center space-x-2">
            <Cpu :size="24" class="text-blue-400" />
            <span>我的脚本</span>
          </h2>
          <p class="text-xs text-slate-400 mt-1">管理并执行自动化运维脚本，提高工作效率</p>
        </div>
        <button @click="createNewScript" class="bg-blue-600 hover:bg-blue-500 text-white px-5 py-2.5 rounded-xl text-sm font-bold flex items-center space-x-2 transition-all shadow-lg shadow-blue-500/20">
          <Plus :size="18" />
          <span>新建脚本</span>
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        <div v-if="scriptsStore.scripts.length === 0" class="flex flex-col items-center justify-center h-full text-slate-500 space-y-4">
          <Cpu :size="64" class="opacity-20" />
          <p class="text-lg">暂无脚本，点击右上角新建</p>
        </div>
        
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 pb-20">
          <div v-for="script in scriptsStore.scripts" :key="script.id"
            @click="selectScript(script, false)"
            @contextmenu.prevent="handleContextMenu($event, script)"
            class="group flex flex-col bg-slate-800/40 hover:bg-slate-800/80 p-5 rounded-2xl cursor-pointer transition-all duration-300 border border-slate-700/50 hover:border-blue-500/50 hover:shadow-xl hover:shadow-blue-500/10 relative h-40">
            
            <div class="flex justify-between items-start mb-3">
              <div class="font-bold text-slate-200 text-base group-hover:text-blue-400 transition-colors line-clamp-1 pr-8">{{ script.name }}</div>
            </div>
            
            <div class="text-sm text-slate-400 line-clamp-2 flex-1 leading-relaxed">
              {{ script.description || '暂无描述' }}
            </div>

            <div class="flex items-center mt-3 space-x-2 text-xs">
              <span v-if="script.skip_warning" class="bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded-md border border-emerald-500/20">免告警</span>
              <span v-if="script.description.includes('明文密码')" class="bg-red-500/10 text-red-400 px-2 py-0.5 rounded-md border border-red-500/20 flex items-center space-x-1">
                <AlertTriangle :size="10" /> <span>敏感内容</span>
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Drawer Overlay -->
    <Transition
      enter-active-class="transition-opacity ease-out duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity ease-in duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="activeScript || isEditing" class="absolute inset-0 z-40 bg-black/50 backdrop-blur-[2px]"></div>
    </Transition>

    <!-- Right Drawer -->
    <Transition
      enter-active-class="transition-transform ease-out duration-300"
      enter-from-class="translate-x-full"
      enter-to-class="translate-x-0"
      leave-active-class="transition-transform ease-in duration-200"
      leave-from-class="translate-x-0"
      leave-to-class="translate-x-full"
    >
      <div v-if="activeScript || isEditing" class="absolute right-0 top-0 bottom-0 w-[85%] bg-[#0f172a] shadow-[-20px_0_50px_rgba(0,0,0,0.6)] z-50 flex flex-col border-l border-slate-700/50">
        
        <!-- Drawer Header -->
        <div class="h-16 border-b border-slate-800 px-8 flex items-center justify-between bg-[#1e293b]/50 backdrop-blur-md">
          <div class="flex-1 flex items-center" v-if="isEditing">
            <input v-model="editingName" placeholder="脚本名称" class="bg-transparent border-b-2 border-slate-700 text-xl font-bold text-white focus:outline-none focus:border-blue-500 py-1.5 w-80 transition-colors" />
          </div>
          <div class="flex-1 flex items-center space-x-4" v-else>
            <h2 class="text-xl font-bold text-white flex items-center space-x-2">
              <span>{{ activeScript?.name }}</span>
              <span v-if="isMultiExecMode" class="text-[10px] px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30">多机模式</span>
              <span v-else class="text-[10px] px-2 py-0.5 rounded-full bg-slate-700/50 text-slate-400 border border-slate-600/50 flex items-center space-x-1">
                <span>单机模式</span>
                <span class="w-1 h-1 bg-slate-500 rounded-full mx-1"></span>
                <span :class="serverStore.activeServer ? 'text-blue-400' : 'text-amber-500'">{{ serverStore.activeServer?.name || '未选择服务器' }}</span>
              </span>
            </h2>
            <button @click="editScript" class="text-blue-400 hover:text-white text-sm flex items-center space-x-1.5 px-3 py-1.5 bg-blue-500/10 hover:bg-blue-500/20 rounded-lg transition-colors">
              <Edit2 :size="14" /><span>编辑脚本</span>
            </button>
          </div>
          
          <div class="flex items-center space-x-4" v-if="isEditing">
             <button @click="saveScript" class="bg-blue-600 hover:bg-blue-500 text-white px-5 py-2 rounded-xl text-sm font-bold transition-all shadow-lg shadow-blue-500/20">保存脚本</button>
             <div class="w-px h-6 bg-slate-700"></div>
             <button @click="cancelEdit" class="text-slate-400 hover:text-slate-200 bg-slate-800/80 hover:bg-slate-700 p-2 rounded-xl transition-colors">
               <X :size="20" />
             </button>
          </div>
          <div class="flex items-center space-x-4" v-else>
             <button @click="handleRunClick" :disabled="isExecuting" class="bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white px-5 py-2 rounded-xl text-sm font-bold flex items-center space-x-2 transition-all shadow-lg shadow-emerald-500/20">
               <Play :size="16" />
               <span>一键运行</span>
             </button>
             <div class="w-px h-6 bg-slate-700"></div>
             <button @click="cancelEdit" class="text-slate-400 hover:text-slate-200 bg-slate-800/80 hover:bg-slate-700 p-2 rounded-xl transition-colors">
               <X :size="20" />
             </button>
          </div>
        </div>

        <!-- Content Split (Left Editor, Right Execution) -->
        <div class="flex-1 flex overflow-hidden">
          
          <!-- Left: Code & Info -->
          <div class="flex-1 flex flex-col border-r border-slate-800 p-8 overflow-y-auto space-y-6">
            
            <!-- Description -->
            <div class="space-y-2">
              <div class="flex justify-between items-center">
                <label class="text-xs font-semibold text-slate-400 uppercase tracking-widest">描述与后果</label>
                <button v-if="isEditing" @click="analyzeScript" :disabled="isAnalyzing" class="text-xs bg-indigo-500/10 text-indigo-400 px-3 py-1.5 rounded-lg flex items-center space-x-1.5 hover:bg-indigo-500/20 border border-indigo-500/20 transition-all">
                  <Sparkles :size="14" />
                  <span>{{ isAnalyzing ? 'AI 分析中...' : 'AI 智能生成描述' }}</span>
                </button>
              </div>
              
              <textarea v-if="isEditing" v-model="editingDescription" rows="3" placeholder="描述脚本的作用和执行后的后果..." class="w-full bg-slate-800/30 border border-slate-700/50 rounded-xl p-4 text-sm text-slate-300 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 resize-none font-mono leading-relaxed transition-all"></textarea>
              <div v-else class="w-full bg-slate-800/20 border border-slate-700/30 rounded-xl p-4 text-sm text-slate-400 whitespace-pre-wrap font-mono min-h-[5rem] leading-relaxed">
                {{ activeScript?.description || '暂无描述' }}
              </div>
              
              <!-- Risk Highlight -->
              <div v-if="(isEditing ? editingDescription : activeScript?.description)?.includes('明文密码')" class="bg-red-500/10 border border-red-500/30 p-4 rounded-xl flex items-start space-x-3 mt-4">
                <AlertTriangle class="text-red-500 shrink-0 mt-0.5" :size="18" />
                <div class="text-sm text-red-400 font-medium leading-relaxed">高危警告：AI 识别到脚本中可能包含明文密码或 Token 信息。请确认执行环境安全，避免敏感信息泄漏。</div>
              </div>
            </div>

            <!-- Code Editor -->
            <div class="flex-1 flex flex-col min-h-[350px]">
              <label class="text-xs font-semibold text-slate-400 uppercase tracking-widest mb-3">脚本代码</label>
              <div class="flex-1 border border-slate-700/50 rounded-xl overflow-hidden flex bg-[#0a0f1c] shadow-inner">
                <textarea v-if="isEditing" v-model="editingContent" class="w-full h-full bg-transparent p-5 text-[13px] text-blue-300 font-mono focus:outline-none resize-none leading-loose" spellcheck="false"></textarea>
                <pre v-else class="w-full h-full p-5 text-[13px] text-emerald-300 font-mono overflow-auto leading-loose">{{ activeScript?.content }}</pre>
              </div>
            </div>

            <!-- Options -->
            <div v-if="isEditing" class="flex items-center space-x-3 pt-2">
              <input type="checkbox" id="skipWarning" v-model="editingSkipWarning" class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-blue-500 focus:ring-blue-500/20 focus:ring-offset-slate-900" />
              <label for="skipWarning" class="text-sm text-slate-400 cursor-pointer select-none hover:text-slate-300 transition-colors">我熟悉该脚本，执行前跳过高危告警提示</label>
            </div>
          </div>

          <!-- Right: Execution & Targets -->
          <div class="w-[420px] flex flex-col bg-[#0f172a]/80 relative border-l border-slate-800/50">
            <div v-if="isMultiExecMode" class="border-b border-slate-800/50 bg-[#1e293b]/30">
              <button @click="isTargetListExpanded = !isTargetListExpanded" class="w-full p-4 flex justify-between items-center hover:bg-slate-800/50 transition-colors focus:outline-none">
                <div class="flex items-center space-x-2 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  <Layers :size="14" /> <span>执行目标选择 (已选 {{ selectedTargetIds.length }} 台)</span>
                </div>
                <ChevronDown :size="16" class="text-slate-500 transition-transform duration-300" :class="{'rotate-180': isTargetListExpanded}" />
              </button>
              
              <div v-show="isTargetListExpanded" class="px-6 pb-6 pt-2">
                <div class="space-y-2.5 max-h-48 overflow-y-auto pr-2 custom-scrollbar">
                  <div v-if="serverStore.servers.length === 0" class="text-sm text-amber-500/70 italic p-3 bg-amber-500/5 rounded-lg border border-amber-500/10">
                    您尚未添加任何服务器。
                  </div>
                  <div v-for="server in serverStore.servers" :key="server.id" class="flex items-center space-x-3 text-sm bg-slate-800/50 p-2.5 rounded-xl border border-slate-700/50 transition-colors hover:border-blue-500/30 cursor-pointer" @click="toggleTarget(server.id)">
                    <input type="checkbox" :value="server.id" v-model="selectedTargetIds" class="w-4 h-4 rounded bg-slate-800 border-slate-700 text-blue-500 focus:ring-blue-500/20 focus:ring-offset-slate-900 cursor-pointer" @click.stop />
                    <span class="text-slate-300 font-medium">{{ server.name }}</span>
                    <div class="flex-1"></div>
                    <div class="w-2 h-2 rounded-full" 
                         :class="server.status === 'online' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]' : 'bg-slate-600 shadow-none'"></div>
                  </div>
                </div>
              </div>
            </div>

            <div class="flex-1 overflow-y-auto p-6 flex flex-col relative">
              <div class="flex items-center justify-between mb-4">
                <h4 class="text-xs font-bold text-slate-400 uppercase tracking-widest flex items-center space-x-2">
                  <span>执行结果</span>
                  <span v-if="!isMultiExecMode" class="px-2 py-0.5 bg-slate-800 rounded-md text-[10px] normal-case text-slate-500 border border-slate-700/50 flex items-center space-x-1">
                    <span>目标:</span>
                    <span :class="serverStore.activeServer ? 'text-emerald-400' : 'text-amber-500/70'">{{ serverStore.activeServer?.name || '无目标' }}</span>
                  </span>
                </h4>
              </div>
              
              <div v-if="isExecuting" class="flex items-center justify-center space-x-3 text-blue-400 text-sm animate-pulse my-8 bg-blue-500/5 py-4 rounded-xl border border-blue-500/10">
                <Cpu class="animate-spin" :size="20" />
                <span class="font-medium">正在下发并执行...</span>
              </div>
              
              <div v-if="executionLogs.length === 0 && !isExecuting" class="flex-1 flex flex-col items-center justify-center text-slate-600 space-y-3 opacity-50">
                <Terminal :size="32" />
                <p class="text-sm">尚未执行</p>
              </div>

              <div class="space-y-4">
                <div v-for="(log, idx) in executionLogs" :key="idx" class="border border-slate-700/50 rounded-xl overflow-hidden shadow-sm">
                  <div :class="['px-4 py-2.5 text-[13px] font-bold border-b border-slate-700/50 flex justify-between items-center', log.error ? 'bg-red-500/10 text-red-400' : 'bg-slate-800/80 text-slate-300']">
                    <span>[ {{ log.server }} ]</span>
                    <span :class="log.error ? 'text-red-500' : 'text-emerald-400'">{{ log.error ? '执行失败' : '执行完成' }}</span>
                  </div>
                  <pre class="p-4 text-[12px] font-mono text-slate-400 whitespace-pre-wrap bg-[#0a0f1c] overflow-x-auto max-h-80 leading-relaxed">{{ log.output || '(无输出)' }}</pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Password Modal -->
    <Teleport to="body">
      <div v-if="showPasswordModal" class="fixed inset-0 z-[110] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in">
        <div class="bg-[#1e293b] border border-slate-700 w-full max-w-sm rounded-2xl shadow-2xl overflow-hidden p-6 animate-in zoom-in-95">
          <div class="flex items-center justify-center w-12 h-12 rounded-full bg-rose-500/10 mx-auto mb-4">
            <Key class="text-rose-500" :size="24" />
          </div>
          <h3 class="text-lg font-bold text-center text-white mb-2">安全验证</h3>
          <p class="text-xs text-slate-400 text-center mb-6">请输入“安全保护密码”以继续操作。</p>
          
          <input 
            v-model="passwordInput" 
            type="password" 
            placeholder="输入安全密码..." 
            class="w-full bg-slate-800 border border-slate-700 rounded-lg py-2.5 px-4 mb-2 focus:outline-none focus:border-rose-500 text-sm text-slate-200"
            @keyup.enter="confirmPassword"
          />
          <p v-if="passwordError" class="text-xs text-rose-500 text-center mb-4">{{ passwordError }}</p>
          
          <div class="flex space-x-3 mt-6">
            <button @click="showPasswordModal = false" class="flex-1 py-2 rounded-lg text-slate-400 hover:bg-slate-800 transition-colors text-sm">取消</button>
            <button @click="confirmPassword" class="flex-1 py-2 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-bold transition-all shadow-lg shadow-rose-600/20 text-sm">验证</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Run Confirmation Modal -->
    <Teleport to="body">
      <div v-if="showRunConfirm" class="fixed inset-0 z-[110] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in">
        <div class="bg-[#1e293b] border border-slate-700 w-full max-w-lg rounded-2xl shadow-2xl overflow-hidden p-6 animate-in zoom-in-95">
          <div class="flex items-center space-x-3 mb-6">
            <div :class="['w-10 h-10 rounded-full flex items-center justify-center', hasPasswordLeak ? 'bg-red-500/20 text-red-500' : 'bg-amber-500/20 text-amber-500']">
              <ShieldAlert v-if="hasPasswordLeak" :size="20" />
              <AlertTriangle v-else :size="20" />
            </div>
            <div>
              <h3 class="text-lg font-bold text-white">执行确认</h3>
              <p class="text-xs text-slate-400">请确认即将执行的脚本后果</p>
            </div>
          </div>
          
          <div class="bg-slate-800/50 border border-slate-700 rounded-lg p-4 mb-6">
            <h4 class="text-xs font-bold text-slate-500 uppercase mb-2">脚本用途及后果描述</h4>
            <div class="text-sm text-slate-300 whitespace-pre-wrap font-mono leading-relaxed">
              {{ activeScript?.description || '暂无描述' }}
            </div>
          </div>
          
          <div class="flex justify-end space-x-3">
            <button @click="showRunConfirm = false" class="px-4 py-2 rounded-lg text-slate-400 hover:bg-slate-800 transition-colors text-sm">取消</button>
            <button @click="executeScript" :class="['px-6 py-2 rounded-lg text-white font-bold transition-all shadow-lg text-sm flex items-center space-x-2', hasPasswordLeak ? 'bg-red-600 hover:bg-red-500 shadow-red-600/20' : 'bg-amber-600 hover:bg-amber-500 shadow-amber-600/20']">
              <Play :size="16" />
              <span>确认执行</span>
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="showContextMenu" 
        class="fixed z-[100] w-48 bg-[#1e293b]/95 backdrop-blur-xl border border-slate-700 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
        <div class="px-3 py-2 border-b border-slate-800/50 mb-1">
          <div class="text-[10px] font-bold text-slate-500 uppercase tracking-widest line-clamp-1">{{ contextMenuScript?.name }}</div>
        </div>
        <button @click="handleContextSingleRun" class="w-full text-left px-3 py-2 text-xs hover:bg-emerald-500/10 text-slate-300 hover:text-emerald-400 transition-all flex items-center space-x-2 group/item">
          <MonitorPlay :size="14" class="group-hover/item:scale-110 transition-transform" /> 
          <span class="group-hover/item:translate-x-1 transition-transform">单机执行</span>
        </button>
        <button @click="handleContextMultiRun" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-500/10 text-slate-300 hover:text-blue-400 transition-all flex items-center space-x-2 group/item">
          <Layers :size="14" class="group-hover/item:scale-110 transition-transform" /> 
          <span class="group-hover/item:translate-x-1 transition-transform">多机执行</span>
        </button>
        
        <div class="h-px bg-slate-700/50 my-1"></div>
        
        <button @click="handleContextEdit" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
          <Edit2 :size="14" class="group-hover/item:scale-110 transition-transform" /> 
          <span class="group-hover/item:translate-x-1 transition-transform">编辑脚本</span>
        </button>
        
        <div class="h-px bg-slate-700/50 my-1"></div>
        
        <button @click="handleContextDelete" class="w-full text-left px-3 py-2 text-xs hover:bg-red-600/90 text-red-500 hover:text-white transition-all flex items-center space-x-2 group/item">
          <Trash2 :size="14" class="group-hover/item:scale-110 transition-transform" /> 
          <span class="group-hover/item:translate-x-1 transition-transform">删除脚本</span>
        </button>
      </div>
    </Teleport>

  </div>
</template>
