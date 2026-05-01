<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted, watch, defineProps } from 'vue';
import type { Server } from '../stores/server';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readFile, writeFile, stat, readDir } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { useServerStore } from '../stores/server';
import { useUIStore } from '../stores/ui';
import { File, Folder, Upload, Download, Trash2, RefreshCw, Loader2, ChevronLeft, Home, HardDrive, ShieldAlert, LayoutGrid, List, Plus, Copy, ChevronDown, Terminal } from 'lucide-vue-next';
import InputModal from './InputModal.vue';

const props = defineProps<{ server: Server; activeTab: string }>();
const serverStore = useServerStore();
const ui = useUIStore();
const currentPath = ref('/root');
const files = ref<any[]>([]);
const isLoading = ref(false);
const isUploading = ref(false);
const isDownloading = ref(false);
const errorMessage = ref('');

// Conflict resolution state
const globalConflictAction = ref<any>(null);
const applyConflictToAll = ref(false);

const resetConflictState = () => {
  globalConflictAction.value = null;
  applyConflictToAll.value = false;
};

// Operation Log
type LogLevel = 'success' | 'error' | 'info';
interface LogEntry {
  id: number;
  time: string;
  level: LogLevel;
  message: string;
}
const logs = ref<LogEntry[]>([]);
const logIdCounter = ref(0);
const logPanelOpen = ref(true);
const logContainer = ref<HTMLElement | null>(null);

const addLog = (message: string, level: LogLevel = 'info') => {
  const now = new Date();
  const time = now.toTimeString().slice(0, 8);
  logIdCounter.value += 1;
  logs.value.push({ id: logIdCounter.value, time, level, message });
  // Keep last 200 entries
  if (logs.value.length > 200) logs.value.splice(0, logs.value.length - 200);
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  });
};

const clearLogs = () => { logs.value = []; };

// Persist view mode changes
const viewMode = ref<'grid' | 'list'>((localStorage.getItem('ssh_view_mode') as 'grid' | 'list') || 'list');
watch(viewMode, (newMode) => {
  localStorage.setItem('ssh_view_mode', newMode);
});

// Selection State
const selectedFiles = ref(new Set<string>());

const isSelected = (name: string) => selectedFiles.value.has(name);

const toggleSelect = (name: string, multi: boolean) => {
  if (multi) {
    const next = new Set(selectedFiles.value);
    next.has(name) ? next.delete(name) : next.add(name);
    selectedFiles.value = next;
  } else {
    selectedFiles.value = new Set([name]);
  }
};

const selectAll = () => {
  selectedFiles.value = new Set(files.value.map((f: any) => f.name));
};

const clearSelection = () => {
  selectedFiles.value = new Set();
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'a' && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    selectAll();
  } else if (e.key === 'Escape') {
    clearSelection();
  }
};

// Context Menu State
const showContextMenu = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });
const contextMenuFile = ref<any>(null); // null means click on background

// Modal State
const showInputModal = ref(false);
const inputModalConfig = ref({
  title: '',
  defaultValue: '',
  placeholder: '',
  type: 'rename' as 'rename' | 'newFolder',
  target: null as any
});

const loadFiles = async () => {
  if (!props.server || props.server.status !== 'online') {
    files.value = [];
    return;
  }

  isLoading.value = true;
  errorMessage.value = '';
  files.value = []; // Clear current list while loading
  try {
    const results = await invoke<any[]>('list_remote_files', {
      serverName: props.server.name,
      path: currentPath.value,
    });
    files.value = results;
  } catch (error) {
    console.error('Failed to load files:', error);
    errorMessage.value = String(error);
  } finally {
    isLoading.value = false;
  }
};

const navigateTo = (path: string) => {
  // Normalize path: replace multiple slashes and ensure it starts with /
  let normalizedPath = path.replace(/\/+/g, '/');
  if (normalizedPath === '') normalizedPath = '/';
  if (!normalizedPath.startsWith('/')) normalizedPath = '/' + normalizedPath;
  currentPath.value = normalizedPath;
  clearSelection();
  loadFiles();
};

const handleFolderClick = (folderName: string) => {
  const newPath = currentPath.value === '/'
    ? `/${folderName}`
    : `${currentPath.value}/${folderName}`;
  navigateTo(newPath);
};

