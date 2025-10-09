import json
import subprocess
import os
import time

# ==========================================================
# KONFIGURASI
# ==========================================================
MAIN_TOKEN_CONFIG = "ghp_" # Ganti dengan token utama Anda
MAIN_ACCOUNT_USERNAME = "Kyugito666" 
BLUEPRINT_REPO_NAME = "Mawari-Nexus-Multi"
# ==========================================================

TOKEN_CACHE_FILE = 'token_cache.json'
INVITED_USERS_FILE = 'invited_users.txt' # File baru untuk tracking

def run_command(command, env=None, input_data=None):
    try:
        process = subprocess.run(
            command, shell=True, check=True, capture_output=True,
            text=True, encoding='utf-8', env=env, input=input_data
        )
        return (True, process.stdout.strip())
    except subprocess.CalledProcessError as e:
        return (False, f"{e.stdout.strip()} {e.stderr.strip()}")

def load_json_file(filename):
    try:
        with open(filename, 'r') as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return {} # Mengembalikan dictionary kosong jika file tidak ada atau error

def save_json_file(filename, data):
    with open(filename, 'w') as f:
        json.dump(data, f, indent=4)

def load_lines_from_file(filename):
    try:
        with open(filename, 'r') as f:
            return {line.strip() for line in f if line.strip()}
    except FileNotFoundError:
        return set()

def save_lines_to_file(filename, lines):
    with open(filename, 'a') as f:
        for line in lines:
            f.write(f"{line}\n")

# --- FUNGSI UNTUK MENU 1: INVITE COLLABORATOR ---
def invite_collaborators():
    print("\n--- Opsi 1: Auto Invite Collaborator & Get Username ---\n")
    
    tokens_data = load_json_file('tokens.json')
    if not tokens_data or 'tokens' not in tokens_data:
        print("❌ FATAL: tokens.json tidak ditemukan atau formatnya salah.")
        return

    tokens = tokens_data['tokens']
    token_cache = load_json_file(TOKEN_CACHE_FILE)
    invited_users = load_lines_from_file(INVITED_USERS_FILE)
    
    print(f"ℹ️  Ditemukan {len(invited_users)} user yang sudah pernah diundang.")

    usernames_to_invite = []
    
    # Tahap 1: Validasi token dan kumpulkan username yang belum diundang
    for index, token in enumerate(tokens):
        print(f"\n--- Memproses Token {index + 1}/{len(tokens)} ---")
        username = token_cache.get(token)
        if not username:
            print("   - Memvalidasi token via API...")
            env = os.environ.copy(); env['GH_TOKEN'] = token
            success, result = run_command("gh api user --jq .login", env=env)
            if success:
                username = result
                print(f"     ✅ Token valid untuk @{username}")
                token_cache[token] = username
            else:
                print(f"     ⚠️  Token tidak valid. Pesan: {result}")
                continue
        
        if username and username not in invited_users:
            usernames_to_invite.append(username)
            print(f"   - @{username} adalah user baru yang akan diundang.")
        elif username:
            print(f"   - @{username} sudah ada di daftar undangan (dilewati).")

    save_json_file(TOKEN_CACHE_FILE, token_cache)
    print("\n✅ Cache token-username telah diperbarui.")

    if not usernames_to_invite:
        print("\n✅ Tidak ada user baru untuk diundang. Semua sudah up-to-date.")
        return

    # Tahap 2: Kirim undangan
    print(f"\n--- Mengundang {len(usernames_to_invite)} Akun Baru ke Repo ---")
    env = os.environ.copy(); env['GH_TOKEN'] = MAIN_TOKEN_CONFIG
    newly_invited = set()

    for username in usernames_to_invite:
        if username.lower() == MAIN_ACCOUNT_USERNAME.lower():
            continue
        print(f"   - Mengirim undangan ke @{username}...")
        command = f"gh api repos/{MAIN_ACCOUNT_USERNAME}/{BLUEPRINT_REPO_NAME}/collaborators/{username} -f permission=push --silent"
        success, result = run_command(command, env=env)
        if success:
            print("     ✅ Undangan berhasil dikirim!")
            newly_invited.add(username)
        elif "already a collaborator" in result.lower():
            print("     ℹ️  Sudah menjadi kolaborator.")
            newly_invited.add(username) # Tandai juga sebagai sudah diundang
        else:
            print(f"     ⚠️  Gagal. Pesan: {result}")
        time.sleep(1)
        
    if newly_invited:
        save_lines_to_file(INVITED_USERS_FILE, newly_invited)
        print(f"\n✅ {len(newly_invited)} user baru berhasil ditambahkan ke tracking file {INVITED_USERS_FILE}.")


