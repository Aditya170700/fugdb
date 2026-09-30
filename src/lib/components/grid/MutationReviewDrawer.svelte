<script lang="ts">
  import { 
    Check, 
    ChevronDown, 
    ChevronUp, 
    Copy, 
    Database, 
    FileCode, 
    Layers, 
    Play, 
    RotateCcw, 
    Save, 
    Sparkles, 
    Trash2, 
    X, 
    AlertCircle,
    CheckCircle2,
    ArrowRight
  } from 'lucide-svelte';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { ColumnMetadata, DriverType } from '$lib/api/types';
  import { TabMutationState, type VisualDiffItem } from '$lib/state/mutations.svelte';

  let { 
    mutationState, 
    columns = [], 
    driver = 'postgres', 
    connectionId = '', 
    tableName = 'table_name' 
  }: {
    mutationState: TabMutationState;
    columns: ColumnMetadata[];
    driver: DriverType;
    connectionId: string;
    tableName: string;
  } = $props();

  let isCopied = $state(false);
  let isExpanded = $state(true);

  const generatedSql = $derived(
    mutationState.generateSql(driver, columns, tableName)
  );

  const visualDiffs = $derived(
    mutationState.generateVisualDiffList(tableName)
  );

  function copySql() {
    navigator.clipboard.writeText(generatedSql);
    isCopied = true;
    setTimeout(() => isCopied = false, 1500);
  }

  async function handleCommit() {
    await mutationState.commit(connectionId, driver, columns, tableName);
  }

  function formatDisplayVal(val: any): string {
    if (val === null || val === undefined) return 'NULL';
    if (typeof val === 'boolean') return val ? 'TRUE' : 'FALSE';
    if (typeof val === 'object') return JSON.stringify(val);
    return String(val);
  }
</script>

<div 
  transition:fly={{ y: 160, duration: 240, easing: cubicOut }}
  class="bg-surface-950 flex flex-col transition-all duration-200 z-20 absolute inset-x-0 bottom-0 shadow-2xl {isExpanded ? 'top-9 h-auto' : 'h-10 border-t border-slate-200 dark:border-slate-800'}"
