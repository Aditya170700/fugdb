# Technical Specification & PRD: FugDB (Rust + Svelte 5 Edition)

> **FugDB** adalah *ultra-lightweight*, *blazing-fast*, dan *privacy-first* cross-platform Open Source Database Client (PostgreSQL, MySQL, SQLite, MSSQL, MariaDB, CockroachDB, DuckDB). Dirancang untuk menggabungkan performa instan ala TablePlus, kelengkapan fitur DataGrip/DBeaver, kemudahan QA testing, serta kapabilitas modern *vibe coding* dan AI-native.

---

## 1. Core Architecture & Tech Stack

```
+-------------------------------------------------------------------------+
|                           Svelte 5 Frontend                             |
|  - Svelte Runes ($state, $derived, $effect)                             |
|  - CodeMirror 6 (SQL Editor + Custom LSP & Autocomplete)                |
|  - svelte-virtual / Custom Canvas-DOM Virtual Grid (100k+ rows @ 60FPS) |
|  - Svelte Flow (@xyflow/svelte) (Interactive ERD Diagram)               |
|  - LayerChart / Unovis (Instant 1-Click Query Result Visualization)     |
|  - Tailwind CSS + Bits UI / Melt UI (Headless primitives & themes)      |
+-------------------------------------------------------------------------+
                                    ↕  Tauri v2 IPC (Binary / 2D Serde JSON / Stream)
+-------------------------------------------------------------------------+
|                          Rust Backend (Tauri v2)                        |
|  - Async Runtime: Tokio (Multi-threaded async I/O)                      |
|  - DB Engine: SQLx (Postgres, MySQL, SQLite, MSSQL) + DuckDB/ClickHouse |
|  - Stream & ETL: Tokio Channels, Arrow/Parquet, csv-async, calamine     |
|  - Secure Storage: keyring-rs (OS Native Keychain / Credential Vault)   |
|  - SSH Tunneling: russh / async-ssh2-lite + Native OpenSSL/rustls       |
|  - Mock Generator: fake-rs + Schema Constraint Engine                   |
|  - Local AI Bridge: Ollama (Local) & Cloud LLM API (OpenAI/Anthropic/…) |
+-------------------------------------------------------------------------+
```

### Target Metrik Performa

| Metrik | Target | Tolok Ukur Kompetitor |
| :--- | :--- | :--- |
| **Idle Memory (RAM)** | **~30 MB – 50 MB** | DBeaver (~600MB-1GB), DataGrip (~800MB-1.5GB) |
| **Cold Startup Time** | **< 800 ms** | DBeaver (5-10s), DataGrip (8-15s) |
| **App Binary Size** | **~10 MB – 18 MB** | Electron Apps (~150MB-250MB) |
| **Large Grid Render** | **100,000+ rows @ 60 FPS** | Zero lag scrolling via Virtual Grid |
| **Cross-DB ETL Throughput** | **10,000+ rows/sec stream** | Non-blocking background worker with progress bar |

---

## 2. Fitur Utama & Spesifikasi Fungsional (Comprehensive Feature Matrix)

### A. Connection Management & Environment Safety
1. **Multi-Driver Support**: PostgreSQL, MySQL, MariaDB, SQLite, MSSQL, CockroachDB, serta embedded DuckDB untuk analisis file lokal.
2. **Environment Tagging & Safety Guard**:
   - Label warna (*Dev* = Hijau, *Staging* = Kuning, *Production* = Merah).
   - **Production Guard**: Konfirmasi ganda / proteksi modal ketika menjalankan query destruktif (`DROP`, `TRUNCATE`, `ALTER`, `DELETE`/`UPDATE` tanpa klausa `WHERE`) pada database Production.
3. **Native SSH Tunneling & SSL/TLS**:
   - Koneksi aman via SSH (Password, Private Key, SSH Agent passphrase).
   - Dukungan mode SSL/TLS lengkap (*Require, Verify-CA, Verify-Full*).
4. **OS-Level Keyring**: Kredensial tidak disimpan dalam plain text melainkan via `keyring-rs` (macOS Keychain, Windows Credential Manager, Linux Secret Service).
5. **Session & Transaction Control**:
   - Toggle Auto-commit vs Manual Transaction (`BEGIN`, `COMMIT`, `ROLLBACK`).
   - Indikator status transaksi aktif di Statusbar.

---

### B. High-Performance SQL Editor
1. **CodeMirror 6 Engine**: Editor super responsif dengan kustomisasi font (JetBrains Mono, Fira Code) dan ligature.
2. **Context-Aware Autocomplete**:
   - Prediksi cerdas nama tabel, kolom, alias (`users u -> u.id`), dan fungsi SQL berdasarkan skema aktif.
   - Rekomendasi JOIN otomatis berdasarkan relasi *Foreign Key*.
