<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import type { Server } from '../stores/server';
import { loadReports as dbLoadReports, loadReportDetail, deleteReport } from '../utils/db';
import {
  ArrowLeft, CheckCircle2, AlertTriangle, XCircle,
  User, RefreshCw, Trash2, ChevronRight, GitCompare,
  SearchCheck, Clock, Loader2, Sparkles
} from 'lucide-vue-next';
import MarkdownIt from 'markdown-it';

const props = defineProps<{ server: Server }>();
const emit = defineEmits(['back']);

interface InspectionReport {
  id?: number;
  server_id: number;
  triggered_by: string;
  status: string;
  overall_result?: string;
  summary?: string;
  started_at?: string;
  completed_at?: string;
}

interface CheckItem {
  id?: number;
  rule_name: string;
  category: string;
  status: string;
  message: string;
  detail?: string;
}

interface ReportDetail {
  report: InspectionReport;
  checks: CheckItem[];
}

const reports = ref<InspectionReport[]>([]);
const selectedReportId = ref<number | null>(null);
const reportDetail = ref<ReportDetail | null>(null);
const isLoading = ref(false);
const isLoadingDetail = ref(false);
const compareMode = ref(false);
const compareA = ref<number | null>(null);
const compareB = ref<number | null>(null);
const compareDetailA = ref<ReportDetail | null>(null);
const compareDetailB = ref<ReportDetail | null>(null);

const md = new MarkdownIt({ html: false, breaks: true });

const formattedTime = (str?: string) => {
  if (!str) return '—';
  try {
    const d = new Date(str.replace(' ', 'T') + 'Z');
    return d.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return str;
  }
};

const triggerLabel = (t: string) => {
  const map: Record<string, string> = { manual: '手动触发', scheduled: '定时巡检' };
  return map[t] || t;
};

