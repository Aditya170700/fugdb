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
    Sparkles, 
    Plus
  } from 'lucide-svelte';

  onMount(() => {
    connectionStore.loadSchema(connectionStore.activeConnectionId);
  });
</script>

<aside class="w-64 border-r border-slate-800 bg-surface-900/50 flex flex-col h-full select-none">
  <!-- Sidebar Header & Search -->
  <div class="p-3 border-b border-slate-800/80 flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold uppercase tracking-wider text-slate-400">Schema Explorer</span>
      <button 
        onclick={() => connectionStore.loadSchema(connectionStore.activeConnectionId)}
        class="text-slate-400 hover:text-indigo-400 transition-colors p-1"
        title="Refresh Schema"
      >
        <RefreshCw size={13} class={connectionStore.isLoading ? 'animate-spin' : ''} />
      </button>
    </div>

    <!-- Quick Search Input -->
    <div class="relative">
      <Search size={13} class="absolute left-2.5 top-2.5 text-slate-500" />
      <input 
        type="text" 
        bind:value={connectionStore.activeSchemaSearch}
        placeholder="Quick search tables (Cmd+P)..." 
        class="w-full bg-surface-950/70 text-slate-200 border border-slate-800 text-xs rounded-md pl-8 pr-2.5 py-1.5 focus:outline-none focus:border-indigo-500 placeholder:text-slate-600"
      />
    </div>
  </div>

  <!-- Table Tree List -->
  <div class="flex-1 overflow-y-auto p-2 space-y-1">
    {#if connectionStore.isLoading}
      <div class="p-4 text-center text-xs text-slate-500 animate-pulse">
        Loading schema metadata...
      </div>
    {:else if connectionStore.filteredTables.length === 0}
      <div class="p-4 text-center text-xs text-slate-500">
        No tables found.
      </div>
    {:else}
      <div class="text-[11px] font-semibold text-slate-500 px-2 py-1 flex items-center gap-1">
        <Folder size={12} />
        <span>PUBLIC ({connectionStore.filteredTables.length})</span>
      </div>

      {#each connectionStore.filteredTables as table (table.name)}
        <div 
          class="group flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs text-slate-300 hover:bg-surface-800 hover:text-white cursor-pointer transition-colors"
          ondblclick={() => tabsStore.openTableGridTab(table.name)}
          onclick={() => tabsStore.openTableGridTab(table.name)}
        >
          <div class="flex items-center gap-2 truncate">
            {#if table.tableType === 'view'}
              <Eye size={14} class="text-amber-400 shrink-0" />
            {:else}
              <Table size={14} class="text-indigo-400 shrink-0" />
            {/if}
            <span class="truncate font-medium">{table.name}</span>
          </div>

          {#if table.rowCountEstimate !== undefined}
            <span class="text-[10px] text-slate-500 font-mono group-hover:text-slate-400">
              ~{table.rowCountEstimate.toLocaleString()}
            </span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Footer Stats -->
  <div class="p-2.5 border-t border-slate-800/80 text-[11px] text-slate-500 flex items-center justify-between">
    <span>DB: <strong class="text-slate-300">{connectionStore.activeSchemaTree?.currentDatabase || 'None'}</strong></span>
    <span class="font-mono text-emerald-400">Online</span>
  </div>
</aside>