const goBack = () => {
  if (currentPath.value === '/' || currentPath.value === '') return;
  const parts = currentPath.value.split('/').filter(Boolean);
  parts.pop();
  const newPath = '/' + parts.join('/');
  navigateTo(newPath);
};

const formatSize = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// Check if a remote path exists
const checkRemoteExists = async (remotePath: string): Promise<boolean> => {
  if (!props.server) return false;
  try {
    const result = await invoke<string>('execute_remote_command', {
      server_name: props.server.name,
      command: `[ -e "${remotePath}" ] && echo "exists"`
    });
    return result.trim() === 'exists';
  } catch (e) {
    return false;
  }
};

// Handle naming conflicts
const resolvePathConflict = async (remotePath: string): Promise<{ finalPath: string, skip: boolean }> => {
  if (!await checkRemoteExists(remotePath)) {
    return { finalPath: remotePath, skip: false };
  }

  const fileName = remotePath.split('/').pop() || 'file';

  // Use global choice if "Apply to all" was checked
  let action = globalConflictAction.value;

  if (!applyConflictToAll.value || !action) {
    const result = await ui.showConflict({ fileName });
    action = result.action;
    if (result.applyToAll) {
      applyConflictToAll.value = true;
      globalConflictAction.value = action;
    }
  }

  if (action === 'skip') {
    return { finalPath: remotePath, skip: true };
  }

  if (action === 'rename') {
    const extIdx = remotePath.lastIndexOf('.');
    const base = extIdx > 0 ? remotePath.slice(0, extIdx) : remotePath;
    const ext = extIdx > 0 ? remotePath.slice(extIdx) : '';
    let counter = 1;
    let newPath = `${base} (${counter})${ext}`;
    while (await checkRemoteExists(newPath)) {
      counter++;
      newPath = `${base} (${counter})${ext}`;
    }
    return { finalPath: newPath, skip: false };
  }

  // Overwrite is default behavior (just use the same path)
  return { finalPath: remotePath, skip: false };
};

// Low-level: upload one file to an explicit remote path, no list refresh
const uploadSingleFile = async (content: Uint8Array, remoteFullPath: string): Promise<boolean> => {
  if (!props.server) return false;
  try {
    await invoke('upload_to_server', {
      serverName: props.server.name,
      remotePath: remoteFullPath,
      fileContent: Array.from(content),
    });
    addLog(`✓ 上传成功: ${remoteFullPath}`, 'success');
    return true;
  } catch (error) {
    addLog(`✗ 上传失败: ${remoteFullPath} — ${String(error)}`, 'error');
    return false;
  }
};

// Upload a single file into currentPath, then refresh list
const uploadFile = async (fileName: string, content: Uint8Array) => {
  if (!props.server) return;
  isUploading.value = true;
  resetConflictState(); // New task starts

  const initialRemotePath = currentPath.value === '/'
    ? `/${fileName}`
    : `${currentPath.value}/${fileName}`;

  const { finalPath, skip } = await resolvePathConflict(initialRemotePath);

  if (skip) {
    addLog(`跳过上传: ${fileName}`, 'info');
    isUploading.value = false;
    return;
  }

  addLog(`上传中: ${fileName} (${formatSize(content.length)}) → ${currentPath.value}`, 'info');
  const ok = await uploadSingleFile(content, finalPath);
  if (ok) {
    ui.showToast(`✓ 上传成功: ${fileName}`, 'success');
    await loadFiles();
  } else {
    ui.showToast(`上传失败: ${fileName}`, 'error', 5000);
  }
  isUploading.value = false;
};

// Join local path with an entry name, respecting OS separator
const joinLocalPath = (base: string, name: string): string => {
  const sep = base.includes('\\') ? '\\' : '/';
  return base.endsWith(sep) ? base + name : base + sep + name;
};

