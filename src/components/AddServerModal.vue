<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { X, Save, Server as ServerIcon, User, Lock, Globe } from 'lucide-vue-next';
import { useServerStore, type Server } from '../stores/server';

const props = defineProps<{
  editServer?: Server | null;
}>();

const emit = defineEmits(['close']);
const serverStore = useServerStore();

const name = ref('');
const host = ref('');
const username = ref('');
const password = ref('');

onMounted(() => {
  if (props.editServer) {
    name.value = props.editServer.name;
    host.value = props.editServer.host;
    username.value = props.editServer.username;
  }
});

const handleSave = () => {
  if (!name.value || !host.value || !username.value) return;
  
  if (props.editServer) {
    serverStore.updateServer(props.editServer.id, {
      name: name.value,
      host: host.value,
      username: username.value,
    });
  } else {
    serverStore.addServer({
      name: name.value,
      host: host.value,
      username: username.value,
    });
  }
  
  emit('close');
};
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
      <div class="bg-[#1e293b] border border-slate-700 w-full max-w-md rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200">
        <!-- Modal Header -->
        <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-900/50">
          <h3 class="text-lg font-bold text-white flex items-center space-x-2">
            <ServerIcon :size="20" class="text-blue-500" />
            <span>{{ props.editServer ? '编辑服务器配置' : '添加新服务器' }}</span>
          </h3>
          <button @click="$emit('close')" class="p-1 hover:bg-slate-800 rounded-full text-slate-400 transition-colors">
            <X :size="20" />
          </button>
        </div>

        <!-- Modal Body -->
        <div class="p-6 space-y-4">
          <div class="space-y-1">
            <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider">服务器名称</label>
            <div class="relative">
              <Globe class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
              <input v-model="name" type="text" placeholder="例如：我的阿里云" class="w-full bg-slate-800 border border-slate-700 rounded-lg py-2 pl-10 pr-4 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-sm text-slate-200" />
            </div>
          </div>

          <div class="space-y-1">
            <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider">主机地址 (IP/Host)</label>
            <div class="relative">
              <ServerIcon class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
              <input v-model="host" type="text" placeholder="1.2.3.4" class="w-full bg-slate-800 border border-slate-700 rounded-lg py-2 pl-10 pr-4 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-sm text-slate-200" />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-1">
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider">用户名</label>
              <div class="relative">
                <User class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
                <input v-model="username" type="text" placeholder="root" class="w-full bg-slate-800 border border-slate-700 rounded-lg py-2 pl-10 pr-4 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-sm text-slate-200" />
              </div>
            </div>
            <div class="space-y-1">
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider">密码</label>
              <div class="relative">
                <Lock class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" :size="16" />
                <input v-model="password" type="password" placeholder="••••••••" class="w-full bg-slate-800 border border-slate-700 rounded-lg py-2 pl-10 pr-4 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all text-sm text-slate-200" />
              </div>
            </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-6 py-4 bg-slate-900/50 border-t border-slate-800 flex justify-end space-x-3">
          <button @click="$emit('close')" class="px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors">
            取消
          </button>
          <button @click="handleSave" class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-bold flex items-center space-x-2 shadow-lg shadow-blue-600/20 transition-all">
            <Save :size="16" />
            <span>{{ props.editServer ? '保存修改' : '保存服务器' }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
