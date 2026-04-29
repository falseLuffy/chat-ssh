mod deepseek;
mod ssh_manager;

use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use tauri::{State, Emitter, Manager};
use std::io::{Read, Write};

struct AppState {
    sessions: Mutex<HashMap<String, Arc<ssh_manager::SshSession>>>,
    senders: Mutex<HashMap<String, std::sync::mpsc::Sender<Vec<u8>>>>,
}

#[tauri::command]
async fn disconnect_ssh(
    name: &str, 
    state: State<'_, AppState>
) -> Result<(), String> {
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.remove(name);
    }
    {
        let mut senders = state.senders.lock().unwrap();
        senders.remove(name);
    }
    Ok(())
}

#[tauri::command]
async fn connect_ssh(
    name: &str, 
    host: &str, 
    user: &str, 
    pass: &str, 
    state: State<'_, AppState>
) -> Result<String, String> {
    let session = ssh_manager::SshSession::connect(host, user, Some(pass))?;
    let mut sessions = state.sessions.lock().unwrap();
    sessions.insert(name.to_string(), Arc::new(session));
    Ok(format!("Successfully connected to {}", name))
}

#[tauri::command]
async fn upload_to_server(
    server_name: &str,
    remote_path: &str,
    file_content: Vec<u8>,
    state: State<'_, AppState>
) -> Result<String, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(server_name) {
        session.upload_file(&file_content, remote_path)?;
        Ok("Upload successful".to_string())
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn open_terminal(
    server_name: String,
    cols: u32,
    rows: u32,
    window: tauri::Window,
    state: State<'_, AppState>
) -> Result<(), String> {
    let session = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&server_name).cloned().ok_or("Server not connected")?
    };

    let mut channel = session.start_shell(cols, rows)?;
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    
    {
        let mut senders = state.senders.lock().unwrap();
        senders.insert(server_name.clone(), tx);
    }

    // Spawn a thread to read and write
    std::thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        session.set_blocking(false);
        
        println!("Terminal thread started for: {}", server_name);
        let session_mutex = session.get_session();

        loop {
            // 1. Read from SSH (Non-blocking)
            let read_res = {
                let _lock = session_mutex.lock().unwrap();
                channel.read(&mut buffer)
            };

            match read_res {
                Ok(n) if n > 0 => {
                    let data = buffer[..n].to_vec();
                    window.emit("ssh-data", serde_json::json!({
                        "server": server_name,
                        "data": data
                    })).unwrap();
                }
                Ok(_) => {}, // No data yet (or EOF)
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Normal, no data to read
                }
                Err(e) => {
                    println!("Read error on {}: {}", server_name, e);
                    break;
                }
            }

            // 2. Check for input to write (Non-blocking)
            if let Ok(input) = rx.try_recv() {
                let mut written = 0;
                while written < input.len() {
                    let write_res = {
                        let _lock = session_mutex.lock().unwrap();
                        channel.write(&input[written..])
                    };

                    match write_res {
                        Ok(n) => {
                            written += n;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // Session is busy, wait a bit and retry
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        }
                        Err(e) => {
                            println!("Write error on {}: {}", server_name, e);
                            break;
                        }
                    }
                }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        println!("Terminal thread exited for: {}", server_name);
    });

    Ok(())
}

#[tauri::command]
async fn write_to_terminal(
    server_name: String,
    data: Vec<u8>,
    state: State<'_, AppState>
) -> Result<(), String> {
    let senders = state.senders.lock().unwrap();
    if let Some(tx) = senders.get(&server_name) {
        tx.send(data).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Terminal session not found".to_string())
    }
}

#[tauri::command]
async fn generate_ai_command(prompt: &str, api_key: &str) -> Result<String, String> {
    deepseek::generate_command(prompt, api_key).await
}

#[tauri::command]
async fn review_command_risk(command: &str, api_key: &str) -> Result<String, String> {
    deepseek::analyze_risk(command, api_key).await
}

mod xshell;

#[tauri::command]
async fn import_xshell_sessions(paths: Vec<String>) -> Result<Vec<xshell::XshellSession>, String> {
    let mut results = Vec::new();
    for path_str in paths {
        let path = std::path::Path::new(&path_str);
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        
        if ext == "xts" {
            let mut xts_sessions = xshell::parse_xts_file(path)?;
            results.append(&mut xts_sessions);
        } else {
            let session = xshell::parse_xsh_file(path)?;
            results.push(session);
        }
    }
    Ok(results)
}

