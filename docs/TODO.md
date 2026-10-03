# FugDB — PRD Implementation TODO & Action Plan

> Dokumen pelacak implementasi fitur FugDB berdasarkan spesifikasi pada [`technical-specification-and-prd.md`](./technical-specification-and-prd.md).

---

## 📊 Status Matriks Fitur (Current State vs PRD)

| Kategori Fitur | Status | Progress | Keterangan & Catatan |
| :--- | :---: | :---: | :--- |
| **A. Connection & Safety** | 🟢 Selesai | 100% | PG, MySQL, SQLite, MSSQL ✅, Production Guard ✅, SSH Tunnel ✅, OS Keyring ✅, Tx Control ✅ |
| **B. High-Performance SQL Editor** | 🟢 Selesai | 100% | CodeMirror 6 ✅, Autocomplete ✅, History/Favorites ✅, Visual EXPLAIN ✅, Shortcuts ✅ |
| **C. In-Cell Grid & Staged Mutations** | 🟢 Selesai | 100% | Virtual Grid 100k @ 60FPS ✅, Staged Diff Drawer ✅, Rich Cell Inspectors ✅, Quick Filter & Sort ✅ |
| **D. Multi-Source ETL & Data Transfer** | 🟢 Selesai | 100% | Tokio Streaming Pipeline (CSV, JSON, Excel, SQL Dump) ✅, DB-to-DB Direct Transfer ✅ |
| **E. Schema Explorer & ERD** | 🟢 Selesai | 100% | Schema Tree ✅, Live FK ERD Visualizer (@xyflow/svelte) ✅, 1-Click Data Dictionary ✅ |
| **F. QA & Mock Data Engine** | 🟢 Selesai | 100% | Smart Semantic Column Inference ✅, FK Relational Validation ✅, Chunked Batch Inserts ✅ |
| **G. "Vibe Coding" & AI Integration** | 🟢 Selesai | 100% | Privacy-First NL-to-SQL (Ollama + Cloud) ✅, 1-Click Fix with AI ✅, AI EXPLAIN Optimizer ✅ |
| **H. Data Visualization & Mini-BI** | 🟢 Selesai | 100% | 1-Click Instant Data Charting (Bar/Line/Pie/Scatter) ✅ |
| **I. Next-Level Server Intelligence** | ⚪ Baru (Phase 5) | 0% | Live Process Monitor ⏳, Schema Diff ⏳, SQL Notebooks ⏳, Redis ⏳, Automation ⏳ |

---

## 🚀 Fitur yang Sudah Selesai (Completed)

- [x] **Multi-Driver Database Engine (Rust Backend)**:
  - [x] PostgreSQL driver (via `sqlx`) dengan SSL/TLS & UUID support.
  - [x] MySQL & MariaDB driver (via `sqlx`).
  - [x] SQLite driver (via `sqlx`).
  - [x] Microsoft SQL Server (MSSQL) driver natif (via `tiberius` + `bb8-tiberius` + `rustls` TDS 7.3).
- [x] **High-Performance Virtual Scrolling Grid (Svelte 5)**:
  - [x] DOM Spacer Windowing Technique (`topPadding` / `bottomPadding` via `ResizeObserver`).
  - [x] Mampu me-render 10.000–100.000+ baris data pada 60 FPS tanpa UI freezing.
  - [x] Scrollbar kustom yang jelas dan kontras tinggi di Light & Dark mode.
- [x] **Staged Mutations Buffer (TablePlus Draft Mode)**:
  - [x] In-cell double-click editing, add row baru, mark deleted rows.
  - [x] Slide-out review drawer dengan SQL diff generator per dialek database (PostgreSQL, MySQL, SQLite, MSSQL).
  - [x] Batch transaction commit & local rollback.
- [x] **Dynamic SQL Query Sync & Syntax Handling**:
  - [x] Dialect-aware query generator (MSSQL `TOP (100)` vs `LIMIT 100`, quoting `[col]` vs `"col"` / `` `col` ``).
  - [x] Reactive editor document sync di CodeMirror 6 saat ganti tabel/tab.
