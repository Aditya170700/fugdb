<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { 
    Table, 
    Search, 
    RefreshCw, 
    Eye, 
    Database, 
    Folder, 
    Sparkles
  } from 'lucide-svelte';

  let { width = 260 }: { width?: number } = $props();

  onMount(() => {
    connectionStore.loadSchema(connectionStore.activeConnectionId);
  });
</script>

<aside style="width: {width}px;" class="bg-surface-900/50 flex flex-col h-full select-none shrink-0 overflow-hidden">
  <!-- Sidebar Header & Search -->
  <div class="p-3 border-b border-slate-200 dark:border-slate-800 flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">Schema Explorer</span>
      <button 
        type="button"
        onclick={() => connectionStore.loadSchema(connectionStore.activeConnectionId)}
        class="text-slate-500 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors p-1 rounded hover:bg-surface-800"
        title="Refresh Schema"
      >
        <RefreshCw size={13} class={connectionStore.isLoading ? 'animate-spin' : ''} />
      </button>
    </div>

    <!-- Quick Search Input -->
    <div class="relative">
      <Search size={13} class="absolute left-2.5 top-2.5 text-slate-400 dark:text-slate-500" />
      <input 
        type="text" 
        bind:value={connectionStore.activeSchemaSearch}
        placeholder="Quick search tables (Cmd+P)..." 
        class="w-full bg-surface-900 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-700/80 text-xs rounded-md pl-8 pr-2.5 py-1.5 focus:outline-none focus:border-indigo-500 placeholder:text-slate-400 dark:placeholder:text-slate-500 shadow-xs"
      />
    </div>
  </div>

  <!-- Table Tree List -->
  <div class="flex-1 overflow-y-auto p-2 space-y-0.5">
    {#if connectionStore.isLoading}
      <div class="p-4 text-center text-xs text-slate-500 animate-pulse flex flex-col items-center gap-2">
        <RefreshCw size={16} class="animate-spin text-indigo-400" />
        <span>Loading schema metadata...</span>
      </div>
    {:else if connectionStore.errorMessage}
      <div class="p-3 m-1 bg-rose-500/10 border border-rose-500/30 rounded-lg text-xs flex flex-col gap-2">
        <span class="font-bold text-[11px] uppercase tracking-wider text-rose-600 dark:text-rose-400">Connection Error</span>
        <p class="text-[11px] leading-relaxed break-words text-rose-950 dark:text-rose-100 font-medium">{connectionStore.errorMessage}</p>
        <button 
          type="button"
          onclick={() => connectionStore.loadSchema(connectionStore.activeConnectionId)}
          class="self-start px-2 py-1 bg-rose-600/20 hover:bg-rose-600/40 text-rose-800 dark:text-rose-200 rounded text-[10px] font-semibold transition-colors border border-rose-500/30"
        >
          Retry Connect
        </button>
      </div>
    {:else if connectionStore.filteredTables.length === 0}
      <div class="p-4 text-center text-xs text-slate-500">
        No tables found.
      </div>
    {:else}
      <div class="text-[11px] font-bold text-slate-600 dark:text-slate-400 px-2 py-1.5 flex items-center gap-1.5 uppercase tracking-wider">
        <Folder size={12} class="text-slate-500 dark:text-slate-400" />
        <span>PUBLIC ({connectionStore.filteredTables.length})</span>
      </div>

      {#each connectionStore.filteredTables as table (table.name)}
        <button 
          type="button"
          class="w-full group flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs text-slate-800 dark:text-slate-200 hover:bg-surface-800 hover:text-indigo-600 dark:hover:text-indigo-300 cursor-pointer transition-colors text-left font-medium"
          ondblclick={() => tabsStore.openTableGridTab(table.name, table.schema)}
          onclick={() => tabsStore.openTableGridTab(table.name, table.schema)}
        >
          <div class="flex items-center gap-2 truncate">
            {#if table.tableType === 'view'}
              <Eye size={14} class="text-amber-500 dark:text-amber-400 shrink-0" />
            {:else}
              <Table size={14} class="text-indigo-600 dark:text-indigo-400 shrink-0" />
            {/if}
            <span class="truncate">{table.name}</span>
          </div>

          {#if table.rowCountEstimate != null}
            <span class="text-[10px] text-slate-500 dark:text-slate-400 font-mono group-hover:text-slate-800 dark:group-hover:text-slate-200">
              ~{table.rowCountEstimate.toLocaleString()}
            </span>
          {/if}
        </button>
      {/each}
    {/if}
  </div>

  <!-- Footer Stats -->
  <div class="p-2.5 border-t border-slate-200 dark:border-slate-800 text-[11px] text-slate-600 dark:text-slate-400 flex items-center justify-between">
    <span>DB: <strong class="text-slate-800 dark:text-slate-100 font-semibold">{connectionStore.activeSchemaTree?.currentDatabase || 'None'}</strong></span>
    <span class="font-mono text-emerald-600 dark:text-emerald-400 font-semibold">Online</span>
  </div>
</aside>