3. **Execution Modes**:
   - `Cmd/Ctrl + Enter`: Eksekusi query di bawah posisi kursor.
   - `Cmd/Ctrl + Shift + Enter`: Eksekusi seluruh script / batch query.
   - Run selected text only.
4. **Visual EXPLAIN & Query Optimizer**:
   - Visualisasi grafis hasil `EXPLAIN (ANALYZE, BUFFERS)` (Postgres/MySQL) untuk mengidentifikasi bottleneck (Seq Scan vs Index Scan).
5. **Persistent Query History & Favorites**:
   - Riwayat eksekusi tersimpan di SQLite lokal dengan pencarian teks, filter durasi/status error, dan pin/bookmark query favorit.
6. **Parameterized Queries**:
   - Mendukung format placeholder (`:user_id`, `$1`) yang memunculkan form input dinamis sebelum query dijalankan.

---

### C. In-Cell Data Grid & Staged Mutations (TablePlus Style)
1. **Virtualized Data Grid**: Mampu menampilkan puluhan ribu baris tanpa penurunan performa dengan konsumsi RAM minimal.
2. **Staged Mutations Buffer (Draft Mode)**:
   - Edit nilai sel, tambah baris baru, atau tandai baris yang dihapus secara lokal terlebih dahulu.
   - Muncul panel preview SQL diff (`UPDATE ...`, `INSERT ...`, `DELETE ...`) sebelum user menekan tombol **Commit / Apply** (`Ctrl+S`).
3. **Rich Cell Inspectors**:
   - **JSON Viewer & Editor**: Tree view interaktif + formatted JSON code editor.
   - **UUID, Hex, Base64 Decoder**: Inspeksi data biner dan hash.
   - **Date-Time Picker**: Konversi timezone instan (Local vs UTC).
   - **Image & Markdown Previewer**: Preview langsung untuk tipe data blob/text panjang.
4. **Excel-Style Instant Filtering**:
   - Filter cepat per kolom (*contains, equals, regex, is null*) dan multi-column sorting tanpa perlu mengetik SQL manual.

---

### D. Multi-Source High-Performance Export / Import & Cross-DB Transfer 🚀

Mesin import/export yang dirancang *stream-based* di layer Rust Tokio worker untuk menangani data skala gigabyte tanpa menghabiskan memori frontend:

#### 1. Multi-Format Import & Export
- **Format yang Didukung**:
  - **CSV / TSV**: Custom delimiter, custom quote/escape char, deteksi otomatis header, type guessing.
  - **JSON & JSON Lines (ndjson)**: Array of objects atau newline-delimited JSON.
  - **SQL Dump**: Opsi ekspor *Structure only*, *Data only*, atau *Structure + Data* dengan batch INSERT configurable.
  - **Excel (.xlsx)**: Import sheet tertentu & export multi-table ke multi-sheet.
  - **Apache Parquet / Arrow**: Ekspor berkecepatan tinggi dan terkompresi untuk kebutuhan data engineer / analytics.

#### 2. Cross-Database Direct Transfer (DB-to-DB Streaming)
- Migrasi data langsung dari **Koneksi Sumber** ke **Koneksi Target** (misal: *Postgres Staging $\rightarrow$ Local MySQL*, atau *Production $\rightarrow$ Local SQLite*) tanpa perlu file perantara.
- **Smart Column Mapping**: Pemetaan tipe data otomatis (misal `VARCHAR` $\leftrightarrow$ `TEXT`, `INT4` $\leftrightarrow$ `INT`), dengan opsi rename/skip kolom.
- **Conflict Resolution**: Opsi *Fail on error*, *Ignore/Skip duplicates*, atau *Upsert / ON CONFLICT DO UPDATE*.

#### 3. Remote Sources & URL Ingestion
- Import data langsung dari endpoint HTTP/REST API (JSON/CSV) atau public S3 bucket URL.

#### 4. Background Job & Real-time Progress
- Menampilkan status realtime via Tauri Event: *Rows processed*, *Throughput (rows/sec)*, *Elapsed time*, *Estimated Time Remaining (ETA)*, dan tombol *Cancel / Abort* instan.
- **Dry-Run / Preview Mode**: Pratinjau 50 baris pertama hasil parsing sebelum commit ke database target.

---