>
  <!-- Drawer Header Bar -->
  <div class="h-10 px-3 bg-surface-900/80 border-b border-slate-200 dark:border-slate-800/60 flex items-center justify-between shrink-0 select-none">
    <div class="flex items-center gap-2.5">
      <!-- Collapse / Expand Toggle Button -->
      <button 
        type="button"
        onclick={() => isExpanded = !isExpanded}
        class="text-slate-600 dark:text-slate-400 hover:text-slate-950 dark:hover:text-white p-1 rounded hover:bg-surface-800 transition-colors"
        title={isExpanded ? 'Collapse Drawer' : 'Expand Drawer'}
      >
        {#if isExpanded}
          <ChevronDown size={14} />
        {:else}
          <ChevronUp size={14} />
        {/if}
      </button>

      <!-- Badge & Title -->
      <div class="flex items-center gap-2">
        <div class="flex items-center gap-1.5 font-bold text-xs text-amber-600 dark:text-amber-400">
          <Sparkles size={14} class="animate-pulse" />
          <span>Staged Changes</span>
        </div>

        <span class="text-slate-400 dark:text-slate-600 font-mono text-[11px]">•</span>

        <!-- Change breakdown chips -->
        <div class="flex items-center gap-1.5 text-[11px] font-mono">
          {#if mutationState.modifiedRowCount > 0}
            <span class="bg-amber-500/15 text-amber-800 dark:text-amber-300 border border-amber-500/40 px-1.5 py-0.5 rounded font-medium">
              {mutationState.modifiedRowCount} modified
            </span>
          {/if}

          {#if mutationState.insertedRowCount > 0}
            <span class="bg-emerald-500/15 text-emerald-800 dark:text-emerald-300 border border-emerald-500/40 px-1.5 py-0.5 rounded font-medium">
              {mutationState.insertedRowCount} inserted
            </span>
          {/if}

          {#if mutationState.deletedRowCount > 0}
            <span class="bg-rose-500/15 text-rose-800 dark:text-rose-300 border border-rose-500/40 px-1.5 py-0.5 rounded font-medium">
              {mutationState.deletedRowCount} deleted
            </span>
          {/if}
        </div>
      </div>

      <!-- View Switcher Tabs (Diff vs SQL) -->
      {#if isExpanded}
        <div class="ml-4 flex items-center bg-surface-800/80 p-0.5 rounded-lg border border-slate-200 dark:border-slate-700/60 text-xs">
          <button
            type="button"
            onclick={() => mutationState.activeDrawerTab = 'diff'}
            class="px-2.5 py-1 rounded text-[11px] font-medium transition-colors flex items-center gap-1 {mutationState.activeDrawerTab === 'diff' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-600 dark:text-slate-400 hover:text-slate-950 dark:hover:text-slate-200'}"
          >
            <Layers size={11} />
            <span>Visual Diff</span>
          </button>
          <button
            type="button"
            onclick={() => mutationState.activeDrawerTab = 'sql'}
            class="px-2.5 py-1 rounded text-[11px] font-medium transition-colors flex items-center gap-1 {mutationState.activeDrawerTab === 'sql' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-600 dark:text-slate-400 hover:text-slate-950 dark:hover:text-slate-200'}"
          >
            <FileCode size={11} />
            <span>SQL Script</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Actions Area -->
    <div class="flex items-center gap-2">
      <!-- Discard All Button -->
      <button 
        type="button"
        onclick={() => mutationState.clearAll()}
        disabled={mutationState.isCommitting}
        class="flex items-center gap-1 text-slate-700 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/10 px-2.5 py-1 rounded text-[11px] font-medium transition-colors border border-transparent hover:border-rose-500/20 disabled:opacity-50"
        title="Discard all pending changes"
      >
        <RotateCcw size={12} class="text-rose-600 dark:text-rose-400" />
        <span>Discard All</span>
      </button>

      <!-- Apply / Commit (Cmd+S) Button -->
      <button 
        type="button"
        onclick={handleCommit}
        disabled={mutationState.isCommitting || !mutationState.hasChanges}
        class="flex items-center gap-1.5 bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white font-bold px-3.5 py-1 rounded-md text-[11px] shadow-md shadow-emerald-950/40 hover:shadow-emerald-900/60 active:scale-95 transition-all disabled:opacity-50"
        title="Apply changes to database (Cmd+S / Ctrl+S)"
      >
        {#if mutationState.isCommitting}
          <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          <span>Executing...</span>
        {:else}
          <Save size={12} />
          <span>Commit (Cmd+S)</span>
        {/if}
      </button>

      <!-- Close Drawer Button -->
      <button 
        type="button"
        onclick={() => mutationState.isDrawerOpen = false}
        class="text-slate-600 dark:text-slate-400 hover:text-slate-950 dark:hover:text-white p-1 rounded hover:bg-surface-800 transition-colors ml-1"
        title="Hide Review Panel"
      >
        <X size={14} />
      </button>
    </div>
  </div>

  <!-- Drawer Body Content -->
  {#if isExpanded}
    <div class="flex-1 overflow-hidden flex flex-col bg-surface-950">
      <!-- Alert Error Banner -->
      {#if mutationState.errorMessage}
        <div class="px-4 py-2.5 bg-rose-500/15 border-b border-rose-500/40 text-xs flex items-center justify-between shrink-0 font-medium">
          <div class="flex items-center gap-2">
            <AlertCircle size={15} class="text-rose-600 dark:text-rose-400 shrink-0" />
            <span class="font-mono text-rose-950 dark:text-rose-100 font-semibold">{mutationState.errorMessage}</span>
          </div>
          <button 
            type="button"
            onclick={() => mutationState.errorMessage = null}
            class="text-rose-600 dark:text-rose-400 hover:text-rose-950 dark:hover:text-white p-0.5 rounded transition-colors"
            title="Dismiss error"
          >
            <X size={13} />
          </button>
        </div>
      {/if}

      <!-- Alert Success Banner -->
      {#if mutationState.successMessage}
        <div class="px-4 py-2.5 bg-emerald-500/15 border-b border-emerald-500/40 text-xs flex items-center justify-between shrink-0 font-medium">
          <div class="flex items-center gap-2">
            <CheckCircle2 size={15} class="text-emerald-600 dark:text-emerald-400 shrink-0" />
            <span class="text-emerald-950 dark:text-emerald-100 font-semibold">{mutationState.successMessage}</span>
          </div>
          <button 
            type="button"
            onclick={() => mutationState.successMessage = null}
            class="text-emerald-600 dark:text-emerald-400 hover:text-emerald-950 dark:hover:text-white p-0.5 rounded transition-colors"
            title="Dismiss success message"
          >
            <X size={13} />
          </button>
        </div>
      {/if}

      <!-- Visual Diff Mode -->
      {#if mutationState.activeDrawerTab === 'diff'}
        <div in:fade={{ duration: 120 }} class="flex-1 overflow-auto p-3 space-y-2">
          {#if visualDiffs.length === 0}
            <div class="h-full flex items-center justify-center text-slate-500 text-xs">
              No staged changes. Double-click any cell to edit, or click "+ Add Row".
            </div>
          {:else}
            {#each visualDiffs as diff (diff.id)}
              <div class="bg-surface-900/90 border border-slate-200 dark:border-slate-800 rounded-lg p-2.5 text-xs shadow-xs hover:border-slate-300 dark:hover:border-slate-700 transition-colors">
                <div class="flex items-center justify-between mb-2 pb-1.5 border-b border-slate-200 dark:border-slate-800/60">
                  <div class="flex items-center gap-2">
                    {#if diff.type === 'update'}
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase bg-amber-500/15 text-amber-800 dark:text-amber-400 border border-amber-500/40">UPDATE</span>
                    {:else if diff.type === 'insert'}
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase bg-emerald-500/15 text-emerald-800 dark:text-emerald-400 border border-emerald-500/40">INSERT</span>
                    {:else if diff.type === 'delete'}
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase bg-rose-500/15 text-rose-800 dark:text-rose-400 border border-rose-500/40">DELETE</span>
                    {/if}

                    <span class="font-bold text-slate-900 dark:text-slate-200">{diff.tableName}</span>
                    <span class="text-slate-500 dark:text-slate-400 font-mono text-[11px]">{diff.rowIdentifier}</span>
                  </div>

                  <!-- Revert specific row button -->
                  <button 
                    type="button"
                    onclick={() => {
                      if (diff.type === 'insert') {
                        mutationState.removeInsertedRow(diff.rowKey as string);
                      } else {
                        mutationState.revertRow(diff.rowKey as string);
                      }
                    }}
                    class="text-slate-600 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-surface-800 p-1 rounded transition-colors text-[11px] flex items-center gap-1 font-medium"
                    title="Revert this change"
                  >
                    <RotateCcw size={11} class="text-rose-600 dark:text-rose-400" />
                    <span>Revert</span>
                  </button>
                </div>

                <!-- Update diff items -->
                {#if diff.type === 'update' && diff.changes}
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-2 font-mono text-[11px]">
                    {#each diff.changes as change}
                      <div class="bg-surface-950/80 border border-slate-200 dark:border-slate-800/60 rounded p-1.5 flex items-center justify-between gap-2">
                        <span class="text-slate-600 dark:text-slate-400 font-medium truncate">{change.columnName}:</span>
                        <div class="flex items-center gap-1.5 truncate">
                          <span class="text-rose-600 dark:text-rose-400 line-through truncate max-w-[120px]">{formatDisplayVal(change.oldValue)}</span>
                          <ArrowRight size={10} class="text-slate-400 dark:text-slate-500 shrink-0" />
                          <span class="text-emerald-700 dark:text-emerald-400 font-bold truncate max-w-[120px]">{formatDisplayVal(change.newValue)}</span>
                        </div>
                      </div>
                    {/each}
                  </div>
                {:else if diff.type === 'insert' && diff.insertedValues}
                  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-1.5 font-mono text-[11px]">
                    {#each Object.entries(diff.insertedValues) as [col, val]}
                      <div class="bg-surface-950/80 border border-slate-200 dark:border-slate-800/60 rounded p-1.5 flex flex-col">
                        <span class="text-slate-500 text-[10px]">{col}</span>
                        <span class="text-emerald-700 dark:text-emerald-400 font-medium truncate">{formatDisplayVal(val)}</span>
                      </div>
                    {/each}
                  </div>
                {:else if diff.type === 'delete' && diff.deletedValues}
                  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-1.5 font-mono text-[11px] opacity-75">
                    {#each Object.entries(diff.deletedValues) as [col, val]}
                      <div class="bg-surface-950/80 border border-slate-200 dark:border-slate-800/60 rounded p-1.5 flex flex-col">
                        <span class="text-slate-500 text-[10px]">{col}</span>
                        <span class="text-rose-600 dark:text-rose-400 line-through truncate">{formatDisplayVal(val)}</span>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>

      <!-- SQL Script Mode -->
      {:else}
        <div in:fade={{ duration: 120 }} class="flex-1 overflow-hidden flex flex-col relative bg-surface-950">
          <div class="absolute top-2 right-3 z-10">
            <button
              type="button"
              onclick={copySql}
              class="flex items-center gap-1 bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 hover:text-slate-950 dark:hover:text-white px-2.5 py-1 rounded text-[11px] font-medium transition-colors border border-slate-200 dark:border-slate-700"
            >
              {#if isCopied}
                <Check size={12} class="text-emerald-600 dark:text-emerald-400" />
                <span class="text-emerald-600 dark:text-emerald-400 font-bold">Copied!</span>
              {:else}
                <Copy size={12} />
                <span>Copy SQL</span>
              {/if}
            </button>
          </div>

          <div class="flex-1 overflow-auto p-4 font-mono text-xs leading-relaxed bg-surface-950">
            <pre class="select-text whitespace-pre-wrap"><code class="text-emerald-700 dark:text-emerald-400 font-medium">{generatedSql || '-- No mutations pending.'}</code></pre>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