- [x] **Theme & UI Contrast Polish**:
  - [x] Perbaikan kontras teks, alert banner, status badge, dan drawer pada Light & Dark mode.

---

## 📋 TODO Implementasi Sesuai PRD

### ⚡ Phase 1: Editor Ergonomics & Core Safety (Immediate Priority)

- [x] **1.1 Schema-Aware Contextual Autocomplete (SQL Editor)**
  - [x] Ambil metadata skema aktif (`tables`, `columns`, `views`, `functions`) dari `connectionStore.activeSchemaTree`.
  - [x] Integrasikan schema completions ke `@codemirror/autocomplete` dan `@codemirror/lang-sql`.
  - [x] Tambahkan auto-suggest klausa `JOIN ... ON ...` cerdas berdasarkan relasi Foreign Key.
  - [x] Expandable sidebar schema tree untuk eksplorasi kolom langsung dengan indikator PK/FK.
  - *Files Terkait*: `src/lib/components/editor/SqlEditor.svelte`, `src/lib/components/editor/sqlCompletion.ts`, `src/lib/components/sidebar/ConnectionTree.svelte`

- [x] **1.2 Production Safety Guard Interceptor**
  - [x] Deteksi environment koneksi (`production` / badge merah).
  - [x] Parser / regex interceptor untuk query berisiko tinggi (`DROP`, `TRUNCATE`, `ALTER`, `DELETE`/`UPDATE` tanpa `WHERE`).
  - [x] Tampilkan modal konfirmasi dengan validasi pengetikan nama database / tombol override yang jelas.
  - [x] Toolbar SQL editor dengan indikator real-time Production Guard dan badge risiko destruktif.
  - *Files Terkait*: `src/lib/state/tabs.svelte.ts`, `src/lib/state/safety.svelte.ts`, `src/lib/utils/safetyGuard.ts`, `src/lib/components/ui/SafetyModal.svelte`, `src/lib/components/editor/SqlEditor.svelte`

- [x] **1.3 Persistent Query History & Favorites**
  - [x] Simpan histori eksekusi query (SQL, timestamp, duration ms, status, row count, database name) ke persistent storage (localStorage up to 500 items).
  - [x] UI drawer riwayat (Cmd+H / Ctrl+H) dengan status filter (All, Favorites, Success, Error), connection filter, full-text search.
  - [x] Bookmark / pin query favorit lengkap dengan tag custom label editing (e.g. "Monthly Active Users").
  - [x] Fitur 1-click "Use Query", "Open in Tab", "Copy SQL", dan "Delete/Clear".
  - *Files Terkait*: `src/lib/state/history.svelte.ts`, `src/lib/components/editor/QueryHistoryDrawer.svelte`, `src/lib/components/layout/Navbar.svelte`, `src/lib/components/editor/SqlEditor.svelte`, `src/App.svelte`

- [x] **1.4 Rich Cell Inspectors (Data Grid)**
  - [x] **JSON Viewer & Editor**: Tree view interaktif dengan syntax formatting, collapse/expand, prettify/minify, filter key search, dan real-time validation editor.
  - [x] **Date & Timezone Inspector**: Konversi real-time UTC $\leftrightarrow$ Local Time $\leftrightarrow$ Global Timezones (WIB, SGT, JST, EST, GMT), relative time ("3 hours ago"), dan Unix Epoch seconds/ms.
  - [x] **Binary & Hash Decoder**: UUID v1/v4/v7 validator & generator, Base64 encoder/decoder (UTF-8 $\leftrightarrow$ Base64), dan Hexadecimal memory dump view (offset, bytes, ASCII).
  - [x] **Blob & Long Text Previewer**: Markdown rich preview, Image/SVG live previewer, character/word statistics, dan Download as file.
  - [x] **Data Grid Integration**: Right-click context menu, shortcut keyboard (<kbd>Cmd+I</kbd> / <kbd>Ctrl+I</kbd>), quick click badge JSON/Date, dan 1-click **Apply to Cell** yang tersinkronisasi langsung ke Staged Mutations buffer.
  - *Files Terkait*: `src/lib/state/inspector.svelte.ts`, `src/lib/components/inspectors/`, `src/lib/components/grid/DataGrid.svelte`, `src/App.svelte`

