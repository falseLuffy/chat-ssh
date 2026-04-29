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
        let sftp_res = session.sftp().map_err(|e| e.to_string());
        let res = match sftp_res {
            Ok(sftp) => {
                if is_dir {
                    sftp.rmdir(Path::new(path)).map_err(|e| e.to_string())
                } else {
                    sftp.unlink(Path::new(path)).map_err(|e| e.to_string())
                }
            }
            Err(e) => Err(e),
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

    pub fn get_session(&self) -> &Mutex<Session> {
        &self.session
    }
}
