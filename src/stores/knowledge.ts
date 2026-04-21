import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import staticKnowledge from '../assets/knowledge.json';

export interface KnowledgeItem {
  id?: number;
  command: string;
  description: string;
  tags: string[];
  isUser?: boolean;
}

export const useKnowledgeStore = defineStore('knowledge', () => {
  const userKnowledge = ref<KnowledgeItem[]>([]);
  const isLoaded = ref(false);

  const combinedKnowledge = computed(() => {
    return [...staticKnowledge, ...userKnowledge.value];
  });

  const initStore = async () => {
    if (isLoaded.value) return;
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      
      // 0. Ensure table exists (Failsafe)
      await db.execute(`
        CREATE TABLE IF NOT EXISTS user_knowledge (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          command TEXT NOT NULL,
          description TEXT,
          tags TEXT,
          created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
      `);
      
      // 1. Check if migration from localStorage is needed
      const oldData = localStorage.getItem('user_knowledge_base');
      if (oldData) {
        try {
          const parsed: KnowledgeItem[] = JSON.parse(oldData);
          for (const item of parsed) {
            await db.execute(
              'INSERT INTO user_knowledge (command, description, tags) VALUES (?, ?, ?)',
              [item.command, item.description, item.tags.join(',')]
            );
          }
          localStorage.removeItem('user_knowledge_base');
          console.log('Migrated knowledge from localStorage to SQL');
        } catch (e) {
          console.error('Failed to migrate knowledge:', e);
        }
      }

      // 2. Load from Database
      const result = await db.select<{ id: number, command: string, description: string, tags: string }[]>(
        'SELECT * FROM user_knowledge ORDER BY created_at DESC'
      );
      
      userKnowledge.value = result.map(item => ({
        id: item.id,
        command: item.command,
        description: item.description,
        tags: item.tags.split(',').filter(t => t),
        isUser: true
      }));

      isLoaded.value = true;
    } catch (error) {
      console.error('Failed to init knowledge store:', error);
    }
  };

  const addCommand = async (command: string, description: string) => {
    if (!isLoaded.value) await initStore();
    
    const normalizedCmd = command.trim();
    const exists = combinedKnowledge.value.some(
      item => item.command.trim().toLowerCase() === normalizedCmd.toLowerCase()
    );

    if (exists) {
      // Don't alert if command exists, just treat as success
      return { success: true, message: '该指令已在知识库中' };
    }

    const tagsArr = Array.from(new Set([
      ...description.toLowerCase().split(/[\s,，.。!！?？]+/).filter(w => w.length > 1),
      ...normalizedCmd.split(/\s+/).filter(w => w.length > 1 && !w.startsWith('-'))
    ]));

    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      const result = await db.execute(
        'INSERT INTO user_knowledge (command, description, tags) VALUES (?, ?, ?)',
        [normalizedCmd, description, tagsArr.join(',')]
      );

      userKnowledge.value.unshift({
        id: result.lastInsertId,
        command: normalizedCmd,
        description,
        tags: tagsArr,
        isUser: true
      });

      return { success: true, message: '指令已存入本地数据库' };
    } catch (error) {
      console.error('Failed to save command to SQL:', error);
      return { success: false, message: '数据库保存失败: ' + (error instanceof Error ? error.message : String(error)) };
    }
  };

  const deleteCommand = async (id: number) => {
    try {
      const db = await Database.load('sqlite:chat_ssh.db');
      await db.execute('DELETE FROM user_knowledge WHERE id = ?', [id]);
      userKnowledge.value = userKnowledge.value.filter(item => item.id !== id);
      return { success: true };
    } catch (error) {
      console.error('Failed to delete command:', error);
      return { success: false };
    }
  };

  const searchLocal = (prompt: string) => {
    if (!prompt.trim()) return null;

    const words = prompt.toLowerCase().split(/[\s,，.。!！?？]+/).filter(w => w.length > 0);
    let bestMatch: KnowledgeItem | null = null;
    let highestScore = 0;

    for (const item of combinedKnowledge.value) {
      let score = 0;
      for (const word of words) {
        if (item.tags.some(tag => tag.toLowerCase().includes(word))) score += 2;
        if (item.description.toLowerCase().includes(word)) score += 1;
        if (item.command.toLowerCase().includes(word)) score += 1;
      }

      const normalizedScore = score / words.length;
      if (normalizedScore > highestScore) {
        highestScore = normalizedScore;
        bestMatch = item;
      }
    }

    if (highestScore >= 0.8) return bestMatch;
    return null;
  };

  return {
    userKnowledge,
    combinedKnowledge,
    isLoaded,
    initStore,
    addCommand,
    deleteCommand,
    searchLocal
  };
});
