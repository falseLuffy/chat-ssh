<script setup lang="ts">
  import { ref, onMounted } from 'vue';
  import { useSettingsStore } from './stores/settings';
  import { useServerStore } from './stores/server';
  import { useKnowledgeStore } from './stores/knowledge';
  import AiChatBox from './components/AiChatBox.vue';
  import ServerSidebar from './components/ServerSidebar.vue';
  import TerminalView from './views/TerminalView.vue';
  import FileBrowser from './components/FileBrowser.vue';
  import SettingsModal from './components/SettingsModal.vue';
  import { Terminal, Shield, Settings as SettingsIcon, Server } from 'lucide-vue-next';

  const settingsStore = useSettingsStore();
  const serverStore = useServerStore();
  const knowledgeStore = useKnowledgeStore();
  
  const activeTab = ref('terminal');
  const showSettingsModal = ref(false);

  onMounted(async () => {
    // Parallel init
    await Promise.all([
      settingsStore.loadSettings(),
      serverStore.initStore(),
      knowledgeStore.initStore()
    ]);
  });
</script>

<template>
  <div class="flex h-screen w-screen bg-[#0f172a] text-slate-200 overflow-hidden font-sans">
    <SettingsModal v-if="showSettingsModal" @close="showSettingsModal = false" />
    <!-- Sidebar -->
    <ServerSidebar class="w-64 border-r border-slate-800 bg-[#1e293b]/50 backdrop-blur-xl" />

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col relative overflow-hidden">
      <!-- Header / Tabs -->
      <header class="h-12 border-b border-slate-800 flex items-center px-4 bg-[#1e293b]/30 backdrop-blur-md z-10">
        <div class="flex space-x-4">
          <button @click="activeTab = 'terminal'"
            :class="['flex items-center space-x-2 px-3 py-1 rounded-md transition-all', activeTab === 'terminal' ? 'bg-blue-600/20 text-blue-400 border border-blue-500/30' : 'hover:bg-slate-800']">
            <Terminal :size="16" />
            <span class="text-sm font-medium">终端</span>
          </button>
          <button @click="activeTab = 'files'"
            :class="['flex items-center space-x-2 px-3 py-1 rounded-md transition-all', activeTab === 'files' ? 'bg-blue-600/20 text-blue-400 border border-blue-500/30' : 'hover:bg-slate-800']">
            <Server :size="16" />
            <span class="text-sm font-medium">文件</span>
          </button>
        </div>

        <div class="ml-auto flex items-center space-x-3">
          <button class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-400 transition-colors">
            <Shield :size="18" title="安全审计" />
          </button>
          <button @click="showSettingsModal = true"
            class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-400 transition-colors">
            <SettingsIcon :size="18" title="设置" />
          </button>
        </div>
      </header>

      <!-- View Area -->
      <div class="flex-1 min-h-0 bg-black/20">
        <TerminalView v-show="activeTab === 'terminal'" :active-tab="activeTab" />
        <FileBrowser v-show="activeTab === 'files'" :active-tab="activeTab" />
      </div>

      <!-- Floating AI Chatbox -->
      <AiChatBox />
    </main>
  </div>
</template>

<style>

  /* Global Glassmorphism effects */
  .glass {
    background: rgba(30, 41, 59, 0.4);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  ::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }

  ::-webkit-scrollbar-track {
    background: transparent;
  }

  ::-webkit-scrollbar-thumb {
    background: #334155;
    border-radius: 10px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background: #475569;
  }
</style>