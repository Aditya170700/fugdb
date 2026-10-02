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
    Eye,
    Maximize2,
    Code,
    Clock,
    Binary,
    FileText,
    ArrowUp,
    ArrowDown,
    ArrowUpDown,
    Search,
    FilterX,
    SlidersHorizontal
  } from 'lucide-svelte';
  import { mutationStore, TabMutationState } from '$lib/state/mutations.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { inspectorStore } from '$lib/state/inspector.svelte';
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
  let selectedColIdx = $state<number | null>(null);
  let copied = $state(false);

  // Filter & Multi-Column Sorting States
  let isFilterRowVisible = $state(false);
  let globalQuickSearch = $state('');
  let columnFilters = $state<Record<string, { operator: string; value: string }>>({});
  let sortCriteria = $state<Array<{ columnName: string; colIdx: number; direction: 'asc' | 'desc' }>>([]);

  // Cell Context Menu State
  let contextMenu = $state<{
    x: number;
    y: number;
    row: any[];
    rowIdx: number;
    colName: string;
    colIdx: number;
    isInserted?: boolean;
    tempId?: string;
  } | null>(null);

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

  // Filter & Sort Pipeline
  const processedRows = $derived.by(() => {
    if (!result?.rows) return [];

    // Map each row with its original row index
    let rows = result.rows.map((row, idx) => ({
      originalRowIdx: idx,
      row
    }));

    // 1. Global Quick Search
    if (globalQuickSearch.trim()) {
      const q = globalQuickSearch.trim().toLowerCase();
      rows = rows.filter(({ row, originalRowIdx }) => {
        return result.columns.some((col, colIdx) => {
          const val = getEffectiveCellValue(row, originalRowIdx, col.name, colIdx);
          if (val === null || val === undefined) return false;
          return String(val).toLowerCase().includes(q);
        });
      });
    }

    // 2. Column Filters
    const activeFilters = Object.entries(columnFilters).filter(([_, f]) => {
      if (f.operator === 'is_null' || f.operator === 'not_null') return true;
      return f.value.trim().length > 0;
    });

    if (activeFilters.length > 0) {
      rows = rows.filter(({ row, originalRowIdx }) => {
        return activeFilters.every(([colName, filter]) => {
          const colIdx = result.columns.findIndex(c => c.name === colName);
          if (colIdx === -1) return true;

          const rawVal = getEffectiveCellValue(row, originalRowIdx, colName, colIdx);
          const filterVal = filter.value.trim().toLowerCase();

          if (filter.operator === 'is_null') {
            return rawVal === null || rawVal === undefined;
          }
          if (filter.operator === 'not_null') {
            return rawVal !== null && rawVal !== undefined;
          }

          if (rawVal === null || rawVal === undefined) {
            return false;
          }

          const cellStr = String(rawVal).toLowerCase();

          // Numeric comparisons if both are numbers
          const numCell = Number(rawVal);
          const numFilter = Number(filter.value);
          const areBothNumeric = !isNaN(numCell) && !isNaN(numFilter) && filter.value.trim() !== '';

          switch (filter.operator) {
            case 'contains':
              return cellStr.includes(filterVal);
            case 'equals':
              return areBothNumeric ? numCell === numFilter : cellStr === filterVal;
            case 'neq':
              return areBothNumeric ? numCell !== numFilter : cellStr !== filterVal;
            case 'starts_with':
              return cellStr.startsWith(filterVal);
            case 'ends_with':
              return cellStr.endsWith(filterVal);
            case 'gt':
              return areBothNumeric ? numCell > numFilter : cellStr > filterVal;
            case 'gte':
              return areBothNumeric ? numCell >= numFilter : cellStr >= filterVal;
            case 'lt':
              return areBothNumeric ? numCell < numFilter : cellStr < filterVal;
            case 'lte':
              return areBothNumeric ? numCell <= numFilter : cellStr <= filterVal;
            case 'regex':
              try {
                const re = new RegExp(filter.value, 'i');
                return re.test(String(rawVal));
              } catch {
                return false;
              }
            default:
              return cellStr.includes(filterVal);
          }
        });
      });
    }

    // 3. Multi-Column Sorting
    if (sortCriteria.length > 0) {
      rows = [...rows].sort((a, b) => {
        for (const sort of sortCriteria) {
          const valA = getEffectiveCellValue(a.row, a.originalRowIdx, sort.columnName, sort.colIdx);
          const valB = getEffectiveCellValue(b.row, b.originalRowIdx, sort.columnName, sort.colIdx);

          if (valA === null && valB === null) continue;
          if (valA === null) return 1;
          if (valB === null) return -1;

          let cmp = 0;
          if (typeof valA === 'number' && typeof valB === 'number') {
            cmp = valA - valB;
          } else if (typeof valA === 'boolean' && typeof valB === 'boolean') {
            cmp = (valA === valB) ? 0 : valA ? 1 : -1;
          } else {
            const strA = String(valA).toLowerCase();
            const strB = String(valB).toLowerCase();
            cmp = strA.localeCompare(strB, undefined, { numeric: true });
          }

          if (cmp !== 0) {
            return sort.direction === 'asc' ? cmp : -cmp;
          }
        }
        return 0;
      });
    }

    return rows;
  });

  const totalRowCount = $derived(result?.rows.length || 0);
  const totalFilteredRowCount = $derived(processedRows.length);
  const activeFilterCount = $derived(
    Object.keys(columnFilters).length + (globalQuickSearch.trim() ? 1 : 0)
  );

  const startIndex = $derived(
    Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER_COUNT)
  );
  const endIndex = $derived(
    Math.min(totalFilteredRowCount, Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + BUFFER_COUNT)
  );

  const visibleRows = $derived.by(() => {
    return processedRows.slice(startIndex, endIndex);
  });

  const topPadding = $derived(startIndex * ROW_HEIGHT);
  const bottomPadding = $derived(
    Math.max(0, (totalFilteredRowCount - endIndex) * ROW_HEIGHT)
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

  function handleColumnSortClick(colName: string, colIdx: number, e: MouseEvent) {
    const isMulti = e.shiftKey || e.metaKey || e.ctrlKey;
    const existingIdx = sortCriteria.findIndex(s => s.columnName === colName);

    if (!isMulti) {
      if (existingIdx >= 0 && sortCriteria.length === 1) {
        if (sortCriteria[0].direction === 'asc') {
          sortCriteria = [{ columnName: colName, colIdx, direction: 'desc' }];
        } else {
          sortCriteria = [];
        }
      } else {
        sortCriteria = [{ columnName: colName, colIdx, direction: 'asc' }];
      }
    } else {
      if (existingIdx >= 0) {
        const current = sortCriteria[existingIdx];
        if (current.direction === 'asc') {
          sortCriteria = sortCriteria.map((s, idx) => idx === existingIdx ? { ...s, direction: 'desc' } : s);
        } else {
          sortCriteria = sortCriteria.filter((_, idx) => idx !== existingIdx);
        }
      } else {
        sortCriteria = [...sortCriteria, { columnName: colName, colIdx, direction: 'asc' }];
      }
    }
  }

  function getSortInfo(colName: string): { direction: 'asc' | 'desc'; index: number } | null {
    const idx = sortCriteria.findIndex(s => s.columnName === colName);
    if (idx === -1) return null;
    return { direction: sortCriteria[idx].direction, index: idx + 1 };
  }

  function setColumnFilter(colName: string, colIdx: number, operator: string, value: string) {
    if (!value && operator !== 'is_null' && operator !== 'not_null') {
      const next = { ...columnFilters };
      delete next[colName];
      columnFilters = next;
    } else {
      columnFilters = {
        ...columnFilters,
        [colName]: { operator, value }
      };
    }
  }

  function clearAllFilters() {
    columnFilters = {};
    globalQuickSearch = '';
  }

  function clearSort() {
    sortCriteria = [];
  }

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
      } else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'i') {
        if (selectedRowIdx !== null && selectedColIdx !== null && result && result.rows[selectedRowIdx]) {
          e.preventDefault();
          const row = result.rows[selectedRowIdx];
          const col = result.columns[selectedColIdx];
          openCellInspector(row, selectedRowIdx, col.name, selectedColIdx);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });

  function openCellInspector(
    row: any[], 
    rowIdx: number, 
    colName: string, 
    colIdx: number, 
    isInserted = false, 
    tempId?: string,
    preferredTab: 'auto' | 'json' | 'datetime' | 'binary' | 'text' = 'auto'
  ) {
    if (isRowDeleted(row, rowIdx)) return;

    let cellVal: any;
    let rowKey: string;
    let origVal: any;

    if (isInserted && tempId) {
      const insertedRow = mutationState.insertedRows.find(r => r.tempId === tempId);
      cellVal = insertedRow?.values[colName];
      rowKey = tempId;
      origVal = null;
    } else {
      rowKey = getRowKey(row, rowIdx);
      origVal = row[colIdx];
      cellVal = getEffectiveCellValue(row, rowIdx, colName, colIdx);
    }

    const col = result?.columns[colIdx];

    inspectorStore.open({
      columnName: colName,
      dataType: col?.dataType,
      tableName: resolvedTableName,
      rowKey,
      colIdx,
      isInserted,
      tempId,
      value: cellVal,
      onApply: (newVal) => {
        if (isInserted && tempId) {
          mutationState.updateInsertedRowCell(tempId, colName, newVal);
        } else {
          mutationState.setCell(rowKey, colName, colIdx, origVal, newVal);
        }
      }
    }, preferredTab);
  }

  function handleCellContextMenu(
    e: MouseEvent,
    row: any[],
    rowIdx: number,
    colName: string,
    colIdx: number,
    isInserted = false,
    tempId?: string
  ) {
    e.preventDefault();
    selectedRowIdx = rowIdx;
    selectedColIdx = colIdx;
    contextMenu = {
      x: Math.min(window.innerWidth - 220, e.clientX),
      y: Math.min(window.innerHeight - 260, e.clientY),
      row,
      rowIdx,
      colName,
      colIdx,
      isInserted,
      tempId
    };
  }

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
    <div class="h-9 border-b border-slate-200 dark:border-slate-800/80 bg-surface-900/70 px-3 flex items-center justify-between text-xs shrink-0 gap-2">
      <div class="flex items-center gap-2 truncate">
        <span class="text-slate-600 dark:text-slate-400 font-mono font-medium">
          {#if activeFilterCount > 0}
            <span class="text-indigo-600 dark:text-indigo-400 font-bold">{totalFilteredRowCount}</span>/{result.rows.length} rows
          {:else}
            {result.rows.length + mutationState.insertedRowCount} rows
          {/if}
        </span>
        <span class="text-slate-400 dark:text-slate-600">•</span>
        <span class="text-emerald-600 dark:text-emerald-400 font-mono font-semibold">{result.executionTimeMs.toFixed(1)}ms</span>

        <!-- Table name pill -->
        <span class="text-slate-400 dark:text-slate-600">•</span>
        <span class="text-indigo-700 dark:text-indigo-400 font-mono text-[11px] font-bold bg-indigo-500/10 border border-indigo-500/30 px-1.5 py-0.5 rounded">
          {resolvedTableName}
        </span>

        <!-- Quick Filter Toggle Button -->
        <button
          type="button"
          onclick={() => isFilterRowVisible = !isFilterRowVisible}
          class="flex items-center gap-1.5 px-2 py-1 rounded text-[11px] font-semibold transition-colors border shadow-xs cursor-pointer ml-1 {isFilterRowVisible || activeFilterCount > 0 ? 'bg-indigo-600/15 border-indigo-500/40 text-indigo-600 dark:text-indigo-300' : 'bg-surface-900 border-slate-200 hover:border-slate-300 dark:border-slate-700/80 text-slate-700 dark:text-slate-300'}"
          title="Toggle Column Filters Row"
        >
          <Filter size={12} class={activeFilterCount > 0 ? 'text-indigo-500 fill-indigo-500/20' : ''} />
          <span>Filters</span>
          {#if activeFilterCount > 0}
            <span class="px-1.5 py-0.2 rounded-full bg-indigo-600 text-white text-[9px] font-bold">
              {activeFilterCount}
            </span>
          {/if}
        </button>

        <!-- Active Sort Reset Button -->
        {#if sortCriteria.length > 0}
          <button
            type="button"
            onclick={clearSort}
            class="flex items-center gap-1 px-2 py-0.5 bg-surface-950 border border-slate-700 rounded text-[10.5px] text-slate-300 hover:text-rose-400 cursor-pointer"
            title="Clear active multi-column sort"
          >
            <ArrowUpDown size={11} class="text-indigo-400" />
            <span>Sorted ({sortCriteria.length})</span>
            <X size={10} />
          </button>
        {/if}

        <!-- Active Filter Reset Button -->
        {#if activeFilterCount > 0}
          <button
            type="button"
            onclick={clearAllFilters}
            class="flex items-center gap-1 px-2 py-0.5 bg-rose-500/10 border border-rose-500/30 rounded text-[10.5px] text-rose-500 hover:text-rose-400 cursor-pointer"
            title="Clear all active column filters"
          >
            <FilterX size={11} />
            <span>Reset Filters</span>
          </button>
        {/if}

        <!-- Row Manipulation Buttons -->
        <div class="ml-1 flex items-center gap-1.5">
          <button
            type="button"
            onclick={handleAddRow}
            class="flex items-center gap-1 bg-surface-900 hover:bg-surface-800 text-slate-800 dark:text-slate-200 hover:text-slate-950 dark:hover:text-white px-2 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs"
            title="Insert new row (+ Add Row)"
          >
            <Plus size={12} class="text-emerald-600 dark:text-emerald-400" />
            <span>Add Row</span>
          </button>

          <button
            type="button"
            onclick={handleDeleteSelectedRow}
            disabled={selectedRowIdx === null}
            class="flex items-center gap-1 bg-surface-900 hover:bg-surface-800 text-slate-800 dark:text-slate-200 hover:text-rose-600 dark:hover:text-rose-300 px-2 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs disabled:opacity-40"
            title="Mark selected row for deletion"
          >
            <Trash2 size={12} class="text-rose-600 dark:text-rose-400" />
            <span>Delete</span>
          </button>
        </div>

        <!-- Staged Changes Notification Badge & Quick Actions -->
        {#if mutationState.hasChanges}
          <div class="ml-2 flex items-center gap-2 bg-amber-500/15 border border-amber-500/50 dark:border-amber-500/40 text-amber-900 dark:text-amber-300 px-2.5 py-0.5 rounded-md text-[11px] shadow-sm font-medium animate-fade-in">
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
        <!-- Global Quick Search Input -->
        <div class="relative w-36 sm:w-48">
          <Search size={11} class="absolute left-2.5 top-2 text-slate-400" />
          <input
            type="text"
            bind:value={globalQuickSearch}
            placeholder="Quick search grid..."
            class="w-full bg-surface-950 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-800 rounded-md pl-7 pr-5 py-0.5 text-[11px] focus:outline-none focus:border-indigo-500 placeholder:text-slate-500"
          />
          {#if globalQuickSearch}
            <button
              type="button"
              onclick={() => globalQuickSearch = ''}
              class="absolute right-1.5 top-1 text-slate-400 hover:text-slate-200"
            >
              <X size={10} />
            </button>
          {/if}
        </div>

        <button 
          type="button"
          onclick={copyAsMarkdown} 
          class="flex items-center gap-1 text-slate-800 dark:text-slate-200 hover:text-slate-950 dark:hover:text-white bg-surface-900 hover:bg-surface-800 px-2 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs"
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
          class="flex items-center gap-1 text-slate-800 dark:text-slate-200 hover:text-slate-950 dark:hover:text-white bg-surface-900 hover:bg-surface-800 px-2 py-1 rounded text-[11px] font-semibold transition-colors border border-slate-200 hover:border-slate-300 dark:border-slate-700/80 shadow-xs"
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
          <!-- 1. Column Header Row (Sortable) -->
          <tr class="border-b border-slate-200 dark:border-slate-800">
            <th class="px-3 py-2 text-[11px] font-semibold text-slate-600 dark:text-slate-400 w-12 text-center border-r border-slate-200 dark:border-slate-800/60">#</th>
            {#each result.columns as col, colIdx (col.name + '_' + colIdx)}
              {@const sortInfo = getSortInfo(col.name)}
              <th 
                onclick={(e) => handleColumnSortClick(col.name, colIdx, e)}
                class="px-3 py-2 text-[11px] font-semibold border-r border-slate-200 dark:border-slate-800/60 truncate cursor-pointer hover:bg-surface-800 transition-colors group/header {col.isPrimaryKey || col.name.toLowerCase() === 'id' ? 'bg-amber-500/5' : ''} {sortInfo ? 'bg-indigo-500/10' : ''}"
                title="Click to sort (Shift+Click for multi-column sort)"
              >
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-1 truncate">
                    {#if col.isPrimaryKey || col.name.toLowerCase() === 'id'}
                      <Key size={11} class="text-amber-500 dark:text-amber-400 shrink-0" />
                    {/if}
                    <span class="truncate font-bold {col.isPrimaryKey || col.name.toLowerCase() === 'id' ? 'text-amber-600 dark:text-amber-400' : 'text-slate-800 dark:text-slate-100'}">{col.name}</span>
                  </div>

                  <div class="flex items-center gap-1.5 shrink-0">
                    <span class="text-[10px] text-slate-500 dark:text-slate-400 font-normal uppercase">{col.dataType}</span>
                    
                    {#if sortInfo}
                      <span class="flex items-center text-indigo-600 dark:text-indigo-400 font-bold font-mono text-[10px] bg-indigo-500/15 px-1 py-0.2 rounded border border-indigo-500/30">
                        {#if sortInfo.direction === 'asc'}
                          <ArrowUp size={11} />
                        {:else}
                          <ArrowDown size={11} />
                        {/if}
                        {#if sortCriteria.length > 1}
                          <span>{sortInfo.index}</span>
                        {/if}
                      </span>
                    {:else}
                      <ArrowUpDown size={11} class="opacity-0 group-hover/header:opacity-40 text-slate-400 transition-opacity" />
                    {/if}
                  </div>
                </div>
              </th>
            {/each}
          </tr>

          <!-- 2. Column Instant Filter Row (Toggleable) -->
          {#if isFilterRowVisible}
            <tr class="bg-surface-950 border-b border-slate-200 dark:border-slate-800 text-[11px] animate-in fade-in duration-100">
              <th class="px-2 py-1 text-center border-r border-slate-200 dark:border-slate-800 font-normal text-slate-500">
                <Filter size={11} class="mx-auto text-indigo-400" />
              </th>
              {#each result.columns as col, colIdx (col.name + '_filter_' + colIdx)}
                {@const currentFilter = columnFilters[col.name]}
                {@const colType = col.dataType.toLowerCase()}
                {@const isNum = colType.includes('int') || colType.includes('float') || colType.includes('double') || colType.includes('numeric') || colType.includes('real') || colType.includes('dec')}
                <th class="p-1 border-r border-slate-200 dark:border-slate-800 font-normal">
                  <div class="flex items-center gap-1 bg-surface-900 border {currentFilter ? 'border-indigo-500/60 bg-indigo-500/5' : 'border-slate-200 dark:border-slate-800'} rounded px-1.5 py-0.5">
                    <!-- Filter Operator Select -->
                    <select
                      value={currentFilter?.operator || (isNum ? 'equals' : 'contains')}
                      onchange={(e) => setColumnFilter(col.name, colIdx, (e.currentTarget as HTMLSelectElement).value, currentFilter?.value || '')}
                      class="bg-transparent text-[10px] text-indigo-600 dark:text-indigo-400 font-bold focus:outline-none shrink-0 cursor-pointer"
                    >
                      <option value="contains">contains</option>
                      <option value="equals">=</option>
                      <option value="neq">≠</option>
                      <option value="starts_with">starts</option>
                      <option value="ends_with">ends</option>
                      <option value="gt">&gt;</option>
                      <option value="gte">≥</option>
                      <option value="lt">&lt;</option>
                      <option value="lte">≤</option>
                      <option value="is_null">is null</option>
                      <option value="not_null">not null</option>
                      <option value="regex">regex</option>
                    </select>

                    {#if currentFilter?.operator !== 'is_null' && currentFilter?.operator !== 'not_null'}
                      <input
                        type="text"
                        value={currentFilter?.value || ''}
                        oninput={(e) => setColumnFilter(col.name, colIdx, currentFilter?.operator || (isNum ? 'equals' : 'contains'), (e.currentTarget as HTMLInputElement).value)}
                        placeholder="filter..."
                        class="w-full bg-transparent text-slate-800 dark:text-slate-200 text-[11px] focus:outline-none placeholder:text-slate-500"
                      />
                    {/if}

                    {#if currentFilter}
                      <button
                        type="button"
                        onclick={() => {
                          const next = { ...columnFilters };
                          delete next[col.name];
                          columnFilters = next;
                        }}
                        class="text-slate-400 hover:text-slate-200 p-0.5"
                        title="Clear filter"
                      >
                        <X size={10} />
                      </button>
                    {/if}
                  </div>
                </th>
              {/each}
            </tr>
          {/if}
        </thead>
        <tbody class="divide-y divide-slate-200 dark:divide-slate-800/60 text-slate-900 dark:text-slate-100 font-normal">
          <!-- Virtual Spacer Top -->
          {#if topPadding > 0}
            <tr style="height: {topPadding}px;" aria-hidden="true">
              <td colspan={result.columns.length + 1} class="p-0 border-0 pointer-events-none"></td>
            </tr>
          {/if}

          <!-- 1. Visible Existing Rows (Filtered & Sorted) -->
          {#each visibleRows as { row, originalRowIdx }, i (originalRowIdx)}
            {@const rowKey = getRowKey(row, originalRowIdx)}
            {@const isDeleted = isRowDeleted(row, originalRowIdx)}
            {@const isSelected = selectedRowIdx === originalRowIdx}
            <tr 
              onclick={() => selectedRowIdx = originalRowIdx}
              style="height: {ROW_HEIGHT}px;"
              class="group transition-colors {isDeleted ? 'bg-rose-500/10 text-rose-700 dark:text-rose-300 opacity-65 line-through' : isSelected ? 'bg-indigo-500/15' : 'hover:bg-indigo-500/5'}"
            >
              <!-- Row Index Column -->
              <td class="px-2 py-1.5 text-center border-r border-slate-200 dark:border-slate-800/60 select-none text-[10px] font-mono relative {isDeleted ? 'bg-rose-950/40 text-rose-400 font-bold' : isSelected ? 'bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 font-bold' : 'bg-surface-900/60 text-slate-600 dark:text-slate-400'}">
                {#if isDeleted}
                  <span class="text-rose-600 dark:text-rose-400 font-bold text-[9px] px-1 bg-rose-500/20 rounded">DEL</span>
                {:else}
                  <span class="group-hover:hidden">{originalRowIdx + 1}</span>
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
                {@const modified = isCellModified(row, originalRowIdx, col.name)}
                {@const cellVal = getEffectiveCellValue(row, originalRowIdx, col.name, colIdx)}
                {@const isEditingThis = editingCell && !editingCell.isInserted && editingCell.rowKey === rowKey && editingCell.colName === col.name}
                {@const isCellSelected = isSelected && selectedColIdx === colIdx}

                <td 
                  onclick={() => { selectedRowIdx = originalRowIdx; selectedColIdx = colIdx; }}
                  ondblclick={() => startEditing(row, originalRowIdx, col.name, colIdx)}
                  oncontextmenu={(e) => handleCellContextMenu(e, row, originalRowIdx, col.name, colIdx)}
                  class="px-3 py-1.5 border-r border-slate-200 dark:border-slate-800/60 truncate max-w-[260px] relative text-slate-900 dark:text-slate-100 transition-colors group/cell {modified ? 'bg-amber-500/20 dark:bg-amber-500/20 text-amber-950 dark:text-amber-200 font-medium' : ''} {isCellSelected ? 'ring-1 ring-inset ring-indigo-500 bg-indigo-500/10' : ''}"
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
                          toggleBooleanCell(row, originalRowIdx, col.name, colIdx);
                        }}
                        class="cursor-pointer hover:opacity-80 transition-opacity"
                        title="Click to toggle boolean"
                      >
                        <span class={cellVal ? 'text-emerald-700 dark:text-emerald-400 font-bold bg-emerald-500/15 px-1.5 py-0.5 rounded' : 'text-rose-700 dark:text-rose-400 font-bold bg-rose-500/15 px-1.5 py-0.5 rounded'}>
                          {cellVal ? 'TRUE' : 'FALSE'}
                        </span>
                      </button>
                    {:else if typeof cellVal === 'object'}
                      <button
                        type="button"
                        onclick={(e) => {
                          e.stopPropagation();
                          openCellInspector(row, originalRowIdx, col.name, colIdx, false, undefined, 'json');
                        }}
                        class="inline-flex items-center gap-1 text-sky-700 dark:text-sky-400 hover:underline cursor-pointer truncate max-w-full text-left"
                        title="Click to open in JSON Inspector"
                      >
                        <span class="px-1 py-0.2 rounded bg-sky-500/10 text-sky-400 font-bold text-[10px] border border-sky-500/20">JSON</span>
                        <span class="truncate">{JSON.stringify(cellVal)}</span>
                      </button>
                    {:else}
                      <span>{cellVal}</span>
                    {/if}

                    <!-- Hover Quick Inspect Button -->
                    <button
                      type="button"
                      onclick={(e) => {
                        e.stopPropagation();
                        openCellInspector(row, originalRowIdx, col.name, colIdx);
                      }}
                      class="hidden group-hover/cell:flex absolute right-1 top-1.5 p-0.5 rounded bg-surface-800 text-slate-400 hover:text-indigo-400 shadow-xs border border-slate-700/80 items-center justify-center cursor-pointer z-1"
                      title="Inspect Cell Value (JSON / Date / UUID / Text)"
                    >
                      <Maximize2 size={10} />
                    </button>
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
                  oncontextmenu={(e) => handleCellContextMenu(e, [], -1, col.name, colIdx, true, newRow.tempId)}
                  class="px-3 py-1.5 border-r border-slate-200 dark:border-slate-800/60 truncate max-w-[260px] relative font-medium text-slate-900 dark:text-slate-100 group/cell"
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

                    <!-- Hover Quick Inspect Button -->
                    <button
                      type="button"
                      onclick={(e) => {
                        e.stopPropagation();
                        openCellInspector([], -1, col.name, colIdx, true, newRow.tempId);
                      }}
                      class="hidden group-hover/cell:flex absolute right-1 top-1.5 p-0.5 rounded bg-surface-800 text-slate-400 hover:text-indigo-400 shadow-xs border border-slate-700/80 items-center justify-center cursor-pointer z-1"
                      title="Inspect Cell Value"
                    >
                      <Maximize2 size={10} />
                    </button>
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Floating Right-Click Context Menu -->
    {#if contextMenu}
      <div 
        class="fixed inset-0 z-50 select-none" 
        onclick={() => contextMenu = null}
        oncontextmenu={(e) => { e.preventDefault(); contextMenu = null; }}
        role="presentation"
      ></div>

      <div 
        style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
        class="fixed z-50 w-56 bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-xl shadow-2xl p-1 text-xs space-y-0.5 font-sans animate-in fade-in-50 zoom-in-95 duration-100 select-none text-slate-800 dark:text-slate-200"
      >
        <div class="px-2.5 py-1.5 text-[10px] text-slate-400 font-mono border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
          <span class="font-bold truncate">{contextMenu.colName}</span>
          <span>{contextMenu.isInserted ? 'Inserted' : `Row #${contextMenu.rowIdx + 1}`}</span>
        </div>

        <button
          type="button"
          onclick={() => {
            const { row, rowIdx, colName, colIdx, isInserted, tempId } = contextMenu!;
            contextMenu = null;
            openCellInspector(row, rowIdx, colName, colIdx, isInserted, tempId);
          }}
          class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-indigo-600 hover:text-white rounded-lg transition-colors text-left cursor-pointer font-semibold text-indigo-600 dark:text-indigo-400"
        >
          <Maximize2 size={13} />
          <span>Inspect Cell Value...</span>
          <span class="text-[10px] opacity-75 font-mono ml-auto">⌘I</span>
        </button>

        <div class="border-t border-slate-200 dark:border-slate-800 my-0.5"></div>

        <button
          type="button"
          onclick={() => {
            const { row, rowIdx, colName, colIdx, isInserted, tempId } = contextMenu!;
            contextMenu = null;
            startEditing(row, rowIdx, colName, colIdx, isInserted, tempId);
          }}
          class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-surface-800 rounded-lg transition-colors text-left cursor-pointer"
        >
          <Sparkles size={13} class="text-slate-400" />
          <span>Edit In-Place (Double-Click)</span>
        </button>

        <button
          type="button"
          onclick={() => {
            const { row, rowIdx, colName, colIdx, isInserted, tempId } = contextMenu!;
            contextMenu = null;
            if (isInserted && tempId) {
              mutationState.updateInsertedRowCell(tempId, colName, null);
            } else {
              setCellNull(row, rowIdx, colName, colIdx);
            }
          }}
          class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-surface-800 rounded-lg transition-colors text-left cursor-pointer"
        >
          <Undo size={13} class="text-amber-500" />
          <span>Set value to NULL</span>
        </button>

        <button
          type="button"
          onclick={() => {
            const { row, rowIdx, colName, colIdx, isInserted, tempId } = contextMenu!;
            const val = isInserted && tempId 
              ? mutationState.insertedRows.find(r => r.tempId === tempId)?.values[colName]
              : getEffectiveCellValue(row, rowIdx, colName, colIdx);
            navigator.clipboard.writeText(val === null ? 'NULL' : typeof val === 'object' ? JSON.stringify(val) : String(val));
            contextMenu = null;
          }}
          class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-surface-800 rounded-lg transition-colors text-left cursor-pointer"
        >
          <Copy size={13} class="text-slate-400" />
          <span>Copy Value</span>
        </button>

        {#if !contextMenu.isInserted}
          <div class="border-t border-slate-200 dark:border-slate-800 my-0.5"></div>

          <button
            type="button"
            onclick={() => {
              const { row, rowIdx } = contextMenu!;
              contextMenu = null;
              mutationState.toggleRowDeleted(getRowKey(row, rowIdx), getRowObject(row));
            }}
            class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-rose-500/10 hover:text-rose-400 text-rose-500 rounded-lg transition-colors text-left cursor-pointer font-semibold"
          >
            <Trash2 size={13} />
            <span>Delete Row</span>
          </button>
        {/if}
      </div>
    {/if}

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
