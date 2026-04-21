import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Database from '@tauri-apps/plugin-sql';

export interface Server {
  id: number;
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  auth_type: 'password' | 'key';
  auth_secret?: string;
  status: 'online' | 'offline' | 'connecting';
}

export const useServerStore = defineStore('server', () => {
  const servers = ref<Server[]>([]);
  const isLoaded = ref(false);
  const activeServerId = ref<number | null>(null);
  const activeServer = ref<Server | null>(null);

  const initStore = async () => {
    if (isLoaded.value) return;
    try {
      const db = await Database.load('sqlite:chat_ssh.db');

      // 0. Ensure table exists (Failsafe)
      await db.execute(`
        CREATE TABLE IF NOT EXISTS servers (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          host TEXT NOT NULL,
          username TEXT NOT NULL,
          port INTEGER NOT NULL DEFAULT 22,
          auth_type TEXT NOT NULL DEFAULT 'password',
          auth_secret TEXT,
          created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
      `);

      // 1. Migration from localStorage
      const savedServers = localStorage.getItem('ssh_servers');
      if (savedServers) {
        try {
          const parsed = JSON.parse(savedServers);
          for (const s of parsed) {
            await db.execute(
              'INSERT INTO servers (name, host, username, port, auth_type, auth_secret) VALUES (?, ?, ?, ?, ?, ?)',
              [s.name, s.host, s.username, s.port || 22, 'password', s.password || '']
            );
          }
          localStorage.removeItem('ssh_servers');
          console.log('Migrated servers from localStorage to SQL');
        } catch (e) {
          console.error('Failed to migrate servers:', e);
        }
      }

      // 2. Load from Database
      const result = await db.select<any[]>('SELECT * FROM servers ORDER BY created_at DESC');
      servers.value = result.map(s => ({
        id: s.id,
        name: s.name,
        host: s.host,
        username: s.username,
        port: s.port,
        auth_type: s.auth_type as 'password' | 'key',
        auth_secret: s.auth_secret,
        password: s.auth_type === 'password' ? s.auth_secret : undefined,
        status: 'offline'
      }));

      isLoaded.value = true;
    } catch (error) {
      console.error('Failed to init server store:', error);
    }
  };

  const setActiveServer = (id: number) => {
    activeServerId.value = id;
    activeServer.value = servers.value.find(s => s.id === id) || null;
  };

  const addServer = async (server: Omit<Server, 'id' | 'status'>) => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      const result = await db.execute(
        'INSERT INTO servers (name, host, username, port, auth_type, auth_secret) VALUES (?, ?, ?, ?, ?, ?)',
        [server.name, server.host, server.username, server.port, server.auth_type || 'password', server.auth_secret || server.password || '']
      );

      const newServer: Server = {
        ...server,
        id: result.lastInsertId,
        status: 'offline',
      };
      servers.value.unshift(newServer);
      return newServer;
    } catch (e) {
      console.error('Failed to add server to SQL:', e);
      throw e;
    }
  };

  const updateServer = async (id: number, updatedData: Partial<Server>) => {
    const index = servers.value.findIndex(s => s.id === id);
    if (index !== -1) {
      try {
        const db = await Database.load('sqlite:chat_ssh.db');
        const s = { ...servers.value[index], ...updatedData };
        await db.execute(
          'UPDATE servers SET name = ?, host = ?, username = ?, port = ?, auth_type = ?, auth_secret = ? WHERE id = ?',
          [s.name, s.host, s.username, s.port, s.auth_type, s.auth_secret || s.password, id]
        );
        
        servers.value[index] = s;
        if (activeServerId.value === id) {
          activeServer.value = s;
        }
      } catch (e) {
        console.error('Failed to update server in SQL:', e);
      }
    }
  };

  const deleteServer = async (id: number) => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      await db.execute('DELETE FROM servers WHERE id = ?', [id]);
      servers.value = servers.value.filter(s => s.id !== id);
      if (activeServerId.value === id) {
        activeServerId.value = null;
        activeServer.value = null;
      }
    } catch (e) {
      console.error('Failed to delete server from SQL:', e);
    }
  };

  const updateStatus = (id: number, status: 'online' | 'offline' | 'connecting') => {
    const index = servers.value.findIndex(s => s.id === id);
    if (index !== -1) {
      servers.value[index].status = status;
    }
  };

  const connectServer = async (server: Server) => {
    try {
      updateStatus(server.id, 'connecting');
      await invoke('connect_ssh', {
        name: server.name,
        host: server.port ? `${server.host}:${server.port}` : server.host,
        user: server.username,
        pass: server.auth_secret || server.password || '',
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
    isLoaded,
    activeServerId,
    activeServer,
    initStore,
    setActiveServer,
    addServer,
    updateServer,
    deleteServer,
    updateStatus,
    connectServer,
    disconnectServer,
  };
});
