<script lang="ts">
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { 
    Database, 
    Play, 
    Plus, 
    X, 
    Network, 
    ArrowLeftRight, 
    Sparkles, 
    Dices,
    ShieldAlert,
    Share2
  } from 'lucide-svelte';

  let { 
    onOpenErd, 
    onOpenTransfer, 
    onOpenMockData,
    onOpenNewConnection
  }: { 
    onOpenErd: () => void; 
    onOpenTransfer: () => void; 
    onOpenMockData: () => void; 
    onOpenNewConnection: () => void;
  } = $props();

  const activeConn = $derived(connectionStore.activeConnection);
  const activeTab = $derived(tabsStore.activeTab);

  function getEnvBadgeColor(env?: string) {
    switch (env) {
      case 'production': return 'bg-rose-500/20 text-rose-300 border-rose-500/30';
      case 'staging': return 'bg-amber-500/20 text-amber-300 border-amber-500/30';
      default: return 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30';
    }
  }
  import { ChevronDown, Check } from 'lucide-svelte';

  let isConnDropdownOpen = $state(false);

  function getDriverIcon(driver: string) {
    switch (driver) {
      case 'postgres': return '🐘';
      case 'mysql': return '🐬';
      case 'sqlite': return '🪶';
      case 'mssql': return '🪟';
      case 'duckdb': return '🦆';
      default: return '🗄️';
    }
  }
</script>

