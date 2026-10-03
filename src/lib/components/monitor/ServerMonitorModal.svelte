<script lang="ts">
  import { monitorStore, type ProcessFilterStatus } from '$lib/state/monitor.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { 
    Activity, 
    RefreshCw, 
    X, 
    Search, 
    AlertTriangle, 
    Check, 
    Clock, 
    Lock, 
    Zap, 
    ShieldAlert, 
    Database, 
    User, 
    Cpu, 
    Server, 
    Copy, 
    ExternalLink, 
    PlayCircle, 
    AlertCircle,
    SlidersHorizontal,
    Maximize2
  } from 'lucide-svelte';

  const stats = $derived(monitorStore.stats);
  const processes = $derived(monitorStore.filteredProcesses);
  const activeConn = $derived(connectionStore.activeConnection);
  const selectedProc = $derived(monitorStore.selectedProcess);
  const killTarget = $derived(monitorStore.killConfirmTarget);

  const blockedProcessesCount = $derived(processes.filter(p => Boolean(p.blockedBy) || (p.waitEvent && p.waitEvent.toLowerCase().includes('lock'))).length);
  const slowProcessesCount = $derived(processes.filter(p => p.durationSeconds >= 3.0).length);
  const totalProcessesCount = $derived(stats?.processes.length || 0);
  const activeProcessesCount = $derived(stats?.activeConnections || 0);
  const idleProcessesCount = $derived(stats?.idleConnections || 0);
  const maxConnLimit = $derived(stats?.maxConnections);
  const connectionUsagePct = $derived(maxConnLimit ? Math.min(100, Math.round(((stats?.totalConnections || 0) / maxConnLimit) * 100)) : 15);

  let copiedSql = $state(false);

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    copiedSql = true;
    setTimeout(() => { copiedSql = false; }, 2000);
  }

  function openQueryInNewTab(sql: string) {
    tabsStore.openNewSqlTab(sql, 'Monitored Query');
    monitorStore.close();
  }

  function getDurationColor(sec: number): string {
    if (sec < 1.0) return 'text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
    if (sec < 5.0) return 'text-amber-600 dark:text-amber-400 bg-amber-500/10 border-amber-500/20';
    return 'text-rose-600 dark:text-rose-400 bg-rose-500/10 border-rose-500/20 font-bold';
  }

  function getStateBadgeClass(state: string, blockedBy?: string): string {
    if (blockedBy) return 'bg-rose-500/15 text-rose-700 dark:text-rose-300 border-rose-500/30';
    const s = state.toLowerCase();
    if (s.includes('active') || s.includes('run') || s.includes('exec')) {
      return 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-500/30';
    }
    if (s.includes('transaction') || s.includes('wait') || s.includes('lock')) {
      return 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/30';
    }
    return 'bg-slate-200/60 dark:bg-surface-800 text-slate-700 dark:text-slate-400 border-slate-300/60 dark:border-slate-700';
  }
</script>

