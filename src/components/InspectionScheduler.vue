<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useServerStore } from '../stores/server';
import { loadSchedules as dbLoadSchedules, createSchedule, updateSchedule, deleteSchedule } from '../utils/db';
import {
  X, Plus, Trash2, ToggleLeft, ToggleRight,
  Clock, Calendar, RefreshCw
} from 'lucide-vue-next';

const emit = defineEmits(['close']);
const serverStore = useServerStore();

interface Schedule {
  id?: number;
  name: string;
  server_ids: string;
  cron_expression: string;
  enabled: boolean;
  last_run_at?: string | null;
  next_run_at?: string | null;
  created_at?: string | null;
}

const schedules = ref<Schedule[]>([]);
const isLoading = ref(false);
const showForm = ref(false);
const editingSchedule = ref<Schedule | null>(null);
const isNewSchedule = ref(false);

const cronPresets = [
  { label: '每小时', expr: '0 * * * *' },
  { label: '每6小时', expr: '0 */6 * * *' },
  { label: '每12小时', expr: '0 */12 * * *' },
  { label: '每天 (08:00)', expr: '0 8 * * *' },
  { label: '每周一 08:00', expr: '0 8 * * 1' },
];

const fetchSchedules = async () => {
  isLoading.value = true;
  try {
    schedules.value = await dbLoadSchedules();
  } catch (e) {
    console.error('Failed to load schedules:', e);
  } finally {
    isLoading.value = false;
  }
};

const toggleSchedule = async (sched: Schedule) => {
  try {
    if (sched.id != null) {
      await updateSchedule(sched.id, { ...sched, enabled: !sched.enabled });
      sched.enabled = !sched.enabled;
    }
  } catch (e) {
    console.error('Failed to toggle schedule:', e);
  }
};

const openAdd = () => {
  isNewSchedule.value = true;
  editingSchedule.value = {
    name: '',
    server_ids: '[]',
    cron_expression: '0 8 * * *',
    enabled: true,
  };
  showForm.value = true;
};

const openEdit = (sched: Schedule) => {
  isNewSchedule.value = false;
  editingSchedule.value = JSON.parse(JSON.stringify(sched));
  showForm.value = true;
};

const closeForm = () => {
  showForm.value = false;
  editingSchedule.value = null;
};

const saveSchedule = async () => {
  if (!editingSchedule.value) return;

  if (isNewSchedule.value) {
    try {
      await createSchedule(editingSchedule.value);
      await fetchSchedules();
      closeForm();
    } catch (e) {
      console.error('Failed to create schedule:', e);
    }
  } else {
    try {
      await updateSchedule(editingSchedule.value.id!, editingSchedule.value);
      await fetchSchedules();
      closeForm();
    } catch (e) {
      console.error('Failed to update schedule:', e);
    }
  }
};

const deleteScheduleItem = async (id: number) => {
  try {
    await deleteSchedule(id);
    await fetchSchedules();
  } catch (e) {
    console.error('Failed to delete schedule:', e);
  }
};

const formatTime = (str?: string | null) => {
  if (!str) return '—';
  try {
    const d = new Date(str.replace(' ', 'T') + 'Z');
    return d.toLocaleString('zh-CN', {
      month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
    });
  } catch {
    return str;
  }
};

const getServerName = (id: number) => {
  return serverStore.servers.find(s => s.id === id)?.name || `服务器 #${id}`;
};

const parseServerIds = (idsStr: string): number[] => {
  try {
    return JSON.parse(idsStr);
  } catch {
    return [];
  }
};

const setCronPreset = (expr: string) => {
  if (editingSchedule.value) {
    editingSchedule.value.cron_expression = expr;
  }
};

