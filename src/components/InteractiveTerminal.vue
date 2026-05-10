<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import 'xterm/css/xterm.css';

const props = withDefaults(defineProps<{
  serverName: string;
  readEvent?: string;
  writeCommand?: string;
  initialInput?: string;
}>(), {
  readEvent: 'ssh-data',
  writeCommand: 'write_to_terminal',
  initialInput: '',
});

interface TerminalSession {
  term: Terminal;
  fitAddon: FitAddon;
}

const terminalSession = ref<TerminalSession | null>(null);
const terminalContainer = ref<HTMLElement | null>(null);
const resizeObserver = ref<ResizeObserver | null>(null);

let unlistenData: UnlistenFn | null = null;

async function initTerminal() {
  await nextTick();
  const container = terminalContainer.value;
  if (!container || terminalSession.value) return;

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

  if (container.offsetWidth > 0 && container.offsetHeight > 0) {
    fitAddon.fit();
  }

  term.onData(async (data) => {
    const bytes = new TextEncoder().encode(data);
    try {
      await invoke(props.writeCommand, {
        serverName: props.serverName,
        data: Array.from(bytes),
      });
    } catch (e) {
      console.error('Failed to send input to backend:', e);
    }
  });

  resizeObserver.value = new ResizeObserver(() => {
    try { fitAddon.fit(); } catch (_) {}
  });
  resizeObserver.value.observe(container);

  terminalSession.value = { term, fitAddon };
  term.focus();

  // Write initial input if provided
  if (props.initialInput) {
    const bytes = new TextEncoder().encode(props.initialInput);
    try {
      await invoke(props.writeCommand, {
        serverName: props.serverName,
        data: Array.from(bytes),
      });
    } catch (e) {
      console.error('Failed to send initial input:', e);
    }
  }
}

function write(data: string) {
  const bytes = new TextEncoder().encode(data);
  invoke(props.writeCommand, {
    serverName: props.serverName,
    data: Array.from(bytes),
  }).catch((e) => console.error('Failed to write to terminal:', e));
}

function writeBytes(data: number[]) {
  invoke(props.writeCommand, {
    serverName: props.serverName,
    data,
  }).catch((e) => console.error('Failed to write bytes to terminal:', e));
}

function fit() {
  terminalSession.value?.fitAddon.fit();
}

function focus() {
  terminalSession.value?.term.focus();
}

function clear() {
  terminalSession.value?.term.clear();
}

function dispose() {
  resizeObserver.value?.disconnect();
  resizeObserver.value = null;
  terminalSession.value?.term.dispose();
  terminalSession.value = null;
}

onMounted(async () => {
  unlistenData = await listen(props.readEvent, (event) => {
    const payload = event.payload as { server: string; data: number[] };
    if (payload.server !== props.serverName) return;
    const session = terminalSession.value;
    if (session && payload.data.length > 0) {
      session.term.write(new Uint8Array(payload.data));
    }
  });

  await nextTick();
  initTerminal();
});

onBeforeUnmount(() => {
  if (unlistenData) unlistenData();
  dispose();
});

defineExpose({ write, writeBytes, fit, focus, clear, dispose });
</script>

<template>
  <div
    ref="terminalContainer"
    class="terminal-component w-full h-full"
  ></div>
</template>

<style>
.terminal-component .xterm-viewport::-webkit-scrollbar {
  display: none;
}

.terminal-component .xterm {
  height: 100%;
  padding: 8px;
}
</style>
