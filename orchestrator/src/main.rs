mod config;
mod github;
mod billing;

use std::thread;
use std::time::{Duration, Instant};
use std::env;

const STATE_FILE: &str = "state.json";
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(4 * 3600); // 4 jam

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("❌ ERROR: Nama repo belum diberikan!");
        eprintln!("Gunakan: cargo run --release -- username/nama-repo");
        return;
    }
    let repo_name = &args[1];

    println!("==================================================");
    println!("   MAWARI & NEXUS MULTI-NODE ORCHESTRATOR");
    println!("==================================================");
    
    println!("\nMemuat tokens.json...");
    let config = match config::load_config("tokens.json") {
        Ok(cfg) => cfg,
        Err(e) => { eprintln!("FATAL: {}", e); return; }
    };
    
    println!("Berhasil memuat {} token", config.tokens.len());
    println!("Target Repo: {}", repo_name);

    let mut state = config::load_state(STATE_FILE).unwrap_or_default();
    let mut current_token_index = state.current_account_index;

    if current_token_index > 0 {
        println!("Melanjutkan dari token indeks: {}", current_token_index);
    }

    loop {
        let token = &config.tokens[current_token_index];
        
        println!("\n==================================================");
        println!("Menggunakan Token #{} dari {}", current_token_index + 1, config.tokens.len());
        println!("==================================================");
        
        let username = match github::get_username(token) {
            Ok(u) => { println!("✅ Token valid untuk: @{}", u); u }
            Err(github::GHError::AuthError(msg)) => {
                eprintln!("❌ Token TIDAK VALID: {}", msg.lines().next().unwrap_or(""));
                current_token_index = (current_token_index + 1) % config.tokens.len();
                state.current_account_index = current_token_index;
                config::save_state(STATE_FILE, &state).ok();
                thread::sleep(Duration::from_secs(3));
                continue;
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                current_token_index = (current_token_index + 1) % config.tokens.len();
                state.current_account_index = current_token_index;
                config::save_state(STATE_FILE, &state).ok();
                thread::sleep(Duration::from_secs(3));
                continue;
            }
        };

        println!("\nMengecek kuota billing...");
        let billing = billing::get_billing_info(token, &username).unwrap();

        if !billing.is_quota_ok {
            eprintln!("   Kuota tidak cukup. Beralih ke akun berikutnya...\n");
            current_token_index = (current_token_index + 1) % config.tokens.len();
            state.current_account_index = current_token_index;
            config::save_state(STATE_FILE, &state).ok();
            thread::sleep(Duration::from_secs(3));
            continue;
        }

        let (mawari_name, nexus_name) = match github::ensure_healthy_codespaces(token, repo_name) {
            Ok(names) => names,
            Err(e) => {
                eprintln!("❌ Deployment gagal: {}", e);
                eprintln!("   Mencoba lagi dalam 5 menit...\n");
                thread::sleep(Duration::from_secs(5 * 60));
                continue;
            }
        };

        println!("\n==================================================");
        println!("         ✅ DEPLOYMENT BERHASIL");
        println!("==================================================");
        println!("Akun     : @{}", username);
        println!("Mawari CS: {}", mawari_name);
        println!("Nexus CS : {}", nexus_name);
        
        state.current_account_index = current_token_index;
        state.mawari_codespace_name = mawari_name.clone();
        state.nexus_codespace_name = nexus_name.clone();
        config::save_state(STATE_FILE, &state).ok();
        
        println!("State berhasil disimpan.");
        
        let run_duration_hours = (billing.hours_remaining - 1.0).max(1.0).min(20.0);
        let run_duration = Duration::from_secs((run_duration_hours * 3600.0) as u64);
        
        println!("\nNode akan berjalan selama {:.1} jam", run_duration_hours);
        println!("Keep-alive akan dijalankan setiap 4 jam.\n");
        
        let start_time = Instant::now();
        
        while start_time.elapsed() < run_duration {
            let remaining_time = run_duration.saturating_sub(start_time.elapsed());
            let sleep_time = std::cmp::min(KEEP_ALIVE_INTERVAL, remaining_time);

            if sleep_time.as_secs() > 60 {
                 println!("Siklus keep-alive berikutnya dalam {:.1} jam...", sleep_time.as_secs_f32() / 3600.0);
                 thread::sleep(sleep_time);
            } else { break; }

            if start_time.elapsed() >= run_duration { break; }
            
            println!("\n--- MENJALANKAN SIKLUS KEEP-ALIVE ---");
            github::wait_and_run_startup_script(token, &mawari_name, "/workspaces/Mawari-Nexus-Multi/mawari/auto-start.sh").ok();
            github::wait_and_run_startup_script(token, &nexus_name, "/workspaces/Mawari-Nexus-Multi/nexus/auto-start.sh").ok();
            println!("--- SIKLUS KEEP-ALIVE SELESAI ---\n");
        }
        
        println!("\n==================================================");
        println!("Siklus Selesai! Durasi: {:.1} jam", run_duration_hours);
        println!("Beralih ke token berikutnya...");
        println!("==================================================\n");
        
        current_token_index = (current_token_index + 1) % config.tokens.len();
        state.current_account_index = current_token_index;
        config::save_state(STATE_FILE, &state).ok();
    }
}