- [x] **1.5 Grid Instant Column Filter & Multi-Column Sorting**
  - [x] **Instant Column Filter Row**: Filter toggleable per kolom dengan operator cerdas (*contains, equals, ≠, starts, ends, >, ≥, <, ≤, is null, not null, regex*).
  - [x] **Global Quick Search**: Input pencarian realtime di toolbar grid untuk menemukan data seketika di seluruh baris & kolom.
  - [x] **Multi-Column Sorting**: Klik header kolom untuk sort ASC $\rightarrow$ DESC $\rightarrow$ None. Dukungan multi-column sort (<kbd>Shift+Click</kbd>) dengan badge urutan prioritas sort (1, 2, 3...).
  - [x] **Preserved Virtual Windowing**: Performa 60 FPS tetap terjaga bahkan dengan filtering & multi-level sorting aktif pada dataset besar.
  - [x] **Seamless Mutation Mapping**: Pengeditan, staging mutations, dan inspeksi sel tetap memetakan baris asli secara presisi.
  - *Files Terkait*: `src/lib/components/grid/DataGrid.svelte`

---

### 🛠️ Phase 2: Schema Architecture, Live ERD & Streaming ETL

- [x] **2.1 Live Interactive ERD Visualizer (@xyflow/svelte)**
  - [x] **FK Metadata Backend Driver**: Ekstraksi Foreign Key metadata di seluruh driver PostgreSQL, MySQL, SQLite, dan SQL Server (`generate_erd_metadata`).
  - [x] **Custom Table Node Card**: Node tabel kustom interaktif dengan indikator PK/FK, badge tipe kolom, not-null indicator, dan tombol aksi langsung (*Open Data Grid*, *Query Table*, *Copy Name*).
  - [x] **Layered Auto-Layout Engine**: Algoritma topological sort BFS multi-layer untuk penataan otomatis hirarki tabel dan relasi foreign key tanpa tumpang tindih.
  - [x] **Real-time Table / Column Search**: Filter pencarian instan dengan efek highlight / dimming pada node yang tidak relevan.
  - [x] **Multi-Format Export Suite**: Ekspor diagram skema ke **PNG Image**, **Vector SVG**, **JSON Schema**, dan **SQL DDL Script**.
  - [x] **MiniMap & Interactive Controls**: Zoom, pan, fit view, stats pill count (tabel & relasi), dan SvelteFlow store synchronization.
  - *Files Terkait*: `src/lib/components/erd/ErdModal.svelte`, `src/lib/components/erd/ErdTableNode.svelte`, `src/lib/components/erd/erdLayout.ts`

- [x] **2.2 High-Performance Streaming Import / Export Engine**
  - [x] **CSV / TSV Streaming Engine**: Stream parser & writer dengan auto-detect delimiter (comma, semicolon, tab, pipe), custom quotes, dan header row toggle.
  - [x] **JSON & NDJSON**: Streaming import & export untuk array of objects dan line-delimited JSON (NDJSON/JSONL) dengan opsi pretty-print.
  - [x] **Excel (.xlsx) Multi-Engine**: Parser lembar kerja menggunakan `calamine` dan generator spreadsheet berperforma tinggi via `rust_xlsxwriter` (auto column width & header styling).
  - [x] **SQL Dump Generator**: Ekspor DDL `CREATE TABLE` skema lengkap + batch parameterized `INSERT INTO` statements dengan ukuran chunk dinamis.
  - [x] **File Inspector & Preview**: Deteksi format file instan, ekstraksi skema kolom, dan tabel preview 10-baris sebelum proses import dimulai.
  - [x] **Real-time Tokio Progress Monitor**: Event background streaming dengan metrik live (*rows/sec, bytes processed, percent complete, ETA countdown*) dan tombol **Cancel Job** seketika.
  - *Files Terkait*: `src-tauri/src/transfer/`, `src-tauri/src/commands/transfer.rs`, `src/lib/components/transfer/TransferModal.svelte`, `src/lib/api/client.ts`

