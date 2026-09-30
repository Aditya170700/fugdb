<script lang="ts">
  import { connectionStore } from '$lib/state/connection.svelte';
  import { ArrowRight, CheckCircle2, Database, Download, FileSpreadsheet, FileText, Upload, X } from 'lucide-svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  let mode = $state<'export' | 'import' | 'cross_db'>('cross_db');
  let selectedFormat = $state<'csv' | 'json' | 'excel' | 'parquet' | 'sql'>('csv');
  let sourceConnId = $state<string>(connectionStore.activeConnectionId);
  let targetConnId = $state<string>(connectionStore.connections[1]?.id || connectionStore.activeConnectionId);
  let selectedTable = $state<string>('users');
  let isRunning = $state(false);
  let progressPercent = $state(0);
  let processedRows = $state(0);

  function startTransfer() {
    isRunning = true;
    progressPercent = 0;
    processedRows = 0;

    const interval = setInterval(() => {
      processedRows += 250;
      progressPercent += 10;
      if (progressPercent >= 100) {
        clearInterval(interval);
        isRunning = false;
      }
    }, 200);
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-surface-900 border border-slate-700 w-full max-w-2xl rounded-xl shadow-2xl overflow-hidden flex flex-col">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-indigo-500/20 text-indigo-400 flex items-center justify-center">
            <Upload size={18} />
          </div>
          <div>
            <h3 class="font-bold text-sm text-slate-100">Multi-Source Data Transfer & ETL</h3>
            <p class="text-[11px] text-slate-400">Stream data between databases, CSV, JSON, Excel, or Parquet</p>
          </div>
        </div>
        <button onclick={onClose} class="text-slate-400 hover:text-slate-100 hover:bg-surface-800 p-1.5 rounded-lg transition-colors" title="Close">
          <X size={16} />
        </button>
      </div>

      <!-- Mode Selector -->
      <div class="flex border-b border-slate-800 bg-surface-950/60 p-1.5 gap-1.5 text-xs">
        <button 
          onclick={() => mode = 'cross_db'}
          class="flex-1 py-1.5 rounded-md font-medium transition-colors {mode === 'cross_db' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
        >
          Direct DB $\leftrightarrow$ DB Migration
        </button>
        <button 
          onclick={() => mode = 'export'}
          class="flex-1 py-1.5 rounded-md font-medium transition-colors {mode === 'export' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
        >
          Export to File
        </button>
        <button 
          onclick={() => mode = 'import'}
          class="flex-1 py-1.5 rounded-md font-medium transition-colors {mode === 'import' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
        >
          Import from File / URL
        </button>
      </div>

      <!-- Content Form -->
      <div class="p-6 space-y-4 text-xs">
        {#if mode === 'cross_db'}
          <div class="grid grid-cols-5 gap-3 items-center">
            <!-- Source DB -->
            <div class="col-span-2 space-y-1.5">
              <label for="src-conn-select" class="font-semibold text-slate-400 uppercase text-[10px]">Source Connection</label>
              <select id="src-conn-select" bind:value={sourceConnId} class="w-full bg-surface-950 border border-slate-800 rounded-md p-2 text-slate-200 focus:outline-none focus:border-indigo-500">
                {#each connectionStore.connections as conn}
                  <option value={conn.id}>{conn.name}</option>
                {/each}
              </select>
              <label for="src-table-input" class="font-semibold text-slate-400 uppercase text-[10px]">Table</label>
              <input id="src-table-input" bind:value={selectedTable} class="w-full bg-surface-950 border border-slate-800 rounded-md p-2 text-slate-200" placeholder="e.g. users" />
            </div>

            <div class="flex justify-center text-indigo-400">
              <ArrowRight size={24} class="animate-pulse" />
            </div>

            <!-- Target DB -->
            <div class="col-span-2 space-y-1.5">
              <label for="target-conn-select" class="font-semibold text-slate-400 uppercase text-[10px]">Target Connection</label>
              <select id="target-conn-select" bind:value={targetConnId} class="w-full bg-surface-950 border border-slate-800 rounded-md p-2 text-slate-200 focus:outline-none focus:border-indigo-500">
                {#each connectionStore.connections as conn}
                  <option value={conn.id}>{conn.name}</option>
                {/each}
              </select>
              <label for="target-table-input" class="font-semibold text-slate-400 uppercase text-[10px]">Target Table</label>
              <input id="target-table-input" value={selectedTable} class="w-full bg-surface-950 border border-slate-800 rounded-md p-2 text-slate-200" />
            </div>
          </div>
        {:else}
          <!-- Format Selector for Export/Import -->
          <div class="space-y-2">
            <span class="font-semibold text-slate-400 uppercase text-[10px] block">Select Format</span>
            <div class="grid grid-cols-5 gap-2">
              {#each ['csv', 'json', 'excel', 'parquet', 'sql'] as fmt}
                <button 
                  type="button"
                  onclick={() => selectedFormat = fmt as any}
                  class="p-3 rounded-lg border text-center font-mono uppercase text-xs transition-all {selectedFormat === fmt ? 'bg-indigo-600/20 border-indigo-500 text-indigo-300 font-bold' : 'border-slate-800 text-slate-400 hover:border-slate-700'}"
                >
                  {fmt}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Progress Bar if Running -->
        {#if isRunning || progressPercent === 100}
          <div class="bg-surface-950 p-4 rounded-lg border border-slate-800 space-y-2">
            <div class="flex items-center justify-between text-xs">
              <span class="font-medium text-slate-300">{isRunning ? 'Streaming rows in Tokio channel...' : 'Transfer Completed!'}</span>
              <span class="font-mono text-emerald-400">{processedRows.toLocaleString()} rows</span>
            </div>
            <div class="w-full h-2 bg-slate-800 rounded-full overflow-hidden">
              <div class="h-full bg-gradient-to-r from-indigo-500 to-emerald-400 transition-all duration-300" style="width: {progressPercent}%"></div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer Actions -->
      <div class="px-5 py-3 border-t border-slate-800 bg-surface-950/40 flex justify-end gap-2">
        <button onclick={onClose} class="px-3 py-1.5 text-xs text-slate-400 hover:text-slate-200">Close</button>
        <button 
          onclick={startTransfer}
          disabled={isRunning}
          class="px-4 py-1.5 text-xs font-semibold rounded-md bg-indigo-600 hover:bg-indigo-500 text-white disabled:bg-slate-800 transition-all shadow-md shadow-indigo-600/20"
        >
          {isRunning ? 'Processing...' : 'Start Transfer'}
        </button>
      </div>
    </div>
  </div>
{/if}
