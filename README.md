# Mawari & Nexus Multi-Node Orchestrator

Repositori ini adalah **blueprint terpadu** untuk menjalankan **multi-node Mawari dan Nexus** secara otomatis di GitHub Codespaces. Proyek ini dilengkapi dengan **orkestrator cerdas** yang mengelola rotasi akun, monitoring kuota billing, health check, dan keep-alive otomatis.

---

## 🎯 Fitur Utama

- ✅ **Multi-Account Rotation**: Otomatis beralih antar akun GitHub saat kuota habis
- ✅ **Hybrid Node Setup**: Mendukung node statis (manual config) dan dinamis (seed phrase)
- ✅ **Smart Health Check**: Verifikasi kesehatan node sebelum deployment
- ✅ **Auto Keep-Alive**: Menjaga node tetap aktif dengan siklus 4 jam
- ✅ **Billing Monitor**: Tracking otomatis penggunaan kuota Codespace
- ✅ **Zero Downtime**: Seamless transition antar akun
- ✅ **Automated Setup**: Setup tools untuk invite collaborator dan sync secrets

---

## 📁 Struktur Proyek

```
Mawari-Nexus-Multi/
├── .devcontainer/
│   └── devcontainer.json          # Konfigurasi Codespace environment
├── mawari/
│   ├── first-setup.sh             # Setup awal 6 wallet Mawari (hybrid)
│   └── auto-start.sh              # Auto-start 6 container Mawari
├── nexus/
│   └── auto-start.sh              # Auto-start container Nexus (hybrid)
├── orchestrator/
│   ├── src/
│   │   ├── main.rs                # Logic utama orkestrator
│   │   ├── config.rs              # Manajemen config & state
│   │   ├── github.rs              # GitHub API & Codespace operations
│   │   └── billing.rs             # Billing quota monitoring
│   ├── Cargo.toml                 # Rust dependencies
│   ├── setup.py                   # Python tool untuk setup multi-account
│   ├── secrets.json               # Template secrets (JANGAN commit!)
│   ├── tokens.json                # List token GitHub (JANGAN commit!)
│   ├── start.bat                  # Windows launcher
│   └── monitor.bat                # Windows monitoring dashboard
├── .gitignore
└── README.md
```

---

## 📋 Persyaratan

### 1. GitHub Account & Tokens
- Minimal **2 akun GitHub** (bisa lebih untuk durasi lebih panjang)
- **Personal Access Token (Classic)** dengan scope: `repo`, `admin:org`, `codespace`, `user`, `admin:public_key`

### 2. Codespace Secrets

Atur secrets di `Settings > Secrets and variables > Codespaces` untuk SETIAP repositori fork:

#### Secrets untuk Mawari (6 Node Hybrid)