<header class="h-12 border-b border-slate-800/80 bg-surface-900/90 backdrop-blur-md flex items-center justify-between px-3 z-20">
  <!-- Left Brand & Active Connection Indicator -->
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2 font-bold tracking-wider text-sm text-indigo-400">
      <div class="w-7 h-7 rounded-lg bg-gradient-to-tr from-indigo-600 to-violet-500 flex items-center justify-center text-white shadow-md shadow-indigo-500/20">
        <Database size={16} />
      </div>
      <span>FUG<span class="text-slate-100 font-extrabold">DB</span></span>
    </div>

    <div class="h-4 w-[1px] bg-slate-700/60 mx-1"></div>

    <!-- Active Connection Selector Custom Dropdown -->
    <div class="relative flex items-center gap-2 text-xs">
      <button 
        type="button"
        onclick={() => isConnDropdownOpen = !isConnDropdownOpen}
        class="bg-surface-800/90 text-slate-200 border border-slate-700/80 hover:border-indigo-500/80 rounded-lg px-2.5 py-1 text-xs flex items-center gap-2 transition-colors cursor-pointer shadow-sm"
      >
        {#if activeConn}
          <span class="text-sm">{getDriverIcon(activeConn.driver)}</span>
          <span class="font-medium max-w-[140px] truncate text-slate-100">{activeConn.name}</span>
          <span class="px-1.5 py-0.2 text-[9px] font-semibold uppercase rounded-full border {getEnvBadgeColor(activeConn.environment)}">
            {activeConn.environment}
          </span>
        {:else}
          <span class="text-slate-400">Select Connection</span>
        {/if}
        <ChevronDown size={13} class="text-slate-400 transition-transform {isConnDropdownOpen ? 'rotate-180 text-indigo-400' : ''}" />
      </button>

      <button 
        type="button"
        onclick={onOpenNewConnection}
        class="p-1 text-slate-400 hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors"
        title="Add New Connection (+)"
      >
        <Plus size={14} />
      </button>

      <!-- Dropdown Popover -->
      {#if isConnDropdownOpen}
        <div 
          class="fixed inset-0 z-40" 
          onclick={() => isConnDropdownOpen = false}
          role="presentation"
        ></div>

        <div class="absolute left-0 top-full mt-1.5 z-50 w-64 bg-surface-900 border border-slate-700 rounded-xl shadow-2xl p-1 space-y-0.5">
          {#each connectionStore.connections as conn (conn.id)}
            <button
              type="button"
              onclick={() => { connectionStore.selectConnection(conn.id); isConnDropdownOpen = false; }}
              class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors {conn.id === connectionStore.activeConnectionId ? 'bg-indigo-600/20 text-indigo-300 font-semibold border border-indigo-500/30' : 'hover:bg-surface-800 text-slate-300'}"
            >
              <div class="flex items-center gap-2 truncate">
                <span>{getDriverIcon(conn.driver)}</span>
                <span class="truncate">{conn.name}</span>
              </div>
              <span class="px-1.5 py-0.2 text-[9px] font-semibold uppercase rounded-full border {getEnvBadgeColor(conn.environment)}">
                {conn.environment}
              </span>
            </button>
          {/each}

          <div class="border-t border-slate-800 pt-1 mt-1">
            <button
              type="button"
              onclick={() => { isConnDropdownOpen = false; onOpenNewConnection(); }}
              class="w-full flex items-center gap-2 px-2.5 py-1.5 text-xs text-indigo-400 hover:bg-surface-800 rounded-lg transition-colors font-medium"
            >
              <Plus size={13} />
              <span>Create New Connection...</span>
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Center Tab Bar -->
  <div class="flex-1 flex items-center gap-1.5 overflow-x-auto px-4 max-w-2xl scrollbar-none">
    {#each tabsStore.tabs as tab (tab.id)}
      <div 
        role="button"
        tabindex="0"
        class="group flex items-center gap-2 px-3 py-1 text-xs rounded-md border transition-all cursor-pointer select-none {tab.id === tabsStore.activeTabId ? 'bg-surface-800 text-indigo-300 border-indigo-500/40 shadow-sm' : 'text-slate-400 border-transparent hover:bg-surface-800/50 hover:text-slate-200'}"
        onclick={() => tabsStore.activeTabId = tab.id}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') tabsStore.activeTabId = tab.id; }}
      >
        <span class="truncate max-w-[130px] font-medium">{tab.title}</span>
        <button 
          type="button"
          class="opacity-0 group-hover:opacity-100 hover:text-rose-400 transition-opacity p-0.5 rounded"
          onclick={(e) => { e.stopPropagation(); tabsStore.closeTab(tab.id); }}
        >
          <X size={12} />
        </button>
      </div>
    {/each}

    <button 
      onclick={() => tabsStore.openNewSqlTab()}
      class="p-1 rounded-md text-slate-400 hover:text-indigo-300 hover:bg-surface-800 transition-colors"
      title="New SQL Tab (Cmd+T)"
    >
      <Plus size={14} />
    </button>
  </div>

  <!-- Right Actions Toolbar -->
  <div class="flex items-center gap-2">
    <!-- Execute Query Button -->
    <button 
      disabled={!activeTab || activeTab.isExecuting}
      onclick={() => activeTab && tabsStore.runTabQuery(activeTab.id)}
      class="flex items-center gap-1.5 px-3 py-1 bg-indigo-600 hover:bg-indigo-500 disabled:bg-slate-800 disabled:text-slate-600 text-white font-semibold text-xs rounded-md shadow-sm transition-all active:scale-95"
      title="Run Current Query (Cmd+Enter)"
    >
      <Play size={13} class={activeTab?.isExecuting ? 'animate-spin' : 'fill-white'} />
      <span>{activeTab?.isExecuting ? 'Running...' : 'Execute'}</span>
    </button>

    <div class="h-4 w-[1px] bg-slate-700/60 mx-1"></div>

    <!-- Power Tools Icons -->
    <button 
      onclick={onOpenTransfer}
      class="p-1.5 text-slate-300 hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="Multi-source Transfer & Import/Export"
    >
      <ArrowLeftRight size={15} />
    </button>

    <button 
      onclick={onOpenErd}
      class="p-1.5 text-slate-300 hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="Interactive ERD Visualizer"
    >
      <Network size={15} />
    </button>

    <button 
      onclick={onOpenMockData}
      class="p-1.5 text-slate-300 hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="QA Smart Mock Data Generator"
    >
      <Dices size={15} />
    </button>
  </div>
</header>
