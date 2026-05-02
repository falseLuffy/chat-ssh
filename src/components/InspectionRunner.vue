<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../stores/settings';
import type { Server } from '../stores/server';
import {
  SearchCheck, Loader2, CheckCircle2, AlertTriangle, XCircle,
  Clock, Sparkles, ShieldCheck, ChevronDown, ChevronUp, Copy, Check,
  Cpu, MemoryStick, HardDrive, Box, Settings, Activity, Wifi, Plus,
  History
} from 'lucide-vue-next';
import MarkdownIt from 'markdown-it';
import { loadRules, createReport, insertCheckResult, finalizeReport, saveReportSummary } from '../utils/db';

const props = defineProps<{ server: Server }>();
const emit = defineEmits(['viewHistory']);
const settingsStore = useSettingsStore();

const isRunning = ref(false);
const isGeneratingSummary = ref(false);
const checks = ref<CheckItem[]>([]);
const summary = ref('');
const reportId = ref<number | null>(null);
const isCopied = ref(false);
const expandedChecks = ref<Set<number>>(new Set());

interface CheckItem {
  ruleName: string;
  category: string;
  status: 'pending' | 'running' | 'pass' | 'warning' | 'critical' | 'error' | 'skipped';
  message: string;
  detail?: string;
  index: number;
  ruleId?: number | null;
}

const categoryIcons: Record<string, any> = {
  cpu: Cpu,
  memory: MemoryStick,
  disk: HardDrive,
  docker: Box,
  service: Settings,
  process: Activity,
  network: Wifi,
  custom: Plus,
};

const statusConfig: Record<string, { icon: any; color: string; bg: string; label: string }> = {
  pending: { icon: Clock, color: 'text-slate-500', bg: 'bg-slate-800/50', label: '等待中' },
  running: { icon: Loader2, color: 'text-blue-400', bg: 'bg-blue-500/10', label: '执行中' },
  pass: { icon: CheckCircle2, color: 'text-emerald-400', bg: 'bg-emerald-500/10', label: '正常' },
  warning: { icon: AlertTriangle, color: 'text-amber-400', bg: 'bg-amber-500/10', label: '警告' },
  critical: { icon: XCircle, color: 'text-red-400', bg: 'bg-red-500/10', label: '严重' },
  error: { icon: XCircle, color: 'text-red-400', bg: 'bg-red-500/10', label: '错误' },
};

const progress = computed(() => {
  if (checks.value.length === 0) return 0;
  const done = checks.value.filter(c => c.status !== 'pending' && c.status !== 'running').length;
  return Math.round((done / checks.value.length) * 100);
});

const progressText = computed(() => {
  const done = checks.value.filter(c => c.status !== 'pending' && c.status !== 'running').length;
  return `${done} / ${checks.value.length}`;
});

const overallResult = computed(() => {
  if (checks.value.length === 0) return null;
  if (checks.value.some(c => c.status === 'critical')) return { icon: XCircle, color: 'text-red-400', label: '严重异常' };
  if (checks.value.some(c => c.status === 'warning')) return { icon: AlertTriangle, color: 'text-amber-400', label: '存在警告' };
  if (checks.value.every(c => c.status === 'pass')) return { icon: CheckCircle2, color: 'text-emerald-400', label: '全部正常' };
  return null;
});

const md = new MarkdownIt({ html: false, breaks: true });
const renderedSummary = computed(() => {
  if (!summary.value) return '';
  return md.render(summary.value);
});

const startInspection = async () => {
  if (!props.server) return;

  isRunning.value = true;
  isGeneratingSummary.value = false;
  checks.value = [];
  summary.value = '';
  reportId.value = null;
  isCopied.value = false;
  expandedChecks.value = new Set();

  try {
    // 1. Load rules from local DB
    const rules = await loadRules(props.server.id);

    // 2. Initialize checks from rules
    checks.value = rules
      .filter((r: any) => r.enabled)
      .map((r: any, i: number): CheckItem => ({
        ruleName: r.name,
        category: r.category,
        status: 'pending',
        message: '',
        ruleId: r.id,
        index: i,
      }));

    if (checks.value.length === 0) {
      throw new Error('没有启用的巡检规则');
    }

    // Show all checks as pending briefly
    await new Promise(r => setTimeout(r, 500));

    // Set all to running
    checks.value = checks.value.map(c => ({ ...c, status: 'running' as const }));

    // 3. Execute inspection (pass rules directly to Rust)
    const results = await invoke<any[]>('run_inspection', {
      serverName: props.server.name,
      serverId: props.server.id,
      rules: rules,
      triggeredBy: 'manual',
    });

    // 4. Save report to local DB
    const newReportId = await createReport(props.server.id, 'manual');
    reportId.value = newReportId;

    // 5. Save each check result, compute overall status
    let overallStatus = 'pass';
    for (const r of results) {
      if (r.status === 'critical') overallStatus = 'critical';
      else if (r.status === 'warning' && overallStatus === 'pass') overallStatus = 'warning';
      await insertCheckResult(newReportId, r);
    }

    // 6. Finalize report
    await finalizeReport(newReportId, overallStatus);

    // 7. Map results back to checks for display
    checks.value = results.map((r: any, i: number): CheckItem => ({
      index: i,
      ruleName: r.rule_name,
      category: r.category,
      status: r.status || 'error',
      message: r.message || '',
      detail: r.detail || undefined,
      ruleId: r.rule_id,
    }));

    // 8. Generate AI summary
    if (settingsStore.deepseekApiKey) {
      isGeneratingSummary.value = true;
      try {
        const result = await invoke<string>('generate_inspection_summary', {
          checks: results,
          apiKey: settingsStore.deepseekApiKey,
        });
        summary.value = result;
        // Save summary to DB
        await saveReportSummary(newReportId, result);
      } catch (e) {
        console.error('Failed to generate summary:', e);
        summary.value = '> 生成 AI 总结失败：' + String(e);
      }
      isGeneratingSummary.value = false;
    }
  } catch (e) {
    console.error('Inspection failed:', e);
    if (checks.value.length === 0) {
      checks.value = [{
        index: 0,
        ruleName: '巡检执行失败',
        category: 'custom',
        status: 'error',
        message: String(e),
      }];
    }
  } finally {
    isRunning.value = false;
  }
};

