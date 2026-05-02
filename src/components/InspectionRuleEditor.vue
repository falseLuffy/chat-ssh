<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Server } from '../stores/server';
import { loadRules as dbLoadRules, createRule, updateRule, deleteRule } from '../utils/db';
import {
  X, Plus, Trash2, ToggleLeft, ToggleRight,
  Cpu, MemoryStick, HardDrive, Box, Settings, Activity, Wifi, ShieldCheck,
  AlertTriangle, Info
} from 'lucide-vue-next';

const props = defineProps<{ server: Server }>();
const emit = defineEmits(['close']);

interface Rule {
  id?: number;
  name: string;
  category: string;
  check_type: string;
  config: any;
  enabled: boolean;
  server_id?: number | null;
  sort_order: number;
}

const rules = ref<Rule[]>([]);
const isLoading = ref(false);
const editingRule = ref<Rule | null>(null);
const showEditor = ref(false);
const isNewRule = ref(false);

const categories = [
  { id: 'cpu', label: 'CPU', icon: Cpu },
  { id: 'memory', label: '内存', icon: MemoryStick },
  { id: 'disk', label: '磁盘', icon: HardDrive },
  { id: 'docker', label: 'Docker', icon: Box },
  { id: 'service', label: '服务', icon: Settings },
  { id: 'process', label: '进程', icon: Activity },
  { id: 'network', label: '网络', icon: Wifi },
  { id: 'custom', label: '自定义', icon: ShieldCheck },
];

const checkTypes = [
  { id: 'threshold', label: '阈值检查' },
  { id: 'service_status', label: '服务状态' },
  { id: 'docker_status', label: 'Docker状态' },
  { id: 'command_output', label: '命令输出' },
];

const defaultConfigs: Record<string, any> = {
  threshold: { command: '', unit: 'percent', threshold_type: 'value', warning: 80, critical: 95 },
  service_status: { services: [] },
  docker_status: { command: 'docker ps -a --format \'{{json .}}\'' },
  command_output: { command: '', fail_if_output: false, fail_if_zero: false },
};

const fetchRules = async () => {
  isLoading.value = true;
  try {
    rules.value = await dbLoadRules(props.server.id);
  } catch (e) {
    console.error('Failed to load rules:', e);
  } finally {
    isLoading.value = false;
  }
};

const toggleRule = async (rule: Rule) => {
  try {
    if (rule.id != null) {
      await updateRule(rule.id, { ...rule, enabled: !rule.enabled });
      rule.enabled = !rule.enabled;
    }
  } catch (e) {
    console.error('Failed to toggle rule:', e);
  }
};

const openAddRule = () => {
  isNewRule.value = true;
  editingRule.value = {
    name: '',
    category: 'custom',
    check_type: 'command_output',
    config: { ...defaultConfigs.command_output },
    enabled: true,
    sort_order: rules.value.length + 1,
  };
  showEditor.value = true;
};

const openEditRule = (rule: Rule) => {
  isNewRule.value = false;
  editingRule.value = JSON.parse(JSON.stringify(rule));
  showEditor.value = true;
};

const closeEditor = () => {
  showEditor.value = false;
  editingRule.value = null;
};

const saveRule = async () => {
  if (!editingRule.value) return;
  const rule = editingRule.value;

  // Rebuild config based on check_type
  if (isNewRule.value) {
    rule.server_id = props.server.id;
    try {
      rule.config = typeof rule.config === 'string' ? rule.config : JSON.stringify(rule.config);
      await createRule(rule);
      await fetchRules();
      closeEditor();
    } catch (e) {
      console.error('Failed to create rule:', e);
    }
  } else {
    try {
      rule.config = typeof rule.config === 'string' ? rule.config : JSON.stringify(rule.config);
      await updateRule(rule.id!, rule);
      await fetchRules();
      closeEditor();
    } catch (e) {
      console.error('Failed to update rule:', e);
    }
  }
};

const removeRule = async (id: number) => {
  try {
    await deleteRule(id);
    await fetchRules();
  } catch (e) {
    console.error('Failed to delete rule:', e);
  }
};

