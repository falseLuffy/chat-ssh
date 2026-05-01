<script setup lang="ts">
import type { Server } from '../stores/server';
import TerminalView from '../views/TerminalView.vue';
import ServerManagement from './ServerManagement.vue';
import FileBrowser from './FileBrowser.vue';
import OpsView from '../views/OpsView.vue';

defineProps<{
  server: Server;
  activeTab: string;
}>();
</script>

<template>
  <div class="h-full flex flex-col absolute overflow-hidden">
    <!-- Terminal: absolute + z-index 保持容器永远有尺寸，避免 xterm 坍缩 -->
    <div
      class="absolute inset-0 transition-none"
      :class="activeTab === 'terminal' ? 'z-10 visible opacity-100' : 'z-0 invisible opacity-0'"
    >
      <TerminalView :server="server" :active-tab="activeTab" />
    </div>
    <!-- 其他视图用 v-show 没问题 -->
    <ServerManagement v-show="activeTab === 'management'" :server="server" />
    <FileBrowser v-show="activeTab === 'files'" :server="server" :active-tab="activeTab" />
    <OpsView v-show="activeTab === 'ops'" :server="server" />
  </div>
</template>
