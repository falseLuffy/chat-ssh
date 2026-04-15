use std::net::TcpStream;
use ssh2::Session;
use std::io::prelude::*;
use std::path::Path;

pub struct SshSession {
    session: Session,
}

impl SshSession {
    pub fn connect(host: &str, username: &str, password: Option<&str>) -> Result<Self, String> {
        let tcp = TcpStream::connect(host).map_err(|e| e.to_string())?;
        let mut session = Session::new().map_err(|e| e.to_string())?;
        session.set_tcp_stream(tcp);
        session.handshake().map_err(|e| e.to_string())?;

        if let Some(pw) = password {
            session.userauth_password(username, pw).map_err(|e| e.to_string())?;
        } else {
            // For now, only password auth is implemented. Key auth can be added later.
            return Err("Password required for authentication".to_string());
        }

        if !session.authenticated() {
            return Err("Authentication failed".to_string());
        }

        Ok(SshSession { session })
    }

    pub fn execute_command(&self, command: &str) -> Result<String, String> {
        let mut channel = self.session.channel_session().map_err(|e| e.to_string())?;
        channel.exec(command).map_err(|e| e.to_string())?;
        let mut s = String::new();
        channel.read_to_string(&mut s).map_err(|e| e.to_string())?;
        channel.wait_close().map_err(|e| e.to_string())?;
        Ok(s)
    }

    pub fn upload_file(&self, local_data: &[u8], remote_path: &str) -> Result<(), String> {
        let mut remote_file = self.session
            .scp_send(Path::new(remote_path), 0o644, local_data.len() as u64, None)
            .map_err(|e| e.to_string())?;
        remote_file.write_all(local_data).map_err(|e| e.to_string())?;
        remote_file.send_eof().map_err(|e| e.to_string())?;
        remote_file.wait_eof().map_err(|e| e.to_string())?;
        remote_file.close().map_err(|e| e.to_string())?;
        remote_file.wait_close().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<String>, String> {
        let sftp = self.session.sftp().map_err(|e| e.to_string())?;
        let mut dir = sftp.readdir(Path::new(path)).map_err(|e| e.to_string())?;
        let mut files = Vec::new();
        for (path, stat) in dir.drain(..) {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if !name.is_empty() {
                let file_type = if stat.is_dir() { "[DIR]" } else { "[FILE]" };
                files.push(format!("{} {}", file_type, name));
            }
        }
        Ok(files)
    }

    pub fn start_shell(&self, cols: u32, rows: u32) -> Result<ssh2::Channel, String> {
        let mut channel = self.session.channel_session().map_err(|e| e.to_string())?;
        channel.request_pty("xterm-256color", None, Some((cols, rows, 0, 0))).map_err(|e| e.to_string())?;
        channel.shell().map_err(|e| e.to_string())?;
        Ok(channel)
    }

    pub fn resize_pty(&self, channel: &mut ssh2::Channel, cols: u32, rows: u32) -> Result<(), String> {
        channel.request_pty_size(cols, rows, None, None).map_err(|e| e.to_string())
    }

    pub fn set_blocking(&self, blocking: bool) {
        self.session.set_blocking(blocking);
    }
}