const resultBadge = (result?: string) => {
  switch (result) {
    case 'pass': return { icon: CheckCircle2, color: 'text-emerald-400', bg: 'bg-emerald-500/10', border: 'border-emerald-500/30', label: '正常' };
    case 'warning': return { icon: AlertTriangle, color: 'text-amber-400', bg: 'bg-amber-500/10', border: 'border-amber-500/30', label: '警告' };
    case 'critical': return { icon: XCircle, color: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/30', label: '严重' };
    default: return { icon: Clock, color: 'text-slate-400', bg: 'bg-slate-800/50', border: 'border-slate-800', label: '未知' };
  }
};

const statusLabel = (st: string) => {
  const map: Record<string, { label: string; color: string }> = {
    pass: { label: '正常', color: 'text-emerald-400' },
    warning: { label: '警告', color: 'text-amber-400' },
    critical: { label: '严重', color: 'text-red-400' },
    error: { label: '错误', color: 'text-red-400' },
    pending: { label: '等待', color: 'text-slate-500' },
    running: { label: '运行中', color: 'text-blue-400' },
  };
  return map[st] || { label: st, color: 'text-slate-500' };
};

const fetchReports = async () => {
  if (!props.server) return;
  isLoading.value = true;
  try {
    reports.value = await dbLoadReports(props.server.id, 50, 0);
  } catch (e) {
    console.error('Failed to load reports:', e);
    reports.value = [];
  } finally {
    isLoading.value = false;
  }
};

const openDetail = async (reportId: number) => {
  selectedReportId.value = reportId;
  isLoadingDetail.value = true;
  reportDetail.value = null;
  try {
    const detail = await loadReportDetail(reportId);
    if (detail) {
      reportDetail.value = detail as ReportDetail;
    }
  } catch (e) {
    console.error('Failed to load report detail:', e);
  } finally {
    isLoadingDetail.value = false;
  }
};

const removeReport = async (id: number) => {
  try {
    await deleteReport(id);
    reports.value = reports.value.filter(r => r.id !== id);
    if (selectedReportId.value === id) {
      selectedReportId.value = null;
      reportDetail.value = null;
    }
  } catch (e) {
    console.error('Failed to delete report:', e);
  }
};

const backToList = () => {
  selectedReportId.value = null;
  reportDetail.value = null;
};

const enterCompare = () => {
  compareMode.value = true;
  compareA.value = null;
  compareB.value = null;
  compareDetailA.value = null;
  compareDetailB.value = null;
};

const cancelCompare = () => {
  compareMode.value = false;
};

const loadCompare = async () => {
  if (compareA.value == null || compareB.value == null) return;
  try {
    const [a, b] = await Promise.all([
      loadReportDetail(compareA.value),
      loadReportDetail(compareB.value),
    ]);
    if (a) compareDetailA.value = a as ReportDetail;
    if (b) compareDetailB.value = b as ReportDetail;
  } catch (e) {
    console.error('Failed to load compare data:', e);
  }
};

// Compare helpers
const compareItems = computed(() => {
  if (!compareDetailA.value || !compareDetailB.value) return [];
  const checksA = compareDetailA.value.checks;
  const checksB = compareDetailB.value.checks;
  const allNames = new Set([
    ...checksA.map(c => c.rule_name),
    ...checksB.map(c => c.rule_name),
  ]);
  return Array.from(allNames).map(name => ({
    name,
    a: checksA.find(c => c.rule_name === name),
    b: checksB.find(c => c.rule_name === name),
    diff: (checksA.find(c => c.rule_name === name)?.status !== checksB.find(c => c.rule_name === name)?.status),
  }));
});

const renderedSummary = computed(() => {
  if (!reportDetail.value?.report?.summary) return '';
  return md.render(reportDetail.value.report.summary);
});

watch(() => props.server?.id, () => {
  fetchReports();
  backToList();
  cancelCompare();
});

onMounted(() => {
  fetchReports();
});
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-slate-800/50 bg-[#1e293b]/30 backdrop-blur-md">
      <div class="flex items-center space-x-2">
        <button
          v-if="selectedReportId != null || compareMode"
          @click="compareMode ? cancelCompare() : backToList()"
          class="p-1.5 hover:bg-slate-800 rounded-lg transition-colors text-slate-500"
        >
          <ArrowLeft :size="18" />
        </button>
        <div class="p-2 bg-blue-500/20 rounded-xl text-blue-400">
          <SearchCheck :size="18" />
        </div>
        <h3 class="text-sm font-bold text-white">
          {{ compareMode ? '对比巡检报告' : selectedReportId != null ? '巡检详情' : '巡检历史' }}
        </h3>
      </div>
      <div class="flex items-center space-x-2">
        <button
          v-if="selectedReportId == null && !compareMode"
          @click="enterCompare"
          class="flex items-center space-x-1.5 px-3 py-1.5 text-xs text-slate-400 hover:text-slate-200 bg-slate-800/50 hover:bg-slate-800 rounded-lg transition-colors"
        >
          <GitCompare :size="14" />
          <span>对比模式</span>
        </button>
        <button
          v-if="selectedReportId == null"
          @click="fetchReports"
          class="p-1.5 hover:bg-slate-800 rounded-lg transition-colors text-slate-500"
        >
          <RefreshCw :size="14" :class="{ 'animate-spin': isLoading }" />
        </button>
        <button
          @click="emit('back')"
          class="flex items-center space-x-1.5 px-3 py-1.5 text-xs text-slate-400 hover:text-slate-200 bg-slate-800/50 hover:bg-slate-800 rounded-lg transition-colors"
        >
          <ArrowLeft :size="14" />
          <span>返回巡检</span>
        </button>
      </div>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-6 custom-scrollbar">
      <!-- === COMPARE MODE === -->
      <div v-if="compareMode" class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="text-xs text-slate-500 mb-1 block">报告 A</label>
            <select
              v-model="compareA"
              class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200"
            >
              <option :value="null" disabled>选择报告</option>
              <option v-for="r in reports" :key="r.id" :value="r.id">
                {{ formattedTime(r.started_at) }} — {{ r.overall_result }}
              </option>
            </select>
          </div>
          <div>
            <label class="text-xs text-slate-500 mb-1 block">报告 B</label>
            <select
              v-model="compareB"
              class="w-full bg-slate-900/50 border border-slate-800 rounded-xl px-3 py-2 text-sm text-slate-200"
            >
              <option :value="null" disabled>选择报告</option>
              <option v-for="r in reports" :key="r.id" :value="r.id">
                {{ formattedTime(r.started_at) }} — {{ r.overall_result }}
              </option>
            </select>
          </div>
        </div>

        <button
          @click="loadCompare"
          :disabled="compareA == null || compareB == null"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-500 text-white text-sm font-bold rounded-xl transition-all"
        >
          开始对比
        </button>

        <div v-if="compareDetailA && compareDetailB" class="space-y-2">
          <div class="grid grid-cols-2 gap-4 mb-4">
            <div class="text-xs text-slate-500 p-3 bg-slate-900/30 rounded-xl">
              报告A: {{ formattedTime(compareDetailA.report.started_at) }}
              — {{ resultBadge(compareDetailA.report.overall_result).label }}
            </div>
            <div class="text-xs text-slate-500 p-3 bg-slate-900/30 rounded-xl">
              报告B: {{ formattedTime(compareDetailB.report.started_at) }}
              — {{ resultBadge(compareDetailB.report.overall_result).label }}
            </div>
          </div>

          <div
            v-for="item in compareItems"
            :key="item.name"
            class="grid grid-cols-2 gap-4 p-3 rounded-xl"
            :class="item.diff ? 'bg-amber-500/5 border border-amber-500/20' : 'bg-slate-900/20 border border-slate-800/50'"
          >
            <div class="flex items-center space-x-2">
              <span
                v-if="item.a"
                class="text-[10px] font-bold px-2 py-0.5 rounded"
                :class="{
                  'bg-emerald-500/10 text-emerald-400': item.a.status === 'pass',
                  'bg-amber-500/10 text-amber-400': item.a.status === 'warning',
                  'bg-red-500/10 text-red-400': item.a.status === 'critical' || item.a.status === 'error',
                }"
              >
                {{ statusLabel(item.a.status).label }}
              </span>
              <span v-else class="text-slate-600 text-xs">—</span>
              <span class="text-xs text-slate-400">{{ item.name }}</span>
            </div>
            <div class="flex items-center space-x-2">
              <span
                v-if="item.b"
                class="text-[10px] font-bold px-2 py-0.5 rounded"
                :class="{
                  'bg-emerald-500/10 text-emerald-400': item.b.status === 'pass',
                  'bg-amber-500/10 text-amber-400': item.b.status === 'warning',
                  'bg-red-500/10 text-red-400': item.b.status === 'critical' || item.b.status === 'error',
                }"
              >
                {{ statusLabel(item.b.status).label }}
              </span>
              <span v-else class="text-slate-600 text-xs">—</span>
              <span class="text-xs text-slate-400">{{ item.name }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- === DETAIL VIEW === -->
      <div v-else-if="selectedReportId != null" class="space-y-6">
        <div v-if="isLoadingDetail" class="flex items-center justify-center py-16">
          <Loader2 class="animate-spin text-blue-400" :size="32" />
        </div>

        <div v-else-if="reportDetail" class="space-y-6">
          <!-- Report metadata -->
          <div class="rounded-xl border border-slate-800/50 bg-[#1e293b]/20 p-4 flex items-center justify-between">
            <div class="space-y-1">
              <div class="text-xs text-slate-500">
                触发: {{ triggerLabel(reportDetail.report.triggered_by) }}
              </div>
              <div class="text-xs text-slate-500">
                时间: {{ formattedTime(reportDetail.report.started_at) }}
              </div>
            </div>
            <div class="flex items-center space-x-3">
              <div
                class="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg"
                :class="resultBadge(reportDetail.report.overall_result).bg + ' border ' + resultBadge(reportDetail.report.overall_result).border"
              >
                <component :is="resultBadge(reportDetail.report.overall_result).icon" :size="14" :class="resultBadge(reportDetail.report.overall_result).color" />
                <span class="text-xs font-bold" :class="resultBadge(reportDetail.report.overall_result).color">
                  {{ resultBadge(reportDetail.report.overall_result).label }}
                </span>
              </div>
              <button
                @click="removeReport(reportDetail.report.id!)"
                class="p-1.5 hover:bg-red-500/20 rounded-lg text-slate-500 hover:text-red-400 transition-colors"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>

          <!-- Check results -->
          <div class="space-y-2">
            <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider">检查项</h4>
            <div
              v-for="(check, i) in reportDetail.checks"
              :key="i"
              class="flex items-center px-4 py-3 rounded-xl border border-slate-800/50 bg-[#1e293b]/20"
            >
              <component
                :is="check.status === 'pass' ? CheckCircle2 : check.status === 'warning' ? AlertTriangle : XCircle"
                :size="16"
                :class="{
                  'text-emerald-400': check.status === 'pass',
                  'text-amber-400': check.status === 'warning',
                  'text-red-400': check.status === 'critical' || check.status === 'error',
                  'text-slate-500': check.status === 'pending',
                }"
                class="mr-3"
              />
              <div class="flex-1">
                <div class="text-sm text-slate-200">{{ check.rule_name }}</div>
                <div class="text-xs text-slate-500">{{ check.message }}</div>
              </div>
              <span
                class="text-[10px] font-bold px-2 py-0.5 rounded"
                :class="{
                  'bg-emerald-500/10 text-emerald-400': check.status === 'pass',
                  'bg-amber-500/10 text-amber-400': check.status === 'warning',
                  'bg-red-500/10 text-red-400': check.status === 'critical' || check.status === 'error',
                }"
              >
                {{ statusLabel(check.status).label }}
              </span>
            </div>
          </div>

          <!-- AI Summary -->
          <div v-if="reportDetail.report.summary" class="rounded-xl border border-slate-800/50 bg-[#1e293b]/20 p-6">
            <div class="flex items-center space-x-2 mb-4">
              <Sparkles :size="16" class="text-amber-400" />
              <span class="text-sm font-bold text-white">AI 巡检总结</span>
            </div>
            <div class="markdown-content text-sm text-slate-300 leading-relaxed" v-html="renderedSummary"></div>
          </div>
        </div>

        <div v-else class="text-center py-16 text-slate-500">未找到报告详情</div>
      </div>

      <!-- === LIST VIEW === -->
      <div v-else class="space-y-2">
        <div v-if="isLoading" class="flex items-center justify-center py-16">
          <Loader2 class="animate-spin text-blue-400" :size="32" />
        </div>

        <div v-else-if="reports.length === 0" class="flex flex-col items-center justify-center py-16 space-y-4">
          <SearchCheck :size="48" class="text-slate-700" />
          <p class="text-sm text-slate-500">暂无巡检记录</p>
          <p class="text-xs text-slate-600">执行一次巡检后，结果将显示在这里</p>
        </div>

        <div
          v-for="report in reports"
          :key="report.id"
          @click="report.id != null && openDetail(report.id)"
          class="rounded-xl border cursor-pointer transition-all hover:bg-white/5"
          :class="resultBadge(report.overall_result).border + ' ' + (resultBadge(report.overall_result).bg || 'bg-slate-900/20')"
        >
          <div class="flex items-center justify-between p-4">
            <div class="flex items-center space-x-3">
              <component
                :is="resultBadge(report.overall_result).icon"
                :size="20"
                :class="resultBadge(report.overall_result).color"
              />
              <div>
                <div class="flex items-center space-x-2">
                  <span
                    class="text-xs font-bold px-2 py-0.5 rounded"
                    :class="resultBadge(report.overall_result).bg + ' ' + resultBadge(report.overall_result).color"
                  >
                    {{ resultBadge(report.overall_result).label }}
                  </span>
                  <span class="text-xs text-slate-500">{{ triggerLabel(report.triggered_by) }}</span>
                </div>
                <div class="text-xs text-slate-400 mt-1">{{ formattedTime(report.started_at) }}</div>
              </div>
            </div>
            <div class="flex items-center space-x-2">
              <ChevronRight :size="16" class="text-slate-600" />
            </div>
          </div>
        </div>
      </div>
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
