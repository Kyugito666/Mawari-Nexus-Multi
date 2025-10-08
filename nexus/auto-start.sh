#!/bin/bash
# nexus/auto-start.sh - (Versi Hybrid Final)

WORKDIR="/workspaces/Mawari-Nexus-Multi/nexus"
LOG_FILE="$WORKDIR/autostart.log"
NEXUS_IMAGE_NAME="nexusxyz/nexus-cli:latest"

exec > >(tee -a "$LOG_FILE") 2>&1

echo "╔════════════════════════════════════════════════╗"
echo "║       NEXUS: HYBRID MULTI-NODE AUTO START      ║"
echo "╚════════════════════════════════════════════════╝"
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"

if [[ "$CODESPACE_NAME" != *"nexus"* ]]; then
    echo "ℹ️  Bukan Codespace Nexus, skrip dilewati."
    exit 0
fi

echo "🐋 Menarik (pull) image Nexus terbaru: ${NEXUS_IMAGE_NAME}..."
docker pull ${NEXUS_IMAGE_NAME}
echo ""

started_count=0
node_counter=1

# --- TAHAP 1: Proses Node Statis (Metode Lama) ---
echo "--- Tahap 1: Memproses Node dari Secret Lama (NEXUS_NODE_ID) ---"
if [ -n "$NEXUS_NODE_ID" ]; then
    container_name="nexus-node-${node_counter}"
    echo "🔧 Memproses Node #${node_counter} (Statis)..."
    echo "   - Node ID: $NEXUS_NODE_ID"

    if docker ps --format '{{.Names}}' | grep -q "^${container_name}$"; then
        echo "   ℹ️  Container ${container_name} sudah berjalan."
        started_count=$((started_count + 1))
    else
        echo "   🚀 Memulai container ${container_name}..."
        docker rm -f "$container_name" 2>/dev/null || true
        
        docker run -d \
          --name "$container_name" \
          --restart unless-stopped \
          ${NEXUS_IMAGE_NAME} \
          start --headless --node-id "$NEXUS_NODE_ID"
          
        if [ $? -eq 0 ]; then
            echo "   ✅ Container ${container_name} berhasil dimulai."
            started_count=$((started_count + 1))
        else
            echo "   ❌ ERROR: Gagal memulai container ${container_name}."
        fi
    fi
    node_counter=$((node_counter + 1))
else
    echo "ℹ️  Secret lama (NEXUS_NODE_ID) tidak ditemukan, dilewati."
fi

# --- TAHAP 2: Proses Node Tambahan (Metode Baru) ---
echo -e "\n--- Tahap 2: Memproses Node Tambahan dari Secret Baru (NEXUS_NODE_IDS_MULTI) ---"
if [ -n "$NEXUS_NODE_IDS_MULTI" ]; then
    IFS=',' read -r -a node_ids <<< "$NEXUS_NODE_IDS_MULTI"
    total_dynamic_nodes=${#node_ids[@]}
    echo "✅ Terdeteksi ${total_dynamic_nodes} node tambahan untuk dijalankan."

    for i in $(seq 0 $(($total_dynamic_nodes - 1))); do
        node_id=${node_ids[$i]}
        container_name="nexus-node-${node_counter}"
        
        echo "🔧 Memproses Node #${node_counter} (Tambahan)..."
        echo "   - Node ID: $node_id"
        
        if docker ps --format '{{.Names}}' | grep -q "^${container_name}$"; then
            echo "   ℹ️  Container ${container_name} sudah berjalan."
            started_count=$((started_count + 1))
        else
            echo "   🚀 Memulai container ${container_name}..."
            docker rm -f "$container_name" 2>/dev/null || true
            
            docker run -d \
              --name "$container_name" \
              --restart unless-stopped \
              ${NEXUS_IMAGE_NAME} \
              start --headless --node-id "$node_id"
              
            if [ $? -eq 0 ]; then
                echo "   ✅ Container ${container_name} berhasil dimulai."
                started_count=$((started_count + 1))
            else
                echo "   ❌ ERROR: Gagal memulai container ${container_name}."
            fi
            sleep 2
        fi
        node_counter=$((node_counter + 1))
    done
else
    echo "ℹ️  Secret baru (NEXUS_NODE_IDS_MULTI) tidak ditemukan, dilewati."
fi

echo "════════════════════════════════════════════════"
echo "✅ Auto-start Nexus selesai! Total ${started_count} node aktif."
echo "════════════════════════════════════════════════"
docker ps --format "table {{.Names}}\t{{.Status}}" | grep nexus-node

touch /tmp/nexus_auto_start_done
