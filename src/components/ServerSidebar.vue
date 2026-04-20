<script setup lang="ts">
  import { ref } from 'vue';
  import { invoke } from '@tauri-apps/api/core';
  import { open, message } from '@tauri-apps/plugin-dialog';
  import { Plus, Server as ServerIcon, Globe, MoreVertical, FileUp } from 'lucide-vue-next';
  import { useServerStore, type Server } from '../stores/server';
  import AddServerModal from './AddServerModal.vue';
  import PasswordPromptModal from './PasswordPromptModal.vue';

  const serverStore = useServerStore();
  const showAddModal = ref(false);
  const showPasswordRetry = ref(false);
  const editingServer = ref<Server | null>(null);
  const retryServer = ref<Server | null>(null);
  const activeMenuId = ref<number | null>(null);
  
  // Context Menu State
  const showContextMenu = ref(false);
  const contextMenuPos = ref({ x: 0, y: 0 });
  const contextMenuServer = ref<Server | null>(null);

  const handleContextMenu = (e: MouseEvent, server: Server | null = null) => {
    e.preventDefault();
    contextMenuPos.value = { x: e.clientX, y: e.clientY };
    contextMenuServer.value = server;
    showContextMenu.value = true;
    
    const closeMenu = () => {
      showContextMenu.value = false;
      window.removeEventListener('click', closeMenu);
    };
    window.addEventListener('click', closeMenu);
  };

  const importXshell = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'Xshell Session', extensions: ['xsh', 'xts'] }]
      });

      if (selected && Array.isArray(selected)) {
        const results = await invoke('import_xshell_sessions', { paths: selected }) as any[];
        let count = 0;
        let passCount = 0;
        
        for (const session of results) {
          serverStore.addServer({
            name: session.name,
            host: session.host,
            port: session.port,
            username: session.user,
            password: session.password || ''
          });
          count++;
          if (session.password) passCount++;
        }

        await message(`成功导入 ${count} 个会话${passCount > 0 ? `（其中 ${passCount} 个密码已自动还原）` : ''}`, {
          title: '导入成功',
          kind: 'info'
        });
      }
    } catch (e) {
      console.error('Import failed:', e);
      await message(String(e), { title: '导入失败', kind: 'error' });
    }
  };

  const selectServer = (id: number) => {
    serverStore.setActiveServer(id);
  };

  const handleDblClick = async (server: Server) => {
    if (server.status === 'online') {
      serverStore.setActiveServer(server.id);
      emit('switch-tab', 'terminal');
      return;
    }

    try {
      // Use store to connect
      const res = await serverStore.connectServer(server);
      
      if (!res.success) {
        throw new Error(res.error);
      }
      
      serverStore.setActiveServer(server.id);

      // Inform parent to switch tab (this usually happens via a shared state/event)
      // For now we'll assume the user switches to terminal
    } catch (error) {
      const errorMsg = String(error);
      console.error('Connection failed:', errorMsg);
      serverStore.updateStatus(server.id, 'offline');
      
      // Check if it's an authentication failure
      if (errorMsg.includes('Authentication failed')) {
        retryServer.value = server;
        showPasswordRetry.value = true;
      } else {
        // Show normal error to user
        await message(errorMsg, {
          title: '连接失败',
          kind: 'error',
        });
      }
    }
  }

  const handleRetrySubmit = async (newPassword: string) => {
    if (retryServer.value) {
      // Update password in store
      serverStore.updateServer(retryServer.value.id, { password: newPassword });
      
      // Close modal
      const serverToRetry = serverStore.servers.find(s => s.id === retryServer.value?.id);
      showPasswordRetry.value = false;
      retryServer.value = null;

      // Retry connection
      if (serverToRetry) {
        handleDblClick(serverToRetry);
      }
    }
  };

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
    serverStore.deleteServer(id);
    activeMenuId.value = null;
  };

  const handleDisconnect = async (event: Event, id: number) => {
    event.stopPropagation();
    await serverStore.disconnectServer(id);
    activeMenuId.value = null;
  };
</script>

