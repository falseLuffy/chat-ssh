use std::net::TcpStream;
use ssh2::Session;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

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
    pub os_info: String,
    pub net: NetInfo,
    pub processes: Vec<ProcessInfo>,
}

#[derive(serde::Serialize)]
pub struct NetInfo {
    pub rx_speed: f64, // bytes/sec
    pub tx_speed: f64, // bytes/sec
}

#[derive(serde::Serialize)]
pub struct ProcessInfo {
    pub pid: String,
    pub user: String,
    pub cpu: f32,
    pub mem: f32,
    pub command: String,
}

#[derive(serde::Serialize)]
pub struct FirewallRule {
    pub action: String,
    pub from: String,
    pub to: String,
    pub proto: String,
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DockerContainer {
    #[serde(alias = "ID")]
    pub id: String,
    #[serde(alias = "Names")]
    pub names: String,
    #[serde(alias = "Image")]
    pub image: String,
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(alias = "State")]
    pub state: String,
    #[serde(alias = "Ports")]
    pub ports: String,
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
    last_net_stats: Mutex<Option<(u64, u64, Instant)>>,
}

impl SshSession {
    pub fn new(session: Session) -> Self {
        Self {
            session: Mutex::new(session),
            last_net_stats: Mutex::new(None),
        }
    }

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

        Ok(SshSession::new(session))
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
        // 5. Net info (rx/tx bytes)
        // 6. Top 5 processes
        let cmd = "hostname; uptime -p; cat /proc/loadavg; free -b; df -k / | tail -n 1; cat /etc/os-release | grep PRETTY_NAME | cut -d'\"' -f2; cat /proc/net/dev | grep -E 'eth|ens|eno|wlan' | awk '{print $1, $2, $10}'; ps -eo pid,user,%cpu,%mem,comm --sort=-%cpu --no-headers | head -n 5";
        let raw_output = self.execute_command(cmd)?;
        let lines: Vec<&str> = raw_output.lines().collect();

        if lines.len() < 7 {
            return Err("Failed to get enough system info lines".to_string());
        }

        let hostname = lines[0].to_string();
        let uptime = lines[1].to_string();
        // free -b takes 3 lines (headers, Mem, Swap)
        // os_info is now at index 7
        let os_info = lines.get(7).unwrap_or(&"Unknown Linux").to_string();
        
