<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import CustomSelect from '$lib/components/ui/CustomSelect.svelte';
  import { 
    Table, 
    Search, 
    RefreshCw, 
    Eye, 
    Folder, 
    ChevronRight,
    ChevronDown,
    KeyRound,
    Link2,
    Hash,
    Type,
    Calendar,
    Check,
    BookOpen,
    Database,
    Terminal,
    Key,
    Layers
  } from 'lucide-svelte';

  let { 
    width = 260,
    onOpenDictionary
  }: { 
    width?: number;
    onOpenDictionary?: () => void;
  } = $props();

  let expandedTables = $state<Record<string, boolean>>({});

  function toggleTable(tableName: string, e: MouseEvent) {
    e.stopPropagation();
    expandedTables[tableName] = !expandedTables[tableName];
  }

  function getColumnTypeIcon(dataType: string) {
    const dt = dataType.toLowerCase();
    if (dt.includes('int') || dt.includes('number') || dt.includes('decimal') || dt.includes('numeric') || dt.includes('float')) {
      return Hash;
    }
    if (dt.includes('date') || dt.includes('time')) {
      return Calendar;
    }
    if (dt.includes('bool') || dt.includes('bit')) {
      return Check;
    }
    return Type;
  }

  const currentDatabase = $derived(connectionStore.activeSchemaTree?.currentDatabase || connectionStore.activeConnection?.database || '');
  const databases = $derived(connectionStore.activeSchemaTree?.databases || []);

  const databaseOptions = $derived(
    databases.map(dbName => ({
      value: dbName,
      label: dbName,
      badge: isSystemDb(dbName) ? 'System' : undefined,
      badgeColor: isSystemDb(dbName) ? 'bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/30' : undefined,
      dotColor: isSystemDb(dbName) ? 'bg-amber-500' : 'bg-indigo-500'
    }))
  );

  function isSystemDb(name: string) {
    const n = name.toLowerCase();
    return ['master', 'model', 'msdb', 'tempdb', 'information_schema', 'performance_schema', 'sys'].includes(n);
  }

  onMount(() => {
    connectionStore.loadSchema(connectionStore.activeConnectionId);
  });
</script>

