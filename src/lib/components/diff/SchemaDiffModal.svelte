<script lang="ts">
  import { diffStore } from '$lib/state/diff.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { 
    GitCompare, 
    ArrowRight, 
    ArrowLeftRight, 
    RefreshCw, 
    X, 
    Search, 
    Plus, 
    Trash2, 
    Edit3, 
    CheckCheck, 
    Check, 
    Copy, 
    Download, 
    Play, 
    AlertTriangle, 
    AlertCircle, 
    ShieldAlert, 
    Database, 
    Table, 
    Columns, 
    Clock,
    FileCode,
    Layers,
    Code2
  } from 'lucide-svelte';

  const connections = $derived(connectionStore.connections);
  const result = $derived(diffStore.result);
  const tableDiffs = $derived(diffStore.filteredTableDiffs);
  const selectedTable = $derived(diffStore.selectedTableDiff);

  const srcConn = $derived(connections.find(c => c.id === diffStore.sourceConnectionId));
  const tgtConn = $derived(connections.find(c => c.id === diffStore.targetConnectionId));

  let copiedSql = $state(false);
  let isConfirmApplyOpen = $state(false);

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    copiedSql = true;
    setTimeout(() => { copiedSql = false; }, 2000);
  }

  function downloadSqlFile() {
    if (!result?.fullMigrationSql) return;
    const blob = new Blob([result.fullMigrationSql], { type: 'text/sql;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `migration_${srcConn?.name || 'source'}_to_${tgtConn?.name || 'target'}_${Date.now()}.sql`;
    link.click();
    URL.revokeObjectURL(url);
  }

  function getActionBadge(action: string) {
    switch (action) {
      case 'create':
        return { label: 'Create', color: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-500/30' };
      case 'alter':
        return { label: 'Alter', color: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/30' };
      case 'drop':
        return { label: 'Drop', color: 'bg-rose-500/15 text-rose-700 dark:text-rose-300 border-rose-500/30' };
      default:
        return { label: 'Synced', color: 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20' };
    }
  }
</script>

{#if diffStore.isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-slate-950/70 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
    onclick={(e) => {
      if (e.target === e.currentTarget && !isConfirmApplyOpen) {
        diffStore.close();
      }
    }}
  >
    <!-- Modal Window -->
    <div class="w-full max-w-7xl h-[92vh] bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-150 relative">
      
      <!-- Top Header & Connection Selectors -->
      <div class="px-5 py-3 border-b border-slate-200 dark:border-slate-800 bg-surface-900 flex flex-wrap items-center justify-between gap-3 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20">
            <GitCompare size={20} />
          </div>
          <div>
            <h2 class="text-base font-bold text-slate-900 dark:text-slate-100 flex items-center gap-2">
              <span>Schema & Data Diff Sync Tool</span>
              {#if result}
                <span class="text-xs font-mono font-normal text-slate-500">({result.executionTimeMs.toFixed(1)} ms)</span>
              {/if}
            </h2>
            <p class="text-xs text-slate-500 dark:text-slate-400">
              Compare schema structures across environments and generate sync migration scripts
            </p>
          </div>
        </div>

        <!-- Connection Pair Controls -->
        <div class="flex items-center gap-2 bg-surface-950 p-1 rounded-lg border border-slate-200 dark:border-slate-800">
          <!-- Source DB -->
          <div class="flex items-center gap-1.5 px-2">
            <span class="text-[11px] font-bold text-slate-500 uppercase">Source:</span>
            <select
              bind:value={diffStore.sourceConnectionId}
              onchange={() => diffStore.compare()}
              class="bg-surface-800 border border-slate-200 dark:border-slate-700 rounded px-2 py-1 text-xs font-medium text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500"
            >
              {#each connections as conn}
                <option value={conn.id}>{conn.name} ({conn.driver.toUpperCase()})</option>
              {/each}
            </select>
          </div>

          <!-- Swap Button -->
          <button
            type="button"
            onclick={() => diffStore.swap()}
            class="p-1.5 text-slate-500 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded transition-colors cursor-pointer"
            title="Swap Source and Target"
          >
            <ArrowLeftRight size={14} />
          </button>

          <!-- Target DB -->
          <div class="flex items-center gap-1.5 px-2">
            <span class="text-[11px] font-bold text-slate-500 uppercase">Target:</span>
            <select
              bind:value={diffStore.targetConnectionId}
              onchange={() => diffStore.compare()}
              class="bg-surface-800 border border-slate-200 dark:border-slate-700 rounded px-2 py-1 text-xs font-medium text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500"
            >
              {#each connections as conn}
                <option value={conn.id}>{conn.name} ({conn.driver.toUpperCase()})</option>
              {/each}
            </select>
          </div>

          <!-- Run Compare Button -->
          <button
            type="button"
            onclick={() => diffStore.compare()}
            disabled={diffStore.isComparing}
            class="flex items-center gap-1.5 px-3 py-1 bg-indigo-600 hover:bg-indigo-500 text-white font-bold rounded text-xs transition-colors cursor-pointer disabled:opacity-50 shadow-xs ml-1"
          >
            <RefreshCw size={12} class={diffStore.isComparing ? 'animate-spin' : ''} />
            <span>{diffStore.isComparing ? 'Comparing...' : 'Compare'}</span>
          </button>
        </div>

        <!-- Close Button -->
        <button
          type="button"
          onclick={() => diffStore.close()}
          class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-1 rounded-lg hover:bg-slate-100 dark:hover:bg-surface-800 transition-colors cursor-pointer"
          title="Close (Esc)"
        >
          <X size={18} />
        </button>
      </div>

      <!-- Toast Messages -->
      {#if diffStore.successMessage}
        <div class="px-5 py-2 bg-emerald-500/15 border-b border-emerald-500/30 flex items-center justify-between text-xs text-emerald-800 dark:text-emerald-300 animate-in fade-in duration-150 shrink-0">
          <div class="flex items-center gap-2">
            <Check size={14} class="text-emerald-500" />
            <span class="font-medium">{diffStore.successMessage}</span>
          </div>
          <button type="button" onclick={() => diffStore.successMessage = null} class="text-emerald-600 hover:text-emerald-800 dark:hover:text-emerald-200">
            <X size={13} />
          </button>
        </div>
      {/if}

      {#if diffStore.errorMessage}
        <div class="px-5 py-2 bg-rose-500/15 border-b border-rose-500/30 flex items-center justify-between text-xs text-rose-800 dark:text-rose-300 animate-in fade-in duration-150 shrink-0">
          <div class="flex items-center gap-2">
            <AlertCircle size={14} class="text-rose-500" />
            <span class="font-medium">{diffStore.errorMessage}</span>
          </div>
          <button type="button" onclick={() => diffStore.errorMessage = null} class="text-rose-600 hover:text-rose-800 dark:hover:text-rose-200">
            <X size={13} />
          </button>
        </div>
      {/if}

      {#if result}
        <!-- Summary KPI Cards -->
        <div class="grid grid-cols-4 gap-3 p-4 border-b border-slate-200 dark:border-slate-800 bg-surface-950/60 shrink-0">
          <!-- 1. Tables to Create -->
          <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
            <div>
              <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Tables to Create</span>
              <div class="text-xl font-bold {result.tablesToCreate > 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-900 dark:text-slate-100'} mt-0.5">
                {result.tablesToCreate}
              </div>
            </div>
            <div class="w-8 h-8 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-500">
              <Plus size={16} />
            </div>
          </div>

          <!-- 2. Tables to Alter -->
          <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
            <div>
              <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Tables to Alter</span>
              <div class="text-xl font-bold {result.tablesToAlter > 0 ? 'text-amber-600 dark:text-amber-400' : 'text-slate-900 dark:text-slate-100'} mt-0.5">
                {result.tablesToAlter}
              </div>
            </div>
            <div class="w-8 h-8 rounded-full bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-500">
              <Edit3 size={15} />
            </div>
          </div>

          <!-- 3. Tables to Drop -->
          <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
            <div>
              <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Tables to Drop</span>
              <div class="text-xl font-bold {result.tablesToDrop > 0 ? 'text-rose-600 dark:text-rose-400' : 'text-slate-900 dark:text-slate-100'} mt-0.5">
                {result.tablesToDrop}
              </div>
            </div>
            <div class="w-8 h-8 rounded-full bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-500">
              <Trash2 size={15} />
            </div>
          </div>

          <!-- 4. Synced / Identical -->
          <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
            <div>
              <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Identical Tables</span>
              <div class="text-xl font-bold text-slate-900 dark:text-slate-100 mt-0.5">
                {result.tablesIdentical}
              </div>
            </div>
            <div class="w-8 h-8 rounded-full bg-slate-500/10 border border-slate-500/20 flex items-center justify-center text-slate-400">
              <CheckCheck size={16} />
            </div>
          </div>
        </div>

        <!-- Mode Tab Bar: Visual vs SQL Migration Script -->
        <div class="px-5 bg-surface-900 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between text-xs font-semibold shrink-0">
          <div class="flex items-center gap-4">
            <button
              type="button"
              onclick={() => diffStore.activeTab = 'visual'}
              class="py-2.5 border-b-2 flex items-center gap-1.5 transition-colors cursor-pointer {diffStore.activeTab === 'visual' 
                ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400 font-bold' 
                : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'}"
            >
              <Columns size={13} />
              <span>Visual Schema Diff ({result.tableDiffs.length} tables)</span>
            </button>

            <button
              type="button"
              onclick={() => diffStore.activeTab = 'sql'}
              class="py-2.5 border-b-2 flex items-center gap-1.5 transition-colors cursor-pointer {diffStore.activeTab === 'sql' 
                ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400 font-bold' 
                : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'}"
            >
              <FileCode size={13} />
              <span>Generated Sync DDL Script</span>
            </button>
          </div>

          <!-- Quick Actions on Top Bar -->
          {#if diffStore.activeTab === 'sql'}
            <div class="flex items-center gap-2 py-1.5">
              <button
                type="button"
                onclick={() => copyToClipboard(result?.fullMigrationSql || '')}
                class="flex items-center gap-1 px-2.5 py-1 bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 font-semibold rounded text-xs border border-slate-300/80 dark:border-slate-700 transition-colors"
                title="Copy full migration SQL"
              >
                {#if copiedSql}
                  <Check size={12} class="text-emerald-500" />
                  <span>Copied</span>
                {:else}
                  <Copy size={12} />
                  <span>Copy SQL</span>
                {/if}
              </button>

              <button
                type="button"
                onclick={downloadSqlFile}
                class="flex items-center gap-1 px-2.5 py-1 bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 font-semibold rounded text-xs border border-slate-300/80 dark:border-slate-700 transition-colors"
                title="Save as .sql file"
              >
                <Download size={12} />
                <span>Download .sql</span>
              </button>

              <button
                type="button"
                onclick={() => isConfirmApplyOpen = true}
                class="flex items-center gap-1.5 px-3 py-1 bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white font-bold rounded text-xs shadow-xs transition-colors cursor-pointer"
              >
                <Play size={12} class="fill-current" />
                <span>Apply to Target ({tgtConn?.name})</span>
              </button>
            </div>
          {/if}
        </div>

        <!-- Main Body Tab Content -->
        {#if diffStore.activeTab === 'visual'}
          <div class="flex-1 flex min-h-0 overflow-hidden">
            <!-- Left Pane: Tables List -->
            <div class="w-80 border-r border-slate-200 dark:border-slate-800 flex flex-col bg-surface-950 shrink-0">
              <!-- Filter & Search in sidebar -->
              <div class="p-3 border-b border-slate-200 dark:border-slate-800 space-y-2 shrink-0">
                <div class="relative">
                  <Search size={13} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400" />
                  <input
                    type="text"
                    bind:value={diffStore.searchQuery}
                    placeholder="Search table or column..."
                    class="w-full pl-8 pr-7 py-1 bg-surface-800 border border-slate-200 dark:border-slate-700 rounded-md text-xs text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500"
                  />
                  {#if diffStore.searchQuery}
                    <button
                      type="button"
                      onclick={() => diffStore.searchQuery = ''}
                      class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
                    >
                      <X size={12} />
                    </button>
                  {/if}
                </div>

                <!-- Action Filters -->
                <div class="flex items-center gap-1 flex-wrap text-[11px]">
                  {#each ['all', 'create', 'alter', 'drop', 'identical'] as act}
                    <button
                      type="button"
                      onclick={() => diffStore.filterAction = act as any}
                      class="px-2 py-0.5 rounded font-semibold capitalize transition-colors cursor-pointer border {diffStore.filterAction === act 
                        ? 'bg-indigo-600 text-white border-indigo-600' 
                        : 'bg-surface-800 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:text-slate-900 dark:hover:text-slate-200'}"
                    >
                      {act}
                    </button>
                  {/each}
                </div>
              </div>

              <!-- Table List Items -->
              <div class="flex-1 overflow-auto divide-y divide-slate-200/60 dark:divide-slate-800/40">
                {#each tableDiffs as t (t.tableName)}
                  {@const badge = getActionBadge(t.action)}
                  {@const changedCols = t.columns.filter(c => c.action !== 'identical').length}
                  <button
                    type="button"
                    onclick={() => diffStore.selectedTableDiff = t}
                    class="w-full p-3 text-left transition-colors flex items-center justify-between gap-2 {selectedTable?.tableName === t.tableName ? 'bg-indigo-50/80 dark:bg-indigo-950/40 ring-1 ring-inset ring-indigo-500/40' : 'hover:bg-slate-100/60 dark:hover:bg-surface-900/60'}"
                  >
                    <div class="truncate">
                      <div class="font-mono font-semibold text-xs text-slate-900 dark:text-slate-100 truncate flex items-center gap-1.5">
                        <Table size={13} class="text-slate-400 shrink-0" />
                        <span class="truncate">{t.tableName}</span>
                      </div>
                      <div class="text-[10.5px] text-slate-500 mt-0.5 flex items-center gap-2 font-sans">
                        <span>{t.columns.length} columns</span>
                        {#if changedCols > 0}
                          <span class="text-amber-600 dark:text-amber-400 font-semibold">• {changedCols} changed</span>
                        {/if}
                      </div>
                    </div>

                    <span class="px-2 py-0.5 rounded text-[10px] font-bold border shrink-0 uppercase tracking-wider {badge.color}">
                      {badge.label}
                    </span>
                  </button>
                {/each}
              </div>
            </div>

            <!-- Right Pane: Table Side-by-Side Column Diff Detail -->
            {#if selectedTable}
              {@const badge = getActionBadge(selectedTable.action)}
              <div class="flex-1 flex flex-col min-w-0 bg-surface-950 overflow-hidden">
                <!-- Table Header Bar -->
                <div class="px-5 py-3 border-b border-slate-200 dark:border-slate-800 bg-surface-900/90 flex items-center justify-between shrink-0">
                  <div class="flex items-center gap-3">
                    <span class="font-mono font-bold text-sm text-slate-900 dark:text-slate-100">{selectedTable.tableName}</span>
                    <span class="px-2.5 py-0.5 rounded-full text-xs font-bold border uppercase tracking-wider {badge.color}">
                      {badge.label}
                    </span>
                  </div>

                  <div class="text-xs text-slate-500 font-mono">
                    Rows: Source ({selectedTable.sourceRowCount ?? '—'}) • Target ({selectedTable.targetRowCount ?? '—'})
                  </div>
                </div>

                <!-- Column Comparison Grid -->
                <div class="flex-1 overflow-auto p-4">
                  <table class="w-full text-left text-xs border-collapse font-mono">
                    <thead class="sticky top-0 bg-surface-900 border-b border-slate-200 dark:border-slate-800 text-[11px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-wider select-none z-10 font-sans">
                      <tr>
                        <th class="py-2 px-3 w-44">Column Name</th>
                        <th class="py-2 px-3 w-32">Status</th>
                        <th class="py-2 px-3">Source ({srcConn?.name})</th>
                        <th class="py-2 px-3">Target ({tgtConn?.name})</th>
                        <th class="py-2 px-3">Diff Reason</th>
                      </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-200/80 dark:divide-slate-800/60">
                      {#each selectedTable.columns as col (col.name)}
                        {@const cBadge = getActionBadge(col.action)}
                        <tr class="hover:bg-slate-100/50 dark:hover:bg-surface-900/50 transition-colors {col.action !== 'identical' ? 'bg-amber-500/5' : ''}">
                          <!-- Column Name -->
                          <td class="py-2.5 px-3 font-bold text-slate-900 dark:text-slate-100">
                            {col.name}
                            {#if col.sourcePk || col.targetPk}
                              <span class="ml-1 px-1 py-0.2 rounded text-[9.5px] bg-amber-500/20 text-amber-700 dark:text-amber-300 font-sans font-bold">PK</span>
                            {/if}
                          </td>

                          <!-- Status Badge -->
                          <td class="py-2.5 px-3">
                            <span class="px-2 py-0.5 rounded text-[10px] font-bold border uppercase tracking-wider {cBadge.color}">
                              {cBadge.label}
                            </span>
                          </td>

                          <!-- Source Definition -->
                          <td class="py-2.5 px-3">
                            {#if col.sourceType}
                              <span class="text-indigo-600 dark:text-indigo-400 font-semibold">{col.sourceType}</span>
                              <span class="text-slate-400 text-[11px] ml-1">{col.sourceNullable ? 'NULL' : 'NOT NULL'}</span>
                            {:else}
                              <span class="text-slate-400 italic font-sans">— (Not in source)</span>
                            {/if}
                          </td>

                          <!-- Target Definition -->
                          <td class="py-2.5 px-3">
                            {#if col.targetType}
                              <span class="text-violet-600 dark:text-violet-400 font-semibold">{col.targetType}</span>
                              <span class="text-slate-400 text-[11px] ml-1">{col.targetNullable ? 'NULL' : 'NOT NULL'}</span>
                            {:else}
                              <span class="text-slate-400 italic font-sans">— (Missing in target)</span>
                            {/if}
                          </td>

                          <!-- Diff Reason -->
                          <td class="py-2.5 px-3 text-[11px] text-slate-600 dark:text-slate-400 font-sans">
                            {col.diffReason || 'Identical match'}
                          </td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>

                  <!-- Single Table Sync SQL Snippet -->
                  {#if selectedTable.syncSql}
                    <div class="mt-6 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
                      <div class="px-3 py-2 bg-surface-900 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between text-xs font-bold text-slate-600 dark:text-slate-400 font-sans">
                        <span class="flex items-center gap-1.5">
                          <Code2 size={13} />
                          <span>Generated Sync SQL for {selectedTable.tableName}</span>
                        </span>
                        <button
                          type="button"
                          onclick={() => copyToClipboard(selectedTable.syncSql)}
                          class="px-2 py-0.5 rounded text-[11px] font-medium bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 border border-slate-300/80 dark:border-slate-700 transition-colors"
                        >
                          Copy
                        </button>
                      </div>
                      <pre class="p-3 bg-surface-950 text-slate-800 dark:text-slate-200 text-xs font-mono whitespace-pre-wrap select-text leading-relaxed">{selectedTable.syncSql}</pre>
                    </div>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {:else}
          <!-- Full Generated Migration SQL Tab -->
          <div class="flex-1 flex flex-col bg-surface-950 p-4 overflow-hidden">
            <div class="flex-1 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden flex flex-col bg-surface-900">
              <div class="px-4 py-2 border-b border-slate-200 dark:border-slate-800 bg-surface-950 flex items-center justify-between text-xs text-slate-500 font-mono">
                <span>Target Dialect: {result.targetDriver.toUpperCase()}</span>
                <span>Statements to execute: {result.tablesToCreate + result.tablesToAlter + result.tablesToDrop}</span>
              </div>
              <textarea
                readonly
                value={result.fullMigrationSql}
                class="flex-1 w-full p-4 bg-surface-950 text-slate-900 dark:text-slate-100 font-mono text-xs leading-relaxed resize-none focus:outline-none select-text"
              ></textarea>
            </div>
          </div>
        {/if}
      {:else}
        <!-- Empty State / Loading State -->
        <div class="flex-1 flex flex-col items-center justify-center p-8 text-center text-slate-500 text-xs">
          {#if diffStore.isComparing}
            <RefreshCw size={36} class="text-indigo-500 animate-spin mb-3" />
            <p class="font-bold text-sm text-slate-800 dark:text-slate-200">Comparing schema architectures...</p>
            <p class="text-slate-400 mt-1">Analyzing tables, data types, nullability, and primary keys.</p>
          {:else}
            <GitCompare size={40} class="text-slate-400 mb-3 opacity-60" />
            <p class="font-bold text-sm text-slate-800 dark:text-slate-200">No Schema Comparison Yet</p>
            <p class="text-slate-400 mt-1 max-w-sm">Select a Source and Target database above, then click <strong>Compare</strong> to generate visual diffs and migration scripts.</p>
          {/if}
        </div>
      {/if}

      <!-- Bottom Modal Bar -->
      <div class="px-5 py-2.5 bg-surface-900 border-t border-slate-200 dark:border-slate-800 flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 font-mono shrink-0">
        <span>Source: {srcConn?.name || '—'} ➔ Target: {tgtConn?.name || '—'}</span>
        <button
          type="button"
          onclick={() => diffStore.close()}
          class="px-3 py-1 bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 font-semibold rounded text-xs border border-slate-300/80 dark:border-slate-700 transition-colors"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Confirmation Modal Before Applying Migration to Target -->
{#if isConfirmApplyOpen && result}
  <div class="fixed inset-0 z-60 bg-slate-950/80 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-100">
    <div class="w-full max-w-md bg-surface-950 border border-emerald-500/50 rounded-xl shadow-2xl p-5 animate-in zoom-in-95 duration-150">
      <div class="flex items-center gap-3 text-emerald-600 dark:text-emerald-400 mb-3">
        <div class="p-2 rounded-full bg-emerald-500/15 border border-emerald-500/30">
          <ShieldAlert size={22} />
        </div>
        <div>
          <h3 class="font-bold text-base text-slate-900 dark:text-slate-100">Apply Migration to Target?</h3>
          <span class="text-xs text-slate-500 font-mono">Target: {tgtConn?.name} ({tgtConn?.driver.toUpperCase()})</span>
        </div>
      </div>

      <p class="text-xs text-slate-600 dark:text-slate-300 mb-3 leading-relaxed">
        This will execute the generated DDL synchronization script directly on <strong class="text-slate-900 dark:text-slate-100">{tgtConn?.name}</strong>:
      </p>

      <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg text-xs space-y-1 mb-4 font-mono">
        <div class="text-emerald-600 dark:text-emerald-400 font-semibold">• Create {result.tablesToCreate} new table(s)</div>
        <div class="text-amber-600 dark:text-amber-400 font-semibold">• Alter {result.tablesToAlter} existing table(s)</div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={() => isConfirmApplyOpen = false}
          disabled={diffStore.isApplying}
          class="px-3.5 py-1.5 rounded-lg text-xs font-semibold text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-surface-800 transition-colors cursor-pointer"
        >
          Cancel
        </button>

        <button
          type="button"
          onclick={async () => {
            isConfirmApplyOpen = false;
            await diffStore.applyMigration();
          }}
          disabled={diffStore.isApplying}
          class="px-4 py-1.5 rounded-lg text-xs font-bold bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white shadow-sm transition-colors cursor-pointer disabled:opacity-50 flex items-center gap-1.5"
        >
          {#if diffStore.isApplying}
            <RefreshCw size={12} class="animate-spin" />
            <span>Applying...</span>
          {:else}
            <Play size={12} class="fill-current" />
            <span>Yes, Apply Migration</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
