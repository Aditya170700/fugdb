<script lang="ts">
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { themeStore, type ThemeMode } from '$lib/state/theme.svelte';
  import { 
    Database, 
    Play, 
    Plus, 
    X, 
    BookOpen,
    Network, 
    ArrowLeftRight, 
    Sparkles, 
    Dices,
    ShieldAlert,
    Share2,
    ChevronDown, 
    Check,
    Sun,
    Moon,
    Monitor,
    Lock,
    Activity,
    GitCompare,
    History as HistoryIcon
  } from 'lucide-svelte';
  import { historyStore } from '$lib/state/history.svelte';
  import { aiStore } from '$lib/state/ai.svelte';
  import { monitorStore } from '$lib/state/monitor.svelte';
  import { diffStore } from '$lib/state/diff.svelte';

  let { 
    onOpenErd, 
    onOpenTransfer, 
    onOpenMockData,
    onOpenDictionary,
    onOpenNewConnection
  }: { 
    onOpenErd: () => void; 
    onOpenTransfer: () => void; 
    onOpenMockData: () => void; 
    onOpenDictionary: () => void;
    onOpenNewConnection: () => void;
  } = $props();

  const activeConn = $derived(connectionStore.activeConnection);
  const activeTab = $derived(tabsStore.activeTab);

  let isConnDropdownOpen = $state(false);
  let isThemeDropdownOpen = $state(false);

  function getEnvBadgeColor(env?: string) {
    switch (env) {
      case 'production': return 'bg-rose-500/15 text-rose-800 dark:text-rose-300 border-rose-500/40 font-bold';
      case 'staging': return 'bg-amber-500/15 text-amber-800 dark:text-amber-300 border-amber-500/40 font-bold';
      default: return 'bg-emerald-500/15 text-emerald-800 dark:text-emerald-300 border-emerald-500/40 font-bold';
    }
  }

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

