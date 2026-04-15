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

        loop {
            // 1. Read from SSH (Non-blocking)
            match channel.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let data = buffer[..n].to_vec();
                    window.emit("ssh-data", data).unwrap();
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
                    match channel.write(&input[written..]) {
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
                if written > 0 {
                    // println!("Wrote {} bytes to SSH", written);
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
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            connect_ssh, 
            upload_to_server,
            open_terminal,
            write_to_terminal,
            generate_ai_command, 
            review_command_risk
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
