import Database from '@tauri-apps/plugin-sql';

let _db: Database | null = null;
let _initialized = false;

export async function getDb(): Promise<Database> {
  if (!_db) {
    _db = await Database.load('sqlite:chat_ssh.db');
  }
  if (!_initialized) {
    try {
      // Ensure inspection tables exist (in case migration 5 didn't run)
      await _db.execute(`CREATE TABLE IF NOT EXISTS inspection_rules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        category TEXT NOT NULL,
        check_type TEXT NOT NULL,
        config TEXT NOT NULL,
        enabled INTEGER NOT NULL DEFAULT 1,
        server_id INTEGER,
        sort_order INTEGER NOT NULL DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )`);
      await _db.execute(`CREATE TABLE IF NOT EXISTS inspection_reports (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        server_id INTEGER NOT NULL,
        triggered_by TEXT NOT NULL DEFAULT 'manual',
        status TEXT NOT NULL DEFAULT 'running',
        overall_result TEXT,
        summary TEXT,
        started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        completed_at DATETIME
      )`);
      await _db.execute(`CREATE TABLE IF NOT EXISTS inspection_check_results (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        report_id INTEGER NOT NULL,
        rule_id INTEGER,
        rule_name TEXT NOT NULL,
        category TEXT NOT NULL,
        status TEXT NOT NULL,
        message TEXT,
        detail TEXT,
        executed_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )`);
      await _db.execute(`CREATE TABLE IF NOT EXISTS inspection_schedules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        server_ids TEXT NOT NULL,
        cron_expression TEXT NOT NULL,
        enabled INTEGER NOT NULL DEFAULT 1,
        last_run_at DATETIME,
        next_run_at DATETIME,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )`);
      await _db.execute(`CREATE UNIQUE INDEX IF NOT EXISTS idx_rules_name_server ON inspection_rules(name, COALESCE(server_id, -1))`);

      // Insert default rules if table is empty
      const row = await _db.select<[{ cnt: number }]>('SELECT COUNT(*) as cnt FROM inspection_rules');
      if (row[0].cnt === 0) {
        await _db.execute(`INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
          ('CPU使用率', 'cpu', 'threshold', '{"command":"cat /proc/loadavg | awk ''{print $1}''","unit":"loadavg","threshold_type":"ratio_to_cores","warning":0.7,"critical":0.9}', 1),
          ('内存使用率', 'memory', 'threshold', '{"command":"free | grep Mem | awk ''{printf \\"%.1f\\", $3/$2 * 100}''","unit":"percent","threshold_type":"value","warning":80,"critical":95}', 2),
          ('磁盘使用率', 'disk', 'threshold', '{"command":"df -h / | tail -1 | awk ''{print $5}'' | sed ''s/%//''","unit":"percent","threshold_type":"value","warning":80,"critical":90}', 3),
          ('Docker容器状态', 'docker', 'docker_status', '{"command":"docker ps -a --format ''{{json .}}''"}', 4),
          ('关键服务状态', 'service', 'service_status', '{"services":["nginx","mysql","docker","sshd","cron"]}', 5),
          ('僵尸进程检测', 'process', 'command_output', '{"command":"ps aux | awk ''$8 ~ /Z/ {print $0}''","fail_if_output":true}', 6),
          ('网络连通性', 'network', 'command_output', '{"command":"ping -c 2 -W 2 8.8.8.8 2>&1 | grep -c ''bytes from''","fail_if_zero":true}', 7),
          ('OOM Killer检测', 'process', 'command_output', '{"command":"dmesg -T 2>/dev/null | grep -i ''out of memory'' | tail -5; journalctl -k --no-pager 2>/dev/null | grep -i ''out of memory'' | tail -5","fail_if_output":true}', 8),
          ('Inode使用率', 'disk', 'command_output', '{"command":"df -i / | tail -1 | awk ''{print $5}'' | sed ''s/%//''","warning_threshold":80,"critical_threshold":90}', 9)`);
      }
    } catch (e) {
      console.warn('DB init warning:', e);
    }
    _initialized = true;
  }
  return _db;
}

// ============================================================
// INSPECTION RULES
// ============================================================

export async function loadRules(serverId: number): Promise<any[]> {
  const db = await getDb();
  const rows = await db.select<any[]>(
    `SELECT id, name, category, check_type, config, enabled, server_id, sort_order, created_at
     FROM inspection_rules
     WHERE server_id = ?1 OR server_id IS NULL
     ORDER BY sort_order, id`,
    [serverId]
  );

  // Parse config JSON strings
  const parsed = rows.map((r: any) => ({
    ...r,
    config: typeof r.config === 'string' ? JSON.parse(r.config) : r.config,
    enabled: !!r.enabled,
  }));

  // Deduplicate: server-specific rules override global templates by name
  const merged = new Map<string, any>();
  for (const rule of parsed) {
    if (rule.server_id != null) {
      merged.set(rule.name, rule);
    } else {
      if (!merged.has(rule.name)) {
        merged.set(rule.name, rule);
      }
    }
  }

  return Array.from(merged.values()).sort((a, b) => a.sort_order - b.sort_order);
}