{#if monitorStore.isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-slate-950/70 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
    onclick={(e) => {
      if (e.target === e.currentTarget && !killTarget) {
        monitorStore.close();
      }
    }}
  >
    <!-- Modal Dialog Window -->
    <div class="w-full max-w-7xl h-[92vh] bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-150 relative">
      
      <!-- Top Header -->
      <div class="px-5 py-3 border-b border-slate-200 dark:border-slate-800 bg-surface-900 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20">
            <Activity size={20} class="animate-pulse" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-base font-bold text-slate-900 dark:text-slate-100">Live Server Health & Process Monitor</h2>
              <span class="px-2 py-0.5 rounded-full text-[11px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30 uppercase tracking-wider">
                {activeConn?.driver || 'SQL'}
              </span>
              {#if activeConn?.environment === 'production'}
                <span class="px-2 py-0.5 rounded-full text-[11px] font-bold bg-rose-500/15 text-rose-600 dark:text-rose-400 border border-rose-500/30 uppercase tracking-wider">
                  Production
                </span>
              {/if}
            </div>
            <p class="text-xs text-slate-500 dark:text-slate-400">
              {activeConn?.name || 'Local Database'} • Host: <span class="font-mono">{activeConn?.host || 'localhost'}</span>
            </p>
          </div>
        </div>

        <!-- Controls: Auto-refresh & Actions -->
        <div class="flex items-center gap-3">
          <!-- Auto Refresh Interval Selector -->
          <div class="flex items-center bg-surface-800 border border-slate-200 dark:border-slate-700/80 rounded-lg p-0.5 text-xs font-medium">
            <span class="text-[11px] text-slate-500 px-2 flex items-center gap-1">
              <Clock size={11} />
              <span>Interval:</span>
            </span>
            {#each [0, 2, 3, 5, 10] as sec}
              <button
                type="button"
                onclick={() => monitorStore.setAutoRefresh(sec)}
                class="px-2 py-1 rounded text-xs transition-colors cursor-pointer {monitorStore.autoRefreshInterval === sec 
                  ? 'bg-indigo-600 text-white font-bold shadow-xs' 
                  : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'}"
              >
                {sec === 0 ? 'Pause' : `${sec}s`}
              </button>
            {/each}
          </div>

          <!-- Manual Refresh Button -->
          <button
            type="button"
            onclick={() => monitorStore.fetchProcesses()}
            disabled={monitorStore.isLoading}
            class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 dark:bg-surface-800 hover:bg-slate-200 dark:hover:bg-surface-700 text-slate-700 dark:text-slate-300 font-semibold rounded-lg text-xs border border-slate-300/80 dark:border-slate-700 transition-colors cursor-pointer disabled:opacity-50"
            title="Refresh processes now (Cmd+R)"
          >
            <RefreshCw size={13} class={monitorStore.isLoading ? 'animate-spin text-indigo-500' : ''} />
            <span>Refresh</span>
          </button>

          <!-- Close Button -->
          <button
            type="button"
            onclick={() => monitorStore.close()}
            class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-1 rounded-lg hover:bg-slate-100 dark:hover:bg-surface-800 transition-colors cursor-pointer"
            title="Close (Esc)"
          >
            <X size={18} />
          </button>
        </div>
      </div>

      <!-- Toast Alerts (Success / Error) -->
      {#if monitorStore.successMessage}
        <div class="px-5 py-2 bg-emerald-500/15 border-b border-emerald-500/30 flex items-center justify-between text-xs text-emerald-800 dark:text-emerald-300 animate-in fade-in duration-150">
          <div class="flex items-center gap-2">
            <Check size={14} class="text-emerald-500 shrink-0" />
            <span class="font-medium">{monitorStore.successMessage}</span>
          </div>
          <button type="button" onclick={() => monitorStore.successMessage = null} class="text-emerald-600 hover:text-emerald-800 dark:hover:text-emerald-200">
            <X size={13} />
          </button>
        </div>
      {/if}

      {#if monitorStore.errorMessage}
        <div class="px-5 py-2 bg-rose-500/15 border-b border-rose-500/30 flex items-center justify-between text-xs text-rose-800 dark:text-rose-300 animate-in fade-in duration-150">
          <div class="flex items-center gap-2">
            <AlertCircle size={14} class="text-rose-500 shrink-0" />
            <span class="font-medium">{monitorStore.errorMessage}</span>
          </div>
          <button type="button" onclick={() => monitorStore.errorMessage = null} class="text-rose-600 hover:text-rose-800 dark:hover:text-rose-200">
            <X size={13} />
          </button>
        </div>
      {/if}

      <!-- Top Metric KPI Cards -->
      <div class="grid grid-cols-5 gap-3 p-4 border-b border-slate-200 dark:border-slate-800/80 bg-surface-950/60 shrink-0">
        <!-- Card 1: Active Queries -->
        <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
          <div>
            <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Active Queries</span>
            <div class="text-xl font-bold text-slate-900 dark:text-slate-100 mt-0.5 flex items-baseline gap-1.5">
              <span>{stats?.activeConnections || 0}</span>
              <span class="text-xs font-normal text-slate-500">running</span>
            </div>
          </div>
          <div class="w-8 h-8 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-500">
            <Zap size={16} class="fill-emerald-500 animate-pulse" />
          </div>
        </div>

        <!-- Card 2: Blocked / Locks -->
        <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
          <div>
            <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Lock Contentions</span>
            <div class="text-xl font-bold {blockedProcessesCount > 0 ? 'text-rose-600 dark:text-rose-400' : 'text-slate-900 dark:text-slate-100'} mt-0.5 flex items-baseline gap-1.5">
              <span>{blockedProcessesCount}</span>
              <span class="text-xs font-normal text-slate-500">blocked</span>
            </div>
          </div>
          <div class="w-8 h-8 rounded-full {blockedProcessesCount > 0 ? 'bg-rose-500/20 border-rose-500/40 text-rose-500' : 'bg-slate-500/10 border-slate-500/20 text-slate-400'} border flex items-center justify-center">
            <Lock size={15} />
          </div>
        </div>

        <!-- Card 3: Slow Queries > 3s -->
        <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
          <div>
            <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Slow Queries (>3s)</span>
            <div class="text-xl font-bold {slowProcessesCount > 0 ? 'text-amber-600 dark:text-amber-400' : 'text-slate-900 dark:text-slate-100'} mt-0.5 flex items-baseline gap-1.5">
              <span>{slowProcessesCount}</span>
              <span class="text-xs font-normal text-slate-500">queries</span>
            </div>
          </div>
          <div class="w-8 h-8 rounded-full {slowProcessesCount > 0 ? 'bg-amber-500/20 border-amber-500/40 text-amber-500' : 'bg-slate-500/10 border-slate-500/20 text-slate-400'} border flex items-center justify-center">
            <Clock size={16} />
          </div>
        </div>

        <!-- Card 4: Total Connections / Max -->
        <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
          <div class="w-full mr-2">
            <div class="flex items-center justify-between">
              <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Connections</span>
              <span class="text-[11px] font-mono text-slate-500">{stats?.totalConnections || 0} / {stats?.maxConnections || '∞'}</span>
            </div>
            <div class="w-full bg-slate-200 dark:bg-surface-800 h-1.5 rounded-full mt-2 overflow-hidden">
              <div class="h-full bg-gradient-to-r from-indigo-500 to-violet-500 rounded-full" style="width: {connectionUsagePct}%"></div>
            </div>
          </div>
          <div class="w-8 h-8 rounded-full bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-500 shrink-0">
            <Server size={15} />
          </div>
        </div>

        <!-- Card 5: Engine & Version -->
        <div class="p-3 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-between">
          <div class="truncate mr-2">
            <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">Engine Version</span>
            <p class="text-xs font-mono font-medium text-slate-900 dark:text-slate-100 truncate mt-0.5" title={stats?.version || ''}>
              {stats?.version || 'Connected'}
            </p>
          </div>
          <div class="w-8 h-8 rounded-full bg-slate-500/10 border border-slate-500/20 flex items-center justify-center text-slate-400 shrink-0">
            <Database size={15} />
          </div>
        </div>
      </div>

      <!-- Filter & Search Toolbar -->
      <div class="px-5 py-2.5 bg-surface-900/90 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between gap-4 text-xs shrink-0">
        <!-- Status Filter Pills -->
        <div class="flex items-center gap-1.5">
          <button
            type="button"
            onclick={() => monitorStore.filterStatus = 'all'}
            class="px-2.5 py-1 rounded-md font-semibold transition-colors cursor-pointer border {monitorStore.filterStatus === 'all' 
              ? 'bg-indigo-600 text-white border-indigo-600' 
              : 'bg-surface-800 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            All ({totalProcessesCount})
          </button>

          <button
            type="button"
            onclick={() => monitorStore.filterStatus = 'active'}
            class="flex items-center gap-1 px-2.5 py-1 rounded-md font-semibold transition-colors cursor-pointer border {monitorStore.filterStatus === 'active' 
              ? 'bg-emerald-600 text-white border-emerald-600' 
              : 'bg-surface-800 text-emerald-700 dark:text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/10'}"
          >
            <Zap size={11} class="fill-current" />
            <span>Active ({activeProcessesCount})</span>
          </button>

          <button
            type="button"
            onclick={() => monitorStore.filterStatus = 'blocked'}
            class="flex items-center gap-1 px-2.5 py-1 rounded-md font-semibold transition-colors cursor-pointer border {monitorStore.filterStatus === 'blocked' 
              ? 'bg-rose-600 text-white border-rose-600' 
              : 'bg-surface-800 text-rose-700 dark:text-rose-400 border-rose-500/20 hover:bg-rose-500/10'}"
          >
            <Lock size={11} />
            <span>Blocked / Locks ({blockedProcessesCount})</span>
          </button>

          <button
            type="button"
            onclick={() => monitorStore.filterStatus = 'slow_3s'}
            class="flex items-center gap-1 px-2.5 py-1 rounded-md font-semibold transition-colors cursor-pointer border {monitorStore.filterStatus === 'slow_3s' 
              ? 'bg-amber-600 text-white border-amber-600' 
              : 'bg-surface-800 text-amber-700 dark:text-amber-400 border-amber-500/20 hover:bg-amber-500/10'}"
          >
            <Clock size={11} />
            <span>Slow >3s ({slowProcessesCount})</span>
          </button>

          <button
            type="button"
            onclick={() => monitorStore.filterStatus = 'idle'}
            class="px-2.5 py-1 rounded-md font-semibold transition-colors cursor-pointer border {monitorStore.filterStatus === 'idle' 
              ? 'bg-slate-700 text-white border-slate-700' 
              : 'bg-surface-800 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            Idle ({idleProcessesCount})
          </button>
        </div>

        <!-- Search Bar -->
        <div class="relative w-80">
          <Search size={13} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            type="text"
            bind:value={monitorStore.filterText}
            placeholder="Search PID, User, DB, Query..."
            class="w-full pl-8 pr-7 py-1 bg-surface-800 border border-slate-200 dark:border-slate-700 rounded-md text-xs text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-indigo-500"
          />
          {#if monitorStore.filterText}
            <button
              type="button"
              onclick={() => monitorStore.filterText = ''}
              class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
            >
              <X size={12} />
            </button>
          {/if}
        </div>
      </div>

      <!-- Main Content Area: Process List & Detail Inspector Drawer -->
      <div class="flex-1 flex min-h-0 overflow-hidden">
        <!-- Process Table -->
        <div class="flex-1 overflow-auto border-r border-slate-200 dark:border-slate-800 bg-surface-950">
          {#if processes.length === 0}
            <div class="h-64 flex flex-col items-center justify-center text-slate-400 text-xs">
              <Activity size={32} class="text-slate-500 mb-2 opacity-50" />
              <span class="font-medium">No matching database processes found.</span>
              <span class="text-[11px] text-slate-500 mt-0.5">Try clearing your search filter or refreshing.</span>
            </div>
          {:else}
            <table class="w-full text-left text-xs border-collapse">
              <thead class="sticky top-0 bg-surface-900 border-b border-slate-200 dark:border-slate-800 text-[11px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-wider select-none z-10">
                <tr>
                  <th class="py-2 px-3 w-16">PID</th>
                  <th class="py-2 px-3 w-32">User / Database</th>
                  <th class="py-2 px-3 w-36">Client / App</th>
                  <th class="py-2 px-3 w-36">Status</th>
                  <th class="py-2 px-3 w-24">Duration</th>
                  <th class="py-2 px-3 w-32">Wait Event</th>
                  <th class="py-2 px-3">Current Query</th>
                  <th class="py-2 px-3 text-right w-24">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-200/80 dark:divide-slate-800/60 font-mono">
                {#each processes as proc (proc.pid)}
                  <tr 
                    onclick={() => monitorStore.selectedProcess = proc}
                    class="hover:bg-slate-100/80 dark:hover:bg-surface-900/80 transition-colors cursor-pointer {selectedProc?.pid === proc.pid ? 'bg-indigo-50/70 dark:bg-indigo-950/40 ring-1 ring-inset ring-indigo-500/50' : ''}"
                  >
                    <!-- PID -->
                    <td class="py-2 px-3 font-bold text-slate-900 dark:text-slate-100">
                      {proc.pid}
                    </td>

                    <!-- User / DB -->
                    <td class="py-2 px-3 truncate">
                      <div class="font-medium text-slate-800 dark:text-slate-200 truncate">{proc.user || '—'}</div>
                      <div class="text-[10.5px] text-slate-500 truncate">{proc.database || '—'}</div>
                    </td>

                    <!-- Client / App -->
                    <td class="py-2 px-3 truncate">
                      <div class="text-[11px] text-slate-700 dark:text-slate-300 truncate">{proc.clientAddr || 'local'}</div>
                      {#if proc.applicationName}
                        <div class="text-[10px] text-indigo-600 dark:text-indigo-400 truncate">{proc.applicationName}</div>
                      {/if}
                    </td>

                    <!-- Status -->
                    <td class="py-2 px-3">
                      <div class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10.5px] font-semibold border {getStateBadgeClass(proc.state, proc.blockedBy)}">
                        {#if proc.blockedBy}
                          <Lock size={10} class="shrink-0 text-rose-500" />
                          <span>Blocked by {proc.blockedBy}</span>
                        {:else}
                          <span class="capitalize">{proc.state}</span>
                        {/if}
                      </div>
                    </td>

                    <!-- Duration -->
                    <td class="py-2 px-3">
                      <span class="inline-block px-1.5 py-0.5 rounded text-[11px] font-semibold border {getDurationColor(proc.durationSeconds)}">
                        {proc.durationSeconds >= 60 
                          ? `${(proc.durationSeconds / 60).toFixed(1)}m` 
                          : `${proc.durationSeconds.toFixed(2)}s`}
                      </span>
                    </td>

                    <!-- Wait Event -->
                    <td class="py-2 px-3 text-[11px] text-slate-500 dark:text-slate-400 truncate" title={proc.waitEvent || ''}>
                      {proc.waitEvent || '—'}
                    </td>

                    <!-- Current Query Preview -->
                    <td class="py-2 px-3 truncate max-w-xs text-[11px] text-slate-700 dark:text-slate-300" title={proc.query || ''}>
                      {proc.query ? proc.query.replace(/\s+/g, ' ') : '<idle>'}
                    </td>

                    <!-- Actions: Kill / Cancel -->
                    <td class="py-2 px-3 text-right shrink-0" onclick={(e) => e.stopPropagation()}>
                      <div class="flex items-center justify-end gap-1">
                        <button
                          type="button"
                          onclick={() => monitorStore.killConfirmTarget = proc}
                          class="p-1 rounded text-rose-600 hover:text-white hover:bg-rose-600 dark:hover:bg-rose-600 transition-colors cursor-pointer"
                          title="Terminate Process (Kill PID {proc.pid})"
                        >
                          <Zap size={13} />
                        </button>
                      </div>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>

        <!-- Right Side Detail Inspector Drawer -->
        {#if selectedProc}
          <div class="w-96 bg-surface-900 flex flex-col border-l border-slate-200 dark:border-slate-800 animate-in slide-in-from-right-4 duration-150 shrink-0">
            <!-- Header -->
            <div class="px-4 py-3 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">Process Inspector</span>
                <span class="px-2 py-0.5 rounded font-mono font-bold text-xs bg-indigo-500/15 text-indigo-700 dark:text-indigo-300">
                  PID {selectedProc.pid}
                </span>
              </div>
              <button
                type="button"
                onclick={() => monitorStore.selectedProcess = null}
                class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-0.5 rounded"
              >
                <X size={15} />
              </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-auto p-4 space-y-4 text-xs font-sans">
              <!-- Meta Stats Grid -->
              <div class="grid grid-cols-2 gap-2 text-[11px] font-mono">
                <div class="p-2 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded">
                  <span class="text-slate-500 block text-[10px] uppercase">Database</span>
                  <span class="font-bold text-slate-900 dark:text-slate-100 truncate block">{selectedProc.database || '—'}</span>
                </div>
                <div class="p-2 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded">
                  <span class="text-slate-500 block text-[10px] uppercase">User</span>
                  <span class="font-bold text-slate-900 dark:text-slate-100 truncate block">{selectedProc.user || '—'}</span>
                </div>
                <div class="p-2 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded">
                  <span class="text-slate-500 block text-[10px] uppercase">Status</span>
                  <span class="font-bold text-slate-900 dark:text-slate-100 truncate block capitalize">{selectedProc.state}</span>
                </div>
                <div class="p-2 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded">
                  <span class="text-slate-500 block text-[10px] uppercase">Runtime</span>
                  <span class="font-bold {getDurationColor(selectedProc.durationSeconds)} block">{selectedProc.durationSeconds.toFixed(2)}s</span>
                </div>
                {#if selectedProc.clientAddr}
                  <div class="col-span-2 p-2 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded">
                    <span class="text-slate-500 block text-[10px] uppercase">Client IP / Application</span>
                    <span class="font-medium text-slate-800 dark:text-slate-200 block truncate">{selectedProc.clientAddr} {selectedProc.applicationName ? `(${selectedProc.applicationName})` : ''}</span>
                  </div>
                {/if}
                {#if selectedProc.waitEvent}
                  <div class="col-span-2 p-2 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded">
                    <span class="text-slate-500 block text-[10px] uppercase">Wait Event</span>
                    <span class="font-medium text-amber-600 dark:text-amber-400 block truncate">{selectedProc.waitEvent}</span>
                  </div>
                {/if}
                {#if selectedProc.blockedBy}
                  <div class="col-span-2 p-2 bg-rose-500/15 border border-rose-500/30 rounded text-rose-700 dark:text-rose-300">
                    <span class="block text-[10px] uppercase font-bold">Lock Contention</span>
                    <span class="font-medium block">Blocked by PID: <span class="font-bold font-mono">{selectedProc.blockedBy}</span></span>
                  </div>
                {/if}
              </div>

              <!-- Query Full Text Box -->
              <div>
                <div class="flex items-center justify-between mb-1.5">
                  <span class="text-[11px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-wider">Executed SQL Query</span>
                  {#if selectedProc.query}
                    <div class="flex items-center gap-1">
                      <button
                        type="button"
                        onclick={() => copyToClipboard(selectedProc?.query || '')}
                        class="flex items-center gap-1 px-2 py-0.5 rounded text-[10.5px] font-medium bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 border border-slate-300/80 dark:border-slate-700 transition-colors"
                        title="Copy SQL to clipboard"
                      >
                        {#if copiedSql}
                          <Check size={11} class="text-emerald-500" />
                          <span>Copied</span>
                        {:else}
                          <Copy size={11} />
                          <span>Copy</span>
                        {/if}
                      </button>

                      <button
                        type="button"
                        onclick={() => openQueryInNewTab(selectedProc?.query || '')}
                        class="flex items-center gap-1 px-2 py-0.5 rounded text-[10.5px] font-medium bg-indigo-500/15 hover:bg-indigo-500/25 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30 transition-colors"
                        title="Open in new SQL Editor tab"
                      >
                        <ExternalLink size={11} />
                        <span>Open Tab</span>
                      </button>
                    </div>
                  {/if}
                </div>

                <div class="p-3 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-lg font-mono text-[11px] text-slate-800 dark:text-slate-200 max-h-56 overflow-auto whitespace-pre-wrap select-text leading-relaxed">
                  {selectedProc.query || '<No active SQL query / Idle>'}
                </div>
              </div>

              <!-- Quick Action: Kill / Cancel -->
              <div class="pt-2 border-t border-slate-200 dark:border-slate-800 flex items-center gap-2">
                <button
                  type="button"
                  onclick={() => monitorStore.killConfirmTarget = selectedProc}
                  class="flex-1 flex items-center justify-center gap-1.5 py-2 bg-rose-600 hover:bg-rose-500 active:bg-rose-700 text-white font-bold rounded-lg text-xs shadow-sm transition-colors cursor-pointer"
                >
                  <Zap size={13} />
                  <span>Kill Process (PID {selectedProc.pid})</span>
                </button>

                {#if selectedProc.query && selectedProc.state.toLowerCase().includes('active')}
                  <button
                    type="button"
                    onclick={() => selectedProc && monitorStore.cancelQuery(selectedProc)}
                    class="px-3 py-2 bg-slate-200 dark:bg-surface-800 hover:bg-slate-300 dark:hover:bg-surface-700 text-slate-800 dark:text-slate-200 font-semibold rounded-lg text-xs border border-slate-300/80 dark:border-slate-700 transition-colors cursor-pointer"
                    title="Cancel active query execution without killing backend connection"
                  >
                    <span>Cancel Query</span>
                  </button>
                {/if}
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Bottom Status Bar in Modal -->
      <div class="px-5 py-2 bg-surface-900 border-t border-slate-200 dark:border-slate-800 flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400 select-none shrink-0 font-mono">
        <div class="flex items-center gap-4">
          <span>Showing: {processes.length} of {stats?.totalConnections || 0} processes</span>
          {#if monitorStore.lastUpdated}
            <span>Last polled: {new Date(monitorStore.lastUpdated).toLocaleTimeString()}</span>
          {/if}
        </div>
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full {monitorStore.autoRefreshInterval > 0 ? 'bg-emerald-500 animate-pulse' : 'bg-slate-500'}"></span>
          <span>{monitorStore.autoRefreshInterval > 0 ? `Auto-polling every ${monitorStore.autoRefreshInterval}s` : 'Polling paused'}</span>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- Kill Process Confirmation Modal -->
{#if killTarget}
  <div class="fixed inset-0 z-60 bg-slate-950/80 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-100">
    <div class="w-full max-w-md bg-surface-950 border border-rose-500/50 rounded-xl shadow-2xl p-5 animate-in zoom-in-95 duration-150">
      <div class="flex items-center gap-3 text-rose-600 dark:text-rose-400 mb-3">
        <div class="p-2 rounded-full bg-rose-500/15 border border-rose-500/30">
          <ShieldAlert size={22} />
        </div>
        <div>
          <h3 class="font-bold text-base text-slate-900 dark:text-slate-100">Terminate Database Process?</h3>
          <span class="text-xs text-slate-500 font-mono">PID: {killTarget.pid} • User: {killTarget.user}</span>
        </div>
      </div>

      <p class="text-xs text-slate-600 dark:text-slate-300 mb-3 leading-relaxed">
        This will immediately terminate backend process <span class="font-mono font-bold text-rose-600 dark:text-rose-400">PID {killTarget.pid}</span> on <span class="font-semibold">{killTarget.database}</span>. Any uncommitted transactions in this process will be rolled back.
      </p>

      {#if killTarget.query}
        <div class="mb-4 p-2.5 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded font-mono text-[11px] text-slate-800 dark:text-slate-200 max-h-24 overflow-auto whitespace-pre-wrap">
          {killTarget.query}
        </div>
      {/if}

      <div class="flex items-center justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={() => monitorStore.killConfirmTarget = null}
          disabled={monitorStore.isOperating}
          class="px-3.5 py-1.5 rounded-lg text-xs font-semibold text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-surface-800 transition-colors cursor-pointer"
        >
          Cancel
        </button>

        <button
          type="button"
          onclick={() => killTarget && monitorStore.killProcess(killTarget)}
          disabled={monitorStore.isOperating}
          class="px-4 py-1.5 rounded-lg text-xs font-bold bg-rose-600 hover:bg-rose-500 active:bg-rose-700 text-white shadow-sm transition-colors cursor-pointer disabled:opacity-50 flex items-center gap-1.5"
        >
          {#if monitorStore.isOperating}
            <RefreshCw size={12} class="animate-spin" />
            <span>Terminating...</span>
          {:else}
            <Zap size={12} />
            <span>Yes, Terminate Process</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