### E. Schema Explorer, ERD & Data Dictionary
1. **Interactive Schema Tree**:
   - Navigasi cepat hierarki: Database $\rightarrow$ Schema $\rightarrow$ Tables, Views, Functions, Stored Procedures, Triggers, Indexes, Enums/Custom Types.
   - Fast Search Palette (`Cmd + P`) untuk lompat ke tabel manapun seketika.
2. **Interactive ERD Visualizer (@xyflow/svelte)**:
   - Visualisasi relasi tabel otomatis (*Foreign Keys*).
   - Fitur auto-layout (Dagre/ELK), drag-and-drop table nodes, zoom & pan, serta export ke **SVG / PNG**.
3. **Live Schema Diff & Migration Generator**:
   - Bandingkan struktur skema antara 2 database (misal *Dev* vs *Staging*) dan otomatis hasilkan DDL script `ALTER TABLE` / migrasi.
4. **1-Click Data Dictionary Generator**:
   - Ekspor dokumentasi skema lengkap ke format **Markdown, HTML, atau PDF** (daftar tabel, tipe kolom, nullability, primary/foreign keys, indeks, dan komentar).

---

### F. QA & Testing Superpowers
1. **Smart Mock Data Generator**:
   - Didukung oleh `fake-rs` + context constraint engine.
   - Otomatis mendeteksi tipe semantik kolom (nama, email, alamat, no telepon, timestamp masa lalu/depan, UUID, angka rentang tertentu, dan valid Foreign Key ID).
   - Generate batch dummy data (10 hingga 100.000 baris) dengan integritas referensial terjaga.
2. **Data Diff Tool**:
   - Membandingkan isi data antar tabel atau hasil query di 2 environment berbeda untuk validasi migrasi atau regresi data.

---

### G. "Vibe Coding" & Modern AI Integration (Local & Cloud)
1. **Natural Language to SQL (NL-to-SQL)**:
   - Ubah instruksi teks bebas menjadi query SQL yang valid sesuai DDL skema aktif.
2. **Zero-Data-Leak Privacy Architecture**:
   - **Hanya metadata DDL / nama kolom yang dikirim ke LLM**. Data baris (record pengguna) **100% tidak pernah disentuh** atau dikirim keluar.
3. **Auto-Fix SQL Error**:
   - Jika query menghasilkan error database, tombol *"Fix with AI"* langsung merevisi sintaks query berdasarkan pesan error dan dialek SQL terkait.
4. **AI Model Agnostic (BYOK + Ollama Local)**:
   - Dukungan Cloud: OpenAI, Anthropic Claude, Google Gemini, Groq, DeepSeek.
   - Dukungan Local (Offline & Enterprise): **Ollama / Llama.cpp** untuk isolasi jaringan total.

---

### H. Instant Data Visualization & Mini-BI
1. **1-Click Chart Generator**:
   - Ubah hasil query tabular menjadi grafik visual (**Bar, Line, Area, Pie/Donut, Scatter**) menggunakan *LayerChart* / *Unovis*.
   - Export chart ke image PNG/SVG atau embed ke dashboard ringkas.
2. **SQL Scratchpad / Notebooks**:
   - Dokumen interaktif yang menggabungkan blok Markdown dokumentasi dengan blok SQL yang dapat dijalankan langsung.

---

## 3. Struktur Direktori Proyek (Clean & Modular)

