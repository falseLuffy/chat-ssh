<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useServerStore } from '../stores/server';
import { Server } from 'lucide-vue-next';
import 'xterm/css/xterm.css';

const terminalElement = ref<HTMLElement | null>(null);
const serverStore = useServerStore();
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlisten: UnlistenFn | null = null;

// Watch for status changes to enable/disable cursor
watch(() => serverStore.activeServer?.status, (newStatus) => {
  if (term) {
    if (newStatus === 'online') {
      term.options.cursorBlink = true;
      term.focus();
    } else {
      term.options.cursorBlink = false;
    }
  }
}, { immediate: true });

onMounted(async () => {
  if (terminalElement.value) {
    term = new Terminal({
      cursorBlink: serverStore.activeServer?.status === 'online',
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

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(terminalElement.value);
    
    if (serverStore.activeServer?.status === 'online') {
      term.focus();
    }
    
    fitAddon.fit();

    term.writeln('\x1b[1;34mChat-SSH 终端已就绪...\x1b[0m');
    
    // Listen for SSH data from backend
    unlisten = await listen('ssh-data', (event) => {
      const data = event.payload as number[];
      term?.write(new Uint8Array(data));
    });

    term.onData(async (data) => {
      if (serverStore.activeServer?.status === 'online') {
        const bytes = new TextEncoder().encode(data);
        try {
          await invoke('write_to_terminal', { 
            serverName: serverStore.activeServer.name, 
            data: Array.from(bytes) 
          });
        } catch (e) {
          console.error('Failed to send input to backend:', e);
        }
      }
    });

    window.addEventListener('resize', handleResize);
  }
});

const handleResize = () => {
  fitAddon?.fit();
};

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
  if (unlisten) unlisten();
  term?.dispose();
});
</script>

<template>
  <div class="h-full w-full p-4 overflow-hidden relative">
    <!-- Overlay when disconnected -->
    <div v-if="serverStore.activeServer?.status !== 'online'" 
      class="absolute inset-0 flex flex-col items-center justify-center bg-[#0f172a]/80 backdrop-blur-sm z-20 animate-in fade-in duration-300">
      <div class="p-6 rounded-3xl bg-slate-800/50 border border-slate-700/50 flex flex-col items-center shadow-2xl">
        <div class="w-16 h-16 rounded-2xl bg-blue-500/10 flex items-center justify-center text-blue-400 mb-6 ring-1 ring-blue-500/20">
          <Server :size="32" />
        </div>
        <h2 class="text-xl font-bold text-white mb-2">未连接服务器</h2>
        <p class="text-slate-400 text-sm max-w-[240px] text-center leading-relaxed">
          {{ serverStore.activeServer?.status === 'connecting' ? '正在连接请稍候...' : '双击左侧列表中的服务器进行连接' }}
        </p>
      </div>
    </div>
    
    <div ref="terminalElement" class="h-full w-full"></div>
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

