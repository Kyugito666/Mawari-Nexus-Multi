#!/bin/bash
# mawari/first-setup.sh - (Versi Hybrid)

set -e
WORKDIR="/workspaces/Mawari-Nexus-Multi/mawari"
LOG_FILE="$WORKDIR/setup.log"

exec > >(tee -a "$LOG_FILE") 2>&1

echo "╔════════════════════════════════════════════════╗"
echo "║    MAWARI: HYBRID BURNER WALLET SETUP          ║"
echo "╚════════════════════════════════════════════════╝"
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"

if [[ "$CODESPACE_NAME" != *"mawari"* ]]; then
    echo "ℹ️  Bukan Codespace Mawari, skrip setup Mawari dilewati."
    exit 0
fi

mkdir -p ~/mawari
success_count=0

# --- TAHAP 1: Proses Node Statis (Metode Lama) ---
echo "--- Tahap 1: Memproses Node Statis dari Secret Manual ---"
if [ -n "$MAWARI_OWNER_ADDRESS" ] && [ -n "$MAWARI_BURNER_PRIVATE_KEY" ]; then
    wallet_dir=~/mawari/wallet_1
    config_file=${wallet_dir}/flohive-cache.json
    echo "🔧 Memproses Wallet #1 (Statis)..."
    echo "   - Owner: $MAWARI_OWNER_ADDRESS"
    echo "   - Burner: $MAWARI_BURNER_ADDRESS"
    mkdir -p "$wallet_dir"

    cat > "$config_file" <<EOF
{
  "burnerWallet": {
    "privateKey": "${MAWARI_BURNER_PRIVATE_KEY}",
    "address": "${MAWARI_BURNER_ADDRESS}"
  }
}
EOF
    chmod 600 "$config_file"
    echo "   ✅ Config file untuk node statis berhasil dibuat."
    success_count=$((success_count + 1))
else
    echo "ℹ️  Secret untuk node statis (MAWARI_OWNER_ADDRESS, dll.) tidak ditemukan, dilewati."
fi

# --- TAHAP 2: Proses Node Dinamis (Metode Seed Phrase) ---
echo -e "\n--- Tahap 2: Memproses Node Dinamis dari Seed Phrase ---"
if [ -n "$SEED_PHRASE" ] && [ -n "$MAWARI_OWNERS" ]; then
    echo "✅ Secret SEED_PHRASE dan MAWARI_OWNERS ditemukan."
    
    IFS=',' read -r -a owners <<< "$MAWARI_OWNERS"
    total_dynamic_wallets=${#owners[@]}
    echo "✅ Terdeteksi ${total_dynamic_wallets} owner tambahan untuk dibuat dari seed phrase."

    for i in $(seq 0 $(($total_dynamic_wallets - 1))); do
        wallet_index=$(($i + 2)) # Mulai dari wallet_2
        derivation_path_index=$(($i + 1)) # Derivation path mulai dari 1 (0 untuk statis)
        wallet_dir=~/mawari/wallet_${wallet_index}
        config_file=${wallet_dir}/flohive-cache.json
        owner_address=${owners[$i]}

        echo "🔧 Memproses Wallet #${wallet_index} (Dinamis, Derivation Index: ${derivation_path_index})..."
        mkdir -p "$wallet_dir"

        wallet_json=$(node -e "
            const ethers = require('ethers');
            const wallet = ethers.Wallet.fromMnemonic('${SEED_PHRASE}', \"m/44'/60'/0'/0/${derivation_path_index}\");
            console.log(JSON.stringify({ address: wallet.address, privateKey: wallet.privateKey }));
        " 2>&1)
        
        burner_address=$(echo "$wallet_json" | jq -r .address)
        burner_private_key=$(echo "$wallet_json" | jq -r .privateKey)

        echo "   → Burner Address: ${burner_address}"

        cat > "$config_file" <<EOF
{
  "burnerWallet": {
    "privateKey": "${burner_private_key}",
    "address": "${burner_address}"
  }
}
EOF
        chmod 600 "$config_file"
        echo "   ✅ Config file created."
        success_count=$((success_count + 1))
    done
else
    echo "ℹ️  Secret untuk node dinamis (SEED_PHRASE, MAWARI_OWNERS) tidak ditemukan, dilewati."
fi

echo "════════════════════════════════════════════════"
echo "✅ Setup Mawari Selesai! Total ${success_count} wallet dikonfigurasi."
touch /tmp/mawari_first_setup_done