// Recursively upload a local directory tree to a remote base path
const uploadDirectoryRecursive = async (
  localPath: string,
  remotePath: string,
  counters: { ok: number; fail: number }
): Promise<void> => {
  if (!props.server) return;

  // Create remote directory
  try {
    await invoke('execute_remote_command', {
      serverName: props.server.name,
      command: `mkdir -p "${remotePath}"`
    });
    addLog(`📁 创建目录: ${remotePath}`, 'info');
  } catch (e) {
    addLog(`✗ 创建目录失败: ${remotePath} — ${String(e)}`, 'error');
    counters.fail++;
    return;
  }

  let entries: any[];
  try {
    entries = await readDir(localPath);
  } catch (e) {
    addLog(`✗ 读取目录失败: ${localPath} — ${String(e)}`, 'error');
    counters.fail++;
    return;
  }

  for (const entry of entries) {
    if (!entry.name) continue;
    const localEntry = joinLocalPath(localPath, entry.name);
    const remoteEntry = `${remotePath}/${entry.name}`;

    if (entry.isDirectory) {
      await uploadDirectoryRecursive(localEntry, remoteEntry, counters);
    } else if (entry.isFile) {
      const { finalPath, skip } = await resolvePathConflict(remoteEntry);
      if (skip) {
        addLog(`跳过文件: ${entry.name}`, 'info');
        continue;
      }

      addLog(`上传中: ${entry.name} → ${finalPath}`, 'info');
      try {
        const content = await readFile(localEntry);
        const ok = await uploadSingleFile(content, finalPath);
        ok ? counters.ok++ : counters.fail++;
      } catch (e) {
        addLog(`✗ 读取文件失败: ${localEntry} — ${String(e)}`, 'error');
        counters.fail++;
      }
    }
  }
};

const handleUpload = async () => {
  if (!props.server) return;

  try {
    const selected = await open({
      multiple: false,
      title: '选择上传文件',
    });

    if (selected && typeof selected === 'string') {
      const content = await readFile(selected);
      const fileName = selected.split(/[\\/]/).pop() || 'uploaded_file';
      await uploadFile(fileName, content);
    }
  } catch (error) {
    console.error('选择文件失败:', error);
  }
};

const handleDrop = async (e: DragEvent) => {
  isDragging.value = false;
  // Drag-drop in Tauri webview uses system paths via tauri://drag-drop event
  // This handler is kept for visual feedback only; actual upload is in the unlisten below
};

// Tauri drag-drop listener (provides file paths instead of File objects)
let unlistenDragDrop: (() => void) | null = null;

const setupDragDropListener = async () => {
  if (unlistenDragDrop) return;
  try {
    const webview = getCurrentWebview();
    unlistenDragDrop = await webview.onDragDropEvent(async (event) => {
      if (event.payload.type === 'over') {
        isDragging.value = true;
      } else if (event.payload.type === 'leave' || event.payload.type === 'cancelled') {
        isDragging.value = false;
      } else if (event.payload.type === 'drop') {
        isDragging.value = false;
        // Only process drop if this is the active server's FileBrowser
        if (serverStore.activeServerId !== props.server.id) return;
        if (!props.server || props.server.status !== 'online') return;
        const paths: string[] = event.payload.paths ?? [];
        let totalOk = 0;
        let totalFail = 0;
        isUploading.value = true;
        resetConflictState(); // Start fresh for new drop

        for (const filePath of paths) {
          try {
            const fileStat = await stat(filePath);
            if (fileStat.isDirectory) {
              const dirName = filePath.replace(/\\/g, '/').split('/').pop() || 'upload';
              const remoteDirPath = currentPath.value === '/'
                ? `/${dirName}`
                : `${currentPath.value}/${dirName}`;

              const { finalPath, skip } = await resolvePathConflict(remoteDirPath);
              if (skip) {
                addLog(`跳过目录: ${dirName}`, 'info');
                continue;
              }

              addLog(`📁 开始上传目录: ${dirName} → ${finalPath}`, 'info');
              const counters = { ok: 0, fail: 0 };
              await uploadDirectoryRecursive(filePath, finalPath, counters);
              totalOk += counters.ok;
              totalFail += counters.fail;
              addLog(`📁 目录 "${dirName}" 完成: 成功 ${counters.ok} 个，失败 ${counters.fail} 个`, counters.fail > 0 ? 'error' : 'success');
            } else if (fileStat.isFile) {
              const fileName = filePath.replace(/\\/g, '/').split('/').pop() || 'uploaded_file';
              const initialRemotePath = currentPath.value === '/'
                ? `/${fileName}`
                : `${currentPath.value}/${fileName}`;

              const { finalPath, skip } = await resolvePathConflict(initialRemotePath);
              if (skip) {
                addLog(`跳过文件: ${fileName}`, 'info');
                continue;
              }

              const content = await readFile(filePath);
              addLog(`上传中: ${fileName} (${formatSize(content.length)}) → ${finalPath}`, 'info');
              const ok = await uploadSingleFile(content, finalPath);
              ok ? totalOk++ : totalFail++;
              if (ok) ui.showToast(`✓ 上传成功: ${fileName}`, 'success');
            }
          } catch (err) {
            console.error('拖拽失败:', err);
            ui.showToast(`拖拽上传失败: ${String(err)}`, 'error', 5000);
            addLog(`✗ 拖拽失败: ${filePath} — ${String(err)}`, 'error');
            totalFail++;
          }
        }

        isUploading.value = false;
        if (totalOk > 0 || totalFail > 0) {
          addLog(`— 拖拽上传完成: 成功 ${totalOk} 个文件，失败 ${totalFail} 个`, totalFail > 0 ? 'error' : 'success');
        }
        await loadFiles();
      }
    });
  } catch (err) {
    console.error('无法注册拖拽监听:', err);
  }
};