# --- FUNGSI UNTUK MENU 2: AUTO SET SECRETS ---
def auto_set_secrets():
    print("\n--- Opsi 2: Auto Set Secrets for All Accounts ---\n")
    
    secrets_to_set = load_json_file('secrets.json')
    if not secrets_to_set:
        print("❌ FATAL: secrets.json tidak ditemukan atau kosong. Silakan isi file tersebut.")
        return
    print("✅ Berhasil memuat secrets dari secrets.json.")

    tokens_data = load_json_file('tokens.json')
    if not tokens_data or 'tokens' not in tokens_data: return
    tokens = tokens_data['tokens']
    
    token_cache = load_json_file(TOKEN_CACHE_FILE)
    if not token_cache:
        print("⚠️ Cache token tidak ditemukan. Jalankan Opsi 1 terlebih dahulu.")
        return

    for index, token in enumerate(tokens):
        print(f"\n--- Memproses Akun {index + 1}/{len(tokens)} ---")
        username = token_cache.get(token)
        if not username: 
            print("   - Username tidak ada di cache. Jalankan Opsi 1 untuk update. Dilewati.")
            continue
            
        repo_full_name = f"{username}/{BLUEPRINT_REPO_NAME}"
        print(f"   - Target Repositori: {repo_full_name}")

        env = os.environ.copy(); env['GH_TOKEN'] = token

        # Logic fork (sama seperti sebelumnya)
        print(f"   - Memeriksa fork...")
        success, _ = run_command(f"gh repo view {repo_full_name}", env=env)
        if not success:
            print(f"     - Fork tidak ditemukan. Membuat fork...")
            run_command(f"gh repo fork {MAIN_ACCOUNT_USERNAME}/{BLUEPRINT_REPO_NAME} --clone=false --remote=false", env=env)
            time.sleep(5)
        else:
            print("     - Fork sudah ada.")

        for name, value in secrets_to_set.items():
            print(f"   - Mengatur secret '{name}'...")
            command = f'gh secret set {name} --app codespaces --repo "{repo_full_name}"'
            success, result = run_command(command, env=env, input_data=value)
            if success: print(f"     ✅ Secret '{name}' berhasil diatur.")
            else: print(f"     ⚠️  Gagal mengatur secret '{name}'. Pesan: {result}")
        time.sleep(1)


# --- FUNGSI UNTUK MENU 3: AUTO ACCEPT INVITES ---
def auto_accept_invitations():
    print("\n--- Opsi 3: Auto Accept Collaboration Invitations ---\n")
    tokens_data = load_json_file('tokens.json')
    if not tokens_data or 'tokens' not in tokens_data: return
    tokens = tokens_data['tokens']
    
    target_repo = f"{MAIN_ACCOUNT_USERNAME}/{BLUEPRINT_REPO_NAME}".lower()

    for index, token in enumerate(tokens):
        print(f"\n--- Memproses Akun {index + 1}/{len(tokens)} ---")
        env = os.environ.copy(); env['GH_TOKEN'] = token

        success, username = run_command("gh api user --jq .login", env=env)
        if not success:
            print("   - ⚠️ Token tidak valid, dilewati.")
            continue
        print(f"   - Login sebagai @{username}")

        print("   - Mengecek undangan...")
        success, invitations_json = run_command("gh api /user/repository_invitations", env=env)
        if not success:
            print("     - ⚠️ Gagal mendapatkan daftar undangan.")
            continue

        try:
            invitations = json.loads(invitations_json)
            if not invitations:
                print("     - ✅ Tidak ada undangan yang tertunda.")
                continue

            for inv in invitations:
                inv_id = inv.get("id")
                repo_name = inv.get("repository", {}).get("full_name", "").lower()

                if repo_name == target_repo:
                    print(f"     - Ditemukan undangan untuk {repo_name}. Menerima...")
                    accept_cmd = f"gh api --method PATCH /user/repository_invitations/{inv_id} --silent"
                    success, result = run_command(accept_cmd, env=env)
                    if success: print("       ✅ Undangan berhasil diterima!")
                    else: print(f"       ⚠️ Gagal menerima undangan. Pesan: {result}")
        except (json.JSONDecodeError, AttributeError):
            print("     - ⚠️ Gagal mem-parsing daftar undangan atau tidak ada undangan.")
        
        time.sleep(1)


# --- TAMPILAN MENU UTAMA ---
def main():
    while True:
        print("\n=============================================")
        print("  TOOL MANAJEMEN OTOMATIS GITHUB CODESPACES")
        print("=============================================")
        print("1. Validasi Token & Undang Kolaborator Baru")
        print("2. Auto Set Secrets (dari secrets.json)")
        print("3. Auto Accept Collaboration Invitations")
        print("0. Keluar")
        
        choice = input("Pilih menu (1/2/3/0): ")
        
        if choice == '1': invite_collaborators()
        elif choice == '2': auto_set_secrets()
        elif choice == '3': auto_accept_invitations()
        elif choice == '0':
            print("Terima kasih!"); break
        else:
            print("Pilihan tidak valid.")
        
        input("\nTekan Enter untuk kembali ke menu utama...")

if __name__ == "__main__":
    main()
