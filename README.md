# Pulse Wireframe (`pulse-monitor-rust`)

A brutalist, wireframe-themed system resource monitor built with Rust and Vue.

Dirancang dengan estetika **Archival Raw & Monospaced Wireframe** (Awwwards-level brutalist engineering), menghilangkan semua pola generic AI slop: tidak ada tombol gradient rounded klise, tidak ada card marketing fiktif, tidak ada filler buzzword. Sepenuhnya data metrik kernel OS nyata.

---

## Technical Stack
- **Kernel Telemetry Backend**: Rust (`sysinfo 0.30`, `Axum 0.7`, `Tokio runtime`).
- **Telemetry Engine Port**: `http://127.0.0.1:9099` (Raw JSON and WebSocket stream).
- **Frontend Layer**: Vue 3 Reactivity, custom CSS Wireframe Grid, `Space Mono` + `IBM Plex Mono` typography.
- **Local Web Environment**:
  - **Laravel Herd**: Domain otomatis `http://pulse-monitor-rust.test`
  - **Direct Localhost**: `http://127.0.0.1:9099`

---

## Metrik & Fitur Nyata (Real-time OS Sampling)
1. **CPU Global & Multi-Core Vector Matrix**:
   - Membaca beban per-core prosesor secara presisi dalam representasi ASCII meter `[====------]`.
2. **Physical RAM & Swap Allocation**:
   - Total memory, buffer resident, swap usage terukur byte demi byte.
3. **Network I/O Packet Buffer**:
   - Akumulasi Rx (Received) dan Tx (Transmitted) traffic interface aktif.
4. **Active Process Pool**:
   - Tabel 20 proses sistem teratas berdasarkan pemakaian CPU/Memory aktual (PID, nama proses binary, RSS memori).
5. **Storage Mount Telemetry**:
   - Status partisi disk dan kapasitas byte tersedia.
6. **Raw Telemetry Chrono Dump**:
   - Console logger mikrodetik mencatat heartbeat dan polling cycle.

---

## Cara Menjalankan

### 1. Jalankan Backend Rust
```bash
cd backend
cargo run
```
Engine akan mendengarkan request di: `http://127.0.0.1:9099`

### 2. Akses Dashboard
Buka browser:
- **Via Laravel Herd**: `http://pulse-monitor-rust.test`
- **Via Local File**: Langsung layani folder `public/`