const handleDownload = async (file: any) => {
  if (!props.server || file.is_dir) return;

  try {
    const savePath = await save({
      defaultPath: file.name,
      title: '保存文件',
    });

    if (savePath) {
      isDownloading.value = true;
      const remotePath = currentPath.value.endsWith('/')
        ? `${currentPath.value}${file.name}`
        : `${currentPath.value}/${file.name}`;

      const content = await invoke<number[]>('download_remote_file', {
        serverName: props.server.name,
        path: remotePath,
      });

      await writeFile(savePath, new Uint8Array(content));
    }
  } catch (error) {
    console.error('下载失败:', error);
  } finally {
    isDownloading.value = false;
  }
};

const handleDelete = async (file: any) => {
  if (!props.server) return;

  const confirm = await ui.showConfirm({ title: '删除项目', message: `确定要删除 ${file.name} 吗？`, type: 'danger' });
  if (!confirm) return;

  const remotePath = currentPath.value.endsWith('/')
    ? `${currentPath.value}${file.name}`
    : `${currentPath.value}/${file.name}`;

  addLog(`删除中: ${remotePath}${file.is_dir ? ' (文件夹)' : ''}`, 'info');
  try {
    await invoke('delete_remote_file', {
      serverName: props.server.name,
      path: remotePath,
      isDir: file.is_dir,
    });

    addLog(`✓ 已删除: ${remotePath}`, 'success');
    ui.showToast(`✓ 已删除: ${file.name}`, 'success');
    await loadFiles();
  } catch (error) {
    console.error('删除失败:', error);
    addLog(`✗ 删除失败: ${remotePath} — ${String(error)}`, 'error');
    ui.showToast(`删除失败: ${String(error)}`, 'error', 5000);
  }
};

const handleContextMenu = (e: MouseEvent, file: any = null) => {
  e.preventDefault();
  contextMenuPos.value = { x: e.clientX, y: e.clientY };
  contextMenuFile.value = file;
  showContextMenu.value = true;

  // Close menu when clicking elsewhere
  const closeMenu = () => {
    showContextMenu.value = false;
    window.removeEventListener('click', closeMenu);
  };
  window.addEventListener('click', closeMenu);
};

const copyPath = (file: any) => {
  const path = currentPath.value === '/'
    ? `/${file.name}`
    : `${currentPath.value}/${file.name}`;
  navigator.clipboard.writeText(path);
};

const handleRename = (file: any) => {
  inputModalConfig.value = {
    title: '重命名项目',
    defaultValue: file.name,
    placeholder: '请输入新名称',
    type: 'rename',
    target: file
  };
  showInputModal.value = true;
};

const handleNewFolder = () => {
  inputModalConfig.value = {
    title: '新建文件夹',
    defaultValue: '',
    placeholder: '请输入文件夹名称',
    type: 'newFolder',
    target: null
  };
  showInputModal.value = true;
};