- [x] **2.3 Cross-Database Direct Transfer (DB-to-DB Streaming)**
  - [x] **Zero-Intermediate-File Streaming**: Pipeline transfer langsung dari Source Connection $\rightarrow$ Target Connection via saluran memory-bounded Tokio channel tanpa file perantara.
  - [x] **Smart Type Mapping**: Konversi tipe data otomatis lintas engine (PostgreSQL, MySQL, SQLite, SQL Server) untuk integer, floats, timestamps, JSON, UUID, dan boolean.
  - [x] **Auto-create Target Table & Truncate Support**: Otomatis membuat tabel target jika belum ada (`CREATE TABLE IF NOT EXISTS`) serta opsi truncate tabel tujuan sebelum migrasi.
  - [x] **Conflict Handling & Batching**: Pilihan resolusi konflik (*Fail*, *Ignore / Skip duplicates*, *Upsert*) dengan ukuran batch yang dapat disesuaikan (250 s/d 2.500 baris per batch).
  - [x] **Real-time Pipeline Monitor**: Dashboard pemantauan kecepatan live (*rows/sec*), progres baris, estimasi waktu (ETA), dan pembatalan instan.
  - *Files Terkait*: `src-tauri/src/transfer/db_to_db.rs`, `src-tauri/src/commands/transfer.rs`, `src/lib/components/transfer/TransferModal.svelte`, `src/lib/api/client.ts`

- [x] **2.4 1-Click Data Dictionary Generator**
  - [x] **Multi-Format Compilation Suite**: Generator dokumentasi skema instan ke format **Markdown (`.md`)**, styled modern self-contained **HTML (`.html`)**, dan **JSON Schema (`.json`)**.
  - [x] **Print-Ready & PDF Output**: Layout styling `@media print` untuk mencetak langsung atau menyimpannya sebagai dokumen PDF arsitektur database resmi.
  - [x] **Instant Table & Column Filtering**: Script embedded pencarian cepat pada output HTML untuk navigasi entitas secara instan.
  - [x] **Interactive Dictionary Modal**: Viewer terintegrasi dengan tab HTML iframe sandbox, Markdown monospace viewer, JSON editor, Copy to Clipboard, serta tombol Save File picker.
  - [x] **Quick-Access Integration**: Tombol akses 1-klik di Navbar (`BookOpen`) dan Header Schema Explorer di sidebar.
  - *Files Terkait*: `src-tauri/src/schema/dictionary.rs`, `src-tauri/src/commands/schema.rs`, `src/lib/components/dictionary/DataDictionaryModal.svelte`, `src/lib/api/client.ts`

- [x] **2.5 Smart QA Mock Data Generator**
  - [x] **Semantic Column Inference Engine**: Deteksi kategori kolom otomatis berbasis fuzzy name matching & tipe data (*Full Name, Email, Phone, Address, City, Country, Zip, Company, Job Title, Price, UUID v4, Timestamps, Boolean, Status, Lorem Ipsum, JSON Object, Custom Lists*).
  - [x] **Foreign Key Relational Validation**: Auto-sampling key yang valid dari tabel relasi target sehingga constraint Foreign Key tidak pernah gagal saat batch insert.
  - [x] **High-Speed Chunked Parameterized Inserts**: Eksekusi batch multi-baris (100 s/d 100.000 baris) dengan kecepatan >25.000 rows/sec via query batching.
  - [x] **Multi-Mode Studio**: Tab **Column Rules Configuration**, **Live 10-Row Sample Preview Grid**, dan **Generated SQL Script (.sql)** dengan tombol Copy & File Download.
  - [x] **Direct DataGrid Integration**: Tombol aksi cepat untuk langsung membuka query tabel target di DataGrid setelah proses batch selesai.
  - *Files Terkait*: `src-tauri/src/qa/`, `src-tauri/src/commands/mock_data.rs`, `src/lib/components/qa/MockDataModal.svelte`, `src/lib/api/client.ts`

---

### 🤖 Phase 3: AI Copilot & Mini-BI Visualization