| Nama Secret | Deskripsi | Contoh |
|:------------|:----------|:-------|
| `MAWARI_OWNER_ADDRESS` | **[Node #1]** Alamat owner untuk node pertama | `0x1234...abcd` |
| `MAWARI_BURNER_ADDRESS` | **[Node #1]** Alamat burner wallet pertama | `0x5678...efgh` |
| `MAWARI_BURNER_PRIVATE_KEY` | **[Node #1]** Private key burner wallet pertama | `0xabcd...1234` |
| `SEED_PHRASE` | **[Node #2-6]** Seed phrase untuk generate 5 wallet | `word1 word2 word3 ... word12` |
| `MAWARI_OWNERS` | **[Node #2-6]** Daftar 5 alamat owner (dipisah koma) | `0xAAA...,0xBBB...,0xCCC...,0xDDD...,0xEEE...` |

#### Secrets untuk Nexus (Hybrid)

| Nama Secret | Deskripsi | Contoh |
|:------------|:----------|:-------|
| `NEXUS_WALLET_ADDRESS` | **[Node #1]** Alamat wallet utama (opsional) | `0x9abc...def0` |
| `NEXUS_NODE_ID` | **[Node #1]** Node ID untuk wallet utama | `abc123...xyz` |
| `NEXUS_WALLETS_MULTI` | **[Node #2+]** Daftar wallet tambahan (dipisah koma) | `0xFFF...,0xGGG...` |
| `NEXUS_NODE_IDS_MULTI` | **[Node #2+]** Daftar Node ID tambahan (urutan sesuai wallet) | `id456...,id789...` |

### 3. Software Requirements (untuk Orkestrator)

**Windows:**
- [Rust](https://www.rust-lang.org/tools/install) (Latest stable)
- [GitHub CLI](https://cli.github.com/) (`gh`)
- [Python 3.8+](https://www.python.org/downloads/) (untuk setup tools)

**Linux/Mac:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install GitHub CLI
# Ubuntu/Debian:
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update && sudo apt install gh

# Python biasanya sudah terinstall
```

---

## 🚀 Cara Penggunaan

### Tahap 1: Setup Multi-Account (Python Tool)

1. **Fork repositori ini** ke akun utama Anda
2. **Clone** ke local machine
3. Masuk ke folder `orchestrator/`
4. Buat file **`tokens.json`**:
```json
{
  "tokens": [
    "ghp_TokenAkun1XXXXXXXXX",
    "ghp_TokenAkun2XXXXXXXXX",
    "ghp_TokenAkun3XXXXXXXXX"
  ]
}
```

5. Buat file **`secrets.json`** (sesuaikan dengan data Anda):
```json
{
  "MAWARI_OWNER_ADDRESS": "0xYourMainOwner...",
  "MAWARI_BURNER_ADDRESS": "0xYourMainBurner...",
  "MAWARI_BURNER_PRIVATE_KEY": "0xYourPrivateKey...",
  "SEED_PHRASE": "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12",
  "MAWARI_OWNERS": "0xOwner2...,0xOwner3...,0xOwner4...,0xOwner5...,0xOwner6...",
  "NEXUS_WALLET_ADDRESS": "0xYourNexusWallet...",
  "NEXUS_NODE_ID": "YourNodeID123",
  "NEXUS_WALLETS_MULTI": "0xNexusWallet2...,0xNexusWallet3...",
  "NEXUS_NODE_IDS_MULTI": "NodeID456,NodeID789"
}
```

6. Edit `setup.py`, ubah konfigurasi:
```python
MAIN_TOKEN_CONFIG = "ghp_YourMainAccountToken"
MAIN_ACCOUNT_USERNAME = "YourGitHubUsername"
BLUEPRINT_REPO_NAME = "Mawari-Nexus-Multi"
```

7. **Jalankan Setup Tool**:
```bash
python setup.py
```

Menu yang tersedia:
- **[1] Validasi Token & Undang Kolaborator**: Otomatis invite semua akun ke repo
- **[2] Auto Set Secrets**: Sync semua secrets ke setiap fork
- **[3] Auto Accept Invitations**: Terima undangan kolaborator otomatis

**Jalankan menu 1 → 2 → 3 secara berurutan!**

---

### Tahap 2: Jalankan Orkestrator

#### Windows:

1. Edit `start.bat`, ubah `PATH` ke folder orkestrator Anda
2. Jalankan:
```cmd
start.bat YourUsername/Mawari-Nexus-Multi
```

#### Linux/Mac:

```bash
cd orchestrator
cargo run --release -- YourUsername/Mawari-Nexus-Multi
```

---

## 📊 Monitoring

### Monitoring via Tool (Windows)

Edit `monitor.bat`, ubah `PATH`, lalu jalankan:
```cmd
monitor.bat
```

### Monitoring Manual

```bash
# Cek status codespace
gh codespace list

# SSH ke codespace
gh codespace ssh -c CODESPACE_NAME

# Cek running containers
docker ps

# Lihat logs specific container
docker logs -f mawari-node-1
docker logs -f nexus-node-1

# Cek state orkestrator
cat orchestrator/state.json
```

---

## ⚙️ Cara Kerja Sistem

### 1. Inisialisasi
- Orkestrator membaca `tokens.json` dan `state.json`
- Validasi token dan cek billing quota
- Memilih akun dengan kuota tersedia

### 2. Deployment
- Membuat/menggunakan 2 Codespace: `mawari-nodes` dan `nexus-nodes`
- **Mawari**: `first-setup.sh` membuat 6 wallet (1 statis + 5 dinamis)
- **Nexus**: `auto-start.sh` menjalankan node statis + node tambahan
- Health check via marker file (`/tmp/*_auto_start_done`)

### 3. Keep-Alive (Setiap 4 Jam)
- Orkestrator mengirim perintah SSH ke kedua Codespace
- Re-run `auto-start.sh` untuk memastikan semua container aktif
- Verifikasi health check

### 4. Rotasi Akun
- Monitoring durasi berjalan berdasarkan billing quota
- Otomatis beralih ke token berikutnya saat:
  - Kuota habis (< 1 jam tersisa)
  - Target durasi tercapai
  - Error deployment

---

## 🔐 Keamanan

**PENTING:** File berikut TIDAK BOLEH di-commit ke GitHub:

```
orchestrator/tokens.json          # Berisi GitHub tokens
orchestrator/secrets.json         # Berisi private keys & sensitive data
orchestrator/token_cache.json     # Cache username-token mapping
orchestrator/invited_users.txt    # Tracking undangan kolaborator
orchestrator/state.json           # State orkestrator
```

Semua file di atas sudah ada di `.gitignore`.

**Best Practices:**
- ✅ Gunakan burner wallet untuk Mawari
- ✅ Jangan share token GitHub ke publik
- ✅ Rotasi token secara berkala
- ✅ Gunakan akun GitHub terpisah untuk farming

---

## 🐛 Troubleshooting

### 1. "Token tidak valid"
- Pastikan token memiliki semua scope yang diperlukan
- Token expired? Generate ulang di GitHub Settings

### 2. "Gagal membuat Codespace"
- Cek kuota billing: `gh api /users/USERNAME/settings/billing/usage`
- Pastikan fork sudah ada di akun target
- Cek apakah ada Codespace lama yang perlu dihapus

### 3. "Health check gagal"
- SSH ke Codespace: `gh codespace ssh -c CODESPACE_NAME`
- Cek logs: `cat ~/mawari/setup.log` atau `cat ~/mawari/autostart.log`
- Verifikasi secrets sudah diset dengan benar

### 4. "Container tidak berjalan"
- Masuk ke Codespace dan cek: `docker ps -a`
- Lihat logs container: `docker logs CONTAINER_NAME`
- Cek konfigurasi wallet: `cat ~/mawari/wallet_*/flohive-cache.json`

### 5. Setup.py Error
- Pastikan Python dependencies terinstall: `pip install requests`
- Cek koneksi internet
- Verifikasi format `tokens.json` dan `secrets.json` valid JSON

---

## 📝 Catatan Penting

1. **Kuota Codespace**: GitHub Free tier menyediakan 120 core-hours/bulan
   - 2 Codespace @ 4-core = 8 core total
   - Estimasi runtime: ~15 jam per akun/bulan

2. **Idle Timeout**: Codespace akan auto-stop setelah 4 jam idle
   - Orkestrator akan re-deploy jika diperlukan
   - Keep-alive memastikan tidak idle

3. **Optimasi Multi-Account**:
   - Minimal 2 akun = ~30 jam/bulan
   - 4 akun = ~60 jam/bulan
   - 8 akun = ~120 jam/bulan (hampir 24/7)

---

## 📞 Support

Jika mengalami kendala:
1. Cek section **Troubleshooting** di atas
2. Review logs di `orchestrator/*.log` dan container logs
3. Pastikan semua secrets sudah diset dengan benar

---

## 📜 License

Project ini untuk keperluan edukasi dan personal use. Gunakan dengan bijak dan patuhi Terms of Service GitHub.

---

## 🙏 Credits

Developed by **Kyugito666**  
Mawari Network: [https://mawari.com](https://mawari.com)  
Nexus Network: [https://nexus.xyz](https://nexus.xyz)

---

**⚠️ DISCLAIMER**: Penggunaan tool ini sepenuhnya tanggung jawab pengguna. Pastikan mematuhi kebijakan GitHub dan Terms of Service dari Mawari & Nexus Network.
