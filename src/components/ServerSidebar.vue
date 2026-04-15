<script setup lang="ts">
  import { ref } from 'vue';
  import { invoke } from '@tauri-apps/api/core';
  import { Plus, Server as ServerIcon, Globe, MoreVertical } from 'lucide-vue-next';
  import { useServerStore, type Server } from '../stores/server';
  import AddServerModal from './AddServerModal.vue';

  const serverStore = useServerStore();
  const showAddModal = ref(false);
  const editingServer = ref<Server | null>(null);
  const activeMenuId = ref<number | null>(null);

  const selectServer = (id: number) => {
    serverStore.setActiveServer(id);
  };

  const handleDblClick = async (server: Server) => {
    try {
      serverStore.updateStatus(server.id, 'connecting');

      // Connect to SSH
      await invoke('connect_ssh', {
        name: server.name,
        host: server.host,
        user: server.username,
        pass: 'root', // In a real app, this should come from a secure input/prompt
      });

      // Open terminal session
      await invoke('open_terminal', { 
        serverName: server.name,
        cols: 80,
        rows: 24
      });

      serverStore.updateStatus(server.id, 'online');
      serverStore.setActiveServer(server.id);

      // Inform parent to switch tab (this usually happens via a shared state/event)
      // For now we'll assume the user switches to terminal
    } catch (error) {
      console.error('Connection failed:', error);
      serverStore.updateStatus(server.id, 'offline');
    }
  }
  const openAddModal = () => {
    editingServer.value = null;
    showAddModal.value = true;
  };

  const openEditModal = (event: Event, server: Server) => {
    event.stopPropagation();
    editingServer.value = server;
    showAddModal.value = true;
    activeMenuId.value = null;
  };

  const toggleMenu = (event: Event, id: number) => {
    event.stopPropagation();
    activeMenuId.value = activeMenuId.value === id ? null : id;
  };

  const deleteServer = (event: Event, id: number) => {
    event.stopPropagation();
    serverStore.servers = serverStore.servers.filter(s => s.id !== id);
    if (serverStore.activeServerId === id) {
      serverStore.activeServerId = null;
      serverStore.activeServer = null;
    }
    activeMenuId.value = null;
  };
</script>

<template>
  <aside class="flex flex-col h-full relative" @click="activeMenuId = null">
    <!-- Sidebar Header -->
    <div class="h-12 flex items-center justify-between px-4 border-b border-slate-800">
      <h2 class="text-xs font-bold uppercase tracking-widest text-slate-500">我的服务器</h2>
      <button @click.stop="openAddModal" class="p-1 hover:bg-slate-800 rounded-md text-blue-400 transition-colors">
        <Plus :size="16" />
      </button>
    </div>

    <!-- Add/Edit Server Modal Overlay -->
    <AddServerModal v-if="showAddModal" :editServer="editingServer" @close="showAddModal = false" />

    <!-- Server List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      <div v-for="server in serverStore.servers" :key="server.id" @click="selectServer(server.id)"
        @dblclick="handleDblClick(server)" :class="[
          'group flex items-center px-3 py-2 rounded-xl cursor-pointer transition-all duration-200 border relative',
          serverStore.activeServerId === server.id
            ? 'bg-blue-600/10 border-blue-500/30 text-blue-400 shadow-lg shadow-blue-500/5'
            : 'border-transparent hover:bg-slate-800/50 text-slate-400 hover:text-slate-200'
        ]">
        <div
          :class="['p-1.5 rounded-lg bg-slate-800 mr-3 transition-colors', serverStore.activeServerId === server.id ? 'text-blue-400 bg-blue-500/10' : 'group-hover:bg-slate-700']">
          <Globe v-if="server.host.includes('.')" :size="16" />
          <ServerIcon v-else :size="16" />
        </div>

        <div class="flex-1 min-w-0">
          <div class="text-sm font-medium truncate">{{ server.name }}</div>
        </div>


        <button @click="toggleMenu($event, server.id)"
          class="opacity-0 group-hover:opacity-100 p-1 hover:bg-slate-700 rounded transition-all">
          <MoreVertical :size="14" />
        </button>

        <!-- Dropdown Menu -->
        <div v-if="activeMenuId === server.id"
          class="absolute right-2 top-10 w-32 bg-[#1e293b] border border-slate-700 rounded-xl shadow-2xl z-20 py-1 animate-in fade-in zoom-in-95 duration-100">
          <button @click="openEditModal($event, server)"
            class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800 transition-colors flex items-center space-x-2">
            <span>编辑配置</span>
          </button>
          <button @click="deleteServer($event, server.id)"
            class="w-full text-left px-3 py-2 text-xs hover:bg-red-500/20 text-red-400 transition-colors flex items-center space-x-2">
            <span>删除服务器</span>
          </button>
        </div>
      </div>
    </div>

    <!-- User Section -->
    <div class="p-4 border-t border-slate-800 bg-[#0f172a]/40">
      <div class="flex items-center space-x-3">
        <div
          class="w-8 h-8 rounded-full bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center text-xs font-bold ring-2 ring-slate-800">
          AD
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-xs font-bold truncate">Admin User</div>
          <div class="text-[10px] text-emerald-500 flex items-center">
            <div class="w-1.5 h-1.5 rounded-full bg-emerald-500 mr-1 animate-pulse"></div>
            本地就绪
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>