<header class="h-12 border-b border-slate-200 dark:border-slate-800 bg-surface-900/90 backdrop-blur-md flex items-center px-3 z-40 relative gap-3">
  <!-- 1. Left Brand -->
  <div class="flex items-center gap-2 font-bold tracking-wider text-sm text-indigo-500 dark:text-indigo-400 shrink-0">
    <div class="w-7 h-7 rounded-lg bg-gradient-to-tr from-indigo-600 to-violet-500 flex items-center justify-center text-white shadow-md shadow-indigo-500/20">
      <Database size={16} />
    </div>
    <span>FUG<span class="text-slate-900 dark:text-slate-100 font-extrabold">DB</span></span>
  </div>

  <!-- 2. Active Connection Selector Custom Dropdown -->
  <div class="relative flex items-center shrink-0 text-xs">
    <button 
      type="button"
      onclick={() => isConnDropdownOpen = !isConnDropdownOpen}
      class="bg-surface-800 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-700 hover:border-indigo-500 rounded-lg px-2.5 py-1 text-xs flex items-center gap-2 transition-colors cursor-pointer shadow-xs"
    >
      {#if activeConn}
        <span class="text-sm">{getDriverIcon(activeConn.driver)}</span>
        <span class="font-bold max-w-[140px] truncate text-slate-900 dark:text-slate-100">{activeConn.name}</span>
        <span class="px-1.5 py-0.2 text-[9px] uppercase rounded-full border {getEnvBadgeColor(activeConn.environment)}">
          {activeConn.environment}
        </span>
      {:else}
        <span class="text-slate-500 dark:text-slate-400">Select Connection</span>
      {/if}
      <ChevronDown size={13} class="text-slate-500 dark:text-slate-400 transition-transform {isConnDropdownOpen ? 'rotate-180 text-indigo-600 dark:text-indigo-400' : ''}" />
    </button>

    <!-- Dropdown Popover -->
    {#if isConnDropdownOpen}
      <div 
        class="fixed inset-0 z-40" 
        onclick={() => isConnDropdownOpen = false}
        role="presentation"
      ></div>

      <div class="absolute left-0 top-full mt-1.5 z-50 w-64 bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-xl shadow-2xl p-1 space-y-0.5">
        {#each connectionStore.connections as conn (conn.id)}
          <button
            type="button"
            onclick={() => { connectionStore.selectConnection(conn.id); isConnDropdownOpen = false; }}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors {conn.id === connectionStore.activeConnectionId ? 'bg-indigo-600/15 text-indigo-600 dark:text-indigo-300 font-bold border border-indigo-500/30' : 'hover:bg-surface-800 text-slate-800 dark:text-slate-200'}"
          >
            <div class="flex items-center gap-2 truncate">
              <span>{getDriverIcon(conn.driver)}</span>
              <span class="truncate">{conn.name}</span>
              {#if conn.useKeyring}
                <span title="OS Keyring Protected" class="flex items-center text-emerald-500 shrink-0">
                  <Lock size={11} />
                </span>
              {/if}
            </div>
            <span class="px-1.5 py-0.2 text-[9px] uppercase rounded-full border {getEnvBadgeColor(conn.environment)}">
              {conn.environment}
            </span>
          </button>
        {/each}

        <div class="border-t border-slate-200 dark:border-slate-800 pt-1 mt-1">
          <button
            type="button"
            onclick={() => { isConnDropdownOpen = false; onOpenNewConnection(); }}
            class="w-full flex items-center gap-2 px-2.5 py-1.5 text-xs text-indigo-600 dark:text-indigo-400 hover:bg-surface-800 rounded-lg transition-colors font-semibold"
          >
            <Plus size={13} />
            <span>Create New Connection...</span>
          </button>
        </div>
      </div>
    {/if}
  </div>

  <!-- 3. Tab Bar (Langsung Mepet Tepat di Samping Connection Selector) -->
  <div class="flex items-center gap-1.5 overflow-x-auto scrollbar-none shrink-0 max-w-[calc(100vw-500px)]">
    {#each tabsStore.tabs as tab (tab.id)}
      <div 
        role="button"
        tabindex="0"
        class="group flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-lg border transition-all cursor-pointer select-none shrink-0 {tab.id === tabsStore.activeTabId ? 'bg-indigo-50 dark:bg-surface-800 text-indigo-700 dark:text-indigo-200 border-indigo-400 dark:border-indigo-500/50 shadow-xs font-bold ring-1 ring-indigo-500/20' : 'bg-surface-900/50 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-800 hover:bg-surface-800 hover:text-slate-900 dark:hover:text-slate-100'}"
        onclick={() => tabsStore.activeTabId = tab.id}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') tabsStore.activeTabId = tab.id; }}
      >
        <span class="text-[11px] opacity-80">{tab.type === 'table_grid' ? '📋' : '📝'}</span>
        <span class="truncate max-w-[120px]">{tab.title || 'Untitled Tab'}</span>
        <button 
          type="button"
          class="opacity-40 group-hover:opacity-100 hover:text-rose-500 hover:bg-rose-500/20 p-0.5 rounded transition-all ml-0.5"
          onclick={(e) => { e.stopPropagation(); tabsStore.closeTab(tab.id); }}
          title="Close Tab"
        >
          <X size={11} />
        </button>
      </div>
    {/each}

    <!-- Plus Button to create New Tab -->
    <button 
      type="button"
      onclick={() => tabsStore.openNewSqlTab()}
      class="flex items-center justify-center w-7 h-7 rounded-lg text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 border border-slate-200 dark:border-slate-800 transition-colors shrink-0"
      title="New SQL Tab (Cmd+T)"
    >
      <Plus size={14} />
    </button>
  </div>

  <!-- Spacer to push Right Actions to the right edge -->
  <div class="flex-1"></div>

  <!-- 4. Right Actions Toolbar -->
  <div class="flex items-center gap-1.5 shrink-0">
    <!-- Execute Query Button -->
    <button 
      disabled={!activeTab || activeTab.isExecuting}
      onclick={() => activeTab && tabsStore.runTabQuery(activeTab.id)}
      class="flex items-center gap-1.5 px-3 py-1 bg-indigo-600 hover:bg-indigo-500 disabled:bg-slate-300 dark:disabled:bg-slate-800 disabled:text-slate-500 dark:disabled:text-slate-600 text-white font-semibold text-xs rounded-md shadow-sm transition-all active:scale-95 mr-1"
      title="Run Current Query (Cmd+Enter)"
    >
      <Play size={13} class={activeTab?.isExecuting ? 'animate-spin' : 'fill-white'} />
      <span>{activeTab?.isExecuting ? 'Running...' : 'Execute'}</span>
    </button>

    <!-- Power Tools Icons -->
    <button 
      onclick={() => historyStore.toggleDrawer()}
      class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors relative" 
      title="Query History & Favorites (Cmd+H / Ctrl+H)"
    >
      <HistoryIcon size={15} />
      {#if historyStore.items.length > 0}
        <span class="absolute top-1 right-1 w-1.5 h-1.5 bg-indigo-500 rounded-full"></span>
      {/if}
    </button>

    <button 
      onclick={onOpenTransfer}
      class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="Multi-source Transfer & Import/Export"
    >
      <ArrowLeftRight size={15} />
    </button>

    <button 
      onclick={() => diffStore.toggle()}
      class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="Schema & Data Diff Sync Tool (Cmd+Shift+D / Ctrl+Shift+D)"
    >
      <GitCompare size={15} />
    </button>

    <button 
      onclick={onOpenErd}
      class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="Interactive ERD Visualizer"
    >
      <Network size={15} />
    </button>

    <button 
      onclick={onOpenDictionary}
      class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="1-Click Data Dictionary Generator (Markdown/HTML/PDF)"
    >
      <BookOpen size={15} />
    </button>

    <button 
      onclick={onOpenMockData}
      class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors" 
      title="QA Smart Mock Data Generator"
    >
      <Dices size={15} />
    </button>

    <!-- Live Server Health & Process Monitor Button -->
    <button 
      onclick={() => monitorStore.toggle()}
      class="p-1.5 text-emerald-600 dark:text-emerald-400 hover:text-emerald-700 dark:hover:text-emerald-300 hover:bg-emerald-50 dark:hover:bg-emerald-950/40 rounded-md transition-colors font-semibold flex items-center gap-1 cursor-pointer" 
      title="Live Server Health & Process Monitor (Cmd+Shift+M / Ctrl+Shift+M)"
    >
      <Activity size={15} />
      <span class="text-[10px] hidden sm:inline font-mono px-1 py-0.2 rounded bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 font-bold">Monitor</span>
    </button>

    <!-- AI Copilot Button -->
    <button 
      onclick={() => aiStore.openDrawer()}
      class="p-1.5 text-violet-600 dark:text-violet-400 hover:text-violet-700 dark:hover:text-violet-300 hover:bg-violet-50 dark:hover:bg-violet-950/40 rounded-md transition-colors font-semibold flex items-center gap-1 cursor-pointer" 
      title="NL-to-SQL AI Copilot (Cmd+K / Ctrl+K)"
    >
      <Sparkles size={15} />
      <span class="text-[10px] hidden sm:inline font-mono px-1 py-0.2 rounded bg-violet-500/15 text-violet-700 dark:text-violet-300 font-bold">AI</span>
    </button>

    <!-- Theme Mode Switcher Dropdown (Dark / Light / Auto) -->
    <div class="relative flex items-center ml-1">
      <button 
        type="button"
        onclick={() => isThemeDropdownOpen = !isThemeDropdownOpen}
        class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-md transition-colors flex items-center gap-1" 
        title="Change Theme ({themeStore.mode})"
      >
        {#if themeStore.mode === 'light'}
          <Sun size={15} class="text-amber-500" />
        {:else if themeStore.mode === 'dark'}
          <Moon size={15} class="text-indigo-400" />
        {:else}
          <Monitor size={15} class="text-slate-500 dark:text-slate-400" />
        {/if}
      </button>

      {#if isThemeDropdownOpen}
        <div 
          class="fixed inset-0 z-40" 
          onclick={() => isThemeDropdownOpen = false}
          role="presentation"
        ></div>

        <div class="absolute right-0 top-full mt-1.5 z-50 w-44 bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-xl shadow-2xl p-1 space-y-0.5 text-xs">
          <button
            type="button"
            onclick={() => { themeStore.setMode('dark'); isThemeDropdownOpen = false; }}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs whitespace-nowrap transition-colors {themeStore.mode === 'dark' ? 'bg-indigo-600/15 text-indigo-700 dark:text-indigo-300 font-bold' : 'hover:bg-surface-800 text-slate-800 dark:text-slate-100'}"
          >
            <div class="flex items-center gap-2">
              <Moon size={13} class="text-indigo-600 dark:text-indigo-400 shrink-0" />
              <span>Dark</span>
            </div>
            {#if themeStore.mode === 'dark'}<Check size={13} class="text-indigo-600 dark:text-indigo-400 shrink-0" />{/if}
          </button>

          <button
            type="button"
            onclick={() => { themeStore.setMode('light'); isThemeDropdownOpen = false; }}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs whitespace-nowrap transition-colors {themeStore.mode === 'light' ? 'bg-indigo-600/15 text-indigo-700 dark:text-indigo-300 font-bold' : 'hover:bg-surface-800 text-slate-800 dark:text-slate-100'}"
          >
            <div class="flex items-center gap-2">
              <Sun size={13} class="text-amber-500 shrink-0" />
              <span>Light</span>
            </div>
            {#if themeStore.mode === 'light'}<Check size={13} class="text-indigo-600 dark:text-indigo-400 shrink-0" />{/if}
          </button>

          <button
            type="button"
            onclick={() => { themeStore.setMode('auto'); isThemeDropdownOpen = false; }}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs whitespace-nowrap transition-colors {themeStore.mode === 'auto' ? 'bg-indigo-600/15 text-indigo-700 dark:text-indigo-300 font-bold' : 'hover:bg-surface-800 text-slate-800 dark:text-slate-100'}"
          >
            <div class="flex items-center gap-2">
              <Monitor size={13} class="text-slate-500 dark:text-slate-400 shrink-0" />
              <span>Auto (System)</span>
            </div>
            {#if themeStore.mode === 'auto'}<Check size={13} class="text-indigo-600 dark:text-indigo-400 shrink-0" />{/if}
          </button>
        </div>
      {/if}
    </div>
  </div>
</header>