<aside style="width: {width}px;" class="bg-surface-900/50 flex flex-col h-full select-none shrink-0 overflow-hidden">
  <!-- Sidebar Header & Search -->
  <div class="p-3 border-b border-slate-200 dark:border-slate-800 flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">Schema Explorer</span>
      <div class="flex items-center gap-1">
        {#if onOpenDictionary}
          <button 
            type="button"
            onclick={onOpenDictionary}
            class="text-slate-500 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors p-1 rounded hover:bg-surface-800 cursor-pointer"
            title="Generate Data Dictionary (Markdown/HTML/PDF)"
          >
            <BookOpen size={13} />
          </button>
        {/if}
        <button 
          type="button"
          onclick={() => connectionStore.loadSchema(connectionStore.activeConnectionId)}
          class="text-slate-500 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors p-1 rounded hover:bg-surface-800 cursor-pointer"
          title="Refresh Schema"
        >
          <RefreshCw size={13} class={connectionStore.isLoading ? 'animate-spin' : ''} />
        </button>
      </div>
    </div>

    <!-- Database Switcher Dropdown (CustomSelect Component) -->
    {#if databases.length > 1}
      <div class="w-full">
        <CustomSelect
          size="sm"
          value={currentDatabase}
          options={databaseOptions}
          onchange={(newDb) => connectionStore.switchDatabase(newDb)}
          placeholder="Select Database..."
        />
      </div>
    {/if}

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
          class="self-start px-2 py-1 bg-rose-600/20 hover:bg-rose-600/40 text-rose-800 dark:text-rose-200 rounded text-[10px] font-semibold transition-colors border border-rose-500/30 cursor-pointer"
        >
          Retry Connect
        </button>
      </div>
    {:else if connectionStore.activeConnection?.driver === 'redis'}
      <div class="p-3 flex flex-col gap-3">
        <!-- Redis In-Memory Node Info Card -->
        <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl flex flex-col gap-2 shadow-xs">
          <div class="flex items-center gap-2.5">
            <div class="p-1.5 rounded-lg bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20 shrink-0">
              <Database class="w-4 h-4" />
            </div>
            <div class="flex flex-col min-w-0">
              <span class="font-bold text-xs text-slate-800 dark:text-slate-100 truncate">Redis In-Memory Node</span>
              <span class="text-[11px] text-slate-500 dark:text-slate-400 font-mono">{connectionStore.activeConnection.host || '127.0.0.1'}:{connectionStore.activeConnection.port || 6379}</span>
            </div>
          </div>
          <p class="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">Manage key patterns, strings, hashes, sets, zsets, streams, and run CLI commands.</p>
        </div>

        <!-- Quick Action Buttons -->
        <div class="space-y-2">
          <button
            type="button"
            onclick={() => tabsStore.openRedisTab(connectionStore.activeConnectionId)}
            class="w-full flex items-center justify-between p-2.5 rounded-xl bg-surface-900 hover:bg-surface-800 border border-slate-200 hover:border-indigo-400/60 dark:border-slate-800 dark:hover:border-indigo-500/40 text-slate-800 dark:text-slate-100 text-xs font-semibold transition-all shadow-xs cursor-pointer group"
          >
            <div class="flex items-center gap-2">
              <div class="p-1 rounded-md bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20">
                <Key class="w-3.5 h-3.5 group-hover:scale-110 transition-transform" />
              </div>
              <span class="text-slate-700 dark:text-slate-200 group-hover:text-indigo-600 dark:group-hover:text-indigo-300 transition-colors">Key-Value Explorer</span>
            </div>
            <span class="text-[10px] bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 px-2 py-0.5 rounded font-mono border border-slate-200 dark:border-slate-700">Open Tab</span>
          </button>

          <button
            type="button"
            onclick={() => {
              tabsStore.openRedisTab(connectionStore.activeConnectionId);
              import('../../state/redis.svelte').then(({ redisStore }) => {
                redisStore.ensureState(connectionStore.activeConnectionId).activeSubView = 'cli';
              });
            }}
            class="w-full flex items-center justify-between p-2.5 rounded-xl bg-surface-900 hover:bg-surface-800 border border-slate-200 hover:border-emerald-400/60 dark:border-slate-800 dark:hover:border-emerald-500/40 text-slate-800 dark:text-slate-100 text-xs font-semibold transition-all shadow-xs cursor-pointer group"
          >
            <div class="flex items-center gap-2">
              <div class="p-1 rounded-md bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20">
                <Terminal class="w-3.5 h-3.5 group-hover:scale-110 transition-transform" />
              </div>
              <span class="text-slate-700 dark:text-slate-200 group-hover:text-emerald-600 dark:group-hover:text-emerald-300 transition-colors">Interactive CLI</span>
            </div>
            <span class="text-[10px] bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 px-2 py-0.5 rounded font-mono border border-slate-200 dark:border-slate-700">Terminal</span>
          </button>
        </div>
      </div>
    {:else if connectionStore.filteredTables.length === 0}
      <div class="p-4 text-center text-xs text-slate-500">
        No tables found.
      </div>
    {:else}
      <div class="text-[11px] font-bold text-slate-600 dark:text-slate-400 px-2 py-1.5 flex items-center gap-1.5 uppercase tracking-wider">
        <Folder size={12} class="text-slate-500 dark:text-slate-400" />
        <span>TABLES & VIEWS ({connectionStore.filteredTables.length})</span>
      </div>

      {#each connectionStore.filteredTables as table (table.schema + '.' + table.name)}
        {@const isExpanded = !!expandedTables[table.name]}
        {@const cols = table.columns || []}

        <div class="flex flex-col">
          <div 
            class="w-full group flex items-center justify-between px-2 py-1.5 rounded-md text-xs text-slate-800 dark:text-slate-200 hover:bg-surface-800 hover:text-indigo-600 dark:hover:text-indigo-300 cursor-pointer transition-colors font-medium"
            role="button"
            tabindex="0"
            onclick={() => tabsStore.openTableGridTab(table.name, table.schema)}
            onkeydown={(e) => { if (e.key === 'Enter') tabsStore.openTableGridTab(table.name, table.schema); }}
          >
            <div class="flex items-center gap-1.5 truncate">
              <!-- Expand toggle -->
              {#if cols.length > 0}
                <button 
                  type="button"
                  class="p-0.5 -ml-1 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer rounded"
                  onclick={(e) => toggleTable(table.name, e)}
                  title={isExpanded ? "Collapse Columns" : "Expand Columns"}
                >
                  {#if isExpanded}
                    <ChevronDown size={13} />
                  {:else}
                    <ChevronRight size={13} />
                  {/if}
                </button>
              {:else}
                <div class="w-3"></div>
              {/if}

              {#if table.tableType === 'view'}
                <Eye size={13} class="text-amber-500 dark:text-amber-400 shrink-0" />
              {:else}
                <Table size={13} class="text-indigo-600 dark:text-indigo-400 shrink-0" />
              {/if}
              <span class="truncate">{table.name}</span>
            </div>

            <div class="flex items-center gap-1.5 shrink-0">
              {#if cols.length > 0}
                <span class="text-[10px] text-slate-400 dark:text-slate-500 font-mono">
                  {cols.length} cols
                </span>
              {/if}
              {#if table.rowCountEstimate != null}
                <span class="text-[10px] text-slate-500 dark:text-slate-400 font-mono">
                  ~{table.rowCountEstimate.toLocaleString()}
                </span>
              {/if}
            </div>
          </div>

          <!-- Expanded Columns List -->
          {#if isExpanded && cols.length > 0}
            <div class="pl-6 pr-1 py-1 space-y-0.5 border-l border-slate-200 dark:border-slate-800 ml-3.5 my-0.5">
              {#each cols as col, colIdx (col.name + '_' + colIdx)}
                {@const TypeIcon = getColumnTypeIcon(col.dataType)}
                <div 
                  class="flex items-center justify-between px-2 py-1 rounded text-[11px] text-slate-600 dark:text-slate-300 hover:bg-surface-800/80 hover:text-slate-900 dark:hover:text-slate-100 group/col transition-colors"
                  title={`${col.name} (${col.dataType})${col.isPrimaryKey ? ' [Primary Key]' : ''}${col.isForeignKey ? ' [Foreign Key]' : ''}`}
                >
                  <div class="flex items-center gap-1.5 truncate">
                    {#if col.isPrimaryKey}
                      <KeyRound size={11} class="text-amber-500 shrink-0" />
                    {:else if col.isForeignKey}
                      <Link2 size={11} class="text-sky-500 shrink-0" />
                    {:else}
                      <TypeIcon size={11} class="text-slate-400 dark:text-slate-500 shrink-0" />
                    {/if}
                    <span class="truncate {col.isPrimaryKey ? 'font-semibold text-slate-800 dark:text-slate-100' : ''}">{col.name}</span>
                  </div>
                  <span class="text-[9.5px] font-mono text-slate-400 dark:text-slate-500 group-hover/col:text-slate-600 dark:group-hover/col:text-slate-300 truncate max-w-[80px]">
                    {col.dataType}
                  </span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Footer Stats -->
  <div class="p-2.5 border-t border-slate-200 dark:border-slate-800 text-[11px] text-slate-600 dark:text-slate-400 flex items-center justify-between">
    <span>DB: <strong class="text-slate-800 dark:text-slate-100 font-semibold">{connectionStore.activeSchemaTree?.currentDatabase || 'None'}</strong></span>
    <span class="font-mono text-emerald-600 dark:text-emerald-400 font-semibold">Online</span>
  </div>
</aside>
