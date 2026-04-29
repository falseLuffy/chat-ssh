use std::net::TcpStream;
use ssh2::Session;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;

#[derive(serde::Serialize)]
pub struct RemoteFile {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: u64,
}

#[derive(serde::Serialize)]
pub struct SysInfo {
    pub cpu: CpuInfo,
    pub memory: MemInfo,
    pub disks: Vec<DiskInfo>,
    pub uptime: String,
    pub hostname: String,
}

#[derive(serde::Serialize)]
pub struct CpuInfo {
    pub usage: f64,
}

#[derive(serde::Serialize)]
pub struct MemInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(serde::Serialize)]
pub struct DiskInfo {
    pub mount: String,
    pub total: u64,
    pub used: u64,
    pub percent: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DockerContainer {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Names")]
    pub names: String,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "Ports")]
    pub ports: String,
    #[serde(rename = "Names")]
    pub names_alt: Option<String>, // Some versions might have different casing
}

#[derive(serde::Serialize)]
pub struct SystemService {
    pub name: String,
    pub load: String,
    pub active: String,
    pub sub: String,
    pub description: String,
}

pub struct SshSession {
    session: Mutex<Session>,
}

impl SshSession {
    pub fn connect(host: &str, username: &str, password: Option<&str>) -> Result<Self, String> {
        let final_host = if host.contains(':') {
            host.to_string()
        } else {
            format!("{}:22", host)
        };

        let tcp = TcpStream::connect(&final_host)
            .map_err(|e| format!("Network connection failed ({}): {}", final_host, e))?;
        
        let mut session = Session::new().map_err(|e| e.to_string())?;
        session.set_tcp_stream(tcp);
        session.handshake().map_err(|e| format!("SSH handshake failed: {}", e))?;

        if let Some(pw) = password {
            session.userauth_password(username, pw).map_err(|e| format!("Authentication failed: {}", e))?;
        } else {
            return Err("Password required for authentication".to_string());
        }

        if !session.authenticated() {
            return Err("Authentication failed: User not authorized".to_string());
        }

        Ok(SshSession { session: Mutex::new(session) })
    }