export async function createRule(rule: any): Promise<number> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO inspection_rules (name, category, check_type, config, enabled, server_id, sort_order)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)`,
    [
      rule.name,
      rule.category,
      rule.check_type,
      typeof rule.config === 'string' ? rule.config : JSON.stringify(rule.config),
      rule.enabled ? 1 : 0,
      rule.server_id ?? null,
      rule.sort_order ?? 0,
    ]
  );
  return result.lastInsertId;
}

export async function updateRule(id: number, rule: any): Promise<void> {
  const db = await getDb();
  await db.execute(
    `UPDATE inspection_rules SET name=?1, category=?2, check_type=?3, config=?4, enabled=?5, server_id=?6, sort_order=?7 WHERE id=?8`,
    [
      rule.name,
      rule.category,
      rule.check_type,
      typeof rule.config === 'string' ? rule.config : JSON.stringify(rule.config),
      rule.enabled ? 1 : 0,
      rule.server_id ?? null,
      rule.sort_order ?? 0,
      id,
    ]
  );
}

export async function deleteRule(id: number): Promise<void> {
  const db = await getDb();
  await db.execute('DELETE FROM inspection_rules WHERE id=?1', [id]);
}

// ============================================================
// INSPECTION REPORTS
// ============================================================

export async function createReport(serverId: number, triggeredBy: string): Promise<number> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO inspection_reports (server_id, triggered_by, status) VALUES (?1, ?2, 'running')`,
    [serverId, triggeredBy]
  );
  return result.lastInsertId;
}

export async function insertCheckResult(reportId: number, check: any): Promise<void> {
  const db = await getDb();
  await db.execute(
    `INSERT INTO inspection_check_results (report_id, rule_id, rule_name, category, status, message, detail)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)`,
    [
      reportId,
      check.rule_id ?? null,
      check.rule_name,
      check.category,
      check.status,
      check.message,
      check.detail ?? null,
    ]
  );
}

export async function finalizeReport(reportId: number, overallResult: string): Promise<void> {
  const db = await getDb();
  await db.execute(
    `UPDATE inspection_reports SET status='completed', overall_result=?1, completed_at=datetime('now') WHERE id=?2`,
    [overallResult, reportId]
  );
}

export async function saveReportSummary(reportId: number, summary: string): Promise<void> {
  const db = await getDb();
  await db.execute(
    'UPDATE inspection_reports SET summary=?1 WHERE id=?2',
    [summary, reportId]
  );
}

export async function loadReports(serverId: number, limit = 50, offset = 0): Promise<any[]> {
  const db = await getDb();
  return await db.select<any[]>(
    `SELECT id, server_id, triggered_by, status, overall_result, summary, started_at, completed_at
     FROM inspection_reports
     WHERE server_id=?1
     ORDER BY started_at DESC
     LIMIT ?2 OFFSET ?3`,
    [serverId, limit, offset]
  );
}

export async function loadReportDetail(reportId: number): Promise<{ report: any; checks: any[] } | null> {
  const db = await getDb();
  const reports = await db.select<any[]>(
    `SELECT id, server_id, triggered_by, status, overall_result, summary, started_at, completed_at
     FROM inspection_reports WHERE id=?1`,
    [reportId]
  );
  if (reports.length === 0) return null;

  const checks = await db.select<any[]>(
    `SELECT id, report_id, rule_id, rule_name, category, status, message, detail, executed_at
     FROM inspection_check_results
     WHERE report_id=?1
     ORDER BY executed_at`,
    [reportId]
  );

  return { report: reports[0], checks };
}

export async function deleteReport(id: number): Promise<void> {
  const db = await getDb();
  // CASCADE will delete check results
  await db.execute('DELETE FROM inspection_reports WHERE id=?1', [id]);
}

// ============================================================
// INSPECTION SCHEDULES
// ============================================================

export async function loadSchedules(): Promise<any[]> {
  const db = await getDb();
  return await db.select<any[]>(
    `SELECT id, name, server_ids, cron_expression, enabled, last_run_at, next_run_at, created_at
     FROM inspection_schedules
     ORDER BY id`
  );
}

export async function createSchedule(schedule: any): Promise<number> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO inspection_schedules (name, server_ids, cron_expression, enabled, next_run_at)
     VALUES (?1, ?2, ?3, ?4, ?5)`,
    [
      schedule.name,
      schedule.server_ids,
      schedule.cron_expression,
      schedule.enabled ? 1 : 0,
      schedule.next_run_at ?? null,
    ]
  );
  return result.lastInsertId;
}

export async function updateSchedule(id: number, schedule: any): Promise<void> {
  const db = await getDb();
  await db.execute(
    `UPDATE inspection_schedules SET name=?1, server_ids=?2, cron_expression=?3, enabled=?4, next_run_at=?5 WHERE id=?6`,
    [
      schedule.name,
      schedule.server_ids,
      schedule.cron_expression,
      schedule.enabled ? 1 : 0,
      schedule.next_run_at ?? null,
      id,
    ]
  );
}

export async function deleteSchedule(id: number): Promise<void> {
  const db = await getDb();
  await db.execute('DELETE FROM inspection_schedules WHERE id=?1', [id]);
}

export async function updateScheduleTimes(id: number, nextRunAt: string): Promise<void> {
  const db = await getDb();
  await db.execute(
    `UPDATE inspection_schedules SET last_run_at=datetime('now'), next_run_at=?1 WHERE id=?2`,
    [nextRunAt, id]
  );
}