onMounted(() => {
  fetchSchedules();
});
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
    <div class="bg-[#1e293b] border border-slate-700 w-full max-w-lg rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[80vh]">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <div class="p-2 bg-blue-500/20 rounded-xl text-blue-400">
            <Clock :size="20" />
          </div>
          <h3 class="text-sm font-bold text-white">定时巡检设置</h3>
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

        <div v-else-if="schedules.length === 0" class="flex flex-col items-center justify-center py-12 space-y-3">
          <Clock :size="40" class="text-slate-700" />
          <p class="text-sm text-slate-500">暂无定时任务</p>
          <p class="text-xs text-slate-600">创建定时任务可自动执行巡检</p>
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="sched in schedules"
            :key="sched.id"
            class="rounded-xl border border-slate-800/50 bg-[#1e293b]/20 p-4"
          >
            <div class="flex items-start justify-between">
              <div class="flex items-center space-x-2">
                <button @click="toggleSchedule(sched)" class="text-slate-500 hover:text-slate-300">
                  <ToggleRight v-if="sched.enabled" :size="20" class="text-blue-400" />
                  <ToggleLeft v-else :size="20" />
                </button>
                <div>
                  <div class="text-sm text-slate-200" :class="{ 'text-slate-500': !sched.enabled }">
                    {{ sched.name }}
                  </div>
                  <div class="text-[10px] text-slate-500 font-mono mt-0.5">{{ sched.cron_expression }}</div>
                </div>
              </div>
              <div class="flex items-center space-x-1">
                <button @click="openEdit(sched)" class="px-2 py-1 text-xs text-slate-500 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors">编辑</button>
                <button @click="deleteScheduleItem(sched.id!)" class="p-1 hover:bg-red-500/20 rounded-lg text-slate-500 hover:text-red-400">
                  <Trash2 :size="14" />
                </button>
              </div>
            </div>
            <div class="mt-2 flex flex-wrap gap-1">
              <span
                v-for="sid in parseServerIds(sched.server_ids)"
                :key="sid"
                class="text-[10px] px-2 py-0.5 bg-slate-800/50 text-slate-400 rounded-full"
              >
                {{ getServerName(sid) }}
              </span>
            </div>
            <div class="mt-2 flex items-center space-x-4 text-[10px] text-slate-600">
              <span v-if="sched.last_run_at">上次: {{ formatTime(sched.last_run_at) }}</span>
              <span v-if="sched.next_run_at">下次: {{ formatTime(sched.next_run_at) }}</span>
            </div>
          </div>
        </div>

        <button
          @click="openAdd"
          class="w-full mt-4 flex items-center justify-center space-x-2 px-4 py-3 border-2 border-dashed border-slate-700/50 rounded-xl text-sm text-slate-400 hover:text-slate-200 hover:border-slate-600 transition-colors"
        >
          <Plus :size="16" />
          <span>添加定时任务</span>
        </button>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end">
        <button @click="emit('close')" class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-bold rounded-xl transition-all">
          关闭
        </button>
      </div>
    </div>

    <!-- Schedule Form Modal (nested) -->
    <div v-if="showForm && editingSchedule" class="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
      <div class="bg-[#1e293b] border border-slate-700 w-full max-w-md rounded-3xl shadow-2xl overflow-hidden">
        <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
          <h3 class="text-sm font-bold text-white">{{ isNewSchedule ? '添加定时任务' : '编辑定时任务' }}</h3>
          <button @click="closeForm" class="p-2 hover:bg-slate-800 rounded-xl text-slate-500">
            <X :size="18" />
          </button>
        </div>

        <div class="p-6 space-y-4">
          <div>
            <label class="text-xs text-slate-500 mb-1 block">任务名称</label>
            <input v-model="editingSchedule.name" class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200" placeholder="例如: 每日巡检" />
          </div>

          <div>
            <label class="text-xs text-slate-500 mb-1 block">目标服务器</label>
            <div class="flex flex-wrap gap-2 mb-2">
              <button
                v-for="server in serverStore.servers"
                :key="server.id"
                @click="() => {
                  const ids = parseServerIds(editingSchedule.server_ids);
                  const idx = ids.indexOf(server.id);
                  if (idx >= 0) ids.splice(idx, 1);
                  else ids.push(server.id);
                  editingSchedule.server_ids = JSON.stringify(ids);
                }"
                class="text-xs px-3 py-1.5 rounded-lg border transition-colors"
                :class="parseServerIds(editingSchedule.server_ids).includes(server.id)
                  ? 'bg-blue-500/10 border-blue-500/30 text-blue-400'
                  : 'bg-slate-800/50 border-slate-700/50 text-slate-400 hover:text-slate-200'"
              >
                {{ server.name }}
              </button>
            </div>
          </div>

          <div>
            <label class="text-xs text-slate-500 mb-1 block">Cron 表达式</label>
            <input
              v-model="editingSchedule.cron_expression"
              class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200 font-mono"
              placeholder="0 8 * * *"
            />
            <div class="flex flex-wrap gap-1 mt-2">
              <button
                v-for="preset in cronPresets"
                :key="preset.expr"
                @click="setCronPreset(preset.expr)"
                class="text-[10px] px-2 py-1 rounded-md transition-colors"
                :class="editingSchedule.cron_expression === preset.expr
                  ? 'bg-blue-500/20 text-blue-400'
                  : 'bg-slate-800/50 text-slate-500 hover:text-slate-300'"
              >
                {{ preset.label }}
              </button>
            </div>
          </div>
        </div>

        <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end space-x-3">
          <button @click="closeForm" class="px-4 py-2 text-sm text-slate-400 hover:text-slate-200 transition-colors">取消</button>
          <button @click="saveSchedule" :disabled="!editingSchedule.name" class="px-6 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-500 text-white text-sm font-bold rounded-xl transition-all">保存</button>
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
