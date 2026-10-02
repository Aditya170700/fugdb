<script lang="ts">
  import { Handle, Position } from '@xyflow/svelte';
  import { 
    Database, 
    Key, 
    Link2, 
    Table, 
    Play, 
    FileSpreadsheet, 
    Copy, 
    Check 
  } from 'lucide-svelte';
  import type { ErdNodeData } from './erdLayout';

  let { data }: { data: ErdNodeData } = $props();

  let copied = $state(false);

  function copyTableName(e: MouseEvent) {
    e.stopPropagation();
    navigator.clipboard.writeText(data.name);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }
</script>

<div class="w-72 bg-surface-900 border-2 border-slate-300 dark:border-slate-700/80 hover:border-indigo-500 rounded-xl shadow-2xl overflow-hidden font-sans text-xs transition-all select-none group/node">
  <!-- General Table Target & Source Handles (Top & Bottom) -->
  <Handle type="target" position={Position.Top} id="{data.name}_top" class="!w-2 !h-2 !bg-indigo-500 !border-2 !border-surface-900" />
  <Handle type="source" position={Position.Bottom} id="{data.name}_bottom" class="!w-2 !h-2 !bg-indigo-500 !border-2 !border-surface-900" />

  <!-- Node Header -->
  <div class="px-3.5 py-2.5 bg-gradient-to-r from-surface-950 via-surface-900 to-indigo-950/40 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between gap-2">
    <div class="flex items-center gap-2 truncate">
      <div class="p-1 bg-indigo-500/15 text-indigo-400 rounded border border-indigo-500/30 shrink-0">
        <Table size={13} />
      </div>
      <div class="truncate">
        <span class="font-bold text-slate-900 dark:text-white font-mono text-xs truncate block">{data.name}</span>
        <span class="text-[10px] text-slate-500 font-mono block truncate">{data.schema || 'public'}</span>
      </div>
    </div>

    <!-- Quick Action Buttons -->
    <div class="flex items-center gap-1 shrink-0">
      <button
        type="button"
        onclick={copyTableName}
        class="p-1 rounded text-slate-400 hover:text-white hover:bg-surface-800 transition-colors"
        title="Copy table name"
      >
        {#if copied}
          <Check size={12} class="text-emerald-500" />
        {:else}
          <Copy size={12} />
        {/if}
      </button>

      {#if data.onOpenTable}
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); data.onOpenTable?.(data.name, data.schema); }}
          class="p-1 rounded text-slate-400 hover:text-indigo-400 hover:bg-surface-800 transition-colors"
          title="Open table data grid"
        >
          <FileSpreadsheet size={12} />
        </button>
      {/if}

      {#if data.onQueryTable}
        <button
          type="button"
          onclick={(e) => { e.stopPropagation(); data.onQueryTable?.(data.name, data.schema); }}
          class="p-1 rounded text-slate-400 hover:text-indigo-400 hover:bg-surface-800 transition-colors"
          title="Open query tab"
        >
          <Play size={11} class="fill-current" />
        </button>
      {/if}
    </div>
  </div>

  <!-- Columns List with per-column Connection Handles -->
  <div class="divide-y divide-slate-200 dark:divide-slate-800/60 font-mono text-[11px] max-h-72 overflow-y-auto bg-surface-950/60">
    {#if data.columns && data.columns.length > 0}
      {#each data.columns as col (col.name)}
        <div class="relative px-3 py-1.5 flex items-center justify-between hover:bg-surface-800/50 transition-colors group/col">
          <!-- Column Level Left & Right Handles -->
          <Handle
            type="target"
            position={Position.Left}
            id="{data.name}_{col.name}_left"
            class="!w-1.5 !h-1.5 !-left-1 !bg-indigo-500 !border-0 opacity-0 group-hover/col:opacity-100 transition-opacity"
          />
          <Handle
            type="source"
            position={Position.Right}
            id="{data.name}_{col.name}_right"
            class="!w-1.5 !h-1.5 !-right-1 !bg-indigo-500 !border-0 opacity-0 group-hover/col:opacity-100 transition-opacity"
          />

          <!-- Left Column Name & Indicators -->
          <div class="flex items-center gap-1.5 truncate mr-2">
            {#if col.isPrimaryKey}
              <span title="Primary Key">
                <Key size={11} class="text-amber-500 shrink-0" />
              </span>
            {:else if col.isForeignKey}
              <span title="Foreign Key">
                <Link2 size={11} class="text-sky-400 shrink-0" />
              </span>
            {:else}
              <span class="w-2.5"></span>
            {/if}

            <span class="truncate {col.isPrimaryKey ? 'font-bold text-amber-500 dark:text-amber-400' : col.isForeignKey ? 'text-sky-600 dark:text-sky-300 font-semibold' : 'text-slate-800 dark:text-slate-200'}">
              {col.name}
            </span>
          </div>

          <!-- Right Column DataType -->
          <div class="flex items-center gap-1 shrink-0 text-[10px] text-slate-500 dark:text-slate-400">
            <span class="truncate uppercase max-w-[80px]">{col.dataType}</span>
            {#if !col.nullable && !col.isPrimaryKey}
              <span class="text-rose-500 font-bold" title="Not Null">*</span>
            {/if}
          </div>
        </div>
      {/each}
    {:else}
      <div class="p-3 text-center text-slate-500 text-[11px] font-sans">
        No column metadata
      </div>
    {/if}
  </div>

  <!-- Footer Info -->
  <div class="px-3 py-1 bg-surface-900 border-t border-slate-200 dark:border-slate-800 text-[10px] text-slate-500 flex items-center justify-between">
    <span>{data.columns?.length || 0} columns</span>
    <span class="capitalize">{data.tableType}</span>
  </div>
</div>