<template>
  <aside class="flex flex-col h-full relative" @click="activeMenuId = null" @contextmenu="handleContextMenu($event)">
    <!-- Sidebar Header -->
    <div class="h-12 flex items-center justify-between px-4 border-b border-slate-800">
      <h2 class="text-xs font-bold uppercase tracking-widest text-slate-500">我的服务器</h2>
      <div class="flex items-center space-x-1">
        <button @click.stop="importXshell" class="p-1 hover:bg-slate-800 rounded-md text-emerald-400 transition-colors" title="从 Xshell 导入">
          <FileUp :size="16" />
        </button>
        <button @click.stop="openAddModal" class="p-1 hover:bg-slate-800 rounded-md text-blue-400 transition-colors" title="添加服务器">
          <Plus :size="16" />
        </button>
      </div>
    </div>

    <!-- Add/Edit Server Modal Overlay -->
    <AddServerModal v-if="showAddModal" :editServer="editingServer" @close="showAddModal = false" />

    <!-- Password Retry Modal Overlay -->
    <PasswordPromptModal v-if="showPasswordRetry && retryServer" :server="retryServer" @close="showPasswordRetry = false" @submit="handleRetrySubmit" />

    <!-- Server List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      <div v-for="server in serverStore.servers" :key="server.id" @click="selectServer(server.id)"
        @dblclick="handleDblClick(server)" 
        @contextmenu.stop="handleContextMenu($event, server)"
        :class="[
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
          <div class="text-sm font-medium truncate flex items-center">
            <span>{{ server.name }}</span>
            <div v-if="server.status === 'online'" class="ml-2 w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]"></div>
            <div v-else-if="server.status === 'connecting'" class="ml-2 w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse"></div>
          </div>
        </div>


        <button @click="toggleMenu($event, server.id)"
          class="opacity-0 group-hover:opacity-100 p-1 hover:bg-slate-700 rounded transition-all">
          <MoreVertical :size="14" />
        </button>

        <!-- Dropdown Menu -->
        <div v-if="activeMenuId === server.id"
          class="absolute right-2 top-10 w-32 bg-[#1e293b] border border-slate-700 rounded-xl shadow-2xl z-20 py-1 animate-in fade-in zoom-in-95 duration-100">
          <button v-if="server.status === 'online'" @click="handleDisconnect($event, server.id)"
            class="w-full text-left px-3 py-2 text-xs hover:bg-red-500/10 text-red-400 transition-colors flex items-center space-x-2">
            <span>断开连接</span>
          </button>
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

    <!-- Custom Context Menu -->
    <Teleport to="body">
      <div v-if="showContextMenu" 
        class="fixed z-[100] w-48 bg-[#1e293b]/95 backdrop-blur-xl border border-slate-700 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
        <template v-if="contextMenuServer">
          <button v-if="contextMenuServer.status === 'offline'" @click="handleDblClick(contextMenuServer)" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Plus :size="14" class="group-hover/item:scale-110 transition-transform" /> 
            <span class="group-hover/item:translate-x-1 transition-transform">立即连接</span>
          </button>
          <button v-else @click="handleDisconnect($event, contextMenuServer.id)" class="w-full text-left px-3 py-2 text-xs hover:bg-orange-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <LogOut :size="14" class="group-hover/item:scale-110 transition-transform" /> 
            <span class="group-hover/item:translate-x-1 transition-transform">断开连接</span>
          </button>
          
          <div class="h-px bg-slate-700/50 my-1"></div>
          
          <button @click="openEditModal($event, contextMenuServer)" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Edit2 :size="14" class="group-hover/item:scale-110 transition-transform" /> 
            <span class="group-hover/item:translate-x-1 transition-transform">编辑配置</span>
          </button>

          <div class="h-px bg-slate-700/50 my-1"></div>
          
          <button @click="deleteServer($event, contextMenuServer.id)" class="w-full text-left px-3 py-2 text-xs hover:bg-red-600/90 text-red-500 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Trash2 :size="14" class="group-hover/item:scale-110 transition-transform" /> 
            <span class="group-hover/item:translate-x-1 transition-transform">删除服务器</span>
          </button>
        </template>
        <template v-else>
          <button @click="openAddModal" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Plus :size="14" class="group-hover/item:scale-110 transition-transform" /> 
            <span class="group-hover/item:translate-x-1 transition-transform">添加新服务器</span>
          </button>
          <button @click="importXshell" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <FileUp :size="14" class="group-hover/item:scale-110 transition-transform" /> 
            <span class="group-hover/item:translate-x-1 transition-transform">导入 Xshell 配置</span>
          </button>
        </template>
      </div>
    </Teleport>
  </aside>
</template>
