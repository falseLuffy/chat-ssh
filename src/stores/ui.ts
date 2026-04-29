import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
}

export interface ConfirmOptions {
  title?: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'danger' | 'warning' | 'info';
}

export type ConflictAction = 'overwrite' | 'skip' | 'rename';

export interface ConflictResult {
  action: ConflictAction;
  applyToAll: boolean;
}

export interface ConflictOptions {
  fileName: string;
  message?: string;
}

export const useUIStore = defineStore('ui', () => {
  const toasts = ref<Toast[]>([]);
  let toastIdCounter = 0;

  const showToast = (message: string, type: ToastType = 'info', duration = 3000) => {
    const id = `toast_${++toastIdCounter}`;
    toasts.value.push({ id, message, type });
    setTimeout(() => {
      removeToast(id);
    }, duration);
  };

  const removeToast = (id: string) => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  };

  const confirmState = ref<{
    isOpen: boolean;
    options: ConfirmOptions;
    resolve: ((value: boolean) => void) | null;
  }>({
    isOpen: false,
    options: { message: '' },
    resolve: null
  });

  const showConfirm = (options: ConfirmOptions | string): Promise<boolean> => {
    const opts = typeof options === 'string' ? { message: options } : options;
    return new Promise((resolve) => {
      confirmState.value = {
        isOpen: true,
        options: opts,
        resolve
      };
    });
  };

  const resolveConfirm = (result: boolean) => {
    if (confirmState.value.resolve) {
      confirmState.value.resolve(result);
    }
    confirmState.value.isOpen = false;
    confirmState.value.resolve = null;
  };

  const conflictState = ref<{
    isOpen: boolean;
    options: ConflictOptions;
    resolve: ((value: ConflictResult) => void) | null;
  }>({
    isOpen: false,
    options: { fileName: '' },
    resolve: null
  });

  const showConflict = (options: ConflictOptions): Promise<ConflictResult> => {
    return new Promise((resolve) => {
      conflictState.value = {
        isOpen: true,
        options,
        resolve
      };
    });
  };

  const resolveConflict = (action: ConflictAction, applyToAll: boolean) => {
    if (conflictState.value.resolve) {
      conflictState.value.resolve({ action, applyToAll });
    }
    conflictState.value.isOpen = false;
    conflictState.value.resolve = null;
  };

  return {
    toasts,
    showToast,
    removeToast,
    confirmState,
    showConfirm,
    resolveConfirm,
    conflictState,
    showConflict,
    resolveConflict
  };
});
