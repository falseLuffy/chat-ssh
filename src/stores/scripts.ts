import { defineStore } from 'pinia';
import { ref } from 'vue';
import Database from '@tauri-apps/plugin-sql';

export interface Script {
  id: number;
  name: string;
  description: string;
  content: string;
  skip_warning: boolean;
  created_at?: string;
}

export const useScriptsStore = defineStore('scripts', () => {
  const scripts = ref<Script[]>([]);
  const isLoaded = ref(false);
  const masterPassword = ref('');

  const initStore = async () => {
    if (isLoaded.value) return;
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      
      // Failsafe: Ensure table exists in case backend migrations failed or skipped
      await db.execute(`
        CREATE TABLE IF NOT EXISTS scripts (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          description TEXT,
          content TEXT NOT NULL,
          skip_warning BOOLEAN DEFAULT 0,
          created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
      `);
      
      await db.execute("INSERT OR IGNORE INTO config (key, value) VALUES ('master_password', '')");

      const result = await db.select<any[]>('SELECT * FROM scripts ORDER BY created_at DESC');
      scripts.value = result.map(s => ({
        id: s.id,
        name: s.name,
        description: s.description || '',
        content: s.content,
        skip_warning: s.skip_warning === 1 || s.skip_warning === true,
        created_at: s.created_at
      }));

      // Load master password
      const configRes = await db.select<{value: string}[]>('SELECT value FROM config WHERE key = ?', ['master_password']);
      if (configRes.length > 0) {
        masterPassword.value = configRes[0].value;
      }

      isLoaded.value = true;
    } catch (error) {
      console.error('Failed to init scripts store:', error);
    }
  };

  const addScript = async (script: Omit<Script, 'id' | 'created_at'>) => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      const result = await db.execute(
        'INSERT INTO scripts (name, description, content, skip_warning) VALUES (?, ?, ?, ?)',
        [script.name, script.description, script.content, script.skip_warning ? 1 : 0]
      );
      const newScript: Script = {
        ...script,
        id: result.lastInsertId
      };
      scripts.value.unshift(newScript);
      return newScript;
    } catch (e) {
      console.error('Failed to add script:', e);
      throw e;
    }
  };

  const updateScript = async (id: number, updatedData: Partial<Script>) => {
    const index = scripts.value.findIndex(s => s.id === id);
    if (index !== -1) {
      try {
        const db = await Database.load('sqlite:chat_ssh.db');
        const s = { ...scripts.value[index], ...updatedData };
        await db.execute(
          'UPDATE scripts SET name = ?, description = ?, content = ?, skip_warning = ? WHERE id = ?',
          [s.name, s.description, s.content, s.skip_warning ? 1 : 0, id]
        );
        scripts.value[index] = s;
      } catch (e) {
        console.error('Failed to update script:', e);
        throw e;
      }
    }
  };

  const deleteScript = async (id: number) => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      await db.execute('DELETE FROM scripts WHERE id = ?', [id]);
      scripts.value = scripts.value.filter(s => s.id !== id);
    } catch (e) {
      console.error('Failed to delete script:', e);
      throw e;
    }
  };

  const setMasterPassword = async (password: string) => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      await db.execute('UPDATE config SET value = ? WHERE key = ?', [password, 'master_password']);
      masterPassword.value = password;
    } catch (e) {
      console.error('Failed to set master password:', e);
      throw e;
    }
  };

  const verifyPassword = (password: string) => {
    return masterPassword.value === password;
  };

  return {
    scripts,
    isLoaded,
    masterPassword,
    initStore,
    addScript,
    updateScript,
    deleteScript,
    setMasterPassword,
    verifyPassword
  };
});
