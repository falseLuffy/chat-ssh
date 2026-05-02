use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use tauri::{AppHandle, Emitter};

// ============================================================
// DATA TYPES
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRule {
    pub id: Option<i32>,
    pub name: String,
    pub category: String,
    pub check_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub server_id: Option<i32>,
    pub sort_order: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionReport {
    pub id: Option<i32>,
    pub server_id: i32,
    pub triggered_by: String,
    pub status: String,
    pub overall_result: Option<String>,
    pub summary: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionCheckResult {
    pub id: Option<i32>,
    pub report_id: i32,
    pub rule_id: Option<i32>,
    pub rule_name: String,
    pub category: String,
    pub status: String,
    pub message: String,
    pub detail: Option<String>,
    pub executed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionReportDetail {
    pub report: InspectionReport,
    pub checks: Vec<InspectionCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionSchedule {
    pub id: Option<i32>,
    pub name: String,
    pub server_ids: String,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: Option<String>,
}

// ============================================================
// SSH COMMAND HELPER
// ============================================================

use crate::ssh_manager::SshSession;

fn ssh_exec(session: &SshSession, command: &str) -> Result<String, String> {
    session.execute_command(command)
}

// ============================================================
// CHECK EXECUTORS
// ============================================================

fn execute_check(
    session: &SshSession,
    rule: &InspectionRule,
) -> Result<(String, String, String), String> {
    let config = &rule.config;

    match rule.check_type.as_str() {
        "threshold" => execute_threshold_check(session, rule, config),
        "service_status" => execute_service_check(session, config),
        "docker_status" => execute_docker_check(session),
        "command_output" => execute_command_output_check(session, rule, config),
        _ => Err(format!("Unknown check_type: {}", rule.check_type)),
    }
}

fn execute_threshold_check(
    session: &SshSession,
    rule: &InspectionRule,
    config: &serde_json::Value,
) -> Result<(String, String, String), String> {
    let cmd = config["command"]
        .as_str()
        .ok_or("Missing command in rule config")?;
    let warning = config["warning"]
        .as_f64()
        .ok_or("Missing warning threshold")?;
    let critical = config["critical"]
        .as_f64()
        .ok_or("Missing critical threshold")?;
    let unit = config["unit"].as_str().unwrap_or("percent");
    let threshold_type = config["threshold_type"].as_str().unwrap_or("value");

    let output = ssh_exec(session, cmd)?;
    let trimmed = output.trim();
    let raw_val: f64 = trimmed
        .parse()
        .map_err(|e| format!("Failed to parse value from '{}': {}", trimmed, e))?;

    let (val, display_str) = match threshold_type {
        "ratio_to_cores" => {
            let cores_out = ssh_exec(session, "nproc")?;
            let cores: f64 = cores_out.trim().parse().unwrap_or(1.0);
            let ratio = if cores > 0.0 { raw_val / cores } else { raw_val };
            (
                ratio,
                format!("{:.1}% (load: {:.2}, cores: {:.0})", raw_val / cores * 100.0, raw_val, cores),
            )
        }
        _ => (
            raw_val,
            format!(
                "{:.1}{}",
                raw_val,
                if unit == "percent" { "%" } else { "" }
            ),
        ),
    };

    let status = if val >= critical {
        "critical"
    } else if val >= warning {
        "warning"
    } else {
        "pass"
    };

    let message = format!("{}: {}", rule.name, display_str);
    let detail = serde_json::json!({
        "value": raw_val,
        "threshold_warning": warning,
        "threshold_critical": critical,
        "unit": unit,
        "threshold_type": threshold_type
    })
    .to_string();

    Ok((status.to_string(), message, detail))
}

fn execute_service_check(
    session: &SshSession,
    config: &serde_json::Value,
) -> Result<(String, String, String), String> {
    let services: Vec<String> = config["services"]
        .as_array()
        .ok_or("Missing services array in config")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let mut results = Vec::new();
    let mut overall = "pass".to_string();
    let mut ok_count: i32 = 0;
    let mut fail_count: i32 = 0;

    for svc in &services {
        let cmd = format!("systemctl is-active {} 2>/dev/null || echo 'not-found'", svc);
        let output = ssh_exec(session, &cmd)?;
        let state = output.trim().to_string();

        let status = match state.as_str() {
            "active" => "pass",
            "inactive" => "warning",
            _ => "critical",
        };

        if status == "active" || status == "pass" {
            ok_count += 1;
        } else {
            fail_count += 1;
        }

        results.push(serde_json::json!({ "service": svc, "state": state, "status": status }));

        if status == "critical" {
            overall = "critical".to_string();
        } else if status == "warning" && overall == "pass" {
            overall = "warning".to_string();
        }
    }

    let message = format!("关键服务: {}/{} 正常", ok_count, services.len());
    let detail = serde_json::json!({ "services": results }).to_string();

    Ok((overall, message, detail))
}

fn execute_docker_check(session: &SshSession) -> Result<(String, String, String), String> {
    let output = ssh_exec(session, "docker ps -a --format '{{json .}}' 2>/dev/null || echo ''")?;

    let mut containers = Vec::new();
    let mut overall = "pass".to_string();
    let mut failed_count = 0;
    let mut running_count = 0;
    let mut total = 0;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        total += 1;
        if let Ok(c) = serde_json::from_str::<serde_json::Value>(line) {
            let state = c["State"].as_str().unwrap_or("unknown");
            let is_running = state == "running";
            if is_running {
                running_count += 1;
            } else {
                failed_count += 1;
            }
            containers.push(serde_json::json!({
                "id": c["ID"].as_str().unwrap_or(""),
                "name": c["Names"].as_str().unwrap_or(""),
                "state": c["State"].as_str().unwrap_or(""),
                "status": c["Status"].as_str().unwrap_or("")
            }));
        }
    }

    if total == 0 {
        // Docker not installed or no containers — treat as pass (not an error)
        let detail = serde_json::json!({ "containers": [], "note": "Docker not installed or no containers" }).to_string();
        return Ok(("pass".to_string(), "Docker容器: 无容器或Docker未安装".to_string(), detail));
    }

    if failed_count > 0 {
        overall = "critical".to_string();
    }
    let message = format!(
        "Docker容器: {} 运行中, {} 异常",
        running_count, failed_count
    );
    let detail = serde_json::json!({ "containers": containers }).to_string();

    Ok((overall, message, detail))
}

fn execute_command_output_check(
    session: &SshSession,
    rule: &InspectionRule,
    config: &serde_json::Value,
) -> Result<(String, String, String), String> {
    let cmd = config["command"]
        .as_str()
        .ok_or("Missing command in rule config")?;
    let fail_if_output = config["fail_if_output"].as_bool().unwrap_or(false);
    let fail_if_zero = config["fail_if_zero"].as_bool().unwrap_or(false);
    let warning_threshold = config["warning_threshold"].as_f64();
    let critical_threshold = config["critical_threshold"].as_f64();

    let output = ssh_exec(session, cmd)?;
    let trimmed = output.trim();

    let status = if fail_if_output && !trimmed.is_empty() {
        "warning"
    } else if fail_if_zero {
        let val: f64 = trimmed.parse().unwrap_or(-1.0);
        if val == 0.0 {
            "critical"
        } else {
            "pass"
        }
    } else if let (Some(warn), Some(crit)) = (warning_threshold, critical_threshold) {
        let val: f64 = trimmed.parse().unwrap_or(0.0);
        if val >= crit {
            "critical"
        } else if val >= warn {
            "warning"
        } else {
            "pass"
        }
    } else {
        // No failure condition set; just report the output
        "pass"
    };

    let display = if trimmed.len() > 200 {
        &trimmed[..200]
    } else {
        trimmed
    };
    let message = format!("{}: {}", rule.name, display);
    let detail = serde_json::json!({ "raw_output": trimmed }).to_string();

    Ok((status.to_string(), message, detail))
}

// ============================================================
// INSPECTION ENGINE
// ============================================================

/// Execute a full inspection for a server.
/// Takes a list of rules (pre-filtered and ordered by the frontend).
/// Returns vec of check results (without report_id — the frontend assigns it).
pub fn run_inspection(
    server_name: &str,
    sessions: &Mutex<HashMap<String, Arc<SshSession>>>,
    rules: &[InspectionRule],
    _server_id: i32,
    _triggered_by: &str,
) -> Result<Vec<InspectionCheckResult>, String> {
    // 1. Get SSH session
    let session = {
        let sessions = sessions.lock().map_err(|e| e.to_string())?;
        sessions
            .get(server_name)
            .cloned()
            .ok_or_else(|| format!("Server '{}' not connected", server_name))?
    };

    // 2. Filter enabled rules
    let enabled_rules: Vec<&InspectionRule> = rules.iter().filter(|r| r.enabled).collect();

    if enabled_rules.is_empty() {
        return Err("No enabled inspection rules found".to_string());
    }

    // 3. Execute each rule sequentially
    let mut results: Vec<InspectionCheckResult> = Vec::new();
    let mut overall_status = "pass".to_string();

    for rule in &enabled_rules {
        let (status, message, detail) = match execute_check(&session, rule) {
            Ok((s, msg, det)) => {
                // Update overall status
                if s == "critical" && overall_status != "critical" {
                    overall_status = "critical".to_string();
                } else if s == "warning" && overall_status == "pass" {
                    overall_status = "warning".to_string();
                }
                (s, msg, det)
            }
            Err(e) => {
                eprintln!("Check '{}' failed: {}", rule.name, e);
                (
                    "error".to_string(),
                    format!("{}: 执行错误 - {}", rule.name, e),
                    "{}".to_string(),
                )
            }
        };

        results.push(InspectionCheckResult {
            id: None,
            report_id: 0, // frontend fills this
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            category: rule.category.clone(),
            status,
            message,
            detail: Some(detail),
            executed_at: None,
        });
    }

    // 4. Attach overall status as the last check result's detail or we can return separately.
    // We'll store overall_result in a dedicated field by convention:
    // The frontend will compute overall_result from the results or use this.
    // For now, store overall_status in results metadata.

    Ok(results)
}

// ============================================================
// AI SUMMARY GENERATION
// ============================================================

pub async fn generate_summary_logic(
    checks: &[InspectionCheckResult],
    api_key: &str,
) -> Result<String, String> {
    let context = checks
        .iter()
        .map(|c| format!("- [{}] {}: {}", c.status, c.rule_name, c.message))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "你是一个资深的 Linux 运维专家。以下是一次服务器智能巡检的结果。请分析这些检查项，给出整体评估（pass/warning/critical），总结发现的问题，并提供具体的修复建议。请使用 Markdown 格式回复，语言为中文。\n\n巡检结果:\n{}",
        context
    );

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "deepseek-chat",
            "messages": [
                {"role": "system", "content": "你是一个专业的服务器巡检分析助手。"},
                {"role": "user", "content": prompt}
            ],
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let res_json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let summary = res_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Failed to get content from AI response")?
        .to_string();

    Ok(summary)
}

// ============================================================
// SCHEDULER
// ============================================================

pub fn start_scheduler(app_handle: AppHandle) {
    // Use a dedicated OS thread with its own tokio runtime
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create scheduler runtime");
        rt.block_on(async move {
            loop {
                // Emit a tick event every 60 seconds; frontend will check schedules from its own DB
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

                app_handle
                    .emit("scheduler-tick", serde_json::json!({}))
                    .ok();
            }
        });
    });
}
