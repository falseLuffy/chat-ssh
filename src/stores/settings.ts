import { defineStore } from 'pinia';
import { ref } from 'vue';
import Database from '@tauri-apps/plugin-sql';

export const useSettingsStore = defineStore('settings', () => {
  const deepseekApiKey = ref('');
  const selectedModel = ref('deepseek-chat');
  const aiMode = ref<'auto' | 'local' | 'ai'>('auto');
  const isLoaded = ref(false);

  const loadSettings = async () => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      const result = await db.select<{ key: string, value: string }[]>('SELECT key, value FROM config');
      
      for (const item of result) {
        if (item.key === 'deepseek_api_key') deepseekApiKey.value = item.value;
        if (item.key === 'deepseek_model') selectedModel.value = item.value;
        if (item.key === 'ai_mode') aiMode.value = item.value as 'auto' | 'local' | 'ai';
      }
      isLoaded.value = true;
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  };

  const saveSettings = async (apiKey: string, model: string, mode: 'auto' | 'local' | 'ai') => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      await db.execute('UPDATE config SET value = ? WHERE key = ?', [apiKey, 'deepseek_api_key']);
      await db.execute('UPDATE config SET value = ? WHERE key = ?', [model, 'deepseek_model']);
      await db.execute('UPDATE config SET value = ? WHERE key = ?', [mode, 'ai_mode']);
      
      deepseekApiKey.value = apiKey;
      selectedModel.value = model;
      aiMode.value = mode;
    } catch (error) {
      console.error('Failed to save settings:', error);
      throw error;
    }
  };

  return {
    deepseekApiKey,
    selectedModel,
    aiMode,
    isLoaded,
    loadSettings,
    saveSettings,
  };
});