- [x] **3.1 Privacy-First Natural Language to SQL (NL-to-SQL)**
  - [x] AI prompt drawer untuk mengubah instruksi teks bahasa natural menjadi query SQL yang valid.
  - [x] **Zero-Data-Leak Architecture**: Hanya mengirimkan skema DDL / nama kolom ke AI. Data baris pengguna **tidak pernah dikirim**.
  - [x] Support Multi-Provider: Local **Ollama** (offline/private) + Cloud (OpenAI, Anthropic Claude, Gemini, DeepSeek).
  - *Files Terkait*: `src-tauri/src/ai/`, `src-tauri/src/commands/ai.rs`, `src/lib/components/ai/AiAssistantDrawer.svelte`, `src/lib/components/ai/AiSettingsModal.svelte`, `src/lib/state/ai.svelte.ts`

- [x] **3.2 "Fix with AI" Button pada Query Error**
  - [x] Tombol 1-klik pada alert error query di SQL Editor & DataGrid untuk mendiagnosa dan memperbaiki sintaks query secara otomatis sesuai dialek database aktif (PostgreSQL, MySQL, SQLite, MSSQL).
  - [x] Interactive AI Fix Modal dengan diagnosis akar masalah (Root Cause), preview SQL yang telah diperbaiki, dan aksi **"Apply & Run" (Cmd+Enter)** serta **"Apply to Editor"**.
  - *Files Terkait*: `src-tauri/src/commands/ai.rs`, `src/lib/components/ai/AiFixModal.svelte`, `src/lib/components/editor/SqlEditor.svelte`, `src/lib/components/grid/DataGrid.svelte`, `src/lib/state/ai.svelte.ts`

- [x] **3.3 1-Click Instant Data Charting**
  - [x] Konversi hasil query tabular menjadi grafik visual interaktif (**Bar, Horizontal Bar, Line, Area, Pie/Donut, Scatter**) menggunakan Chart.js & HTML5 Canvas.
  - [x] Auto-detection dimensi X-Axis (kategori/tanggal/teks) dan metrik Y-Axis (angka/numerik/agregasi), multi-series selection, sorting, dan grouping aggregation (Sum, Avg, Count, Min, Max).
  - [x] Top KPI stat summary cards (Data points plotted, Total Sum, Average, Peak/Max).
  - [x] Opsi 1-klik export chart: **High-Resolution PNG Image Download** & **Direct Copy Image to System Clipboard**.
  - *Files Terkait*: `src/lib/components/charts/DataChartModal.svelte`, `src/lib/components/grid/DataGrid.svelte`

- [x] **3.4 Visual Query EXPLAIN Plan**
  - [x] Visualisasi pohon grafis interaktif untuk hasil `EXPLAIN (ANALYZE, COSTS, BUFFERS)` dengan multi-dialect parser (PostgreSQL, MySQL, SQLite, MSSQL).
  - [x] Deteksi otomatis bottleneck & warning badges: Sequential / Full Table Scan vs Index Scan, misprediksi estimasi baris (misprediction ratio > 10x), sort overhead.
  - [x] Node detail inspector panel: Shared Cache Hit Blocks vs Disk Read Blocks, predicate filter conditions, join hashing, loops & actual execution times.
  - [x] Monospace Raw Plan viewer dengan search filtering dan 1-click Copy.
  - [x] ✨ **AI Performance Optimizer Studio**: Analisis gabungan SQL + EXPLAIN plan + skema database, rekomendasi compound indexes (`CREATE INDEX ...`), penulisan ulang query yang dioptimasi, dan 1-klik "Apply to Editor".
  - [x] Shortcut global `Cmd+E` / `Ctrl+E` dan tombol Explain di bottom toolbar SQL Editor.
  - *Files Terkait*: `src/lib/components/editor/ExplainPlanModal.svelte`, `src/lib/components/editor/ExplainTreeNode.svelte`, `src/lib/state/explain.svelte.ts`, `src-tauri/src/commands/query.rs`, `src-tauri/src/commands/ai.rs`

---

### 🔒 Phase 4: Security, Session & System Hardening

