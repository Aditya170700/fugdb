<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { 
    Upload, 
    Download, 
    FileSpreadsheet, 
    FileText, 
    FileCode, 
    Database, 
    ArrowRight, 
    CheckCircle2, 
    AlertCircle, 
    X, 
    FolderOpen, 
    Play, 
    StopCircle, 
    RefreshCw, 
    Sliders, 
    FileCheck, 
    Sparkles,
    Check,
    HelpCircle,
    Eye
  } from 'lucide-svelte';

  import { connectionStore } from '$lib/state/connection.svelte';
  import { api } from '$lib/api/client';
  import type { 
    TransferFormat, 
    ConflictStrategy, 
    TransferProgressEvent, 
    FileInspectionResult, 
    ExportJobRequest, 
    ImportJobRequest 
  } from '$lib/api/types';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  // Mode: 'export' | 'import'
  let activeTab = $state<'export' | 'import'>('export');

  // --- Export State ---
  let exportConnId = $state<string>('');
  let exportSourceMode = $state<'table' | 'query'>('table');
  let exportSelectedTable = $state<string>('');
  let exportCustomSql = $state<string>('SELECT * FROM users LIMIT 5000;');
  let exportFormatType = $state<'csv' | 'tsv' | 'json' | 'excel' | 'sql'>('csv');
  let exportTargetPath = $state<string>('');
  let exportCsvDelimiter = $state<string>(',');
  let exportCsvHasHeader = $state<boolean>(true);
  let exportJsonIsNdjson = $state<boolean>(false);
  let exportJsonPretty = $state<boolean>(true);
  let exportExcelSheet = $state<string>('Sheet1');
  let exportSqlIncludeDdl = $state<boolean>(true);
  let exportSqlBatchSize = $state<number>(250);

  // --- Import State ---
  let importSourcePath = $state<string>('');
  let importConnId = $state<string>('');
  let importTargetSchema = $state<string>('public');
  let importTargetTable = $state<string>('');
  let importFormatType = $state<'csv' | 'tsv' | 'json' | 'excel' | 'sql'>('csv');
  let importCsvDelimiter = $state<string>(',');
  let importConflictStrategy = $state<'fail' | 'ignore' | 'upsert'>('fail');
  let importCreateTableIfMissing = $state<boolean>(true);
  let inspectionResult = $state<FileInspectionResult | null>(null);
  let isInspecting = $state<boolean>(false);

  // --- Job & Progress State ---
  let activeJobId = $state<string | null>(null);
  let isRunning = $state<boolean>(false);
  let progressEvent = $state<TransferProgressEvent | null>(null);
  let transferError = $state<string | null>(null);
  let unlistenProgress: UnlistenFn | null = null;

  // Active Connection & Schema derived
  const connections = $derived(connectionStore.connections);
  const activeConn = $derived(connectionStore.activeConnection);
  const activeSchema = $derived(connectionStore.activeSchemaTree);

  $effect(() => {
    if (isOpen && activeConn) {
      if (!exportConnId) exportConnId = activeConn.id;
      if (!importConnId) importConnId = activeConn.id;
      if (activeSchema?.tables && activeSchema.tables.length > 0 && !exportSelectedTable) {
        exportSelectedTable = activeSchema.tables[0].name;
        if (!importTargetTable) importTargetTable = activeSchema.tables[0].name;
      }
    }
  });

  onMount(async () => {
    unlistenProgress = await api.onTransferProgress((event) => {
      progressEvent = event;
      if (event.status === 'completed' || event.status === 'failed' || event.status === 'cancelled') {
        isRunning = false;
        if (event.status === 'failed') {
          transferError = event.errorMessage || 'Transfer failed';
        }
      }
    });
  });

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress();
    }
  });

  // --- Handlers: Export ---
  async function handleBrowseSave() {
    let ext = 'csv';
    let filterName = 'CSV File (*.csv)';
    switch (exportFormatType) {
      case 'tsv': ext = 'tsv'; filterName = 'TSV File (*.tsv)'; break;
      case 'json': ext = exportJsonIsNdjson ? 'ndjson' : 'json'; filterName = 'JSON File (*.json)'; break;
      case 'excel': ext = 'xlsx'; filterName = 'Excel Spreadsheet (*.xlsx)'; break;
      case 'sql': ext = 'sql'; filterName = 'SQL Dump (*.sql)'; break;
    }

    const defaultName = `${exportSelectedTable || 'export_data'}.${ext}`;
    const selected = await api.pickSaveFile(defaultName, [{ name: filterName, extensions: [ext] }]);
    if (selected) {
      exportTargetPath = selected;
    }
  }

  async function startExport() {
    if (!exportTargetPath) {
      await handleBrowseSave();
      if (!exportTargetPath) return;
    }

    isRunning = true;
    transferError = null;
    progressEvent = null;

    let format: TransferFormat;
    switch (exportFormatType) {
      case 'csv':
        format = { type: 'csv', delimiter: exportCsvDelimiter, hasHeader: exportCsvHasHeader };
        break;
      case 'tsv':
        format = { type: 'tsv', hasHeader: exportCsvHasHeader };
        break;
      case 'json':
        format = { type: 'json', isNdjson: exportJsonIsNdjson, pretty: exportJsonPretty };
        break;
      case 'excel':
        format = { type: 'excel', sheetName: exportExcelSheet };
        break;
      case 'sql':
        format = { type: 'sqlDump', includeDdl: exportSqlIncludeDdl, batchSize: exportSqlBatchSize };
        break;
    }

    const req: ExportJobRequest = {
      connectionId: exportConnId || activeConn?.id || '',
      schema: 'public',
      table: exportSourceMode === 'table' ? exportSelectedTable : undefined,
      query: exportSourceMode === 'query' ? exportCustomSql : undefined,
      targetPath: exportTargetPath,
      format,
    };

    try {
      const jobId = await api.startExportJob(req);
      activeJobId = jobId;
    } catch (err: any) {
      isRunning = false;
      transferError = err?.message || String(err);
    }
  }

  // --- Handlers: Import ---
  async function handleBrowseOpen() {
    const selected = await api.pickOpenFile();
    if (selected) {
      importSourcePath = selected;
      await runFileInspection(selected);
    }
  }

  async function runFileInspection(filePath: string) {
    if (!filePath) return;
    isInspecting = true;
    inspectionResult = null;
    try {
      const res = await api.inspectFile(filePath);
      inspectionResult = res;

      // Auto-configure format from inspection
      if (res.detectedFormat === 'csv' || res.detectedFormat === 'tsv') {
        importFormatType = res.detectedFormat as any;
        if (res.delimiter) importCsvDelimiter = res.delimiter;
      } else if (res.detectedFormat === 'json' || res.detectedFormat === 'ndjson') {
        importFormatType = 'json';
      } else if (res.detectedFormat === 'excel') {
        importFormatType = 'excel';
      }

      // Suggest table name from filename
      const baseName = filePath.split('/').pop()?.split('\\').pop()?.split('.')[0];
      if (baseName && !importTargetTable) {
        importTargetTable = baseName.toLowerCase().replace(/[^a-z0-9_]/g, '_');
      }
    } catch (err) {
      console.error('Inspection failed:', err);
    } finally {
      isInspecting = false;
    }
  }

  async function startImport() {
    if (!importSourcePath) {
      await handleBrowseOpen();
      if (!importSourcePath) return;
    }
    if (!importTargetTable) {
      transferError = 'Please specify target table name';
      return;
    }

    isRunning = true;
    transferError = null;
    progressEvent = null;

    let format: TransferFormat;
    switch (importFormatType) {
      case 'csv':
        format = { type: 'csv', delimiter: importCsvDelimiter, hasHeader: true };
        break;
      case 'tsv':
        format = { type: 'tsv', hasHeader: true };
        break;
      case 'json':
        format = { type: 'json', isNdjson: inspectionResult?.detectedFormat === 'ndjson' };
        break;
      case 'excel':
        format = { type: 'excel', sheetName: inspectionResult?.sheetNames?.[0] };
        break;
      case 'sql':
        format = { type: 'sqlDump' };
        break;
    }

    let conflictStrategy: ConflictStrategy;
    if (importConflictStrategy === 'ignore') {
      conflictStrategy = 'ignore';
    } else if (importConflictStrategy === 'upsert') {
      conflictStrategy = { upsert: { matchColumns: ['id'] } };
    } else {
      conflictStrategy = 'fail';
    }

    const req: ImportJobRequest = {
      connectionId: importConnId || activeConn?.id || '',
      schema: importTargetSchema,
      table: importTargetTable,
      sourcePath: importSourcePath,
      format,
      conflictStrategy,
      createTableIfMissing: importCreateTableIfMissing,
    };

    try {
      const jobId = await api.startImportJob(req);
      activeJobId = jobId;
    } catch (err: any) {
      isRunning = false;
      transferError = err?.message || String(err);
    }
  }

  async function cancelJob() {
    if (activeJobId) {
      await api.cancelTransferJob(activeJobId);
      isRunning = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen && !isRunning) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div 
    class="fixed inset-0 bg-black/75 backdrop-blur-xs z-50 flex items-center justify-center p-3 sm:p-6 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <div 
      class="bg-surface-900 border border-slate-200 dark:border-slate-800 w-full max-w-4xl max-h-[92vh] rounded-2xl shadow-2xl flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 animate-in zoom-in-95 duration-200"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-surface-950/80">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-gradient-to-tr from-indigo-600 to-violet-500 text-white rounded-xl shadow-md shadow-indigo-500/20 shrink-0">
            <Upload size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="font-bold text-sm text-slate-900 dark:text-white">
                Streaming Data Transfer & ETL Engine
              </h3>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-bold border border-emerald-500/30">
                Tokio Streaming Channels
              </span>
            </div>
            <p class="text-[11px] text-slate-500 dark:text-slate-400">
              High-performance batch import & export for CSV, TSV, JSON, Excel (.xlsx), and SQL Dumps
            </p>
          </div>
        </div>
        <button 
          type="button"
          onclick={onClose} 
          disabled={isRunning}
          class="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-surface-800 transition-colors cursor-pointer disabled:opacity-40" 
          title="Close (Esc)"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Mode Selector Tabs -->
      <div class="flex border-b border-slate-200 dark:border-slate-800 bg-surface-950/50 px-5 pt-2 gap-2 text-xs font-semibold">
        <button 
          type="button"
          onclick={() => activeTab = 'export'}
          class="flex items-center gap-2 px-4 py-2 border-b-2 transition-all cursor-pointer {activeTab === 'export' ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400 bg-surface-900/60 rounded-t-lg' : 'border-transparent text-slate-500 hover:text-slate-300'}"
        >
          <Download size={14} />
          <span>Export from Database</span>
        </button>
        <button 
          type="button"
          onclick={() => activeTab = 'import'}
          class="flex items-center gap-2 px-4 py-2 border-b-2 transition-all cursor-pointer {activeTab === 'import' ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400 bg-surface-900/60 rounded-t-lg' : 'border-transparent text-slate-500 hover:text-slate-300'}"
        >
          <Upload size={14} />
          <span>Import to Database</span>
        </button>
      </div>

      <!-- Modal Body (Scrollable) -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6 text-xs">
        {#if activeTab === 'export'}
          <!-- ================= EXPORT TAB ================= -->
          <div class="space-y-5">
            <!-- 1. Source Database & Query/Table Selection -->
            <div class="bg-surface-950/70 p-4 rounded-xl border border-slate-200 dark:border-slate-800/80 space-y-3">
              <span class="font-bold text-slate-800 dark:text-slate-200 text-xs uppercase tracking-wider block">
                1. Source Data
              </span>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label for="export-conn" class="text-[11px] font-medium text-slate-400 block mb-1">Source Connection</label>
                  <select
                    id="export-conn"
                    bind:value={exportConnId}
                    class="w-full bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                  >
                    {#each connections as conn (conn.id)}
                      <option value={conn.id}>{conn.name} ({conn.driver})</option>
                    {/each}
                  </select>
                </div>

                <div>
                  <span class="text-[11px] font-medium text-slate-400 block mb-1">Extraction Scope</span>
                  <div class="flex gap-2">
                    <button
                      type="button"
                      onclick={() => exportSourceMode = 'table'}
                      class="flex-1 py-1.5 rounded-lg border text-xs font-semibold transition-all {exportSourceMode === 'table' ? 'bg-indigo-600/20 border-indigo-500 text-indigo-400' : 'bg-surface-900 border-slate-700 text-slate-400'}"
                    >
                      Entire Table
                    </button>
                    <button
                      type="button"
                      onclick={() => exportSourceMode = 'query'}
                      class="flex-1 py-1.5 rounded-lg border text-xs font-semibold transition-all {exportSourceMode === 'query' ? 'bg-indigo-600/20 border-indigo-500 text-indigo-400' : 'bg-surface-900 border-slate-700 text-slate-400'}"
                    >
                      Custom SQL Query
                    </button>
                  </div>
                </div>
              </div>

              {#if exportSourceMode === 'table'}
                <div>
                  <label for="export-table-select" class="text-[11px] font-medium text-slate-400 block mb-1">Select Table</label>
                  <select
                    id="export-table-select"
                    bind:value={exportSelectedTable}
                    class="w-full bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500 font-mono"
                  >
                    {#if activeSchema?.tables}
                      {#each activeSchema.tables as tbl}
                        <option value={tbl.name}>{tbl.schema}.{tbl.name} ({tbl.rowCountEstimate || 0} rows)</option>
                      {/each}
                    {:else}
                      <option value="users">users</option>
                    {/if}
                  </select>
                </div>
              {:else}
                <div>
                  <label for="export-query-input" class="text-[11px] font-medium text-slate-400 block mb-1">SQL Query to Export</label>
                  <textarea
                    id="export-query-input"
                    bind:value={exportCustomSql}
                    rows="3"
                    class="w-full bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg p-2.5 font-mono text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                    placeholder="SELECT * FROM my_table WHERE active = true..."
                  ></textarea>
                </div>
              {/if}
            </div>

            <!-- 2. Target Format Selector -->
            <div class="bg-surface-950/70 p-4 rounded-xl border border-slate-200 dark:border-slate-800/80 space-y-3">
              <span class="font-bold text-slate-800 dark:text-slate-200 text-xs uppercase tracking-wider block">
                2. Target Format
              </span>

              <div class="grid grid-cols-2 sm:grid-cols-5 gap-2">
                {#each [
                  { id: 'csv', name: 'CSV File', desc: 'Comma Separated', icon: FileText, color: 'text-sky-400' },
                  { id: 'tsv', name: 'TSV File', desc: 'Tab Separated', icon: FileText, color: 'text-cyan-400' },
                  { id: 'json', name: 'JSON Document', desc: 'Array / ndjson', icon: FileCode, color: 'text-amber-400' },
                  { id: 'excel', name: 'Excel (.xlsx)', desc: 'Multi-column formatted', icon: FileSpreadsheet, color: 'text-emerald-400' },
                  { id: 'sql', name: 'SQL Dump', desc: 'DDL + batch INSERT', icon: Database, color: 'text-indigo-400' }
                ] as fmt}
                  <button
                    type="button"
                    onclick={() => exportFormatType = fmt.id as any}
                    class="p-3 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between {exportFormatType === fmt.id ? 'bg-indigo-600/15 border-indigo-500 text-white shadow-md' : 'bg-surface-900 border-slate-800 text-slate-400 hover:border-slate-700'}"
                  >
                    <div class="flex items-center justify-between mb-2">
                      <fmt.icon size={18} class={fmt.color} />
                      {#if exportFormatType === fmt.id}
                        <Check size={14} class="text-indigo-400" />
                      {/if}
                    </div>
                    <div>
                      <div class="font-bold text-xs text-slate-200">{fmt.name}</div>
                      <div class="text-[10px] text-slate-500 truncate">{fmt.desc}</div>
                    </div>
                  </button>
                {/each}
              </div>

              <!-- Format Options Row -->
              <div class="pt-2 border-t border-slate-800/60 grid grid-cols-1 sm:grid-cols-2 gap-3">
                {#if exportFormatType === 'csv'}
                  <div class="flex items-center gap-3">
                    <label for="csv-delim" class="text-[11px] text-slate-400 shrink-0">Delimiter:</label>
                    <select id="csv-delim" bind:value={exportCsvDelimiter} class="bg-surface-900 border border-slate-700 rounded-lg px-2 py-1 text-xs text-slate-200">
                      <option value=",">Comma (,)</option>
                      <option value=";">Semicolon (;)</option>
                      <option value="|">Pipe (|)</option>
                    </select>
                  </div>
                  <div class="flex items-center gap-2">
                    <input type="checkbox" id="csv-header" bind:checked={exportCsvHasHeader} class="rounded text-indigo-600 bg-surface-900 border-slate-700" />
                    <label for="csv-header" class="text-[11px] text-slate-300">Include header row with column names</label>
                  </div>
                {:else if exportFormatType === 'json'}
                  <div class="flex items-center gap-2">
                    <input type="checkbox" id="json-nd" bind:checked={exportJsonIsNdjson} class="rounded text-indigo-600 bg-surface-900 border-slate-700" />
                    <label for="json-nd" class="text-[11px] text-slate-300">NDJSON / JSON Lines (1 JSON object per row)</label>
                  </div>
                  {#if !exportJsonIsNdjson}
                    <div class="flex items-center gap-2">
                      <input type="checkbox" id="json-pretty" bind:checked={exportJsonPretty} class="rounded text-indigo-600 bg-surface-900 border-slate-700" />
                      <label for="json-pretty" class="text-[11px] text-slate-300">Pretty print (Indented 2 spaces)</label>
                    </div>
                  {/if}
                {:else if exportFormatType === 'excel'}
                  <div class="flex items-center gap-3">
                    <label for="excel-sheet" class="text-[11px] text-slate-400 shrink-0">Worksheet Name:</label>
                    <input id="excel-sheet" type="text" bind:value={exportExcelSheet} class="bg-surface-900 border border-slate-700 rounded-lg px-2 py-1 text-xs text-slate-200" placeholder="Sheet1" />
                  </div>
                {:else if exportFormatType === 'sql'}
                  <div class="flex items-center gap-2">
                    <input type="checkbox" id="sql-ddl" bind:checked={exportSqlIncludeDdl} class="rounded text-indigo-600 bg-surface-900 border-slate-700" />
                    <label for="sql-ddl" class="text-[11px] text-slate-300">Include CREATE TABLE statement</label>
                  </div>
                  <div class="flex items-center gap-2">
                    <label for="sql-batch" class="text-[11px] text-slate-400 shrink-0">Batch Size:</label>
                    <input id="sql-batch" type="number" bind:value={exportSqlBatchSize} class="w-20 bg-surface-900 border border-slate-700 rounded-lg px-2 py-1 text-xs text-slate-200" min="10" max="5000" />
                  </div>
                {/if}
              </div>
            </div>

            <!-- 3. Destination File Path -->
            <div class="bg-surface-950/70 p-4 rounded-xl border border-slate-200 dark:border-slate-800/80 space-y-2">
              <span class="font-bold text-slate-800 dark:text-slate-200 text-xs uppercase tracking-wider block">
                3. Save Destination
              </span>
              <div class="flex gap-2">
                <input
                  type="text"
                  bind:value={exportTargetPath}
                  placeholder="Click Browse to select destination file..."
                  class="flex-1 bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
                />
                <button
                  type="button"
                  onclick={handleBrowseSave}
                  class="px-3.5 py-1.5 rounded-lg font-semibold bg-surface-800 hover:bg-surface-700 text-slate-200 border border-slate-700 flex items-center gap-1.5 cursor-pointer shadow-xs"
                >
                  <FolderOpen size={14} class="text-indigo-400" />
                  <span>Browse...</span>
                </button>
              </div>
            </div>
          </div>
        {:else}
          <!-- ================= IMPORT TAB ================= -->
          <div class="space-y-5">
            <!-- 1. Source File Selection -->
            <div class="bg-surface-950/70 p-4 rounded-xl border border-slate-200 dark:border-slate-800/80 space-y-3">
              <span class="font-bold text-slate-800 dark:text-slate-200 text-xs uppercase tracking-wider block">
                1. Source File
              </span>
              <div class="flex gap-2">
                <input
                  type="text"
                  bind:value={importSourcePath}
                  placeholder="Click Browse to select data file (.csv, .tsv, .json, .xlsx, .sql)..."
                  class="flex-1 bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
                />
                <button
                  type="button"
                  onclick={handleBrowseOpen}
                  class="px-3.5 py-1.5 rounded-lg font-semibold bg-surface-800 hover:bg-surface-700 text-slate-200 border border-slate-700 flex items-center gap-1.5 cursor-pointer shadow-xs"
                >
                  <FolderOpen size={14} class="text-indigo-400" />
                  <span>Browse...</span>
                </button>
              </div>

              {#if isInspecting}
                <div class="flex items-center gap-2 text-indigo-400 text-xs py-1">
                  <RefreshCw size={13} class="animate-spin" />
                  <span>Inspecting format, schema, and columns...</span>
                </div>
              {/if}

              <!-- File Inspection Result Preview Card -->
              {#if inspectionResult}
                <div class="p-3 bg-surface-900/90 rounded-xl border border-slate-800 space-y-2">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2 font-bold text-xs text-slate-200">
                      <FileCheck size={15} class="text-emerald-400" />
                      <span>Detected: <strong class="uppercase text-indigo-400">{inspectionResult.detectedFormat}</strong></span>
                      <span class="text-slate-500">•</span>
                      <span class="text-slate-400 font-normal">{(inspectionResult.totalBytes / 1024).toFixed(1)} KB</span>
                      <span class="text-slate-500">•</span>
                      <span class="text-slate-400 font-normal">{inspectionResult.columns.length} columns found</span>
                    </div>
                  </div>

                  <!-- Sample Rows Preview Table -->
                  {#if inspectionResult.sampleRows.length > 0}
                    <div class="overflow-x-auto border border-slate-800 rounded-lg max-h-36">
                      <table class="w-full text-left font-mono text-[10px]">
                        <thead class="bg-surface-950 text-slate-400 uppercase border-b border-slate-800 sticky top-0">
                          <tr>
                            {#each inspectionResult.columns as col}
                              <th class="px-2.5 py-1 font-semibold">{col}</th>
                            {/each}
                          </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-800/60 text-slate-300">
                          {#each inspectionResult.sampleRows as row}
                            <tr class="hover:bg-surface-800/40">
                              {#each row as val}
                                <td class="px-2.5 py-1 truncate max-w-[120px]">{val === null ? 'NULL' : String(val)}</td>
                              {/each}
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- 2. Target Database & Conflict Resolution -->
            <div class="bg-surface-950/70 p-4 rounded-xl border border-slate-200 dark:border-slate-800/80 space-y-3">
              <span class="font-bold text-slate-800 dark:text-slate-200 text-xs uppercase tracking-wider block">
                2. Target Destination
              </span>

              <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div>
                  <label for="import-conn" class="text-[11px] font-medium text-slate-400 block mb-1">Target Connection</label>
                  <select
                    id="import-conn"
                    bind:value={importConnId}
                    class="w-full bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                  >
                    {#each connections as conn (conn.id)}
                      <option value={conn.id}>{conn.name} ({conn.driver})</option>
                    {/each}
                  </select>
                </div>

                <div>
                  <label for="import-schema" class="text-[11px] font-medium text-slate-400 block mb-1">Schema</label>
                  <input
                    id="import-schema"
                    type="text"
                    bind:value={importTargetSchema}
                    class="w-full bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>

                <div>
                  <label for="import-table" class="text-[11px] font-medium text-slate-400 block mb-1">Target Table Name</label>
                  <input
                    id="import-table"
                    type="text"
                    bind:value={importTargetTable}
                    placeholder="e.g. users"
                    class="w-full bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 font-mono focus:outline-none focus:border-indigo-500 font-bold text-indigo-400"
                  />
                </div>
              </div>

              <!-- Options -->
              <div class="pt-3 border-t border-slate-800/60 grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label for="import-conflict" class="text-[11px] font-medium text-slate-400 block mb-1">Conflict Resolution Strategy</label>
                  <select
                    id="import-conflict"
                    bind:value={importConflictStrategy}
                    class="w-full bg-surface-900 border border-slate-700 rounded-lg px-2.5 py-1 text-xs text-slate-200"
                  >
                    <option value="fail">Fail on duplicate / constraint violation</option>
                    <option value="ignore">Ignore duplicates (Skip conflicting rows)</option>
                    <option value="upsert">Upsert / Overwrite existing records</option>
                  </select>
                </div>

                <div class="flex items-center gap-2 pt-4">
                  <input
                    type="checkbox"
                    id="auto-create"
                    bind:checked={importCreateTableIfMissing}
                    class="rounded text-indigo-600 bg-surface-900 border-slate-700"
                  />
                  <label for="auto-create" class="text-[11px] text-slate-300">Auto-create table if not exists (Smart type inference)</label>
                </div>
              </div>
            </div>
          </div>
        {/if}

        <!-- Real-time Progress Monitor Area -->
        {#if isRunning || progressEvent}
          <div class="bg-surface-950 p-4 rounded-xl border border-slate-800 space-y-3">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                {#if isRunning}
                  <RefreshCw size={15} class="animate-spin text-indigo-400" />
                  <span class="font-bold text-xs text-white">Streaming Transfer in Progress...</span>
                {:else if progressEvent?.status === 'completed'}
                  <CheckCircle2 size={16} class="text-emerald-400" />
                  <span class="font-bold text-xs text-emerald-400">Transfer Completed Successfully!</span>
                {:else if progressEvent?.status === 'cancelled'}
                  <StopCircle size={16} class="text-amber-400" />
                  <span class="font-bold text-xs text-amber-400">Transfer Cancelled</span>
                {:else}
                  <AlertCircle size={16} class="text-rose-400" />
                  <span class="font-bold text-xs text-rose-400">Transfer Failed</span>
                {/if}
              </div>

              {#if progressEvent}
                <div class="flex items-center gap-3 text-[11px] font-mono">
                  <span class="text-indigo-400 font-bold">{Math.round(progressEvent.rowsPerSecond || 0).toLocaleString()} rows/sec</span>
                  {#if progressEvent.estimatedSecondsRemaining}
                    <span class="text-slate-400">ETA: {progressEvent.estimatedSecondsRemaining}s</span>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- Progress Bar -->
            <div class="w-full h-2.5 bg-surface-900 rounded-full overflow-hidden border border-slate-800">
              <div 
                class="h-full bg-gradient-to-r from-indigo-500 via-violet-500 to-emerald-400 transition-all duration-200" 
                style="width: {progressEvent?.percentage !== undefined ? Math.min(100, Math.max(0, progressEvent.percentage)) : isRunning ? 50 : 100}%"
              ></div>
            </div>

            <!-- Metrics Grid -->
            <div class="grid grid-cols-3 gap-2 pt-1 font-mono text-[11px]">
              <div class="bg-surface-900 p-2 rounded-lg border border-slate-800/80">
                <span class="text-slate-500 block text-[10px]">PROCESSED ROWS</span>
                <span class="font-bold text-slate-200">{progressEvent?.rowsProcessed.toLocaleString() || 0}</span>
              </div>
              <div class="bg-surface-900 p-2 rounded-lg border border-slate-800/80">
                <span class="text-slate-500 block text-[10px]">BYTES STREAMED</span>
                <span class="font-bold text-slate-200">{((progressEvent?.bytesProcessed || 0) / 1024).toFixed(1)} KB</span>
              </div>
              <div class="bg-surface-900 p-2 rounded-lg border border-slate-800/80">
                <span class="text-slate-500 block text-[10px]">STATUS</span>
                <span class="font-bold uppercase text-indigo-400">{progressEvent?.status || 'running'}</span>
              </div>
            </div>

            {#if progressEvent?.message}
              <p class="text-[11px] text-slate-400 font-mono">{progressEvent.message}</p>
            {/if}
          </div>
        {/if}

        {#if transferError}
          <div class="p-3 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-300 text-xs flex items-center gap-2">
            <AlertCircle size={15} class="shrink-0 text-rose-400" />
            <span class="font-mono">{transferError}</span>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3.5 border-t border-slate-200 dark:border-slate-800 bg-surface-950/70 flex items-center justify-between">
        <div class="text-[11px] text-slate-500">
          Powered by Rust Tokio Async Streams & Calamine Engine
        </div>

        <div class="flex items-center gap-2">
          {#if isRunning}
            <button
              type="button"
              onclick={cancelJob}
              class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-rose-600 hover:bg-rose-500 text-white flex items-center gap-1.5 cursor-pointer shadow-md shadow-rose-600/20"
            >
              <StopCircle size={14} />
              <span>Cancel Job</span>
            </button>
          {:else}
            <button 
              type="button"
              onclick={onClose} 
              class="px-3.5 py-1.5 text-xs font-semibold text-slate-400 hover:text-slate-200 rounded-lg hover:bg-surface-800 transition-colors cursor-pointer"
            >
              Close
            </button>

            {#if activeTab === 'export'}
              <button
                type="button"
                onclick={startExport}
                class="px-5 py-1.5 rounded-lg text-xs font-semibold bg-gradient-to-r from-indigo-600 to-violet-600 hover:from-indigo-500 hover:to-violet-500 text-white flex items-center gap-1.5 cursor-pointer shadow-md shadow-indigo-600/25 transition-all"
              >
                <Download size={14} />
                <span>Start Export</span>
              </button>
            {:else}
              <button
                type="button"
                onclick={startImport}
                class="px-5 py-1.5 rounded-lg text-xs font-semibold bg-gradient-to-r from-indigo-600 to-violet-600 hover:from-indigo-500 hover:to-violet-500 text-white flex items-center gap-1.5 cursor-pointer shadow-md shadow-indigo-600/25 transition-all"
              >
                <Upload size={14} />
                <span>Start Import</span>
              </button>
            {/if}
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