const handleModalConfirm = async (value: string) => {
  if (!props.server) return;

  try {
    if (inputModalConfig.value.type === 'rename') {
      const file = inputModalConfig.value.target;
      const oldPath = currentPath.value === '/' ? `/${file.name}` : `${currentPath.value}/${file.name}`;
      const newPath = currentPath.value === '/' ? `/${value}` : `${currentPath.value}/${value}`;

      await invoke('execute_remote_command', {
        serverName: props.server.name,
        command: `mv "${oldPath}" "${newPath}"`
      });
      ui.showToast(`✓ 已重命名为: ${value}`, 'success');
    } else {
      const newDirPath = currentPath.value === '/' ? `/${value}` : `${currentPath.value}/${value}`;
      await invoke('execute_remote_command', {
        serverName: props.server.name,
        command: `mkdir -p "${newDirPath}"`
      });
      ui.showToast(`✓ 已创建文件夹: ${value}`, 'success');
    }
    await loadFiles();
  } catch (error) {
    console.error('操作失败:', error);
    ui.showToast(`操作失败: ${String(error)}`, 'error', 5000);
  } finally {
    showInputModal.value = false;
  }
};

// Drag state (visual only)
const isDragging = ref(false);

onMounted(() => {
  setupDragDropListener();
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop();
    unlistenDragDrop = null;
  }
  window.removeEventListener('keydown', handleKeyDown);
});

watch(() => props.activeTab, async (newTab) => {
  if (newTab === 'files') {
    // Do nothing, keep the current state to avoid unnecessary refreshes
  }
});

watch(() => props.server.status, (newStatus) => {
  if (newStatus === 'online') {
    loadFiles();
  } else {
    files.value = [];
    errorMessage.value = '服务器未连接';
  }
}, { immediate: true });
</script>

