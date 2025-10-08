#!/bin/bash
# mawari/auto-start.sh

WORKDIR="/workspaces/Mawari-Nexus-Multi/mawari"
LOG_FILE="$WORKDIR/autostart.log"

exec > >(tee -a "$LOG_FILE") 2>&1

echo "╔════════════════════════════════════════════════╗"
echo "║          MAWARI: MULTI-NODE AUTO START         ║"
echo "╚════════════════════════════════════════════════╝"
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"

if [[ "$CODESPACE_NAME" != *"mawari"* ]]; then
    echo "ℹ️  Bukan Codespace Mawari, skrip dilewati."
    exit 0
fi

wallet_dirs=$(find ~/mawari -mindepth 1 -maxdepth 1 -type d -name "wallet_*" 2>/dev/null)
if [ -z "$wallet_dirs" ]; then
    echo "❌ ERROR: Tidak ada folder wallet ditemukan. first-setup.sh mungkin gagal."
    exit 1
fi

ALL_OWNERS="$MAWARI_OWNER_ADDRESS,$MAWARI_OWNERS"
export MNTESTNET_IMAGE=us-east4-docker.pkg.dev/mawarinetwork-dev/mwr-net-d-car-uses4-public-docker-registry-e62e/mawari-node:latest

echo "🐋 Pulling image Mawari terbaru..."
docker pull $MNTESTNET_IMAGE

started_count=0
for dir in $wallet_dirs; do
    wallet_index=$(basename "$dir" | sed 's/wallet_//')
    container_name="mawari-node-${wallet_index}"

    echo "🔄 Memeriksa Node Mawari #${wallet_index}..."

    if docker ps --format '{{.Names}}' | grep -q "^${container_name}$"; then
        echo "   ℹ️  Container ${container_name} sudah berjalan."
        started_count=$((started_count + 1))
    else
        echo "   🚀 Memulai container ${container_name}..."
        docker rm -f "$container_name" 2>/dev/null || true
        
        docker run -d \
            --name "$container_name" \
            --restart unless-stopped \
            -v "${dir}:/app/cache" \
            -e OWNERS_ALLOWLIST="$ALL_OWNERS" \
            $MNTESTNET_IMAGE
        
        if [ $? -eq 0 ]; then
            echo "   ✅ Container ${container_name} berhasil dimulai."
            started_count=$((started_count + 1))
        else
            echo "   ❌ ERROR: Gagal memulai container ${container_name}"
        fi
        sleep 2
    fi
done

echo "════════════════════════════════════════════════"
echo "✅ Auto-start Mawari selesai! ${started_count} node aktif."
echo "════════════════════════════════════════════════"
docker ps --format "table {{.Names}}\t{{.Status}}" | grep mawari-node

touch /tmp/mawari_auto_start_done