```
fugdb/
├── src-tauri/                     # Rust Backend (Tauri v2)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs                # App setup & IPC Command Registration
│       ├── error.rs               # AppError enum (thiserror + serde serialize)
│       ├── state.rs               # AppState (Connection Pool & Worker Registry)
│       ├── drivers/               # DB Driver Abstraction
│       │   ├── mod.rs             # DatabaseAdapter Trait
│       │   ├── postgres.rs        # PostgreSQL Driver (sqlx)
│       │   ├── mysql.rs           # MySQL / MariaDB Driver (sqlx)
│       │   ├── sqlite.rs          # SQLite Driver (sqlx)
│       │   └── mssql.rs           # MSSQL Driver
│       ├── transfer/              # High-Performance Import/Export & ETL Engine
│       │   ├── mod.rs             # Transfer Pipeline & Stream Channels
│       │   ├── csv_handler.rs     # CSV/TSV Reader & Writer (csv-async)
│       │   ├── json_handler.rs    # JSON / ndjson Streamer
│       │   ├── excel_handler.rs   # Excel .xlsx Parser & Exporter (calamine)
│       │   ├── parquet_handler.rs # Arrow & Parquet Handler
│       │   └── db_to_db.rs        # Direct DB-to-DB Streaming Worker
│       ├── commands/              # Tauri IPC Handlers
│       │   ├── connection.rs      # Connect, Disconnect, Ping, Test
│       │   ├── schema.rs          # Tables, Columns, Indexes, Relations
│       │   ├── query.rs           # Query Execution & Result Paging
│       │   ├── mutation.rs        # Apply Staged In-Cell Mutations
│       │   ├── transfer.rs        # Import/Export jobs & Progress Stream
│       │   ├── mock_data.rs       # QA Dummy data generation
│       │   ├── diff.rs            # Schema & Data Diff Engines
│       │   └── ai.rs              # AI NL-to-SQL & Error Fixing bridge
│       ├── models/                # Serde Data Transfer Objects (DTOs)
│       │   ├── connection.rs
│       │   ├── schema.rs
│       │   ├── query.rs
│       │   ├── transfer.rs
│       │   └── ai.rs
│       └── utils/
│           ├── ssh.rs             # SSH Tunnel & Key Manager
│           └── keyring.rs         # OS Keyring wrapper
│
└── src/                           # Svelte 5 Frontend
    ├── app.postcss                # Tailwind CSS + Custom Design System
    ├── App.svelte                 # Root layout & Router
    ├── lib/
    │   ├── api/                   # Typed Tauri invoke client
    │   │   ├── client.ts
    │   │   ├── types.ts           # Auto-generated / synced TypeScript interfaces
    │   │   └── events.ts          # Tauri Event listeners (Progress, etc.)
    │   ├── state/                 # Svelte 5 Runes State Stores (.svelte.ts)
    │   │   ├── connection.svelte.ts
    │   │   ├── tabs.svelte.ts
    │   │   ├── editor.svelte.ts
    │   │   ├── grid.svelte.ts     # In-Cell edit & Staged mutation state
    │   │   └── transfer.svelte.ts # Active Import/Export job tracker
    │   └── components/
    │       ├── layout/            # SplitPanes, Titlebar, Statusbar, CommandPalette
    │       ├── sidebar/           # Schema Tree & Quick Search
    │       ├── editor/            # CodeMirror 6 Wrapper, Autocomplete, History
    │       ├── grid/              # Virtualized Data Grid + Staged Commit Drawer
    │       ├── inspectors/        # JSON, DateTime, Image/Blob, Hex/UUID Modals
    │       ├── erd/               # Svelte Flow ERD Visualizer & Export
    │       ├── transfer/          # Import/Export Wizard & DB-to-DB Migration UI
    │       ├── charts/            # 1-Click LayerChart Visualizer
    │       ├── qa/                # Smart Mock Data & Data Diff Modals
    │       └── ai/                # NL-to-SQL Input & AI Assistant Drawer
```

---

## 4. Desain Kontrak IPC (Rust $\leftrightarrow$ Svelte)

### A. Kontrak Query Execution & Result (2D Array Optimization)

Untuk efisiensi serialisasi transfer data jutaan sel antara Rust dan Webview, baris data ditransmisikan sebagai format **2D Array of Values**:

```rust
// src-tauri/src/models/query.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryRequest {
    pub connection_id: String,
    pub sql: String,
    pub page_size: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub nullable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<serde_json::Value>>, // 2D array: efisiensi payload & memori
    pub affected_rows: u64,
    pub execution_time_ms: u128,
    pub total_rows: Option<u64>,
}
```

### B. Kontrak Multi-Source Import / Export & DB-to-DB Transfer

```rust
// src-tauri/src/models/transfer.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "config")]
pub enum TransferSource {
    File { path: String, format: TransferFormat },
    Table { connection_id: String, schema: String, table: String },
    Query { connection_id: String, sql: String },
    Url { url: String, format: TransferFormat },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "config")]
pub enum TransferTarget {
    File { path: String, format: TransferFormat },
    Table { connection_id: String, schema: String, table: String, conflict_strategy: ConflictStrategy },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransferFormat {
    Csv { delimiter: char, has_header: bool },
    Json { is_ndjson: bool },
    Excel { sheet_name: Option<String> },
    Parquet,
    SqlDump { include_ddl: bool, batch_size: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConflictStrategy {
    Fail,
    Ignore,
    Upsert { match_columns: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferProgressEvent {
    pub job_id: String,
    pub rows_processed: u64,
    pub bytes_processed: u64,
    pub rows_per_second: f64,
    pub estimated_seconds_remaining: Option<u64>,
    pub status: String, // "running" | "completed" | "failed" | "cancelled"
    pub error_message: Option<String>,
}
```

### C. Interface Backend Driver (`DatabaseAdapter` Trait)

