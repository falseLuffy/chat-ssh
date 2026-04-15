<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';
import { useServerStore } from '../stores/server';
import { File, Folder, Upload, Download, Trash2, RefreshCw, Loader2 } from 'lucide-vue-next';

const serverStore = useServerStore();
const currentPath = ref('/root');
const files = ref([
  { name: 'nginx.conf', size: '2.4 KB', type: 'file' },
  { name: 'logs', size: '-', type: 'dir' },
  { name: 'web-app', size: '-', type: 'dir' },
]);
const isUploading = ref(false);

const handleUpload = async () => {
  if (!serverStore.activeServer) return;

  try {
    const selected = await open({
      multiple: false,
      title: '选择上传文件',
    });

    if (selected && typeof selected === 'string') {
      isUploading.value = true;
      const content = await readFile(selected);
      const fileName = selected.split(/[\\/]/).pop();
      const remotePath = `${currentPath.value}/${fileName}`;
      
      await invoke('upload_to_server', {
        serverName: serverStore.activeServer.name,
        remotePath,
        fileContent: Array.from(content), 
      });
      
      alert(`上传成功: ${fileName}`);
    }
  } catch (error) {
    console.error('上传失败:', error);
    alert('上传失败，请检查连接。');
  } finally {
    isUploading.value = false;
  }
};
</script>

<template>
  <div class="flex flex-col h-full bg-[#0f172a]/30">
    <!-- Toolbar -->
    <div class="h-12 border-b border-slate-800 flex items-center px-4 space-x-4 bg-slate-900/50">
      <div class="flex items-center space-x-2 text-xs text-slate-400">
        <Folder :size="14" />
        <span class="font-mono">{{ currentPath }}</span>
      </div>
      
      <div class="flex-1"></div>
      
      <div class="flex items-center space-x-2">
        <button 
          @click="handleUpload"
          :disabled="isUploading || !serverStore.activeServer"
          class="flex items-center space-x-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white rounded-lg text-xs font-semibold transition-all"
        >
          <Loader2 v-if="isUploading" class="animate-spin" :size="14" />
          <Upload v-else :size="14" />
          <span>上传文件</span>
        </button>
        <button class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-400">
          <RefreshCw :size="16" />
        </button>
      </div>
    </div>

    <!-- File List -->
    <div class="flex-1 overflow-y-auto p-4">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        <div 
          v-for="file in files" 
          :key="file.name"
          class="group flex items-center p-3 bg-slate-800/40 border border-slate-700/50 rounded-xl hover:border-blue-500/50 hover:bg-blue-500/5 transition-all cursor-pointer"
        >
          <div class="p-2 rounded-lg bg-slate-800 mr-3 text-slate-400 group-hover:text-blue-400 transition-colors">
            <Folder v-if="file.type === 'dir'" :size="20" />
            <File v-else :size="20" />
          </div>
          
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium truncate text-slate-200">{{ file.name }}</div>
            <div class="text-[10px] text-slate-500 font-mono">{{ file.size }}</div>
          </div>

          <div class="flex space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button class="p-1.5 hover:bg-slate-700 rounded text-slate-400"><Download :size="14" /></button>
            <button class="p-1.5 hover:bg-red-500/20 rounded text-red-500/70"><Trash2 :size="14" /></button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
