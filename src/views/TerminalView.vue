<script setup lang="ts">
  import { onMounted, onBeforeUnmount, ref, watch, nextTick, defineProps } from 'vue';
  import { Terminal } from 'xterm';
  import { FitAddon } from 'xterm-addon-fit';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { useServerStore } from '../stores/server';
  import { Server, Copy, Clipboard, Trash2 } from 'lucide-vue-next';
  import 'xterm/css/xterm.css';

  const serverStore = useServerStore();
  const props = defineProps<{ activeTab: string }>();

  interface TerminalSession {
    term: Terminal;
    fitAddon: FitAddon;
  }

  const terminalSessions = new Map<string, TerminalSession>();
  const terminalContainers = ref<Record<string, HTMLElement>>({});
  
  // Context Menu State
  const showContextMenu = ref(false);
  const contextMenuPos = ref({ x: 0, y: 0 });

  let unlisten: UnlistenFn | null = null;
  let unlistenInput: UnlistenFn | null = null;

  // Watch for status changes to manage terminal instances
  watch(() => serverStore.servers, (newServers) => {
    // Add new online terminals
    newServers.forEach(server => {
      if (server.status === 'online' && !terminalSessions.has(server.name)) {
        initTerminal(server);
      }
    });

    // Cleanup offline terminals
    for (const [name, session] of terminalSessions.entries()) {
      if (!newServers.find(s => s.name === name && s.status === 'online')) {
        session.term.dispose();
        terminalSessions.delete(name);
        delete terminalContainers.value[name];
      }
    }
  }, { deep: true, immediate: true });

  // Watch active server to refocus
  watch(() => serverStore.activeServerId, () => {
    const active = serverStore.activeServer;
    if (active && terminalSessions.has(active.name)) {
      setTimeout(() => {
        const session = terminalSessions.get(active.name)!;
        session.fitAddon.fit();
        session.term.focus();
      }, 50);
    }
  });


  async function initTerminal(server: any) {
    await nextTick();
    const container = terminalContainers.value[server.name];
    if (!container || terminalSessions.has(server.name)) return;

    const term = new Terminal({
      cursorBlink: true,
      cols: 80,
      rows: 24,
      theme: {
        background: 'transparent',
        foreground: '#e2e8f0',
        cursor: '#38bdf8',
        selectionBackground: 'rgba(56, 189, 248, 0.3)',
      },
      fontFamily: '"Fira Code", monospace',
      fontSize: 14,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(container);
    fitAddon.fit();

    term.writeln('\x1b[1;34mAI-SSH 终端已挂载 - ' + server.host + '\x1b[0m');

    term.onData(async (data) => {
      const bytes = new TextEncoder().encode(data);
      try {
        await invoke('write_to_terminal', {
          serverName: server.name,
          data: Array.from(bytes)
        });
      } catch (e) {
        console.error('Failed to send input to backend:', e);
      }
    });

    terminalSessions.set(server.name, { term, fitAddon });
    if (serverStore.activeServerId === server.id) {
      term.focus();
    }
  };

  onMounted(async () => {
    // Listen for SSH data from backend
    unlisten = await listen('ssh-data', (event) => {
      const { server: serverName, data } = event.payload as { server: string, data: number[] };
      const session = terminalSessions.get(serverName);
      if (session) {
        session.term.write(new Uint8Array(data));
      }
    });

    // Listen for AI command input
    unlistenInput = await listen('terminal-input', async (event) => {
      const command = event.payload as string;
      const active = serverStore.activeServer;
      if (active && active.status === 'online') {
        const session = terminalSessions.get(active.name);
        if (session) {
          const bytes = new TextEncoder().encode(command);
          try {
            await invoke('write_to_terminal', {
              serverName: active.name,
              data: Array.from(bytes)
            });
          } catch (e) {
            console.error('Failed to send AI command to backend:', e);
          }
        }
      }
    });

    window.addEventListener('resize', handleResize);
  });

  watch(() => props.activeTab, async (newTab) => {
    if (newTab === 'terminal') {
      await nextTick();
      // Small timeout to ensure DOM has finalized layout
      setTimeout(() => {
        handleResize();
        const active = serverStore.activeServer;
        if (active) {
          terminalSessions.get(active.name)?.term.focus();
        }
      }, 50);
    }
  });

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    contextMenuPos.value = { x: e.clientX, y: e.clientY };
    showContextMenu.value = true;
    
    // Close menu when clicking elsewhere
    const closeMenu = () => {
      showContextMenu.value = false;
      window.removeEventListener('click', closeMenu);
    };
    window.addEventListener('click', closeMenu);
  };

  const copySelection = () => {
    const active = serverStore.activeServer;
    if (active) {
      const session = terminalSessions.get(active.name);
      if (session) {
        const selected = session.term.getSelection();
        if (selected) {
          navigator.clipboard.writeText(selected);
        }
      }
    }
  };

  const pasteFromClipboard = async () => {
    const active = serverStore.activeServer;
    if (active) {
      try {
        const text = await navigator.clipboard.readText();
        if (text) {
          const bytes = new TextEncoder().encode(text);
          await invoke('write_to_terminal', {
            serverName: active.name,
            data: Array.from(bytes)
          });
        }
      } catch (err) {
        console.error('Failed to paste:', err);
      }
    }
  };

  const clearTerminal = async () => {
    const active = serverStore.activeServer;
    if (active) {
      const bytes = new TextEncoder().encode('clear\n');
      await invoke('write_to_terminal', {
        serverName: active.name,
        data: Array.from(bytes)
      });
    }
  };

  const handleResize = () => {
    terminalSessions.forEach(session => session.fitAddon.fit());
  };

  onBeforeUnmount(() => {
    window.removeEventListener('resize', handleResize);
    if (unlisten) unlisten();
    if (unlistenInput) unlistenInput();
    terminalSessions.forEach(session => session.term.dispose());
    terminalSessions.clear();
  });
</script>

<template>
  <div class="h-full w-full p-4 overflow-hidden relative">
    <!-- Overlay when disconnected -->
    <div v-if="serverStore.activeServer?.status !== 'online'"
      class="absolute inset-0 flex flex-col items-center justify-center bg-[#0f172a]/80 backdrop-blur-sm z-20 animate-in fade-in duration-300">
      <div class="p-6 rounded-3xl bg-slate-800/50 border border-slate-700/50 flex flex-col items-center shadow-2xl">
        <div
          class="w-16 h-16 rounded-2xl bg-blue-500/10 flex items-center justify-center text-blue-400 mb-6 ring-1 ring-blue-500/20">
          <Server :size="32" />
        </div>
        <h2 class="text-xl font-bold text-white mb-2">
          {{ serverStore.activeServer?.status === 'connecting' ? '正在建立连接' : '未连接服务器' }}
        </h2>
        <p class="text-slate-400 text-sm max-w-[240px] text-center leading-relaxed">
          {{ serverStore.activeServer?.status === 'connecting'
            ? `正在连接到 ${serverStore.activeServer?.name} (${serverStore.activeServer?.host}:${serverStore.activeServer?.port
            || 22})...`
            : '双击左侧列表中的服务器进行连接' }}
        </p>
      </div>
    </div>

    <div 
      v-for="server in serverStore.servers.filter(s => s.status === 'online')" 
      :key="server.id"
      v-show="serverStore.activeServerId === server.id"
      class="flex-1 min-h-0 relative h-full w-full bg-black/40"
      @contextmenu="handleContextMenu"
    >
      <div 
        :ref="el => { if (el) terminalContainers[server.name] = el as HTMLElement }"
        class="terminal-container w-full h-full p-4"
      ></div>
    </div>

    <!-- Custom Context Menu -->
    <Teleport to="body">
      <div v-if="showContextMenu" 
        class="fixed z-[100] w-40 bg-[#1e293b]/95 backdrop-blur-xl border border-slate-700 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
        <button @click="copySelection" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/menu">
          <Copy :size="14" class="group-hover/menu:scale-110 transition-transform" /> 
          <span class="group-hover/menu:translate-x-1 transition-transform">复制选中</span>
        </button>
        <button @click="pasteFromClipboard" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/menu">
          <Clipboard :size="14" class="group-hover/menu:scale-110 transition-transform" /> 
          <span class="group-hover/menu:translate-x-1 transition-transform">粘贴内容</span>
        </button>
        <div class="h-px bg-slate-800 my-1"></div>
        <button @click="clearTerminal" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/menu">
          <Trash2 :size="14" class="group-hover/menu:scale-110 transition-transform" /> 
          <span class="group-hover/menu:translate-x-1 transition-transform">清除屏幕</span>
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style>
  .xterm-viewport::-webkit-scrollbar {
    display: none;
  }

  .xterm {
    height: 100%;
    padding: 8px;
  }
</style>