```rust
// src-tauri/src/drivers/mod.rs
use async_trait::async_trait;
use crate::models::{query::*, schema::*, transfer::*};
use crate::error::AppError;
use tokio::sync::mpsc;

#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    async fn ping(&self) -> Result<(), AppError>;
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, AppError>;
    async fn fetch_schema_tree(&self) -> Result<SchemaTree, AppError>;
    async fn generate_erd_metadata(&self) -> Result<Vec<RelationEdge>, AppError>;
    async fn insert_mock_batch(&self, table: &str, rows: Vec<serde_json::Value>) -> Result<u64, AppError>;
    
    // Stream-based Batch Insertion & Extraction for ETL
    async fn stream_rows(&self, sql: &str, tx: mpsc::Sender<Vec<serde_json::Value>>) -> Result<(), AppError>;
    async fn batch_insert_rows(&self, table: &str, columns: &[String], rows: &[Vec<serde_json::Value>], strategy: &ConflictStrategy) -> Result<u64, AppError>;
}
```

---

## 5. Roadmap & Tahapan Implementasi

```mermaid
graph TD
    subgraph Phase 1: Core MVP (Speed & Reliability)
        A[Tauri v2 + Tokio + SQLx Backend] --> B[Postgres, MySQL, SQLite Drivers]
        B --> C[Connection Manager + Keyring Security]
        C --> D[CodeMirror 6 SQL Editor & History]
        D --> E[Virtualized In-Cell Data Grid]
        E --> F[Staged Mutation Buffer Review & Commit]
    end

    subgraph Phase 2: Power Tools & Data Transfer
        F --> G[Multi-Source Import/Export: CSV, JSON, Excel, Parquet]
        G --> H[Cross DB-to-DB Direct Streaming Migration]
        H --> I[Interactive Svelte Flow ERD Diagram]
        I --> J[Smart Mock Data Generator via fake-rs]
        J --> K[1-Click Data Dictionary Generator]
    end

    subgraph Phase 3: AI & Next-Gen Vibe
        K --> L[NL-to-SQL Assistant via BYOK & Local Ollama]
        L --> M[Visual EXPLAIN & Query Performance Analyzer]
        M --> N[1-Click Instant Data Charting & Mini-BI]
        N --> O[Live Schema Diff & Migration Generator]
    end
```

---

## 6. Master Prompt untuk Vibe Coding

Gunakan master prompt berikut pada AI Coding Assistant Anda untuk menginisialisasi pondasi project:

```
Act as a Principal Systems Architect and Rust/Svelte 5 Expert.
We are building "FugDB", an ultra-lightweight, high-performance, cross-platform open-source Database Client (PostgreSQL, MySQL, SQLite, MSSQL, MariaDB, DuckDB).

Core Tech Stack:
- Runtime: Tauri v2 (Rust backend with Tokio + SQLx)
- Frontend: Svelte 5 (using Runes: $state, $derived, $effect) + TypeScript + Tailwind CSS
- Performance Target: Cold boot < 800ms, Idle RAM < 50MB, 100k rows grid at 60 FPS
- Key Features:
  1. Multi-driver DB connection with OS Keyring & SSH Tunneling
  2. CodeMirror 6 SQL Editor with smart autocomplete & query history
  3. Virtualized data grid with TablePlus-style Staged Mutations Buffer
  4. Stream-based Multi-Source Export/Import & direct DB-to-DB migration
  5. Interactive ERD visualizer (@xyflow/svelte) & Data Dictionary generator
  6. Smart Mock Data generator (fake-rs) & Data Diff validator
  7. AI NL-to-SQL copilot (Privacy-first, BYOK + Local Ollama support)
  8. 1-Click Instant Charting for query results

Rules:
1. Rust code must be clean, use async/await with SQLx and Tokio, and avoid unwrap() in production paths (use custom thiserror AppError).
2. Frontend state management must use Svelte 5 Runes in .svelte.ts files.
3. Query results and data transfers must use 2D arrays (`Vec<Vec<serde_json::Value>>`) and Tokio mpsc streaming to prevent UI blocking and excessive serialization overhead.
4. Privacy-First AI: Never send raw row data to LLM, only schema DDL metadata.

Task 1:
Initialize the modular project structure.
1. Provide the Cargo.toml dependencies needed for Tauri v2, sqlx, tokio, serde, keyring, fake, csv-async, and calamine.
2. Implement `src-tauri/src/error.rs` using `thiserror` that implements `serde::Serialize`.
3. Implement `src-tauri/src/state.rs` managing active database connection pools in an `Arc<RwLock<HashMap<String, Box<dyn DatabaseAdapter>>>>>`.
4. Implement `src-tauri/src/models/` for query, connection, and transfer DTOs.
```