    pub fn upload_file(&self, local_data: &[u8], remote_path: &str) -> Result<(), String> {
        let session = self.session.lock().unwrap();
        session.set_blocking(true);
        let res = (|| {
            let mut remote_file = session
                .scp_send(Path::new(remote_path), 0o644, local_data.len() as u64, None)
                .map_err(|e| e.to_string())?;
            remote_file.write_all(local_data).map_err(|e| e.to_string())?;
            remote_file.send_eof().map_err(|e| e.to_string())?;
            remote_file.wait_eof().map_err(|e| e.to_string())?;
            remote_file.close().map_err(|e| e.to_string())?;
            remote_file.wait_close().map_err(|e| e.to_string())?;
            Ok(())
        })();
        session.set_blocking(false);
        res
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<RemoteFile>, String> {
        let session = self.session.lock().unwrap();
        session.set_blocking(true);
        let sftp_res = session.sftp().map_err(|e| format!("SFTP init failed: {}", e));
        
        let res = match sftp_res {
            Ok(sftp) => {
                let dir_res = sftp.readdir(Path::new(path)).map_err(|e| format!("Read dir failed for {}: {}", path, e));
                match dir_res {
                    Ok(dir) => {
                        let mut files = Vec::new();
                        for (p, stat) in dir {
                            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                            if !name.is_empty() && name != "." && name != ".." {
                                files.push(RemoteFile {
                                    name,
                                    size: stat.size.unwrap_or(0),
                                    is_dir: stat.is_dir(),
                                    modified: stat.mtime.unwrap_or(0),
                                });
                            }
                        }
                        files.sort_by(|a, b| {
                            if a.is_dir != b.is_dir {
                                b.is_dir.cmp(&a.is_dir)
                            } else {
                                a.name.cmp(&b.name)
                            }
                        });
                        Ok(files)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        };
        
        session.set_blocking(false);
        res
    }

    pub fn delete_file(&self, path: &str, is_dir: bool) -> Result<(), String> {
        let session = self.session.lock().unwrap();
        session.set_blocking(true);
        let res = if is_dir {
            // Use rm -rf for directories (sftp.rmdir only deletes empty dirs)
            let mut channel = session.channel_session().map_err(|e| e.to_string())?;
            let cmd = format!("rm -rf \"{}\" 2>&1", path);
            channel.exec(&cmd).map_err(|e| e.to_string())?;
            let mut output = String::new();
            channel.read_to_string(&mut output).map_err(|e| e.to_string())?;
            channel.wait_close().map_err(|e| e.to_string())?;
            let exit_status = channel.exit_status().map_err(|e| e.to_string())?;
            if exit_status != 0 {
                Err(format!("rm -rf failed (exit {}): {}", exit_status, output.trim()))
            } else {
                Ok(())
            }
        } else {
            let sftp = session.sftp().map_err(|e| e.to_string())?;
            sftp.unlink(Path::new(path)).map_err(|e| e.to_string())
        };
        session.set_blocking(false);
        res
    }

    pub fn download_file(&self, path: &str) -> Result<Vec<u8>, String> {
        let session = self.session.lock().unwrap();
        session.set_blocking(true);
        let sftp_res = session.sftp().map_err(|e| e.to_string());
        let res = match sftp_res {
            Ok(sftp) => {
                let file_res = sftp.open(Path::new(path)).map_err(|e| e.to_string());
                match file_res {
                    Ok(mut file) => {
                        let mut data = Vec::new();
                        let read_res = file.read_to_end(&mut data).map_err(|e| e.to_string());
                        match read_res {
                            Ok(_) => Ok(data),
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        };
        session.set_blocking(false);
        res
    }

    pub fn start_shell(&self, cols: u32, rows: u32) -> Result<ssh2::Channel, String> {
        let session = self.session.lock().unwrap();
        let mut channel = session.channel_session().map_err(|e| e.to_string())?;
        channel.request_pty("xterm-256color", None, Some((cols, rows, 0, 0))).map_err(|e| e.to_string())?;
        channel.shell().map_err(|e| e.to_string())?;
        Ok(channel)
    }

    pub fn resize_pty(&self, channel: &mut ssh2::Channel, cols: u32, rows: u32) -> Result<(), String> {
        channel.request_pty_size(cols, rows, None, None).map_err(|e| e.to_string())
    }

    pub fn set_blocking(&self, blocking: bool) {
        let session = self.session.lock().unwrap();
        session.set_blocking(blocking);
    }

    pub fn execute_command(&self, command: &str) -> Result<String, String> {
        let session = self.session.lock().unwrap();
        session.set_blocking(true);
        let mut channel = session.channel_session().map_err(|e| e.to_string())?;
        
        // Execute the command
        channel.exec(command).map_err(|e| e.to_string())?;
        
        let mut output = String::new();
        channel.read_to_string(&mut output).map_err(|e| e.to_string())?;
        
        // Try to read stderr as well, optionally
        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr).unwrap_or(0);
        
        if !stderr.is_empty() {
            output.push_str("\n--- STDERR ---\n");
            output.push_str(&stderr);
        }

        channel.wait_close().map_err(|e| e.to_string())?;
        session.set_blocking(false);
        Ok(output)
    }

    pub fn get_sys_info(&self) -> Result<SysInfo, String> {
        // Execute multiple commands to get all info
        // 1. Hostname & Uptime
        // 2. CPU Load (1min)
        // 3. Memory info (bytes)
        // 4. Disk info (bytes)
        let cmd = "hostname; uptime -p; cat /proc/loadavg; free -b; df -k / | tail -n 1";
        let raw_output = self.execute_command(cmd)?;
        let lines: Vec<&str> = raw_output.lines().collect();

        if lines.len() < 5 {
            return Err("Failed to get enough system info lines".to_string());
        }

        let hostname = lines[0].to_string();
        let uptime = lines[1].to_string();
        
        // Parse CPU (e.g. 0.05 0.03 0.01 1/123 4567)
        let load_parts: Vec<&str> = lines[2].split_whitespace().collect();
        let load_1min = load_parts.get(0).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        
        // Parse Memory (e.g. Mem: 16453775360 4537753600 11916021760 ...)
        let mut mem_total = 0;
        let mut mem_used = 0;
        let mut mem_free = 0;
        for line in &lines {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                mem_total = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                mem_used = parts.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                mem_free = parts.get(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                break;
            }
        }

        // Parse Disk
        let mut disks = Vec::new();
        for line in &lines {
            // Looking for lines that look like df output for a mount point
            let parts: Vec<&str> = line.split_whitespace().collect();
            // df -k output typically has 6 columns. Data line starts with /dev or similar.
            if parts.len() >= 6 && (parts[0].starts_with('/') || parts[5] == "/") {
                let total_kb = parts[1].parse::<u64>().unwrap_or(0);
                let used_kb = parts[2].parse::<u64>().unwrap_or(0);
                let percent_str = parts[4].replace("%", "");
                let percent = percent_str.parse::<u32>().unwrap_or(0);
                let mount = parts[5].to_string();
                
                // Convert KB to Bytes
                let total = total_kb * 1024;
                let used = used_kb * 1024;
                
                disks.push(DiskInfo { mount, total, used, percent });
            }
        }

        Ok(SysInfo {
            hostname,
            uptime,
            cpu: CpuInfo { usage: load_1min },
            memory: MemInfo { total: mem_total, used: mem_used, free: mem_free },
            disks,
        })
    }

    pub fn list_docker_containers(&self) -> Result<Vec<DockerContainer>, String> {
        let cmd = "docker ps -a --format '{{json .}}'";
        let output = self.execute_command(cmd)?;
        let mut containers = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() || line.starts_with("--- STDERR") {
                continue;
            }
            if let Ok(container) = serde_json::from_str::<DockerContainer>(line) {
                containers.push(container);
            }
        }
        Ok(containers)
    }

    pub fn control_docker_container(&self, container_id: &str, action: &str) -> Result<(), String> {
        let valid_actions = ["start", "stop", "restart", "pause", "unpause", "rm"];
        if !valid_actions.contains(&action) {
            return Err("Invalid docker action".to_string());
        }
        let cmd = format!("docker {} {}", action, container_id);
        self.execute_command(&cmd)?;
        Ok(())
    }

    pub fn list_system_services(&self) -> Result<Vec<SystemService>, String> {
        // systemctl list-units --type=service --all --no-pager --no-legend
        // Output format: UNIT LOAD ACTIVE SUB DESCRIPTION
        let cmd = "systemctl list-units --type=service --all --no-pager --no-legend";
        let output = self.execute_command(cmd)?;
        let mut services = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() || line.starts_with("--- STDERR") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let load = parts[1].to_string();
                let active = parts[2].to_string();
                let sub = parts[3].to_string();
                let description = parts[4..].join(" ");
                services.push(SystemService { name, load, active, sub, description });
            }
        }
        Ok(services)
    }

    pub fn manage_system_service(&self, service_name: &str, action: &str) -> Result<(), String> {
        let valid_actions = ["start", "stop", "restart", "enable", "disable"];
        if !valid_actions.contains(&action) {
            return Err("Invalid service action".to_string());
        }
        let cmd = format!("sudo systemctl {} {}", action, service_name);
        // Note: This might require sudo without password or handled via pty if password needed
        // For now we assume standard cloud user or sudoers config
        self.execute_command(&cmd)?;
        Ok(())
    }

    pub fn get_session(&self) -> &Mutex<Session> {
        &self.session
    }
}