        // Parse CPU (loadavg is index 2)
        let load_parts: Vec<&str> = lines[2].split_whitespace().collect();
        let load_1min = load_parts.get(0).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        
        // Parse Memory
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
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Disk lines must start with /dev or / and have 6 parts.
            // This prevents matching process lines like "/usr/bin/foo /"
            if parts.len() >= 6 && parts[0].starts_with('/') {
                let total_kb = parts[1].parse::<u64>().unwrap_or(0);
                let used_kb = parts[2].parse::<u64>().unwrap_or(0);
                let percent_str = parts[4].replace("%", "");
                let percent = percent_str.parse::<u32>().unwrap_or(0);
                let mount = parts[5].to_string();
                
                // Only include the main disk / or meaningful mounts
                if mount == "/" || mount.starts_with("/mnt") || mount.starts_with("/media") {
                    let total = total_kb * 1024;
                    let used = used_kb * 1024;
                    disks.push(DiskInfo { mount, total, used, percent });
                }
            }
        }

        // Parse Network
        let mut current_rx = 0;
        let mut current_tx = 0;
        for line in &lines {
            if line.contains(':') && (line.contains("eth") || line.contains("ens") || line.contains("eno") || line.contains("wlan")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    // parts[0] is "eth0:", parts[1] is RX, parts[2] is TX
                    current_rx += parts[1].parse::<u64>().unwrap_or(0);
                    current_tx += parts[2].parse::<u64>().unwrap_or(0);
                }
            }
        }

        let mut rx_speed = 0.0;
        let mut tx_speed = 0.0;
        let now = Instant::now();
        {
            let mut last_stats = self.last_net_stats.lock().unwrap();
            if let Some((prev_rx, prev_tx, prev_time)) = *last_stats {
                let duration = now.duration_since(prev_time).as_secs_f64();
                if duration > 0.0 {
                    rx_speed = (current_rx.saturating_sub(prev_rx)) as f64 / duration;
                    tx_speed = (current_tx.saturating_sub(prev_tx)) as f64 / duration;
                }
            }
            *last_stats = Some((current_rx, current_tx, now));
        }

        // Parse Processes
        let mut processes = Vec::new();
        for line in &lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Process lines start with a PID (number)
            if parts.len() >= 5 && parts[0].chars().all(|c| c.is_digit(10)) {
                let pid = parts[0].to_string();
                let user = parts[1].to_string();
                let cpu = parts[2].parse::<f32>().unwrap_or(0.0);
                let mem = parts[3].parse::<f32>().unwrap_or(0.0);
                let command = parts[4..].join(" ");
                processes.push(ProcessInfo { pid, user, cpu, mem, command });
            }
        }

        Ok(SysInfo {
            hostname,
            uptime,
            os_info,
            cpu: CpuInfo { usage: load_1min },
            memory: MemInfo { total: mem_total, used: mem_used, free: mem_free },
            disks,
            net: NetInfo { rx_speed, tx_speed },
            processes,
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
            match serde_json::from_str::<DockerContainer>(line) {
                Ok(container) => containers.push(container),
                Err(e) => println!("Failed to parse docker line: {} - Error: {}", line, e),
            }
        }
        
        if containers.is_empty() && !output.is_empty() {
            println!("No containers parsed. Raw output: {}", output);
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
        self.execute_command(&cmd)?;
        Ok(())
    }

    pub fn kill_process(&self, pid: &str) -> Result<(), String> {
        let cmd = format!("sudo kill -9 {}", pid);
        self.execute_command(&cmd)?;
        Ok(())
    }

    pub fn get_firewall_rules(&self) -> Result<Vec<FirewallRule>, String> {
        let mut rules = Vec::new();
        
        // 1. Try UFW (Ubuntu/Debian)
        let ufw_res = self.execute_command("sudo ufw status | grep -E 'ALLOW|DENY'");
        if let Ok(output) = ufw_res {
            for line in output.lines() {
                // Stop if we hit the STDERR section or skip error/usage lines
                if line.contains("--- STDERR") { break; }
                if line.contains("To") || line.contains("---") || line.contains("sudo:") || 
                   line.to_lowercase().contains("usage:") || line.to_lowercase().contains("not found") {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    rules.push(FirewallRule { 
                        to: parts[0].to_string(), 
                        action: parts[1].to_string(), 
                        from: parts.get(2).unwrap_or(&"anywhere").to_string(), 
                        proto: "any".to_string() 
                    });
                }
            }
            if !rules.is_empty() { return Ok(rules); }
        }

        // 2. Try Firewall-cmd (CentOS/RHEL)
        let fw_cmd_res = self.execute_command("sudo firewall-cmd --list-ports --list-services");
        if let Ok(output) = fw_cmd_res {
            // Stop if we hit the STDERR section
            let clean_output = if let Some(idx) = output.find("--- STDERR") {
                &output[..idx]
            } else {
                &output
            };

            if clean_output.contains("not found") || clean_output.contains("usage:") {
                return Ok(Vec::new());
            }
            
            for line in clean_output.lines() {
                for part in line.split_whitespace() {
                    if part == "sudo:" || part == "---" { continue; }
                    rules.push(FirewallRule {
                        to: part.to_string(),
                        action: "ALLOW (firewalld)".to_string(),
                        from: "anywhere".to_string(),
                        proto: if part.contains('/') { "tcp/udp".to_string() } else { "service".to_string() }
                    });
                }
            }
        }
        
        Ok(rules)
    }

    pub fn manage_firewall_rule(&self, port: &str, action: &str) -> Result<(), String> {
        // Try UFW first
        let ufw_check = self.execute_command("which ufw");
        if ufw_check.is_ok() {
            let cmd = format!("sudo ufw {} {}", action, port);
            self.execute_command(&cmd)?;
        } else {
            // Try firewalld
            let cmd = if action == "allow" {
                format!("sudo firewall-cmd --permanent --add-port={}/tcp && sudo firewall-cmd --reload", port)
            } else {
                format!("sudo firewall-cmd --permanent --remove-port={}/tcp && sudo firewall-cmd --reload", port)
            };
            self.execute_command(&cmd)?;
        }
        Ok(())
    }

    pub fn get_session(&self) -> &Mutex<Session> {
        &self.session
    }
}