<template>
  <div class="flex flex-col h-full bg-[#0f172a]/30">
    <!-- Toolbar -->
    <div class="h-12 border-b border-slate-800 flex items-center px-4 space-x-4 bg-slate-900/50 backdrop-blur-md">
      <div class="flex items-center space-x-2">
        <button @click="goBack" :disabled="currentPath === '/'" class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-400 disabled:opacity-30 transition-all">
          <ChevronLeft :size="18" />
        </button>
        <button @click="navigateTo('/root')" class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-400 transition-all">
          <Home :size="18" />
        </button>
      </div>

      <div class="h-6 w-px bg-slate-800 mx-1"></div>

      <div class="flex items-center flex-1 min-w-0 bg-slate-800/30 px-3 py-1.5 rounded-lg border border-slate-700/50">
        <HardDrive :size="14" class="text-slate-500 mr-2 flex-shrink-0" />
        <span class="text-xs font-mono text-slate-300 truncate">{{ currentPath }}</span>
      </div>

      <div class="flex items-center space-x-2">
        <button
          @click="handleUpload"
          :disabled="isUploading || !props.server || props.server.status !== 'online'"
          class="flex items-center space-x-2 px-4 py-1.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white rounded-lg text-xs font-bold transition-all shadow-lg shadow-blue-600/20"
        >
          <Loader2 v-if="isUploading" class="animate-spin" :size="14" />
          <Upload v-else :size="14" />
          <span>上传</span>
        </button>
        <button @click="loadFiles" :disabled="isLoading" class="p-2 hover:bg-slate-800 rounded-lg text-slate-400 transition-all">
          <RefreshCw :class="{ 'animate-spin': isLoading }" :size="16" />
        </button>

        <div class="h-6 w-px bg-slate-800 mx-1"></div>

        <div class="flex items-center bg-slate-800/50 p-1 rounded-lg">
          <button
            @click="viewMode = 'grid'"
            :class="['p-1.5 rounded-md transition-all', viewMode === 'grid' ? 'bg-blue-600 text-white shadow-lg' : 'text-slate-500 hover:text-slate-300']"
          >
            <LayoutGrid :size="14" />
          </button>
          <button
            @click="viewMode = 'list'"
            :class="['p-1.5 rounded-md transition-all', viewMode === 'list' ? 'bg-blue-600 text-white shadow-lg' : 'text-slate-500 hover:text-slate-300']"
          >
            <List :size="14" />
          </button>
        </div>
      </div>
    </div>

    <!-- Empty State / Error State -->
    <div v-if="(!props.server || props.server.status !== 'online') || errorMessage" class="flex-1 flex flex-col items-center justify-center p-10 text-center">
      <div class="w-20 h-20 bg-slate-800/50 rounded-3xl flex items-center justify-center mb-6 border border-slate-700/50">
        <ShieldAlert v-if="errorMessage" :size="40" class="text-red-500" />
        <HardDrive v-else :size="40" class="text-slate-600" />
      </div>
      <h3 class="text-lg font-bold text-white mb-2">{{ errorMessage ? '获取文件失败' : '未准备就绪' }}</h3>
      <p class="text-slate-500 text-sm max-w-xs">{{ errorMessage || '请先在左侧选择并连接一个服务器来使用 SFTP 文件管理功能。' }}</p>
      <button v-if="errorMessage" @click="loadFiles" class="mt-4 px-6 py-2 bg-slate-800 hover:bg-slate-700 text-white rounded-lg text-xs transition-all">重试</button>
    </div>

    <!-- File List -->
    <div v-else
      @contextmenu="handleContextMenu($event)"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="handleDrop"
      @click.self="clearSelection"
      :class="['flex-1 overflow-y-auto p-4 custom-scrollbar relative transition-colors select-none', isDragging ? 'bg-blue-600/10' : '']"
    >
      <!-- Drag Overlay -->
      <div v-if="isDragging" class="absolute inset-0 z-10 flex items-center justify-center pointer-events-none">
        <div class="bg-blue-600 border border-blue-400 text-white px-8 py-4 rounded-3xl shadow-2xl flex flex-col items-center space-y-2 animate-in zoom-in duration-200">
          <Upload :size="48" class="animate-bounce" />
          <span class="text-lg font-bold">释放以上传到 {{ currentPath }}</span>
        </div>
      </div>

      <div v-if="isLoading && files.length === 0" class="h-full flex items-center justify-center">
        <Loader2 class="animate-spin text-blue-500" :size="32" />
      </div>

      <div v-else-if="files.length === 0" class="h-full flex flex-col items-center justify-center py-20 opacity-50">
        <Folder :size="48" class="text-slate-700 mb-4" />
        <span class="text-sm text-slate-500">目录为空</span>
      </div>

      <!-- Grid View -->
      <div v-else-if="viewMode === 'grid'" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 animate-in fade-in slide-in-from-bottom-4 duration-500">
        <!-- Parent Directory Entry -->
        <div v-if="currentPath !== '/'"
          @dblclick="goBack"
          class="group flex flex-col p-4 bg-slate-800/20 border border-dashed border-slate-700/50 rounded-2xl hover:border-blue-500/50 hover:bg-blue-500/5 transition-all cursor-pointer relative"
        >
          <div class="flex items-start mb-3">
            <div class="p-3 rounded-xl bg-slate-800 text-slate-500 group-hover:text-blue-400 transition-colors">
              <ChevronLeft :size="24" />
            </div>
          </div>
          <div class="min-w-0">
            <div class="text-sm font-bold text-slate-400 group-hover:text-white">..</div>
            <div class="text-[10px] text-slate-600 mt-1">返回上级目录</div>
          </div>
        </div>

        <div
          v-for="file in files"
          :key="file.name"
          @click.exact="toggleSelect(file.name, false)"
          @click.ctrl="toggleSelect(file.name, true)"
          @click.meta="toggleSelect(file.name, true)"
          @dblclick="file.is_dir ? handleFolderClick(file.name) : null"
          @contextmenu.stop="handleContextMenu($event, file)"
          :class="[
            'group flex flex-col p-4 border rounded-2xl transition-all cursor-pointer relative',
            isSelected(file.name)
              ? 'bg-blue-600/20 border-blue-500/70 ring-1 ring-blue-500/40'
              : 'bg-[#1e293b]/40 border-slate-700/50 hover:border-blue-500/50 hover:bg-blue-500/5'
          ]"
        >
          <div class="flex items-start justify-between mb-3">
            <div :class="['p-3 rounded-xl bg-slate-800 mr-3 transition-colors shadow-sm', file.is_dir ? 'text-amber-400 group-hover:bg-amber-400/10' : 'text-blue-400 group-hover:bg-blue-400/10']">
              <Folder v-if="file.is_dir" :size="24" />
              <File v-else :size="24" />
            </div>
          </div>

          <div class="min-w-0">
            <div class="text-sm font-bold truncate text-slate-200 group-hover:text-white transition-colors">{{ file.name }}</div>
            <div class="flex items-center mt-1 text-[10px] text-slate-500 space-x-2">
              <span>{{ file.is_dir ? '文件夹' : formatSize(file.size) }}</span>
              <span v-if="file.modified" class="opacity-30">•</span>
              <span v-if="file.modified">{{ new Date(file.modified * 1000).toLocaleDateString() }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- List View -->
      <div v-else class="flex flex-col bg-[#1e293b]/20 border border-slate-800 rounded-2xl overflow-hidden animate-in fade-in duration-500">
        <div class="flex items-center px-4 py-2 border-b border-slate-800 text-[10px] font-bold text-slate-500 bg-slate-900/50">
          <div class="flex-1">名称</div>
          <div class="w-24 text-right">大小</div>
          <div class="w-32 text-right">修改日期</div>
        </div>

        <!-- Parent Directory Item -->
        <div v-if="currentPath !== '/'" @dblclick="goBack" class="flex items-center px-4 py-3 hover:bg-slate-800/50 border-b border-slate-800/30 group transition-colors cursor-pointer italic text-slate-500">
          <div class="flex-1 flex items-center min-w-0">
            <div class="mr-3">
              <ChevronLeft :size="16" />
            </div>
            <span class="text-xs font-medium">.. (返回上级)</span>
          </div>
          <div class="w-24 text-right text-[10px]">-</div>
          <div class="w-32 text-right text-[10px]">-</div>
        </div>

        <div
          v-for="file in files"
          :key="file.name"
          @click.exact="toggleSelect(file.name, false)"
          @click.ctrl="toggleSelect(file.name, true)"
          @click.meta="toggleSelect(file.name, true)"
          @dblclick="file.is_dir ? handleFolderClick(file.name) : null"
          @contextmenu.stop="handleContextMenu($event, file)"
          :class="[
            'flex items-center px-4 py-3 border-b border-slate-800/30 group transition-colors cursor-pointer',
            isSelected(file.name)
              ? 'bg-blue-600/20'
              : 'hover:bg-slate-800/50'
          ]"
        >
          <div class="flex-1 flex items-center min-w-0">
            <div :class="['mr-3', file.is_dir ? 'text-amber-400' : 'text-blue-400']">
              <Folder v-if="file.is_dir" :size="16" />
              <File v-else :size="16" />
            </div>
            <span class="text-xs font-medium text-slate-200 truncate">{{ file.name }}</span>
          </div>
          <div class="w-24 text-right text-[10px] text-slate-500">{{ file.is_dir ? '-' : formatSize(file.size) }}</div>
          <div class="w-32 text-right text-[10px] text-slate-500">{{ file.modified ? new Date(file.modified * 1000).toLocaleDateString() : '-' }}</div>
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <div v-if="props.server && props.server.status === 'online'" class="h-8 border-t border-slate-800 bg-slate-900/80 px-4 flex items-center justify-between flex-shrink-0">
      <div class="text-[10px] text-slate-500 flex items-center">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 mr-2"></span>
        已连接: {{ props.server.name }}
      </div>
      <div class="text-[10px] flex items-center space-x-3">
        <span v-if="selectedFiles.size > 0" class="text-blue-400 font-medium">
          已选 {{ selectedFiles.size }} 个
        </span>
        <span class="text-slate-500">{{ files.length }} 个项目</span>
      </div>
    </div>

    <!-- Operation Log Panel -->
    <div class="flex-shrink-0 border-t border-slate-800 bg-[#0a0f1a]">
      <!-- Log Header -->
      <div
        class="flex items-center justify-between px-3 py-1.5 cursor-pointer select-none hover:bg-slate-800/40 transition-colors"
        @click="logPanelOpen = !logPanelOpen"
      >
        <div class="flex items-center space-x-2 text-slate-400">
          <Terminal :size="12" />
          <span class="text-[10px] font-bold tracking-widest uppercase">操作日志</span>
          <span v-if="logs.length" class="text-[9px] bg-slate-700 text-slate-400 rounded px-1.5 py-0.5">{{ logs.length }}</span>
        </div>
        <div class="flex items-center space-x-2">
          <button
            v-if="logs.length"
            @click.stop="clearLogs"
            class="text-[9px] text-slate-600 hover:text-slate-400 transition-colors px-1.5 py-0.5 rounded hover:bg-slate-800"
          >清空</button>
          <ChevronDown
            :size="12"
            :class="['text-slate-600 transition-transform duration-200', logPanelOpen ? 'rotate-180' : '']"
          />
        </div>
      </div>

      <!-- Log Body -->
      <div
        v-show="logPanelOpen"
        ref="logContainer"
        class="h-28 overflow-y-auto px-3 pb-2 font-mono text-[11px] leading-5 space-y-0.5 log-scrollbar"
      >
        <div v-if="!logs.length" class="flex items-center justify-center h-full text-slate-700 text-[10px]">
          暂无操作记录
        </div>
        <div
          v-for="entry in logs"
          :key="entry.id"
          :class="[
            'flex items-start space-x-2 py-0.5',
            entry.level === 'success' ? 'text-emerald-400' : entry.level === 'error' ? 'text-red-400' : 'text-slate-500'
          ]"
        >
          <span class="flex-shrink-0 text-slate-700">{{ entry.time }}</span>
          <span class="break-all">{{ entry.message }}</span>
        </div>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showContextMenu"
        class="fixed z-[100] w-48 bg-[#1e293b]/95 backdrop-blur-xl border border-slate-700 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
        <template v-if="contextMenuFile">
          <button v-if="contextMenuFile.is_dir" @click="handleFolderClick(contextMenuFile.name)" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Folder :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">进入文件夹</span>
          </button>
          <button v-else @click="handleDownload(contextMenuFile)" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Download :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">下载文件</span>
          </button>

          <div class="h-px bg-slate-700/50 my-1"></div>

          <button @click="copyPath(contextMenuFile)" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Copy :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">复制绝对路径</span>
          </button>

          <button @click="handleRename(contextMenuFile)" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <RefreshCw :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">重命名</span>
          </button>

          <div class="h-px bg-slate-700/50 my-1"></div>

          <button @click="handleDelete(contextMenuFile)" class="w-full text-left px-3 py-2 text-xs hover:bg-red-600/90 text-red-500 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Trash2 :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">删除项目</span>
          </button>
        </template>
        <template v-else>
          <button @click="handleNewFolder" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Plus :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">新建文件夹</span>
          </button>
          <button @click="handleUpload" class="w-full text-left px-3 py-2 text-xs hover:bg-blue-600 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Upload :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">上传文件</span>
          </button>
          <button @click="loadFiles" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <RefreshCw :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">刷新列表</span>
          </button>

          <div class="h-px bg-slate-700/50 my-1"></div>

          <button @click="navigateTo('/root')" class="w-full text-left px-3 py-2 text-xs hover:bg-slate-800/80 text-slate-300 hover:text-white transition-all flex items-center space-x-2 group/item">
            <Home :size="14" class="group-hover/item:scale-110 transition-transform" />
            <span class="group-hover/item:translate-x-1 transition-transform">回到 Root 目录</span>
          </button>
        </template>
      </div>
    </Teleport>

    <!-- Input Modal -->
    <InputModal
      v-if="showInputModal"
      :show="showInputModal"
      :title="inputModalConfig.title"
      :defaultValue="inputModalConfig.defaultValue"
      :placeholder="inputModalConfig.placeholder"
      @close="showInputModal = false"
      @confirm="handleModalConfirm"
    />
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #1e293b;
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #334155;
}
.log-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.log-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.log-scrollbar::-webkit-scrollbar-thumb {
  background: #1e293b;
  border-radius: 4px;
}
.log-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #334155;
}
</style>
