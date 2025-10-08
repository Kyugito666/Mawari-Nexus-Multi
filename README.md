# Mawari & Nexus Multi-Node Blueprint

Repositori ini adalah *blueprint* terpadu untuk menjalankan *multi-node* Mawari dan Nexus di dalam GitHub Codespaces. Proyek ini dirancang untuk dikelola oleh orkestrator eksternal yang akan membuat **dua Codespace**: satu untuk Mawari (`mawari-nodes`) dan satu untuk Nexus (`nexus-nodes`).

---

## 📋 Persyaratan (GitHub Codespace Secrets)

Atur semua *secrets* di bawah ini di `Settings > Secrets and variables > Codespaces` pada repositori ini.

### Secrets untuk Mawari (1 Codespace, 6 Node Hybrid)

| Nama Secret | Deskripsi |
| :--- | :--- |
| `MAWARI_OWNER_ADDRESS` | **[Node #1]** Alamat *owner* untuk node pertama (statis). |
| `MAWARI_BURNER_ADDRESS` | **[Node #1]** Alamat *burner wallet* untuk node pertama. |
| `MAWARI_BURNER_PRIVATE_KEY` | **[Node #1]** *Private key* dari *burner wallet* pertama. |
| `SEED_PHRASE` | **[Node #2-6]** *Seed phrase* (12/24 kata) untuk membuat 5 *burner wallet* berikutnya. |
| `MAWARI_OWNERS` | **[Node #2-6]** Daftar 5 alamat *owner* lainnya, dipisahkan koma. |

### Secrets untuk Nexus (1 Codespace, Hybrid)

| Nama Secret | Deskripsi |
| :--- | :--- |
| `NEXUS_WALLET_ADDRESS` | **[Node #1]** Alamat wallet untuk node utama Anda yang sudah ada. |
| `NEXUS_NODE_ID` | **[Node #1]** Node ID yang berpasangan dengan wallet lama. |
| `NEXUS_WALLETS_MULTI` | **[Node #2 dst.]** Daftar wallet Nexus **tambahan**, dipisahkan koma. |
| `NEXUS_NODE_IDS_MULTI` | **[Node #2 dst.]** Daftar Node ID **tambahan**, urutannya harus sesuai. |

---

## 🚀 Cara Kerja Automasi

1.  **Orkestrator**: Membuat dua Codespace: `mawari-nodes` dan `nexus-nodes`.
2.  **Inisialisasi**: Saat Codespace Mawari dibuat, `mawari/first-setup.sh` akan mengkonfigurasi 6 *wallet* Mawari (1 statis, 5 dinamis).
3.  **Auto-Start**: Setiap kali Codespace aktif:
    * Di `mawari-nodes`, `mawari/auto-start.sh` akan menjalankan 6 container Docker Mawari.
    * Di `nexus-nodes`, `nexus/auto-start.sh` akan menarik *image* terbaru dan menjalankan container Docker untuk node Nexus statis dan semua node tambahan.

---

## 📊 Monitoring

Gunakan `gh codespace ssh` untuk masuk, lalu gunakan `docker ps` untuk melihat semua node yang berjalan dan `docker logs -f [nama-container]` untuk melihat log spesifik.