const copyReport = () => {
  if (!summary.value) return;
  navigator.clipboard.writeText(summary.value);
  isCopied.value = true;
  setTimeout(() => (isCopied.value = false), 2000);
};

const toggleDetail = (index: number) => {
  if (expandedChecks.value.has(index)) {
    expandedChecks.value.delete(index);
  } else {
    expandedChecks.value.add(index);
  }
  expandedChecks.value = new Set(expandedChecks.value);
};

const formatDetail = (detailStr?: string) => {
  if (!detailStr) return '';
  try {
    const obj = JSON.parse(detailStr);
    return JSON.stringify(obj, null, 2);
  } catch {
    return detailStr;
  }
};

const getCategoryIcon = (category: string) => {
  return categoryIcons[category] || ShieldCheck;
};
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-slate-800/50 bg-[#1e293b]/30 backdrop-blur-md">
      <div class="flex items-center space-x-2">
        <div class="p-2 bg-blue-500/20 rounded-xl text-blue-400">
          <SearchCheck :size="18" />
        </div>
        <h3 class="text-sm font-bold text-white">智能巡检</h3>
      </div>
      <button
        @click="emit('viewHistory')"
        class="flex items-center space-x-1.5 px-3 py-1.5 text-xs text-slate-400 hover:text-slate-200 bg-slate-800/50 hover:bg-slate-800 rounded-lg transition-colors"
      >
        <History :size="14" />
        <span>查看历史</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-6 custom-scrollbar space-y-6">
      <!-- Progress bar -->
      <div v-if="checks.length > 0" class="space-y-2">
        <div class="flex items-center justify-between text-xs">
          <span class="text-slate-500">巡检进度</span>
          <span class="text-slate-400 font-mono">{{ progressText }}</span>
        </div>
        <div class="h-1.5 bg-slate-800 rounded-full overflow-hidden">
          <div
            class="h-full bg-blue-500 rounded-full transition-all duration-500 ease-out"
            :style="{ width: progress + '%' }"
          ></div>
        </div>
      </div>

      <!-- Overall result -->
      <div
        v-if="overallResult && !isRunning && !isGeneratingSummary && checks.length > 0"
        class="flex items-center space-x-2 px-4 py-2 rounded-xl"
        :class="{
          'bg-emerald-500/10 border border-emerald-500/30': overallResult.label === '全部正常',
          'bg-amber-500/10 border border-amber-500/30': overallResult.label === '存在警告',
          'bg-red-500/10 border border-red-500/30': overallResult.label === '严重异常',
        }"
      >
        <component :is="overallResult.icon" :size="18" :class="overallResult.color" />
        <span class="text-sm font-bold text-slate-200">整体状态: {{ overallResult.label }}</span>
      </div>

      <!-- Check items list -->
      <div class="space-y-2">
        <div
          v-for="check in checks"
          :key="check.index"
          class="rounded-xl border border-slate-800/50 bg-[#1e293b]/20 backdrop-blur-sm overflow-hidden transition-all duration-300"
          :class="{
            'border-blue-500/30 shadow-lg shadow-blue-500/5': check.status === 'running',
          }"
        >
          <div
            class="flex items-center px-4 py-3 cursor-pointer hover:bg-white/5 transition-colors"
            @click="check.detail ? toggleDetail(check.index) : null"
          >
            <!-- Status icon -->
            <div class="w-8 h-8 rounded-lg flex items-center justify-center mr-3" :class="statusConfig[check.status]?.bg || 'bg-slate-800/50'">
              <component
                :is="check.status === 'running' ? Loader2 : statusConfig[check.status]?.icon || Clock"
                :size="16"
                :class="[
                  statusConfig[check.status]?.color || 'text-slate-500',
                  check.status === 'running' ? 'animate-spin' : ''
                ]"
              />
            </div>

            <!-- Category icon -->
            <div class="w-8 h-8 rounded-lg flex items-center justify-center mr-3 bg-slate-800/50 text-slate-500">
              <component :is="getCategoryIcon(check.category)" :size="14" />
            </div>

            <!-- Name and message -->
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-slate-200">{{ check.ruleName }}</div>
              <div class="text-xs text-slate-500 truncate">{{ check.message || statusConfig[check.status]?.label }}</div>
            </div>

            <!-- Status badge -->
            <div class="flex items-center space-x-2">
              <span
                class="text-[10px] font-bold px-2 py-0.5 rounded-md"
                :class="{
                  'bg-emerald-500/10 text-emerald-400': check.status === 'pass',
                  'bg-amber-500/10 text-amber-400': check.status === 'warning',
                  'bg-red-500/10 text-red-400': check.status === 'critical' || check.status === 'error',
                  'bg-slate-800 text-slate-500': check.status === 'pending',
                  'bg-blue-500/10 text-blue-400': check.status === 'running',
                }"
              >
                {{ statusConfig[check.status]?.label }}
              </span>
              <ChevronDown
                v-if="check.detail"
                :size="14"
                class="text-slate-600 transition-transform duration-200"
                :class="{ 'rotate-180': expandedChecks.has(check.index) }"
              />
            </div>
          </div>

          <!-- Expanded detail -->
          <div
            v-if="check.detail && expandedChecks.has(check.index)"
            class="px-4 pb-3 pt-0"
          >
            <pre class="text-xs text-slate-400 bg-black/40 rounded-lg p-3 overflow-x-auto font-mono leading-relaxed">{{ formatDetail(check.detail) }}</pre>
          </div>
        </div>
      </div>

      <!-- AI Summary -->
      <div v-if="isGeneratingSummary" class="rounded-xl border border-blue-500/30 bg-blue-500/10 p-6 backdrop-blur-sm">
        <div class="flex items-center space-x-3">
          <Loader2 class="animate-spin text-blue-400" :size="24" />
          <span class="text-sm text-slate-300">正在使用 AI 分析巡检结果...</span>
        </div>
      </div>

      <div
        v-if="summary && !isGeneratingSummary"
        class="rounded-xl border border-slate-800/50 bg-[#1e293b]/20 backdrop-blur-sm p-6"
      >
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center space-x-2">
            <Sparkles :size="16" class="text-amber-400" />
            <span class="text-sm font-bold text-white">AI 巡检总结</span>
          </div>
          <button
            @click="copyReport"
            class="flex items-center space-x-1 text-[10px] text-slate-500 hover:text-slate-300 transition-colors"
          >
            <Check v-if="isCopied" :size="12" class="text-emerald-500" />
            <Copy v-else :size="12" />
            <span>{{ isCopied ? '已复制' : '复制报告' }}</span>
          </button>
        </div>
        <div class="markdown-content text-sm text-slate-300 leading-relaxed" v-html="renderedSummary"></div>
      </div>

      <!-- Empty state -->
      <div
        v-if="checks.length === 0 && !isRunning"
        class="flex flex-col items-center justify-center py-16 space-y-4"
      >
        <SearchCheck :size="48" class="text-slate-700" />
        <p class="text-sm text-slate-500">暂未执行巡检</p>
        <p class="text-xs text-slate-600">点击下方按钮开始智能巡检</p>
      </div>
    </div>

    <!-- Footer / Action buttons -->
    <div class="px-6 py-4 border-t border-slate-800/50 bg-[#1e293b]/30 backdrop-blur-md flex justify-center space-x-3">
      <button
        v-if="checks.length === 0 || !isRunning"
        @click="startInspection"
        class="flex items-center space-x-2 px-6 py-2.5 bg-blue-600 hover:bg-blue-500 text-white text-sm font-bold rounded-xl transition-all shadow-lg shadow-blue-600/20"
      >
        <Sparkles :size="16" />
        <span>{{ checks.length === 0 ? '开始巡检' : '重新巡检' }}</span>
      </button>
      <button
        v-if="checks.length > 0 && !isRunning"
        @click="startInspection"
        class="flex items-center space-x-2 px-4 py-2.5 bg-slate-800 hover:bg-slate-700 text-slate-300 text-sm font-medium rounded-xl transition-all"
      >
        <Loader2 :size="14" />
        <span>重新巡检</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}

.markdown-content :deep(h1), .markdown-content :deep(h2), .markdown-content :deep(h3) {
  color: white;
  font-weight: 700;
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
}
.markdown-content :deep(h1) { font-size: 1.25rem; }
.markdown-content :deep(h2) { font-size: 1.1rem; border-left: 3px solid #3b82f6; padding-left: 0.75rem; }
.markdown-content :deep(p) { margin-bottom: 1rem; }
.markdown-content :deep(strong) { color: #f59e0b; }
.markdown-content :deep(ul) { list-style-type: disc; margin-left: 1.5rem; margin-bottom: 1rem; }
.markdown-content :deep(code) { background: rgba(0,0,0,0.3); padding: 0.1rem 0.3rem; border-radius: 0.25rem; font-family: monospace; color: #10b981; }
</style>
