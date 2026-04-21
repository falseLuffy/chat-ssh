import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import staticKnowledge from '../assets/knowledge.json';

export interface KnowledgeItem {
  command: string;
  description: string;
  tags: string[];
  isUser?: boolean;
}

export const useKnowledgeStore = defineStore('knowledge', () => {
  const userKnowledge = ref<KnowledgeItem[]>(JSON.parse(localStorage.getItem('user_knowledge_base') || '[]'));

  const combinedKnowledge = computed(() => {
    return [...staticKnowledge, ...userKnowledge.value];
  });

  const saveToLocalStorage = () => {
    localStorage.setItem('user_knowledge_base', JSON.stringify(userKnowledge.value));
  };

  const addCommand = (command: string, description: string) => {
    const normalizedCmd = command.trim();
    
    // Deduplication check
    const exists = combinedKnowledge.value.some(
      item => item.command.trim().toLowerCase() === normalizedCmd.toLowerCase()
    );

    if (exists) {
      return { success: false, message: '该指令已存在于知识库中' };
    }

    // Basic auto-tagging: extract words from description
    const tags = Array.from(new Set([
      ...description.toLowerCase().split(/[\s,，.。!！?？]+/).filter(w => w.length > 1),
      ...normalizedCmd.split(/\s+/).filter(w => w.length > 1 && !w.startsWith('-'))
    ]));

    userKnowledge.value.push({
      command: normalizedCmd,
      description,
      tags,
      isUser: true
    });

    saveToLocalStorage();
    return { success: true, message: '指令已存入本地知识库' };
  };

  const searchLocal = (prompt: string) => {
    if (!prompt.trim()) return null;

    const words = prompt.toLowerCase().split(/[\s,，.。!！?？]+/).filter(w => w.length > 0);
    let bestMatch: KnowledgeItem | null = null;
    let highestScore = 0;

    for (const item of combinedKnowledge.value) {
      let score = 0;
      
      for (const word of words) {
        // Match in tags (high priority)
        if (item.tags.some(tag => tag.toLowerCase().includes(word))) {
          score += 2;
        }
        // Match in description
        if (item.description.toLowerCase().includes(word)) {
          score += 1;
        }
        // Match in command
        if (item.command.toLowerCase().includes(word)) {
          score += 1;
        }
      }

      // Normalize score by prompt word count and item tags
      const normalizedScore = score / words.length;

      if (normalizedScore > highestScore) {
        highestScore = normalizedScore;
        bestMatch = item;
      }
    }

    // Confidence threshold
    if (highestScore >= 0.8) {
      return bestMatch;
    }

    return null;
  };

  return {
    userKnowledge,
    combinedKnowledge,
    addCommand,
    searchLocal
  };
});
