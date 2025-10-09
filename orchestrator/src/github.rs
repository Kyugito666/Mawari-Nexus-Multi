use std::process::Command;
use std::fmt;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum GHError {
    CommandError(String),
    AuthError(String),
}

impl fmt::Display for GHError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GHError::CommandError(e) => write!(f, "Command Gagal: {}", e),
            GHError::AuthError(e) => write!(f, "Auth Error: {}", e),
        }
    }
}

fn run_gh_command(token: &str, args: &[&str]) -> Result<String, GHError> {
    eprintln!("DEBUG: gh {}", args.join(" "));
    
    let output = Command::new("gh")
        .args(args)
        .env("GH_TOKEN", token)
        .output()
        .map_err(|e| GHError::CommandError(format!("Gagal eksekusi gh: {}", e)))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    if !output.status.success() {
        if stderr.contains("Bad credentials") || stderr.contains("authentication required") || stderr.contains("HTTP 401") {
            return Err(GHError::AuthError(stderr));
        }
        if stderr.contains("no codespaces found") || stdout.trim().is_empty() {
            return Ok("".to_string());
        }
        return Err(GHError::CommandError(stderr));
    }
    
    Ok(stdout.trim().to_string())
}

pub fn get_username(token: &str) -> Result<String, GHError> {
    run_gh_command(token, &["api", "user", "--jq", ".login"])
}

fn stop_codespace(token: &str, name: &str) -> Result<(), GHError> {
    println!("      Menghentikan '{}'...", name);
    match run_gh_command(token, &["codespace", "stop", "-c", name]) {
        Ok(_) => { println!("      Berhasil dihentikan."); thread::sleep(Duration::from_secs(3)); Ok(()) }
        Err(e) => { eprintln!("      Peringatan saat menghentikan: {}", e); Ok(()) }
    }
}

fn delete_codespace(token: &str, name: &str) -> Result<(), GHError> {
    println!("      Menghapus '{}'...", name);
    match run_gh_command(token, &["codespace", "delete", "-c", name, "--force"]) {
        Ok(_) => { println!("      Berhasil dihapus."); thread::sleep(Duration::from_secs(3)); Ok(()) }
        Err(e) => { eprintln!("      Gagal menghapus, melanjutkan...: {}", e); Ok(()) }
    }
}

fn health_check(token: &str, name: &str, marker_file: &str) -> bool {
    let check_cmd = format!("test -f {} && echo 'healthy'", marker_file);
    match run_gh_command(token, &["codespace", "ssh", "-c", name, "--", &check_cmd]) {
        Ok(output) if output.contains("healthy") => true,
        _ => false,
    }
}

pub fn wait_and_run_startup_script(token: &str, name: &str, script_path: &str) -> Result<(), GHError> {
    println!("   Memverifikasi dan menjalankan node di '{}'...", name);
    
    for attempt in 1..=15 {
        println!("      Attempt {}/15: Mengecek kesiapan SSH...", attempt);
        
        match run_gh_command(token, &["codespace", "ssh", "-c", name, "--", "echo 'ready'"]) {
            Ok(output) if output.contains("ready") => {
                println!("      ✅ SSH sudah siap!");
                let exec_command = format!("bash -l -c 'bash {}'", script_path);
                
                println!("      🚀 Menjalankan skrip auto-start: {}", script_path);
                match run_gh_command(token, &["codespace", "ssh", "-c", name, "--", &exec_command]) {
                    Ok(_) => { println!("      ✅ Perintah eksekusi skrip berhasil dikirim."); return Ok(()); },
                    Err(e) => { eprintln!("      ⚠️  Peringatan saat eksekusi skrip: {}", e.to_string()); return Ok(()); }
                }
            },
            _ => { println!("      ... Belum siap."); }
        }
        
        if attempt < 15 {
            println!("      Menunggu 30 detik...");
            thread::sleep(Duration::from_secs(30));
        }
    }
    
    Err(GHError::CommandError(format!("Timeout: SSH tidak siap untuk '{}'", name)))
}

pub fn ensure_healthy_codespaces(token: &str, repo: &str) -> Result<(String, String), GHError> {
    println!("  Mengecek Codespace yang ada...");
    
    let mut mawari_name = String::new();
    let mut nexus_name = String::new();

    let list_output = run_gh_command(token, &["codespace", "list", "--json", "name,repository,state,displayName"])?;
    
    if !list_output.is_empty() {
        if let Ok(codespaces) = serde_json::from_str::<Vec<serde_json::Value>>(&list_output) {
            for cs in codespaces {
                if cs["repository"].as_str().unwrap_or("") != repo { continue; }

                let name = cs["name"].as_str().unwrap_or("").to_string();
                let state = cs["state"].as_str().unwrap_or("").to_string();
                let display_name = cs["displayName"].as_str().unwrap_or("");

                let process_node = |current_name: &mut String, target_display: &str, marker_file: &str| -> Result<(), GHError> {
                    if display_name == target_display {
                        println!("  -> Ditemukan '{}': {} (State: {})", target_display, name, state);
                        
                        if state == "Available" && health_check(token, &name, marker_file) {
                            println!("    ✅ Health check LULUS. Digunakan kembali.");
                            *current_name = name.clone();
                        } else {
                            println!("    ❌ Health check GAGAL atau state tidak 'Available'. Dibuat ulang...");
                            if state == "Available" || state == "Running" { stop_codespace(token, &name)?; }
                            delete_codespace(token, &name)?;
                        }
                    }
                    Ok(())
                };

                process_node(&mut mawari_name, "mawari-nodes", "/tmp/mawari_auto_start_done")?;
                process_node(&mut nexus_name, "nexus-nodes", "/tmp/nexus_auto_start_done")?;
            }
        }
    }

    if mawari_name.is_empty() {
        println!("  Membuat codespace 'mawari-nodes'...");
        let new_name = run_gh_command(token, &["codespace", "create", "-r", repo, "-m", "standardLinux32gb", "--display-name", "mawari-nodes", "--idle-timeout", "240m"])?;
        if new_name.is_empty() { return Err(GHError::CommandError("Gagal membuat codespace mawari".to_string())); }
        
        mawari_name = new_name;
        println!("     ✅ Berhasil dibuat: {}", mawari_name);
        wait_and_run_startup_script(token, &mawari_name, "/workspaces/Mawari-Nexus-Multi/mawari/auto-start.sh")?;
    }
    
    println!("\n  Menunggu 10 detik sebelum lanjut...\n");
    thread::sleep(Duration::from_secs(10));
    
    if nexus_name.is_empty() {
        println!("  Membuat codespace 'nexus-nodes'...");
        let new_name = run_gh_command(token, &["codespace", "create", "-r", repo, "-m", "standardLinux32gb", "--display-name", "nexus-nodes", "--idle-timeout", "240m"])?;
        if new_name.is_empty() { return Err(GHError::CommandError("Gagal membuat codespace nexus".to_string())); }

        nexus_name = new_name;
        println!("     ✅ Berhasil dibuat: {}", nexus_name);
        wait_and_run_startup_script(token, &nexus_name, "/workspaces/Mawari-Nexus-Multi/nexus/auto-start.sh")?;
    }

    println!("\n  ✅ Kedua codespace siap!");
    Ok((mawari_name, nexus_name))
}
