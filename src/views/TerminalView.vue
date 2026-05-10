<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useServerStore } from '../stores/server';
import { Server, Clipboard, Trash2 } from 'lucide-vue-next';
import type { Server as ServerType } from '../stores/server';
import InteractiveTerminal from '../components/InteractiveTerminal.vue';

const serverStore = useServerStore();
const props = defineProps<{ server: ServerType; activeTab: string }>();

const terminalRef = ref<InstanceType<typeof InteractiveTerminal> | null>(null);
const terminalContainer = ref<HTMLElement | null>(null);

// Context Menu State
const showContextMenu = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });

let unlistenInput: UnlistenFn | null = null;

// Create terminal when server comes online
watch(() => props.server.status, (newStatus, oldStatus) => {
  if (newStatus === 'online' && oldStatus !== 'online') {
    nextTick(() => terminalRef.value?.fit());
  } else if (newStatus !== 'online' && oldStatus === 'online') {
    terminalRef.value?.dispose();
  }
});

onMounted(async () => {
  // Listen for AI command input — only respond if this is the active server's instance
  unlistenInput = await listen('terminal-input', async (event) => {
    if (serverStore.activeServerId !== props.server.id) return;
    const command = event.payload as string;
    if (props.server.status === 'online' && terminalRef.value) {
      terminalRef.value.write(command);
    }
  });

  window.addEventListener('resize', handleResize);

  // 如果组件挂载时服务器已在线，初始化终端
  if (props.server.status === 'online') {
    await nextTick();
    terminalRef.value?.fit();
  }
});

watch(() => props.activeTab, async (newTab) => {
  if (newTab === 'terminal') {
    await nextTick();
    setTimeout(() => {
      handleResize();
      terminalRef.value?.focus();
    }, 50);
  }
});

// When this server becomes the active server, re-fit terminal
watch(() => serverStore.activeServerId, async (newId) => {
  if (newId === props.server.id && props.server.status === 'online') {
    await nextTick();
    setTimeout(() => {
      handleResize();
      terminalRef.value?.focus();
    }, 50);
  }
});

const handleContextMenu = (e: MouseEvent) => {
  e.preventDefault();
  contextMenuPos.value = { x: e.clientX, y: e.clientY };
  showContextMenu.value = true;

  const closeMenu = () => {
    showContextMenu.value = false;
    window.removeEventListener('click', closeMenu);
  };
  window.addEventListener('click', closeMenu);
};

const pasteFromClipboard = async () => {
  try {
    const text = await navigator.clipboard.readText();
    if (text && terminalRef.value) {
      terminalRef.value.write(text);
    }
  } catch (err) {
    console.error('Failed to paste:', err);
  }
};

const clearTerminal = () => {
  terminalRef.value?.write('clear\n');
};

const handleResize = () => {
  terminalRef.value?.fit();
};

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
  if (unlistenInput) unlistenInput();
  terminalRef.value?.dispose();
});
</script>

<template>
  <div class="h-full w-full p-4 overflow-hidden relative">
    <!-- Overlay when disconnected -->
    <div v-if="props.server.status !== 'online'"
      class="absolute inset-0 flex flex-col items-center justify-center bg-[#0f172a]/80 backdrop-blur-sm z-20 animate-in fade-in duration-300">
      <div class="p-6 rounded-3xl bg-slate-800/50 border border-slate-700/50 flex flex-col items-center shadow-2xl">
        <div
          class="w-16 h-16 rounded-2xl bg-blue-500/10 flex items-center justify-center text-blue-400 mb-6 ring-1 ring-blue-500/20">
          <Server :size="32" />
        </div>
        <h2 class="text-xl font-bold text-white mb-2">
          {{ props.server.status === 'connecting' ? '正在建立连接' : '未连接服务器' }}
        </h2>
        <p class="text-slate-400 text-sm max-w-[240px] text-center leading-relaxed">
          {{ props.server.status === 'connecting'
            ? `正在连接到 ${props.server.name} (${props.server.host}:${props.server.port || 22})...`
            : '双击左侧列表中的服务器进行连接' }}
        </p>
      </div>
    </div>

    <div
      v-if="props.server.status === 'online'"
      ref="terminalContainer"
      class="flex-1 min-h-0 relative h-full w-full bg-black/40"
      @contextmenu="handleContextMenu"
    >
      <InteractiveTerminal
        ref="terminalRef"
        :server-name="props.server.name"
        class="w-full h-full p-4"
      />
    </div>

    <!-- Custom Context Menu -->
    <Teleport to="body">
      <div v-if="showContextMenu"
        class="fixed z-[100] w-40 bg-[#1e293b]/95 backdrop-blur-xl border border-slate-700 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
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
