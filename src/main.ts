import { createApp } from "vue";
import { createPinia } from "pinia";
import "./assets/main.css";
import App from "./App.vue";

const app = createApp(App);
app.use(createPinia());
app.mount("#app");

// 全局禁止浏览器默认快捷键（开发环境保留 F12 / Ctrl+Shift+I 等开发者工具）
const isDev = import.meta.env.DEV;

document.addEventListener('keydown', (e) => {
  const isCtrl = e.ctrlKey || e.metaKey;
  const isAlt = e.altKey;

  // 开发环境保留开发者工具快捷键
  if (isDev) {
    const keyRaw = e.key;
    if (keyRaw === 'F12') return;
    if (isCtrl && e.shiftKey && ['i', 'j', 'c'].includes(keyRaw.toLowerCase())) return;
  }

  // 无修饰键的普通按键
  if (!isCtrl && !isAlt) {
    // 阻止功能键 (F1-F12)
    if (e.key.startsWith('F') && !isNaN(parseInt(e.key.substring(1)))) {
      e.preventDefault();
    }
    return;
  }

  const key = e.key.toLowerCase();

  // 允许标准编辑快捷键
  if (isCtrl && !isAlt && ['c', 'v', 'x', 'a', 'z', 'y', 'backspace', 'delete', 'insert'].includes(key)) {
    return;
  }

  // 允许 Ctrl+方向键/Home/End (导航)
  if (isCtrl && !isAlt && (e.key.startsWith('Arrow') || ['home', 'end'].includes(key))) {
    return;
  }

  // 阻止所有其他带修饰键的快捷键（浏览器默认行为：Ctrl+S, Ctrl+F, Ctrl+R, F5 等）
  e.preventDefault();
}, { capture: true });
