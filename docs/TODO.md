# FugDB — PRD Implementation TODO & Action Plan

> Dokumen pelacak implementasi fitur FugDB berdasarkan spesifikasi pada [`technical-specification-and-prd.md`](./technical-specification-and-prd.md).

---

## 📊 Status Matriks Fitur (Current State vs PRD)

| Kategori Fitur | Status | Progress | Keterangan & Catatan |
| :--- | :---: | :---: | :--- |
| **A. Connection & Safety** | 🟡 Sebagian | ~60% | PG, MySQL, SQLite, MSSQL ✅. Production Guard ⏳, SSH Tunnel ⏳, Keyring ⏳ |
| **B. High-Performance SQL Editor** | 🟡 Sebagian | ~50% | CodeMirror 6 ✅, Shortcuts (`Cmd+Enter`) ✅. Autocomplete ⏳, History/Favorites ⏳, Visual EXPLAIN ⏳ |
| **C. In-Cell Grid & Staged Mutations** | 🟢 Selesai | ~85% | Virtual Grid 100k rows @ 60FPS ✅, Staged Mutation Buffer & Diff Drawer ✅. Cell Inspectors ⏳, Quick Filter ⏳ |
| **D. Multi-Source ETL & Data Transfer** | 🟡 Sebagian | ~35% | UI Wizard Modal ✅. Tokio Streaming Pipeline (CSV, JSON, Excel, DB-to-DB) ⏳ |
| **E. Schema Explorer & ERD** | 🟡 Sebagian | ~55% | Schema Tree Navigator ✅, SvelteFlow ERD Component ✅. Live FK Extractor ⏳, Data Dictionary ⏳ |
| **F. QA & Mock Data Engine** | 🟡 Sebagian | ~45% | Mock Data Modal UI ✅, `fake-rs` backend scaffold ✅. Smart Semantics & FK integrity ⏳, Data Diff ⏳ |
| **G. "Vibe Coding" & AI Integration** | ⚪ Backlog | 0% | Zero-Data-Leak NL-to-SQL Drawer ⏳, Auto-Fix Error with AI ⏳, Ollama / Cloud API ⏳ |
| **H. Data Visualization & Mini-BI** | ⚪ Backlog | 0% | 1-Click Chart Generator (Bar/Line/Pie) ⏳, SQL Scratchpad Notebooks ⏳ |

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

- [ ] **1.5 Grid Instant Column Filter & Multi-Column Sorting**
  - [ ] Filter bar per kolom (*contains, equals, regex, is null, not null*).
  - [ ] Multi-column sort toggle (Ascending / Descending) tanpa perlu menulis query SQL manual.
  - *Files Terkait*: `src/lib/components/grid/DataGrid.svelte`, `src/lib/state/tabs.svelte.ts`

---

### 🛠️ Phase 2: Schema Architecture, Live ERD & Streaming ETL

- [ ] **2.1 Live Interactive ERD Visualizer (@xyflow/svelte)**
  - [ ] Implementasikan query Foreign Key metadata pada backend driver (`generate_erd_metadata`).
  - [ ] Hubungkan relasi FK ke node & edge di `ErdModal.svelte`.
  - [ ] Tambahkan auto-layout algoritma (Dagre) untuk penataan otomatis tabel, fitur zoom/pan, dan export diagram ke **PNG / SVG**.
  - *Files Terkait*: `src-tauri/src/drivers/`, `src/lib/components/erd/ErdModal.svelte`

- [ ] **2.2 High-Performance Streaming Import / Export Engine**
  - [ ] **CSV / TSV**: Stream parser dengan auto-detect delimiter dan type guessing (`csv-async`).
  - [ ] **JSON & ndjson**: Streaming import & export array of objects.
  - [ ] **Excel (.xlsx)**: Parser & generator multi-sheet (`calamine`).
  - [ ] **SQL Dump**: Generator DDL + batch `INSERT INTO` statements.
  - [ ] Background worker via Tokio channels dengan event progress realtime (*rows/sec, bytes processed, ETA, cancel button*).
  - *Files Terkait*: `src-tauri/src/transfer/`, `src-tauri/src/commands/transfer.rs`, `src/lib/components/transfer/TransferModal.svelte`

- [ ] **2.3 Cross-Database Direct Transfer (DB-to-DB Streaming)**
  - [ ] Migrasi langsung dari Source Connection $\rightarrow$ Target Connection tanpa file perantara.
  - [ ] Pemetaan tipe data otomatis (Smart Type Mapping) & penanganan konflik (*Fail*, *Ignore*, *Upsert*).
  - *Files Terkait*: `src-tauri/src/transfer/db_to_db.rs`, `src/lib/components/transfer/TransferModal.svelte`

