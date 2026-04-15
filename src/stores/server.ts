import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface Server {
  id: number;
  name: string;
  host: string;
  username: string;
  status: 'online' | 'offline' | 'connecting';
}

export const useServerStore = defineStore('server', () => {
  const servers = ref<Server[]>([
    { id: 1, name: 'Local WSL', host: '127.0.0.1', username: 'root', status: 'online' },
  ]);

  const activeServerId = ref<number | null>(1);
  const activeServer = ref<Server | null>(servers.value[0]);

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
    return newServer;
  };

  const updateServer = (id: number, updatedData: Partial<Server>) => {
    const index = servers.value.findIndex(s => s.id === id);
    if (index !== -1) {
      servers.value[index] = { ...servers.value[index], ...updatedData };
      if (activeServerId.value === id) {
        activeServer.value = servers.value[index];
      }
    }
  };

  const updateStatus = (id: number, status: 'online' | 'offline' | 'connecting') => {
    const server = servers.value.find(s => s.id === id);
    if (server) server.status = status;
  };

  return {
    servers,
    activeServerId,
    activeServer,
    setActiveServer,
    updateStatus,
  };
});
