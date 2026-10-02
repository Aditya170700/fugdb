<script lang="ts">
  import { onMount } from 'svelte';
  import type { QueryResult } from '$lib/api/types';
  import { 
    Check, 
    Copy, 
    Download, 
    Filter, 
    Plus, 
    RotateCcw, 
    Save, 
    Trash2, 
    Undo, 
    Key, 
    Sparkles, 
    MoreHorizontal,
    CornerDownLeft,
    CheckCircle2,
    X,
    Eye
  } from 'lucide-svelte';
  import { mutationStore, TabMutationState } from '$lib/state/mutations.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import MutationReviewDrawer from './MutationReviewDrawer.svelte';

  let { 
    tabId = 'tab-1',
    sql = '',
    tableName = '',
    connectionId = '',
    result, 
    errorMessage 
  }: { 
    tabId?: string;
    sql?: string;
    tableName?: string;
    connectionId?: string;
    result?: QueryResult; 
    errorMessage?: string;
  } = $props();

  const mutationState = $derived(mutationStore.getTabState(tabId));
  const activeConn = $derived(connectionStore.activeConnection);
  const driver = $derived(activeConn?.driver || 'postgres');

  const resolvedTableName = $derived(
    mutationState.resolveTableName(sql, tableName)
  );

  let selectedRowIdx = $state<number | null>(null);
  let filterText = $state('');
  let copied = $state(false);

  // Virtual Scrolling State
  let scrollContainer = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let containerHeight = $state(600);

  function handleScroll(e: Event) {
    const target = e.currentTarget as HTMLElement;
    scrollTop = target.scrollTop;
  }

  const ROW_HEIGHT = 33;
  const BUFFER_COUNT = 15;

  const totalRowCount = $derived(result?.rows.length || 0);
  const startIndex = $derived(
    Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER_COUNT)
  );
  const endIndex = $derived(
    Math.min(totalRowCount, Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + BUFFER_COUNT)
  );

  const visibleRows = $derived.by(() => {
    if (!result?.rows) return [];
    return result.rows.slice(startIndex, endIndex);
  });

  const topPadding = $derived(startIndex * ROW_HEIGHT);
  const bottomPadding = $derived(
    Math.max(0, (totalRowCount - endIndex) * ROW_HEIGHT)
  );

  // Reset scroll on query result change
  $effect(() => {
    if (result) {
      scrollTop = 0;
      if (scrollContainer) {
        scrollContainer.scrollTop = 0;
      }
    }
  });

  // In-cell editing state
  let editingCell = $state<{
    isInserted: boolean;
    rowKey: string;
    tempId?: string;
    colName: string;
    colIdx: number;
    originalVal: any;
    currentVal: string;
  } | null>(null);

  let inputRef = $state<HTMLInputElement | null>(null);

  // Focus and select input on mount of editing
  $effect(() => {
    if (editingCell && inputRef) {
      inputRef.focus();
      inputRef.select();
    }
  });

  // Global Cmd+S / Ctrl+S listener & Container Observer
  onMount(() => {
    if (scrollContainer) {
      containerHeight = scrollContainer.clientHeight;
      const ro = new ResizeObserver((entries) => {
        for (const entry of entries) {
          containerHeight = entry.contentRect.height;
        }
      });
      ro.observe(scrollContainer);
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 's') {
        if (mutationState.hasChanges && !mutationState.isCommitting) {
          e.preventDefault();
          e.stopPropagation();
          mutationState.commit(
            connectionId || activeConn?.id || '',
            driver,
            result?.columns || [],
            resolvedTableName
          );
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });

  function getPrimaryKeyColIdx(): number {
    if (!result) return -1;
    const idx = result.columns.findIndex(c => c.isPrimaryKey || c.name.toLowerCase() === 'id');
    return idx >= 0 ? idx : -1;
  }

  function getRowKey(row: any[], rowIdx: number): string {
    const pkIdx = getPrimaryKeyColIdx();
    if (pkIdx >= 0 && row[pkIdx] !== null && row[pkIdx] !== undefined) {
      return String(row[pkIdx]);
    }
    return String(rowIdx);
  }

  function getRowObject(row: any[]): Record<string, any> {
    if (!result) return {};
    const obj: Record<string, any> = {};
    result.columns.forEach((col, idx) => {
      obj[col.name] = row[idx];
    });
    return obj;
  }

  function getEffectiveCellValue(row: any[], rowIdx: number, colName: string, colIdx: number): any {
    const rowKey = getRowKey(row, rowIdx);
    const modified = mutationState.modifiedCells[rowKey]?.[colName];
    if (modified !== undefined) {
      return modified.current;
    }
    return row[colIdx];
  }

  function isCellModified(row: any[], rowIdx: number, colName: string): boolean {
    const rowKey = getRowKey(row, rowIdx);
    return mutationState.modifiedCells[rowKey]?.[colName] !== undefined;
  }

  function isRowDeleted(row: any[], rowIdx: number): boolean {
    const rowKey = getRowKey(row, rowIdx);
    return mutationState.deletedRowKeys.includes(rowKey);
  }

  // Start cell editing
  function startEditing(
    row: any[], 
    rowIdx: number, 
    colName: string, 
    colIdx: number, 
    isInserted = false, 
    tempId?: string
  ) {
    if (isRowDeleted(row, rowIdx)) return;

    if (isInserted && tempId) {
      const insertedRow = mutationState.insertedRows.find(r => r.tempId === tempId);
      const val = insertedRow?.values[colName];
      editingCell = {
        isInserted: true,
        rowKey: tempId,
        tempId,
        colName,
        colIdx,
        originalVal: null,
        currentVal: val === null || val === undefined ? '' : String(val)
      };
      return;
    }

    const rowKey = getRowKey(row, rowIdx);
    const origVal = row[colIdx];
    const currentVal = getEffectiveCellValue(row, rowIdx, colName, colIdx);

    editingCell = {
      isInserted: false,
      rowKey,
      colName,
      colIdx,
      originalVal: origVal,
      currentVal: currentVal === null || currentVal === undefined ? '' : String(currentVal)
    };
  }

  // Commit current cell edit
  function commitEditing(navigationDirection?: 'next-row' | 'next-col' | 'prev-col') {
    if (!editingCell) return;

    const { isInserted, tempId, rowKey, colName, colIdx, originalVal, currentVal } = editingCell;
    const colType = result?.columns[colIdx]?.dataType?.toLowerCase() || '';
    const isBoolCol = colType.includes('bool') || colType.includes('bit');
    const isNumCol = colType.includes('int') || colType.includes('float') || colType.includes('double') || colType.includes('numeric') || colType.includes('real') || colType.includes('dec') || colType.includes('serial');

    let parsedVal: any = currentVal;
    if (currentVal === '' || currentVal.toUpperCase() === 'NULL') {
      parsedVal = null;
    } else if (isBoolCol) {
      const lower = currentVal.trim().toLowerCase();
      if (['true', 't', '1', 'yes', 'y'].includes(lower)) {
        parsedVal = true;
      } else if (['false', 'f', '0', 'no', 'n'].includes(lower)) {
        parsedVal = false;
      } else {
        parsedVal = currentVal;
      }
    } else if (isNumCol) {
      const trimmed = currentVal.trim();
      if (!isNaN(Number(trimmed)) && trimmed !== '') {
        parsedVal = Number(trimmed);
      } else {
        parsedVal = currentVal;
      }
    } else {
      const lower = currentVal.trim().toLowerCase();
      if (lower === 'true') {
        parsedVal = true;
      } else if (lower === 'false') {
        parsedVal = false;
      } else if (!isNaN(Number(currentVal)) && currentVal.trim() !== '' && !currentVal.includes(':') && !currentVal.includes('-')) {
        parsedVal = Number(currentVal);
      } else {
        parsedVal = currentVal;
      }
    }

    if (isInserted && tempId) {
      mutationState.updateInsertedRowCell(tempId, colName, parsedVal);
    } else {
      mutationState.setCell(rowKey, colName, colIdx, originalVal, parsedVal);
    }

    const savedEditing = { ...editingCell };
    editingCell = null;

    // Handle navigation to next/previous cell
    if (navigationDirection && result) {
      if (navigationDirection === 'next-col' && savedEditing.colIdx < result.columns.length - 1) {
        const nextCol = result.columns[savedEditing.colIdx + 1];
        if (!savedEditing.isInserted) {
          const rowIdx = result.rows.findIndex(r => getRowKey(r, 0) === savedEditing.rowKey);
          if (rowIdx >= 0) {
            startEditing(result.rows[rowIdx], rowIdx, nextCol.name, savedEditing.colIdx + 1);
          }
        }
      } else if (navigationDirection === 'prev-col' && savedEditing.colIdx > 0) {
        const prevCol = result.columns[savedEditing.colIdx - 1];
        if (!savedEditing.isInserted) {
          const rowIdx = result.rows.findIndex(r => getRowKey(r, 0) === savedEditing.rowKey);
          if (rowIdx >= 0) {
            startEditing(result.rows[rowIdx], rowIdx, prevCol.name, savedEditing.colIdx - 1);
          }
        }
      }
    }
  }

  function handleCellKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      commitEditing('next-row');
    } else if (e.key === 'Tab') {
      e.preventDefault();
      commitEditing(e.shiftKey ? 'prev-col' : 'next-col');
    } else if (e.key === 'Escape') {
      e.preventDefault();
      editingCell = null;
    }
  }

  function toggleBooleanCell(row: any[], rowIdx: number, colName: string, colIdx: number) {
    if (isRowDeleted(row, rowIdx)) return;
    const current = getEffectiveCellValue(row, rowIdx, colName, colIdx);
    const rowKey = getRowKey(row, rowIdx);
    const origVal = row[colIdx];
    mutationState.setCell(rowKey, colName, colIdx, origVal, !current);
  }

  function setCellNull(row: any[], rowIdx: number, colName: string, colIdx: number) {
    const rowKey = getRowKey(row, rowIdx);
    const origVal = row[colIdx];
    mutationState.setCell(rowKey, colName, colIdx, origVal, null);
  }

  function handleAddRow() {
    if (!result) return;
    const tempId = mutationState.addInsertedRow(result.columns);
    // Automatically start editing first editable column of new row
    const firstCol = result.columns.find(c => !c.isPrimaryKey && c.name.toLowerCase() !== 'id') || result.columns[0];
    if (firstCol) {
      const colIdx = result.columns.indexOf(firstCol);
      startEditing([], -1, firstCol.name, colIdx, true, tempId);
    }
  }

  function handleDeleteSelectedRow() {
    if (selectedRowIdx === null || !result || !result.rows[selectedRowIdx]) return;
    const row = result.rows[selectedRowIdx];
    const rowKey = getRowKey(row, selectedRowIdx);
    const rowObj = getRowObject(row);
    mutationState.toggleRowDeleted(rowKey, rowObj);
  }

  function copyAsJson() {
    if (!result) return;
    const formatted = result.rows.map((row, rowIdx) => {
      const obj: Record<string, any> = {};
      result.columns.forEach((col, idx) => {
        obj[col.name] = getEffectiveCellValue(row, rowIdx, col.name, idx);
      });
      return obj;
    });
    navigator.clipboard.writeText(JSON.stringify(formatted, null, 2));
    copied = true;
    setTimeout(() => copied = false, 1500);
  }

  function copyAsMarkdown() {
    if (!result) return;
    const header = '| ' + result.columns.map(c => c.name).join(' | ') + ' |';
    const divider = '| ' + result.columns.map(() => '---').join(' | ') + ' |';
    const rows = result.rows.map((r, rowIdx) => '| ' + r.map((_, idx) => {
      const v = getEffectiveCellValue(r, rowIdx, result.columns[idx].name, idx);
      return v === null ? 'NULL' : typeof v === 'object' ? JSON.stringify(v) : v;
    }).join(' | ') + ' |');
    const md = [header, divider, ...rows].join('\n');
    navigator.clipboard.writeText(md);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }
</script>

<div class="w-full h-full flex flex-col bg-surface-950 select-none overflow-hidden relative">
  {#if errorMessage}
    <div class="flex-1 flex flex-col items-center justify-center p-6 text-center">
      <div class="max-w-md bg-rose-500/10 border border-rose-500/30 rounded-xl p-4 text-xs space-y-2 text-left shadow-lg">
        <span class="font-bold text-[11px] uppercase tracking-wider text-rose-600 dark:text-rose-400 block">Query Execution Error</span>
        <p class="font-mono text-rose-950 dark:text-rose-100 text-xs bg-surface-950/70 p-2.5 rounded-lg border border-slate-200 dark:border-slate-800 break-words font-medium">{errorMessage}</p>
      </div>
    </div>
  {:else if !result}
    <div class="flex-1 flex flex-col items-center justify-center text-slate-500 text-sm gap-2">
      <span>No data to display.</span>
      <span class="text-xs text-slate-600">Press <kbd class="px-1.5 py-0.5 rounded bg-surface-800 text-slate-300 font-mono">Cmd+Enter</kbd> to execute query.</span>
    </div>
  {:else}
    <!-- Grid Action Toolbar -->
    <div class="h-9 border-b border-slate-200 dark:border-slate-800/80 bg-surface-900/70 px-3 flex items-center justify-between text-xs shrink-0">
      <div class="flex items-center gap-2">
        <span class="text-slate-600 dark:text-slate-400 font-mono font-medium">{result.rows.length + mutationState.insertedRowCount} rows</span>
        <span class="text-slate-400 dark:text-slate-600">•</span>
        <span class="text-emerald-600 dark:text-emerald-400 font-mono font-semibold">{result.executionTimeMs.toFixed(1)}ms</span>

        <!-- Table name pill -->
        <span class="text-slate-400 dark:text-slate-600">•</span>
        <span class="text-indigo-700 dark:text-indigo-400 font-mono text-[11px] font-bold bg-indigo-500/10 border border-indigo-500/30 px-1.5 py-0.5 rounded">
          {resolvedTableName}
        </span>

        <!-- Row Manipulation Buttons -->
        <div class="ml-2 flex items-center gap-1.5">
          <button
            type="button"
            onclick={handleAddRow}
            class="flex items-center gap-1 bg-surface-900 hover:bg-surface-800 text-slate-800 dark:text-slate-200 hover:text-slate-950 dark:hover:text-white px-2.5 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs"
            title="Insert new row (+ Add Row)"
          >
            <Plus size={12} class="text-emerald-600 dark:text-emerald-400" />
            <span>Add Row</span>
          </button>

          <button
            type="button"
            onclick={handleDeleteSelectedRow}
            disabled={selectedRowIdx === null}
            class="flex items-center gap-1 bg-surface-900 hover:bg-surface-800 text-slate-800 dark:text-slate-200 hover:text-rose-600 dark:hover:text-rose-300 px-2.5 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs disabled:opacity-40"
            title="Mark selected row for deletion"
          >
            <Trash2 size={12} class="text-rose-600 dark:text-rose-400" />
            <span>Delete Row</span>
          </button>
        </div>

        <!-- Staged Changes Notification Badge & Quick Actions -->
        {#if mutationState.hasChanges}
          <div class="ml-3 flex items-center gap-2 bg-amber-500/15 border border-amber-500/50 dark:border-amber-500/40 text-amber-900 dark:text-amber-300 px-2.5 py-0.5 rounded-md text-[11px] shadow-sm font-medium animate-fade-in">
            <Sparkles size={12} class="text-amber-600 dark:text-amber-400 animate-pulse" />
            <span class="font-bold text-amber-900 dark:text-amber-300">{mutationState.totalChangesCount} staged</span>

            <button
              type="button"
              onclick={() => mutationState.isDrawerOpen = !mutationState.isDrawerOpen}
              class="text-indigo-600 dark:text-indigo-300 hover:text-indigo-900 dark:hover:text-white hover:underline flex items-center gap-0.5 font-semibold ml-1 transition-colors"
              title="Toggle SQL Diff Review Drawer"
            >
              <Eye size={11} class="text-indigo-600 dark:text-indigo-300" />
              <span>Review Diff</span>
            </button>

            <button 
              type="button"
              onclick={() => mutationState.commit(connectionId || activeConn?.id || '', driver, result.columns, resolvedTableName)}
              disabled={mutationState.isCommitting}
              class="text-emerald-700 dark:text-emerald-400 hover:text-emerald-950 dark:hover:text-emerald-200 hover:underline flex items-center gap-0.5 font-bold ml-1 transition-colors disabled:opacity-50"
              title="Apply mutations (Cmd+S)"
            >
              <Save size={11} class="text-emerald-600 dark:text-emerald-400" /> 
              <span>Commit (Cmd+S)</span>
            </button>

            <button 
              type="button"
              onclick={() => mutationState.clearAll()}
              class="text-rose-600 dark:text-rose-400 hover:text-rose-900 dark:hover:text-rose-200 hover:underline flex items-center gap-0.5 font-semibold ml-1 transition-colors"
              title="Discard all pending changes"
            >
              <Undo size={11} class="text-rose-600 dark:text-rose-400" /> 
              <span>Discard</span>
            </button>
          </div>
        {/if}
      </div>

      <div class="flex items-center gap-2">
        <button 
          type="button"
          onclick={copyAsMarkdown} 
          class="flex items-center gap-1 text-slate-800 dark:text-slate-200 hover:text-slate-950 dark:hover:text-white bg-surface-900 hover:bg-surface-800 px-2.5 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs"
          title="Copy as Markdown Table"
        >
          {#if copied}
            <Check size={12} class="text-emerald-600 dark:text-emerald-400" />
            <span class="text-emerald-600 dark:text-emerald-400">Copied!</span>
          {:else}
            <Copy size={12} />
            <span>Copy MD</span>
          {/if}
        </button>

        <button 
          type="button"
          onclick={copyAsJson} 
          class="flex items-center gap-1 text-slate-800 dark:text-slate-200 hover:text-slate-950 dark:hover:text-white bg-surface-900 hover:bg-surface-800 px-2.5 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs"
          title="Copy as JSON"
        >
          <Download size={12} />
          <span>JSON</span>
        </button>
      </div>
    </div>

    <!-- Virtual Grid Table Container -->
    <div 
      bind:this={scrollContainer} 
      onscroll={handleScroll}
      class="flex-1 overflow-auto bg-surface-950 relative"
    >
      <table class="w-full text-left border-collapse font-mono text-xs">
        <thead class="bg-surface-900 sticky top-0 z-10 select-none shadow-sm">
          <tr class="border-b border-slate-200 dark:border-slate-800">
            <th class="px-3 py-2 text-[11px] font-semibold text-slate-600 dark:text-slate-400 w-12 text-center border-r border-slate-200 dark:border-slate-800/60">#</th>
            {#each result.columns as col, colIdx (col.name + '_' + colIdx)}
              <th class="px-3 py-2 text-[11px] font-semibold border-r border-slate-200 dark:border-slate-800/60 truncate {col.isPrimaryKey || col.name.toLowerCase() === 'id' ? 'bg-amber-500/5' : ''}">
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-1 truncate">
                    {#if col.isPrimaryKey || col.name.toLowerCase() === 'id'}
                      <Key size={11} class="text-amber-500 dark:text-amber-400 shrink-0" />
                    {/if}
                    <span class="truncate font-bold {col.isPrimaryKey || col.name.toLowerCase() === 'id' ? 'text-amber-600 dark:text-amber-400' : 'text-slate-800 dark:text-slate-100'}">{col.name}</span>
                  </div>
                  <span class="text-[10px] text-slate-500 dark:text-slate-400 font-normal uppercase">{col.dataType}</span>
                </div>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-200 dark:divide-slate-800/60 text-slate-900 dark:text-slate-100 font-normal">
          <!-- Virtual Spacer Top -->
          {#if topPadding > 0}
            <tr style="height: {topPadding}px;" aria-hidden="true">
              <td colspan={result.columns.length + 1} class="p-0 border-0 pointer-events-none"></td>
            </tr>
          {/if}

          <!-- 1. Visible Existing Rows -->
          {#each visibleRows as row, i (getRowKey(row, startIndex + i))}
            {@const rowIdx = startIndex + i}
            {@const rowKey = getRowKey(row, rowIdx)}
            {@const isDeleted = isRowDeleted(row, rowIdx)}
            {@const isSelected = selectedRowIdx === rowIdx}
            <tr 
              onclick={() => selectedRowIdx = rowIdx}
              style="height: {ROW_HEIGHT}px;"
              class="group transition-colors {isDeleted ? 'bg-rose-500/10 text-rose-700 dark:text-rose-300 opacity-65 line-through' : isSelected ? 'bg-indigo-500/15' : 'hover:bg-indigo-500/5'}"
            >
              <!-- Row Index Column -->
              <td class="px-2 py-1.5 text-center border-r border-slate-200 dark:border-slate-800/60 select-none text-[10px] font-mono relative {isDeleted ? 'bg-rose-950/40 text-rose-400 font-bold' : isSelected ? 'bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 font-bold' : 'bg-surface-900/60 text-slate-600 dark:text-slate-400'}">
                {#if isDeleted}
                  <span class="text-rose-600 dark:text-rose-400 font-bold text-[9px] px-1 bg-rose-500/20 rounded">DEL</span>
                {:else}
                  <span class="group-hover:hidden">{rowIdx + 1}</span>
                  <!-- Hover Delete / Undelete button in row number cell -->
                  <button
                    type="button"
                    onclick={(e) => {
                      e.stopPropagation();
                      mutationState.toggleRowDeleted(rowKey, getRowObject(row));
                    }}
                    class="hidden group-hover:inline-flex items-center justify-center text-slate-500 hover:text-rose-600 dark:hover:text-rose-400"
                    title={isDeleted ? 'Undelete row' : 'Delete row'}
                  >
                    <Trash2 size={11} />
                  </button>
                {/if}
              </td>

              <!-- Data Cells -->
              {#each result.columns as col, colIdx (col.name + '_' + colIdx)}
                {@const modified = isCellModified(row, rowIdx, col.name)}
                {@const cellVal = getEffectiveCellValue(row, rowIdx, col.name, colIdx)}
                {@const isEditingThis = editingCell && !editingCell.isInserted && editingCell.rowKey === rowKey && editingCell.colName === col.name}

                <td 
                  ondblclick={() => startEditing(row, rowIdx, col.name, colIdx)}
                  class="px-3 py-1.5 border-r border-slate-200 dark:border-slate-800/60 truncate max-w-[260px] relative text-slate-900 dark:text-slate-100 transition-colors {modified ? 'bg-amber-500/20 dark:bg-amber-500/20 text-amber-950 dark:text-amber-200 font-medium' : ''}"
                  title={modified ? `Modified (Original: ${formatVal(row[colIdx])})` : ''}
                >
                  {#if isEditingThis && editingCell}
                    <!-- Inline Cell Input -->
                    <div class="flex items-center gap-1 -my-1 -mx-2">
                      <input
                        bind:this={inputRef}
                        bind:value={editingCell.currentVal}
                        onkeydown={handleCellKeyDown}
                        onblur={() => commitEditing()}
                        class="w-full bg-surface-900 text-slate-900 dark:text-white font-mono text-xs px-2 py-0.5 rounded border-2 border-indigo-500 focus:outline-none shadow-md placeholder:text-slate-400"
                        placeholder="NULL"
                      />
                      <button
                        type="button"
                        onmousedown={(e) => {
                          e.preventDefault();
                          if (editingCell) editingCell.currentVal = '';
                          commitEditing();
                        }}
                        class="text-[10px] text-slate-700 dark:text-slate-300 hover:text-amber-800 dark:hover:text-amber-300 bg-surface-800 px-1.5 py-0.5 rounded border border-slate-300 dark:border-slate-700 font-bold font-mono"
                        title="Set to NULL"
                      >
                        NULL
                      </button>
                    </div>
                  {:else}
                    <!-- Modified cell corner indicator -->
                    {#if modified}
                      <span class="absolute top-0 right-0 w-2 h-2 border-t-4 border-r-4 border-t-amber-500 border-r-amber-500 border-b-transparent border-l-transparent pointer-events-none"></span>
                    {/if}

                    {#if cellVal === null}
                      <span class="text-slate-400 dark:text-slate-500 italic">NULL</span>
                    {:else if typeof cellVal === 'boolean'}
                      <button 
                        type="button"
                        onclick={(e) => {
                          e.stopPropagation();
                          toggleBooleanCell(row, rowIdx, col.name, colIdx);
                        }}
                        class="cursor-pointer hover:opacity-80 transition-opacity"
                        title="Click to toggle boolean"
                      >
                        <span class={cellVal ? 'text-emerald-700 dark:text-emerald-400 font-bold bg-emerald-500/15 px-1.5 py-0.5 rounded' : 'text-rose-700 dark:text-rose-400 font-bold bg-rose-500/15 px-1.5 py-0.5 rounded'}>
                          {cellVal ? 'TRUE' : 'FALSE'}
                        </span>
                      </button>
                    {:else if typeof cellVal === 'object'}
                      <span class="text-sky-700 dark:text-sky-400">{JSON.stringify(cellVal)}</span>
                    {:else}
                      <span>{cellVal}</span>
                    {/if}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}

          <!-- Virtual Spacer Bottom -->
          {#if bottomPadding > 0}
            <tr style="height: {bottomPadding}px;" aria-hidden="true">
              <td colspan={result.columns.length + 1} class="p-0 border-0 pointer-events-none"></td>
            </tr>
          {/if}

          <!-- 2. Staged Inserted Rows -->
          {#each mutationState.insertedRows as newRow, insertedIdx (newRow.tempId)}
            <tr class="bg-emerald-500/10 text-emerald-950 dark:text-emerald-200 border-l-2 border-l-emerald-500 group">
              <!-- Inserted Row Index Marker -->
              <td class="px-2 py-1.5 text-center border-r border-slate-200 dark:border-slate-800/60 select-none text-[10px] font-mono bg-emerald-950/40 text-emerald-300 font-bold relative">
                <span class="group-hover:hidden">+{insertedIdx + 1}</span>
                <button
                  type="button"
                  onclick={() => mutationState.removeInsertedRow(newRow.tempId)}
                  class="hidden group-hover:inline-flex items-center justify-center text-rose-600 dark:text-rose-400 hover:text-rose-800 dark:hover:text-rose-300"
                  title="Discard this newly added row"
                >
                  <X size={12} />
                </button>
              </td>

              <!-- Inserted Cells -->
              {#each result.columns as col, colIdx (col.name + '_' + colIdx)}
                {@const cellVal = newRow.values[col.name]}
                {@const isEditingThis = editingCell && editingCell.isInserted && editingCell.tempId === newRow.tempId && editingCell.colName === col.name}

                <td 
                  ondblclick={() => startEditing([], -1, col.name, colIdx, true, newRow.tempId)}
                  class="px-3 py-1.5 border-r border-slate-200 dark:border-slate-800/60 truncate max-w-[260px] relative font-medium text-slate-900 dark:text-slate-100"
                >
                  {#if isEditingThis && editingCell}
                    <div class="flex items-center gap-1 -my-1 -mx-2">
                      <input
                        bind:this={inputRef}
                        bind:value={editingCell.currentVal}
                        onkeydown={handleCellKeyDown}
                        onblur={() => commitEditing()}
                        class="w-full bg-surface-900 text-slate-900 dark:text-white font-mono text-xs px-2 py-0.5 rounded border-2 border-emerald-500 focus:outline-none shadow-md placeholder:text-slate-400"
                        placeholder="NULL"
                      />
                    </div>
                  {:else}
                    {#if cellVal === null || cellVal === undefined}
                      <span class="text-slate-400 dark:text-slate-500 italic">NULL</span>
                    {:else if typeof cellVal === 'boolean'}
                      <button 
                        type="button"
                        onclick={() => mutationState.updateInsertedRowCell(newRow.tempId, col.name, !cellVal)}
                        class="cursor-pointer hover:opacity-80"
                      >
                        <span class={cellVal ? 'text-emerald-700 dark:text-emerald-400 font-bold' : 'text-rose-700 dark:text-rose-400 font-bold'}>
                          {cellVal ? 'TRUE' : 'FALSE'}
                        </span>
                      </button>
                    {:else}
                      <span class="text-emerald-800 dark:text-emerald-300 font-medium">{cellVal}</span>
                    {/if}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Bottom Staged Mutation Review Panel / SQL Diff Drawer -->
    {#if mutationState.hasChanges && mutationState.isDrawerOpen}
      <MutationReviewDrawer 
        {mutationState} 
        columns={result.columns} 
        {driver} 
        {connectionId} 
        tableName={resolvedTableName} 
      />
    {/if}
  {/if}
</div>

<script lang="ts" module>
  function formatVal(v: any): string {
    if (v === null || v === undefined) return 'NULL';
    if (typeof v === 'boolean') return v ? 'TRUE' : 'FALSE';
    if (typeof v === 'object') return JSON.stringify(v);
    return String(v);
  }
</script>
