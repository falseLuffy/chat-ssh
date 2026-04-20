use ini::Ini;
use encoding_rs::{UTF_16LE, GB18030};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::path::Path;
use std::io::Read;
use serde::Serialize;
use windows_sys::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB as DATA_BLOB};
use windows_sys::Win32::Foundation::LocalFree;
use std::ptr;

#[derive(Debug, Serialize)]
pub struct XshellSession {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
}

pub fn parse_xsh_bytes(bytes: &[u8], name: String) -> Result<XshellSession, String> {
    // 1. Try UTF-16 LE
    let (decoded, _, malformed) = UTF_16LE.decode(bytes);
    let final_content = if !malformed && decoded.trim().starts_with('[') {
        decoded.to_string()
    } else {
        // 2. Try GB18030 (Common in Chinese Xshell)
        let (decoded_gb, _, malformed_gb) = GB18030.decode(bytes);
        if !malformed_gb && decoded_gb.trim().starts_with('[') {
            decoded_gb.to_string()
        } else {
            // 3. Fallback to UTF-8
            String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| decoded.to_string())
        }
    };

    println!("    Determined content starts with: {}", final_content.chars().take(20).collect::<String>());

    let conf = Ini::load_from_str(&final_content).map_err(|e| {
        format!("INI parse failed for {}: {}", name, e)
    })?;

    let connection = conf.section(Some("Connection")).ok_or_else(|| format!("{}: Missing Connection section", name))?;
    let host = connection.get("Host").ok_or_else(|| format!("{}: Missing Host", name))?.to_string();
    let port = connection.get("Port").unwrap_or("22").parse::<u16>().unwrap_or(22);

    let auth = conf.section(Some("Authentication")).ok_or_else(|| format!("{}: Missing Authentication section", name))?;
    let user = auth.get("UserName").unwrap_or("root").to_string();
    
    let password_enc = auth.get("Password");
    let password = password_enc.and_then(|enc| decrypt_dpapi(enc).ok());

    Ok(XshellSession {
        name,
        host,
        port,
        user,
        password,
    })
}

pub fn parse_xsh_file(path: &Path) -> Result<XshellSession, String> {
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Session")
        .to_string();

    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    parse_xsh_bytes(&bytes, name)
}

pub fn parse_xts_file(path: &Path) -> Result<Vec<XshellSession>, String> {
    println!("Opening XTS file: {:?}", path);
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut sessions = Vec::new();

    println!("ZIP archive contains {} files", archive.len());

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        
        // Attempt to decode filename (many ZIPs use GBK for Chinese filenames)
        let raw_name = file.name_raw();
        let (decoded_name, _, _) = GB18030.decode(raw_name);
        let name_str = if !decoded_name.contains('\u{FFFD}') && decoded_name.contains(|c: char| c as u32 > 127) {
            decoded_name.to_string()
        } else {
            file.name().to_string()
        };

        println!("  Checking file in ZIP: {}", name_str);
        
        if file.is_file() && name_str.to_lowercase().ends_with(".xsh") {
            let mut buffer = Vec::new();
            if let Err(e) = file.read_to_end(&mut buffer) {
                println!("    Failed to read file {}: {}", name_str, e);
                continue;
            }
            
            let filename_path = Path::new(&name_str);
            let session_name = filename_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            println!("    Attempting to parse session: {}", session_name);
            match parse_xsh_bytes(&buffer, session_name.clone()) {
                Ok(session) => {
                    println!("    Successfully parsed session: {}", session_name);
                    sessions.push(session);
                }
                Err(e) => {
                    println!("    Failed to parse session {}: {}", session_name, e);
                }
            }
        }
    }
    
    if sessions.is_empty() {
        println!("No valid .xsh sessions found in the archive.");
    } else {
        println!("Found {} sessions in the archive.", sessions.len());
    }
    
    Ok(sessions)
}

fn decrypt_dpapi(encrypted_base64: &str) -> Result<String, String> {
    let encrypted_bytes = STANDARD.decode(encrypted_base64).map_err(|e| e.to_string())?;
    
    unsafe {
        let mut input = DATA_BLOB {
            cbData: encrypted_bytes.len() as u32,
            pbData: encrypted_bytes.as_ptr() as *mut u8,
        };
        let mut output = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        let result = CryptUnprotectData(
            &mut input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut output,
        );

        if result != 0 {
            let data = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
            let password = String::from_utf8(data.to_vec()).map_err(|e| e.to_string())?;
            LocalFree(output.pbData as _);
            Ok(password)
        } else {
            Err("DPAPI decryption failed".to_string())
        }
    }
}