- [x] **4.1 OS-Level Keyring Integration**
  - [x] Simpan password database, SSH passphrases, dan AI API keys ke OS Native Credential Store via `keyring-rs` (macOS Keychain, Windows Credential Manager, Linux Secret Service).
  - [x] Resolusi otomatis kredensial saat koneksi backend (`resolve_effective_config` & `resolve_effective_ai_config`) sehingga password plaintext tidak terekspos di localStorage/state.
  - [x] Checkbox dan visual protection badge "Store password securely in OS Keychain / Keyring" pada `ConnectionModal.svelte`.
  - [x] Indikator gembok OS Keyring di connection selector dropdown `Navbar.svelte`.
  - [x] Otomatis membersihkan secret dari OS keyring (`delete_keyring_credential`) saat koneksi dihapus.
  - *Files Terkait*: `src-tauri/src/keyring.rs`, `src-tauri/src/commands/connection.rs`, `src-tauri/src/commands/ai.rs`, `src/lib/state/connection.svelte.ts`, `src/lib/components/connection/ConnectionModal.svelte`, `src/lib/components/layout/Navbar.svelte`

- [x] **4.2 Native SSH Tunneling**
  - [x] Dukungan koneksi database via SSH Bastion Host (Password, Private Key `.pem`/`id_rsa`/`id_ed25519` file dengan optional passphrase, SSH Agent).
  - [x] Background asynchronous TCP port forwarding loop (`channel_direct_tcpip`) yang menghubungkan driver database ke remote host via local loopback.
  - [x] Lifecycle management otomatis di `AppState.tunnels` (pembuatan saat connect / pembersihan otomatis saat disconnect).
  - [x] Fitur "Test SSH Tunnel" langsung di tab SSH modal koneksi untuk diagnosa cepat konektivitas Bastion Host.
  - [x] Integrasi native file picker dialog untuk pemilihan private key file.
  - *Files Terkait*: `src-tauri/src/ssh.rs`, `src-tauri/src/commands/connection.rs`, `src-tauri/src/state.rs`, `src/lib/components/connection/ConnectionModal.svelte`

- [x] **4.3 Transaction & Session Control**
  - [x] Toggle Auto-commit (⚡) vs Manual Transaction (🔒) dengan persistensi preferensi per koneksi.
  - [x] Eksekusi backend native transaction commands (`BEGIN` / `START TRANSACTION`, `COMMIT`, `ROLLBACK`) untuk PostgreSQL, MySQL, SQLite, dan SQL Server (MSSQL).
  - [x] Indikator status transaksi aktif di Statusbar bawah dengan counter uncommitted statements dan tombol aksi cepat Commit / Rollback.
  - [x] Integrasi SQL Editor toolbar dengan tombol toggle Auto-commit, tombol Commit & Rollback, serta global keyboard shortcuts (`Cmd+Shift+C` / `Ctrl+Shift+C` untuk Commit, `Cmd+Shift+R` / `Ctrl+Shift+R` untuk Rollback).
  - [x] Automatic query tracking untuk deteksi query mutasi data (`INSERT`, `UPDATE`, `DELETE`, `DDL`) dan perintah eksplisit transaksi.
  - *Files Terkait*: `src-tauri/src/commands/query.rs`, `src/lib/state/session.svelte.ts`, `src/lib/components/layout/Statusbar.svelte`, `src/lib/components/editor/SqlEditor.svelte`

---

### 🚀 Phase 5: Next-Level Server Intelligence & Powerhouse Features

- [ ] **5.1 Live Server Health & Active Process Monitor**
  - [ ] Pemantauan real-time proses & koneksi aktif (`pg_stat_activity` di Postgres, `SHOW FULL PROCESSLIST` di MySQL, `sys.dm_exec_requests` di MSSQL, lock state di SQLite).
  - [ ] Deteksi durasi query runtime, resource CPU/memory, locking dependencies (**`🔒 Blocked by PID`**), serta status koneksi (`active`, `idle in transaction`, `waiting`).
  - [ ] Tombol aksi 1-klik **`⚡ Kill Process / Terminate Backend`** untuk query hanging/blocking dengan konfirmasi safety.
  - [ ] Auto-refresh polling (interval 2s, 5s, atau manual) dan instant filtering (query > 3s, active only, by user/database).
  - *Files Terkait*: `src-tauri/src/commands/monitor.rs`, `src/lib/components/monitor/ServerMonitorModal.svelte`, `src/lib/state/monitor.svelte.ts`, `src/lib/api/client.ts`