#[tauri::command]
async fn list_remote_files(
    server_name: &str,
    path: &str,
    state: State<'_, AppState>
) -> Result<Vec<ssh_manager::RemoteFile>, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(server_name) {
        session.list_directory(path)
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn delete_remote_file(
    server_name: &str,
    path: &str,
    is_dir: bool,
    state: State<'_, AppState>
) -> Result<(), String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(server_name) {
        session.delete_file(path, is_dir)
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn download_remote_file(
    server_name: &str,
    path: &str,
    state: State<'_, AppState>
) -> Result<Vec<u8>, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(server_name) {
        session.download_file(path)
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn execute_script(
    server_name: &str,
    script_content: &str,
    state: State<'_, AppState>
) -> Result<String, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(server_name) {
        session.execute_command(script_content)
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn analyze_script_with_ai(script_content: &str, api_key: &str) -> Result<deepseek::ScriptAnalysisResult, String> {
    deepseek::analyze_script_with_ai(script_content, api_key).await
}

#[tauri::command]
async fn execute_remote_command(
    server_name: &str,
    command: &str,
    state: State<'_, AppState>
) -> Result<String, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(server_name) {
        session.execute_command(command)
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn get_server_sys_info(
    server_name: &str,
    state: State<'_, AppState>
) -> Result<ssh_manager::SysInfo, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(server_name) {
        session.get_sys_info()
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn get_docker_containers(
    serverName: &str,
    state: State<'_, AppState>
) -> Result<Vec<ssh_manager::DockerContainer>, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(serverName) {
        session.list_docker_containers()
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn manage_docker_container(
    serverName: &str,
    containerId: &str,
    action: &str,
    state: State<'_, AppState>
) -> Result<(), String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(serverName) {
        session.control_docker_container(containerId, action)
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn get_system_services(
    serverName: &str,
    state: State<'_, AppState>
) -> Result<Vec<ssh_manager::SystemService>, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(serverName) {
        session.list_system_services()
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn manage_system_service(
    serverName: &str,
    serviceName: &str,
    action: &str,
    state: State<'_, AppState>
) -> Result<(), String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.get(serverName) {
        session.manage_system_service(serviceName, action)
    } else {
        Err("Server not connected".to_string())
    }
}

#[tauri::command]
async fn diagnose_server_issue(
    serverName: &str,
    context: &str,
    apiKey: &str,
) -> Result<String, String> {
    let prompt = format!(
        "你是一个资深的 Linux 运维专家。请根据以下服务器的实时状态上下文，分析是否存在异常（如 CPU 过高、内存不足、容器崩溃、服务故障等），并给出具体的诊断结论和修复建议。\n\n服务器名称: {}\n上下文信息:\n{}\n\n请使用 Markdown 格式回复，语言为中文。",
        serverName, context
    );

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", apiKey))
        .json(&serde_json::json!({
            "model": "deepseek-chat",
            "messages": [
                {"role": "system", "content": "你是一个专业的服务器诊断助手。"},
                {"role": "user", "content": prompt}
            ],
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let res_json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let content = res_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Failed to get content from AI response")?;

    Ok(content.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            sessions: Mutex::new(HashMap::new()),
            senders: Mutex::new(HashMap::new()),
        })
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Some(monitor) = window.current_monitor()? {
                    let size = monitor.size();
                    let scale_factor = monitor.scale_factor();
                    let logical_size = size.to_logical::<f64>(scale_factor);
                    
                    let width = logical_size.width * 0.5;
                    let height = logical_size.height * 0.5;
                    
                    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))?;
                    window.center()?;
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:chat_ssh.db", vec![
                    tauri_plugin_sql::Migration {
                        version: 1,
                        description: "initial",
                        sql: include_str!("../migrations/initial.sql"),
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    },
                    tauri_plugin_sql::Migration {
                        version: 2,
                        description: "add_user_knowledge",
                        sql: "CREATE TABLE IF NOT EXISTS user_knowledge (id INTEGER PRIMARY KEY AUTOINCREMENT, command TEXT NOT NULL, description TEXT, tags TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP);",
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    },
                    tauri_plugin_sql::Migration {
                        version: 3,
                        description: "add_server_auth_fields",
                        sql: "ALTER TABLE servers ADD COLUMN auth_type TEXT NOT NULL DEFAULT 'password'; ALTER TABLE servers ADD COLUMN auth_secret TEXT;",
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    },
                    tauri_plugin_sql::Migration {
                        version: 4,
                        description: "add_scripts_and_password",
                        sql: "CREATE TABLE IF NOT EXISTS scripts (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, description TEXT, content TEXT NOT NULL, skip_warning BOOLEAN DEFAULT 0, created_at DATETIME DEFAULT CURRENT_TIMESTAMP); INSERT OR IGNORE INTO config (key, value) VALUES ('master_password', '');",
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    }
                ])
                .build()
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            disconnect_ssh,
            connect_ssh, 
            upload_to_server,
            open_terminal,
            write_to_terminal,
            generate_ai_command, 
            review_command_risk,
            import_xshell_sessions,
            list_remote_files,
            delete_remote_file,
            download_remote_file,
            execute_script,
            analyze_script_with_ai,
            execute_remote_command,
            get_server_sys_info,
            get_docker_containers,
            manage_docker_container,
            get_system_services,
            manage_system_service,
            diagnose_server_issue
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