- [ ] **2.4 1-Click Data Dictionary Generator**
  - [ ] Ekspor dokumentasi skema lengkap (daftar tabel, tipe kolom, nullability, PK/FK, indeks, deskripsi) ke format **Markdown** dan **HTML**.
  - *Files Terkait*: `src/lib/components/sidebar/SchemaTree.svelte`, `src-tauri/src/commands/schema.rs`

- [ ] **2.5 Smart QA Mock Data Generator**
  - [ ] Deteksi tipe semantik kolom secara otomatis (Nama, Email, Alamat, Nomor Telepon, UUID, Tanggal).
  - [ ] Validasi integritas Foreign Key saat men-generate batch dummy data (100 hingga 100.000 baris).
  - *Files Terkait*: `src-tauri/src/commands/mock_data.rs`, `src/lib/components/qa/MockDataModal.svelte`

---

### 🤖 Phase 3: AI Copilot & Mini-BI Visualization

- [ ] **3.1 Privacy-First Natural Language to SQL (NL-to-SQL)**
  - [ ] AI prompt drawer untuk mengubah instruksi teks bahasa natural menjadi query SQL yang valid.
  - [ ] **Zero-Data-Leak Architecture**: Hanya mengirimkan skema DDL / nama kolom ke AI. Data baris pengguna **tidak pernah dikirim**.
  - [ ] Support Multi-Provider: Local **Ollama** (offline/private) + Cloud (OpenAI, Anthropic Claude, Gemini, DeepSeek).
  - *Files Terkait*: `src-tauri/src/commands/ai.rs`, `src/lib/components/ai/AiAssistantDrawer.svelte`

- [ ] **3.2 "Fix with AI" Button pada Query Error**
  - [ ] Tombol 1-klik pada alert error query untuk mendiagnosa dan memperbaiki sintaks query secara otomatis sesuai dialek database aktif.
  - *Files Terkait*: `src/lib/components/editor/SqlEditor.svelte`, `src/App.svelte`

- [ ] **3.3 1-Click Instant Data Charting**
  - [ ] Konversi hasil query tabular menjadi grafik visual interaktif (**Bar, Line, Area, Pie/Donut, Scatter**) menggunakan LayerChart / Chart.js / Canvas.
  - [ ] Opsi export chart ke gambar PNG/SVG.
  - *Files Terkait*: `src/lib/components/charts/DataChartModal.svelte`, `src/lib/components/grid/DataGrid.svelte`

- [ ] **3.4 Visual Query EXPLAIN Plan**
  - [ ] Visualisasi pohon grafis untuk hasil `EXPLAIN (ANALYZE, BUFFERS)` untuk mempermudah identifikasi bottleneck performa query (Seq Scan vs Index Scan).
  - *Files Terkait*: `src/lib/components/editor/ExplainPlanModal.svelte`

---

### 🔒 Phase 4: Security, Session & System Hardening

- [ ] **4.1 OS-Level Keyring Integration**
  - [ ] Simpan password dan token koneksi ke OS Native Credential Store via `keyring-rs` (macOS Keychain, Windows Credential Manager, Linux Secret Service).
  - *Files Terkait*: `src-tauri/src/utils/keyring.rs`, `src-tauri/src/commands/connection.rs`

- [ ] **4.2 Native SSH Tunneling**
  - [ ] Dukungan koneksi database via SSH Bastion Host (Password, Private Key file, SSH Agent).
  - *Files Terkait*: `src-tauri/src/utils/ssh.rs`

- [ ] **4.3 Transaction & Session Control**
  - [ ] Toggle Auto-commit vs Manual Transaction (`BEGIN`, `COMMIT`, `ROLLBACK`).
  - [ ] Indikator status transaksi aktif di Statusbar bawah.
  - *Files Terkait*: `src/lib/components/layout/Statusbar.svelte`, `src-tauri/src/drivers/`

---

## 🎯 Prioritas Pengerjaan Rekomendasi

1. **Sprint 1 (Immediate)**: `1.1 Autocomplete Skema` ➔ `1.2 Production Safety Guard` ➔ `1.4 Rich Cell Inspectors (JSON/Date)`.
2. **Sprint 2 (Architecture)**: `2.1 Live ERD Visualizer` ➔ `2.2 Streaming Import/Export (CSV/JSON/Excel)` ➔ `2.5 Smart Mock Data`.
3. **Sprint 3 (Vibe & AI)**: `3.1 NL-to-SQL Copilot (Ollama/Cloud)` ➔ `3.2 Fix with AI` ➔ `3.3 1-Click Charting`.
