import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface Server {
  id: number;
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  status: 'online' | 'offline' | 'connecting';
}

export const useServerStore = defineStore('server', () => {
  const servers = ref<Server[]>([]);
  
  // Load servers from localStorage on initialization
  const savedServers = localStorage.getItem('ssh_servers');
  if (savedServers) {
    try {
      const parsed = JSON.parse(savedServers);
      servers.value = parsed.map((s: any) => ({ ...s, status: 'offline' }));
    } catch (e) {
      console.error('Failed to parse saved servers:', e);
      servers.value = [];
    }
  }

  const activeServerId = ref<number | null>(null);
  const activeServer = ref<Server | null>(null);

  const saveToStorage = () => {
    localStorage.setItem('ssh_servers', JSON.stringify(servers.value));
  };

  const setActiveServer = (id: number) => {
    activeServerId.value = id;
    activeServer.value = servers.value.find(s => s.id === id) || null;
  };

  const addServer = (server: Omit<Server, 'id' | 'status'>) => {
    const newId = servers.value.length > 0 ? Math.max(...servers.value.map(s => s.id)) + 1 : 1;
    const newServer: Server = {
      ...server,
      id: newId,
      status: 'offline',
    };
    servers.value.push(newServer);
    saveToStorage();
    return newServer;
  };

  const updateServer = (id: number, updatedData: Partial<Server>) => {
    const index = servers.value.findIndex(s => s.id === id);
    if (index !== -1) {
      servers.value[index] = { ...servers.value[index], ...updatedData };
      if (activeServerId.value === id) {
        activeServer.value = servers.value[index];
      }
      saveToStorage();
    }
  };

  const deleteServer = (id: number) => {
    servers.value = servers.value.filter(s => s.id !== id);
    if (activeServerId.value === id) {
      activeServerId.value = null;
      activeServer.value = null;
    }
    saveToStorage();
  };

  const updateStatus = (id: number, status: 'online' | 'offline' | 'connecting') => {
    const index = servers.value.findIndex(s => s.id === id);
    if (index !== -1) {
      servers.value[index].status = status;
      if (status === 'online') {
        const [server] = servers.value.splice(index, 1);
        servers.value.unshift(server);
        saveToStorage();
      }
    }
  };

  const connectServer = async (server: Server) => {
    try {
      updateStatus(server.id, 'connecting');
      await invoke('connect_ssh', {
        name: server.name,
        host: server.port ? `${server.host}:${server.port}` : server.host,
        user: server.username,
        pass: server.password || '',
      });
      await invoke('open_terminal', { 
        serverName: server.name,
        cols: 80,
        rows: 24
      });
      updateStatus(server.id, 'online');
      return { success: true };
    } catch (e) {
      updateStatus(server.id, 'offline');
      return { success: false, error: String(e) };
    }
  };

  const disconnectServer = async (id: number) => {
    const server = servers.value.find(s => s.id === id);
    if (server) {
      try {
        await invoke('disconnect_ssh', { name: server.name });
      } catch (e) {
        console.error('Failed to disconnect on backend:', e);
      }
      updateStatus(id, 'offline');
    }
  };

  return {
    servers,
    activeServerId,
    activeServer,
    setActiveServer,
    addServer,
    updateServer,
    deleteServer,
    updateStatus,
    connectServer,
    disconnectServer,
  };
});