const getCategoryLabel = (cat: string) => {
  return categories.find(c => c.id === cat)?.label || cat;
};

const getCategoryIcon = (cat: string) => {
  const c = categories.find(c => c.id === cat);
  return c?.icon || ShieldCheck;
};

onMounted(() => {
  fetchRules();
});
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
    <div class="bg-[#1e293b] border border-slate-700 w-full max-w-xl rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[80vh]">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <div class="p-2 bg-blue-500/20 rounded-xl text-blue-400">
            <ShieldCheck :size="20" />
          </div>
          <h3 class="text-sm font-bold text-white">巡检规则管理</h3>
        </div>
        <button @click="emit('close')" class="p-2 hover:bg-slate-800 rounded-xl text-slate-500 transition-colors">
          <X :size="20" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-6 custom-scrollbar">
        <div v-if="isLoading" class="flex items-center justify-center py-12">
          <div class="animate-spin w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full"></div>
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="rule in rules"
            :key="rule.id"
            class="flex items-center px-4 py-3 rounded-xl border border-slate-800/50 bg-[#1e293b]/20"
          >
            <!-- Toggle -->
            <button @click="toggleRule(rule)" class="mr-3 text-slate-500 hover:text-slate-300 transition-colors">
              <ToggleRight v-if="rule.enabled" :size="20" class="text-blue-400" />
              <ToggleLeft v-else :size="20" />
            </button>

            <!-- Icon -->
            <div class="w-8 h-8 rounded-lg flex items-center justify-center mr-3 bg-slate-800/50 text-slate-500">
              <component :is="getCategoryIcon(rule.category)" :size="14" />
            </div>

            <!-- Info -->
            <div class="flex-1 min-w-0">
              <div class="text-sm text-slate-200" :class="{ 'text-slate-500': !rule.enabled }">
                {{ rule.name }}
              </div>
              <div class="text-[10px] text-slate-500">
                {{ getCategoryLabel(rule.category) }} · {{ rule.check_type }}
                <span v-if="rule.server_id != null" class="text-blue-400 ml-1">(服务器专属)</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex items-center space-x-1">
              <button @click="openEditRule(rule)" class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-500 hover:text-slate-200 transition-colors text-xs">
                编辑
              </button>
              <button @click="removeRule(rule.id!)" class="p-1.5 hover:bg-red-500/20 rounded-lg text-slate-500 hover:text-red-400 transition-colors">
                <Trash2 :size="14" />
              </button>
            </div>
          </div>

          <button
            @click="openAddRule"
            class="w-full flex items-center justify-center space-x-2 px-4 py-3 border-2 border-dashed border-slate-700/50 rounded-xl text-sm text-slate-400 hover:text-slate-200 hover:border-slate-600 transition-colors"
          >
            <Plus :size="16" />
            <span>添加自定义规则</span>
          </button>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end">
        <button @click="emit('close')" class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-bold rounded-xl transition-all">
          关闭
        </button>
      </div>
    </div>

    <!-- Rule Editor Modal (nested) -->
    <div v-if="showEditor && editingRule" class="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
      <div class="bg-[#1e293b] border border-slate-700 w-full max-w-lg rounded-3xl shadow-2xl overflow-hidden">
        <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
          <h3 class="text-sm font-bold text-white">{{ isNewRule ? '添加规则' : '编辑规则' }}</h3>
          <button @click="closeEditor" class="p-2 hover:bg-slate-800 rounded-xl text-slate-500">
            <X :size="18" />
          </button>
        </div>

        <div class="p-6 space-y-4">
          <div>
            <label class="text-xs text-slate-500 mb-1 block">规则名称</label>
            <input
              v-model="editingRule.name"
              class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-blue-500/50"
              placeholder="例如: MySQL端口检测"
            />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="text-xs text-slate-500 mb-1 block">分类</label>
              <select
                v-model="editingRule.category"
                class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200"
              >
                <option v-for="cat in categories" :key="cat.id" :value="cat.id">{{ cat.label }}</option>
              </select>
            </div>
            <div>
              <label class="text-xs text-slate-500 mb-1 block">检查类型</label>
              <select
                v-model="editingRule.check_type"
                class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200"
                @change="editingRule.config = { ...defaultConfigs[editingRule.check_type] || { command: '' } }"
              >
                <option v-for="ct in checkTypes" :key="ct.id" :value="ct.id">{{ ct.label }}</option>
              </select>
            </div>
          </div>

          <!-- Threshold config -->
          <div v-if="editingRule.check_type === 'threshold'" class="space-y-3">
            <div>
              <label class="text-xs text-slate-500 mb-1 block">命令 (需返回数字)</label>
              <input v-model="editingRule.config.command" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200 font-mono" placeholder="例如: free | grep Mem | awk 计算内存百分比" />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="text-xs text-slate-500 mb-1 block">警告阈值</label>
                <input v-model.number="editingRule.config.warning" type="number" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200" />
              </div>
              <div>
                <label class="text-xs text-slate-500 mb-1 block">严重阈值</label>
                <input v-model.number="editingRule.config.critical" type="number" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200" />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="text-xs text-slate-500 mb-1 block">阈值类型</label>
                <select v-model="editingRule.config.threshold_type" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200">
                  <option value="value">直接比较</option>
                  <option value="ratio_to_cores">按核心数均分</option>
                </select>
              </div>
              <div>
                <label class="text-xs text-slate-500 mb-1 block">单位</label>
                <select v-model="editingRule.config.unit" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200">
                  <option value="percent">百分比</option>
                  <option value="loadavg">负载值</option>
                  <option value="count">计数</option>
                </select>
              </div>
            </div>
          </div>

          <!-- Service status config -->
          <div v-if="editingRule.check_type === 'service_status'" class="space-y-3">
            <label class="text-xs text-slate-500 mb-1 block">服务列表 (逗号分隔)</label>
            <input
              :value="(editingRule.config.services || []).join(', ')"
              @input="editingRule.config.services = ($event.target as HTMLInputElement).value.split(',').map((s: string) => s.trim()).filter(Boolean)"
              class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200 font-mono"
              placeholder="nginx, mysql, docker, sshd"
            />
            <p class="text-xs text-slate-500">输入服务名称，用逗号分隔</p>
          </div>

          <!-- Command output config -->
          <div v-if="editingRule.check_type === 'command_output'" class="space-y-3">
            <div>
              <label class="text-xs text-slate-500 mb-1 block">命令</label>
              <textarea
                v-model="editingRule.config.command"
                rows="2"
                class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200 font-mono focus:outline-none focus:border-blue-500/50"
                placeholder="输入需要执行的命令"
              ></textarea>
            </div>
            <div class="flex items-center space-x-4">
              <label class="flex items-center space-x-2 text-xs text-slate-400">
                <input type="checkbox" v-model="editingRule.config.fail_if_output" class="rounded bg-slate-800 border-slate-700" />
                <span>有输出则警告</span>
              </label>
              <label class="flex items-center space-x-2 text-xs text-slate-400">
                <input type="checkbox" v-model="editingRule.config.fail_if_zero" class="rounded bg-slate-800 border-slate-700" />
                <span>结果为0则严重</span>
              </label>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="text-xs text-slate-500 mb-1 block">警告阈值 (可选)</label>
                <input v-model.number="editingRule.config.warning_threshold" type="number" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200" />
              </div>
              <div>
                <label class="text-xs text-slate-500 mb-1 block">严重阈值 (可选)</label>
                <input v-model.number="editingRule.config.critical_threshold" type="number" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200" />
              </div>
            </div>
          </div>
        </div>

        <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end space-x-3">
          <button @click="closeEditor" class="px-4 py-2 text-sm text-slate-400 hover:text-slate-200 transition-colors">取消</button>
          <button
            @click="saveRule"
            :disabled="!editingRule.name"
            class="px-6 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-500 text-white text-sm font-bold rounded-xl transition-all"
          >
            保存
          </button>
        </div>
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
</style>
