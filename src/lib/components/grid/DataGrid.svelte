<script lang="ts">
  import type { QueryResult } from '$lib/api/types';
  import { Check, Copy, Download, Filter, Save, Undo } from 'lucide-svelte';

  let { result, errorMessage }: { result?: QueryResult; errorMessage?: string } = $props();

  let stagedChangesCount = $state(0);
  let filterText = $state('');
  let copied = $state(false);

  function copyAsJson() {
    if (!result) return;
    const formatted = result.rows.map(row => {
      const obj: Record<string, any> = {};
      result.columns.forEach((col, idx) => {
        obj[col.name] = row[idx];
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
    const rows = result.rows.map(r => '| ' + r.map(v => v === null ? 'NULL' : typeof v === 'object' ? JSON.stringify(v) : v).join(' | ') + ' |');
    const md = [header, divider, ...rows].join('\n');
    navigator.clipboard.writeText(md);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }
</script>

<div class="w-full h-full flex flex-col bg-surface-950">
  {#if errorMessage}
    <div class="flex-1 flex flex-col items-center justify-center p-6 text-center">
      <div class="max-w-md bg-rose-500/10 border border-rose-500/30 rounded-xl p-4 text-rose-300 text-xs space-y-2 text-left shadow-lg">
        <span class="font-bold text-[11px] uppercase tracking-wider text-rose-400 block">Query Execution Error</span>
        <p class="font-mono text-slate-200 text-xs bg-surface-950/70 p-2.5 rounded-lg border border-slate-800 break-words">{errorMessage}</p>
      </div>
    </div>
  {:else if !result}
    <div class="flex-1 flex flex-col items-center justify-center text-slate-500 text-sm gap-2">
      <span>No data to display.</span>
      <span class="text-xs text-slate-600">Press <kbd class="px-1.5 py-0.5 rounded bg-surface-800 text-slate-300 font-mono">Cmd+Enter</kbd> to execute query.</span>
    </div>
  {:else}
    <!-- Grid Action Toolbar -->
    <div class="h-9 border-b border-slate-800/80 bg-surface-900/60 px-3 flex items-center justify-between text-xs">
      <div class="flex items-center gap-2">
        <span class="text-slate-400 font-mono">{result.rows.length} rows</span>
        <span class="text-slate-600">•</span>
        <span class="text-emerald-400 font-mono">{result.executionTimeMs.toFixed(1)}ms</span>

        {#if stagedChangesCount > 0}
          <div class="ml-3 flex items-center gap-1.5 bg-amber-500/10 border border-amber-500/30 text-amber-300 px-2 py-0.5 rounded text-[11px]">
            <span>{stagedChangesCount} pending changes</span>
            <button class="ml-1 text-emerald-400 hover:underline flex items-center gap-0.5 font-bold">
              <Save size={11} /> Commit
            </button>
            <button class="text-rose-400 hover:underline flex items-center gap-0.5">
              <Undo size={11} /> Revert
            </button>
          </div>
        {/if}
      </div>

      <div class="flex items-center gap-2">
        <button 
          onclick={copyAsMarkdown} 
          class="flex items-center gap-1 text-slate-300 hover:text-white bg-surface-800/80 hover:bg-surface-700 px-2 py-1 rounded text-[11px] transition-colors"
          title="Copy as Markdown Table"
        >
          {#if copied}
            <Check size={12} class="text-emerald-400" />
            <span>Copied!</span>
          {:else}
            <Copy size={12} />
            <span>Copy MD</span>
          {/if}
        </button>

        <button 
          onclick={copyAsJson} 
          class="flex items-center gap-1 text-slate-300 hover:text-white bg-surface-800/80 hover:bg-surface-700 px-2 py-1 rounded text-[11px] transition-colors"
          title="Copy as JSON"
        >
          <Download size={12} />
          <span>JSON</span>
        </button>
      </div>
    </div>

    <!-- Virtual Grid Table -->
    <div class="flex-1 overflow-auto">
      <table class="w-full text-left border-collapse font-mono text-xs">
        <thead class="bg-surface-900 sticky top-0 z-10 select-none shadow-sm">
          <tr class="border-b border-slate-800">
            <th class="px-3 py-2 text-[11px] font-semibold text-slate-400 w-12 text-center border-r border-slate-800/60">#</th>
            {#each result.columns as col (col.name)}
              <th class="px-3 py-2 text-[11px] font-semibold text-slate-100 border-r border-slate-800/60 truncate">
                <div class="flex items-center justify-between gap-2">
                  <span class="truncate font-bold {col.isPrimaryKey ? 'text-amber-600 dark:text-amber-400' : 'text-slate-100'}">{col.name}</span>
                  <span class="text-[10px] text-slate-400 font-normal uppercase">{col.dataType}</span>
                </div>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 text-slate-100 font-normal">
          {#each result.rows as row, rowIdx (rowIdx)}
            <tr class="hover:bg-indigo-500/10 group transition-colors">
              <td class="px-3 py-1.5 text-center text-slate-400 bg-surface-900/60 border-r border-slate-800/60 select-none text-[10px]">
                {rowIdx + 1}
              </td>
              {#each row as cell, cellIdx (cellIdx)}
                <td class="px-3 py-1.5 border-r border-slate-800/60 truncate max-w-[240px] focus:outline-none focus:bg-indigo-500/20 text-slate-100" contenteditable="true">
                  {#if cell === null}
                    <span class="text-slate-400 italic">NULL</span>
                  {:else if typeof cell === 'boolean'}
                    <span class={cell ? 'text-emerald-600 dark:text-emerald-400 font-bold' : 'text-rose-600 dark:text-rose-400 font-bold'}>{cell ? 'TRUE' : 'FALSE'}</span>
                  {:else if typeof cell === 'object'}
                    <span class="text-sky-600 dark:text-sky-400">{JSON.stringify(cell)}</span>
                  {:else}
                    <span>{cell}</span>
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
