-- Inspection rules (global templates when server_id IS NULL, server-specific overrides otherwise)
CREATE TABLE IF NOT EXISTS inspection_rules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  category TEXT NOT NULL,
  check_type TEXT NOT NULL,
  config TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  server_id INTEGER,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- Each inspection run produces one report
CREATE TABLE IF NOT EXISTS inspection_reports (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  server_id INTEGER NOT NULL,
  triggered_by TEXT NOT NULL DEFAULT 'manual',
  status TEXT NOT NULL DEFAULT 'running',
  overall_result TEXT,
  summary TEXT,
  started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  completed_at DATETIME,
  FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- Individual check results within a report
CREATE TABLE IF NOT EXISTS inspection_check_results (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  report_id INTEGER NOT NULL,
  rule_id INTEGER,
  rule_name TEXT NOT NULL,
  category TEXT NOT NULL,
  status TEXT NOT NULL,
  message TEXT,
  detail TEXT,
  executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (report_id) REFERENCES inspection_reports(id) ON DELETE CASCADE,
  FOREIGN KEY (rule_id) REFERENCES inspection_rules(id) ON DELETE SET NULL
);

-- Cron-based inspection schedules
CREATE TABLE IF NOT EXISTS inspection_schedules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  server_ids TEXT NOT NULL,
  cron_expression TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  last_run_at DATETIME,
  next_run_at DATETIME,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Unique index to prevent duplicate rule names per scope (global or per-server)
CREATE UNIQUE INDEX IF NOT EXISTS idx_rules_name_server ON inspection_rules(name, COALESCE(server_id, -1));

-- Default built-in rules (global templates, server_id IS NULL)
INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('CPU使用率', 'cpu', 'threshold',
 '{"command":"cat /proc/loadavg | awk ''{print $1}''","unit":"loadavg","threshold_type":"ratio_to_cores","warning":0.7,"critical":0.9}', 1);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('内存使用率', 'memory', 'threshold',
 '{"command":"free | grep Mem | awk ''{printf \"%.1f\", $3/$2 * 100}''","unit":"percent","threshold_type":"value","warning":80,"critical":95}', 2);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('磁盘使用率', 'disk', 'threshold',
 '{"command":"df -h / | tail -1 | awk ''{print $5}'' | sed ''s/%//''","unit":"percent","threshold_type":"value","warning":80,"critical":90}', 3);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('Docker容器状态', 'docker', 'docker_status',
 '{"command":"docker ps -a --format ''{{json .}}''"}', 4);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('关键服务状态', 'service', 'service_status',
 '{"services":["nginx","mysql","docker","sshd","cron"]}', 5);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('僵尸进程检测', 'process', 'command_output',
 '{"command":"ps aux | awk ''$8 ~ /Z/ {print $0}''","fail_if_output":true}', 6);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('网络连通性', 'network', 'command_output',
 '{"command":"ping -c 2 -W 2 8.8.8.8 2>&1 | grep -c ''bytes from''","fail_if_zero":true}', 7);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('OOM Killer检测', 'process', 'command_output',
 '{"command":"dmesg -T 2>/dev/null | grep -i ''out of memory'' | tail -5; journalctl -k --no-pager 2>/dev/null | grep -i ''out of memory'' | tail -5","fail_if_output":true}', 8);

INSERT OR IGNORE INTO inspection_rules (name, category, check_type, config, sort_order) VALUES
('Inode使用率', 'disk', 'command_output',
 '{"command":"df -i / | tail -1 | awk ''{print $5}'' | sed ''s/%//''","warning_threshold":80,"critical_threshold":90}', 9);