- [ ] **5.2 Schema & Data Diff Sync Tool (Database Comparison)**
  - [ ] Komparasi struktur skema antar 2 database (Source vs Target: Dev vs Staging vs Prod).
  - [ ] Visual side-by-side diff: tabel baru/hilang, tipe kolom berbeda, missing indexes, dan mismatch foreign keys.
  - [ ] Generator otomatis script migrasi DDL sinkronisasi (`ALTER TABLE ...`, `CREATE INDEX ...`, `DROP COLUMN ...`).
  - [ ] Opsi Data Diff (pengecekan selisih baris data via hash/checksum comparison).
  - *Files Terkait*: `src-tauri/src/schema/diff.rs`, `src/lib/components/diff/SchemaDiffModal.svelte`, `src/lib/api/client.ts`

- [ ] **5.3 Interactive SQL Scratchpad Notebooks (`.fugpad`)**
  - [ ] Tab editor canvas bergaya notebook interaktif yang menggabungkan Markdown text blocks, SQL query blocks, dan visualisasi Chart.js interaktif dalam satu file.
  - [ ] Eksekusi independen per blok query (**Run Cell: `Shift+Enter`**) dengan hasil data grid interaktif tersimpan di state.
  - [ ] Ekspor notebook ke standalone HTML report interaktif atau file dokumen `.fugpad` JSON format.
  - *Files Terkait*: `src/lib/components/notebook/SqlNotebookTab.svelte`, `src/lib/state/notebook.svelte.ts`, `src/lib/components/layout/Navbar.svelte`

- [ ] **5.4 Redis & Key-Value Polyglot Inspector**
  - [ ] Driver koneksi native Redis / KeyDB via `redis-rs`.
  - [ ] Key tree pattern explorer (`user:*`, `session:*`, `cache:*` dengan namespace grouping).
  - [ ] Editor interaktif untuk semua tipe data Redis: String, Hash, List, Set, ZSet, dan Stream lengkap dengan TTL inspector & TTL editor.
  - [ ] Live Redis CLI console & Pub/Sub message stream monitor.
  - *Files Terkait*: `src-tauri/src/drivers/redis.rs`, `src/lib/components/redis/RedisKeyViewer.svelte`, `src/lib/components/connection/ConnectionModal.svelte`

- [ ] **5.5 Scheduled Query Automations & Local Backups**
  - [ ] Local background scheduler (Cron pattern / Timer) untuk eksekusi query berkala di background.
  - [ ] Export otomatis hasil query ke direktori lokal (CSV, JSON, Excel) dengan format penamaan berbasis timestamp.
  - [ ] Automated lightweight database backup runner (SQL dump schedule harian/mingguan).
  - *Files Terkait*: `src-tauri/src/scheduler/`, `src/lib/components/scheduler/SchedulerModal.svelte`, `src/lib/api/client.ts`

---

## 🎯 Prioritas Pengerjaan Rekomendasi

1. **Sprint 1 (Immediate - Selesai ✅)**: `1.1 Autocomplete Skema` ➔ `1.2 Production Safety Guard` ➔ `1.4 Rich Cell Inspectors (JSON/Date)`.
2. **Sprint 2 (Architecture - Selesai ✅)**: `2.1 Live ERD Visualizer` ➔ `2.2 Streaming Import/Export` ➔ `2.5 Smart Mock Data`.
3. **Sprint 3 (Vibe & AI - Selesai ✅)**: `3.1 NL-to-SQL Copilot` ➔ `3.2 Fix with AI` ➔ `3.3 1-Click Charting` ➔ `3.4 Visual EXPLAIN Plan`.
4. **Sprint 4 (Security & Hardening - Selesai ✅)**: `4.1 OS Keyring` ➔ `4.2 SSH Bastion Tunneling` ➔ `4.3 Transaction Control`.
5. **Sprint 5 (Powerhouse & Server Ops - Next Up 🚀)**: `5.1 Live Server Health & Process Monitor` ➔ `5.2 Schema Diff & Sync` ➔ `5.3 SQL Notebooks` ➔ `5.4 Redis Inspector` ➔ `5.5 Scheduled Automations`.
