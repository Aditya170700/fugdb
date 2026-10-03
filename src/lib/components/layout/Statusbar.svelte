<script lang="ts">
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { themeStore } from '$lib/state/theme.svelte';
  import { sessionStore } from '$lib/state/session.svelte';
  import { CheckCircle2, Clock, Layers, Cpu, Sun, Moon, Monitor, Zap, Lock, RotateCcw, Check, Loader2 } from 'lucide-svelte';

  const activeTab = $derived(tabsStore.activeTab);
  const result = $derived(activeTab?.queryResult);
  const activeConn = $derived(connectionStore.activeConnection);
  const activeConnId = $derived(connectionStore.activeConnectionId);
  const txState = $derived(sessionStore.getState(activeConnId));
</script>

<footer class="h-6 border-t border-slate-200 dark:border-slate-800 bg-surface-950 text-slate-600 dark:text-slate-400 text-[11px] px-3 flex items-center justify-between select-none z-20">
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400">
      <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
      <span class="font-medium">Ready</span>
    </div>

    {#if result}
      <div class="flex items-center gap-1">
        <Layers size={12} class="text-slate-400 dark:text-slate-500" />
        <span>{result.rows.length} rows</span>
      </div>

      <div class="flex items-center gap-1">
        <Clock size={12} class="text-slate-400 dark:text-slate-500" />
        <span>{result.executionTimeMs.toFixed(1)} ms</span>
      </div>
    {/if}

    <div class="h-3 w-px bg-slate-200 dark:bg-slate-800 mx-0.5"></div>

    <!-- 4.3 Transaction & Session Control Statusbar Widget -->
    {#if activeConnId}
      <div class="flex items-center gap-1.5">
        <!-- Toggle Auto-commit button -->
        <button
          type="button"
          onclick={() => sessionStore.toggleAutoCommit(activeConnId)}
          class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10.5px] font-medium transition-colors cursor-pointer border {txState.autoCommit 
            ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/20 hover:bg-emerald-500/20' 
            : 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/20 hover:bg-amber-500/20'}"
          title={txState.autoCommit ? 'Auto-commit is ON. Click to switch to Manual Transaction mode.' : 'Manual Transaction mode active. Click to switch to Auto-commit.'}
        >
          {#if txState.autoCommit}
            <Zap size={10.5} class="text-emerald-500 fill-emerald-500" />
            <span>Auto-commit: ON</span>
          {:else}
            <Lock size={10.5} class="text-amber-500" />
            <span>Manual Tx</span>
          {/if}
        </button>

        <!-- Transaction Active State & Quick Actions -->
        {#if !txState.autoCommit || txState.inTransaction}
          <div class="flex items-center gap-1 animate-in fade-in duration-200">
            {#if txState.uncommittedCount > 0}
              <span class="px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-600 dark:text-amber-400 font-semibold text-[10px]">
                {txState.uncommittedCount} uncommitted
              </span>
            {/if}

            <button
              type="button"
              onclick={() => sessionStore.commit(activeConnId, activeConn?.driver)}
              disabled={sessionStore.isOperating}
              class="flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10.5px] font-semibold bg-emerald-600 hover:bg-emerald-500 text-white transition-colors cursor-pointer disabled:opacity-50"
              title="Commit current transaction (Cmd+Shift+C / Ctrl+Shift+C)"
            >
              {#if sessionStore.isOperating}
                <Loader2 size={10} class="animate-spin" />
              {:else}
                <Check size={10} strokeWidth={3} />
              {/if}
              <span>Commit</span>
            </button>

            <button
              type="button"
              onclick={() => sessionStore.rollback(activeConnId, activeConn?.driver)}
              disabled={sessionStore.isOperating}
              class="flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10.5px] font-semibold bg-rose-600/90 hover:bg-rose-500 text-white transition-colors cursor-pointer disabled:opacity-50"
              title="Rollback current transaction (Cmd+Shift+R / Ctrl+Shift+R)"
            >
              {#if sessionStore.isOperating}
                <Loader2 size={10} class="animate-spin" />
              {:else}
                <RotateCcw size={10} strokeWidth={2.5} />
              {/if}
              <span>Rollback</span>
            </button>
          </div>
        {/if}

        <!-- Transient notification message -->
        {#if sessionStore.notification}
          <span class="text-[10.5px] font-medium text-slate-700 dark:text-slate-300 animate-in fade-in slide-in-from-left-1 duration-150">
            {sessionStore.notification.message}
          </span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-4">
    <div class="flex items-center gap-1 text-slate-500 dark:text-slate-400">
      <Cpu size={12} />
      <span>Memory: ~34 MB RAM</span>
    </div>

    <!-- Theme quick cycle button in statusbar -->
    <button 
      type="button"
      onclick={() => themeStore.toggleCycle()}
      class="flex items-center gap-1 hover:text-indigo-600 dark:hover:text-indigo-300 transition-colors text-slate-600 dark:text-slate-400"
      title="Theme: {themeStore.mode} (Click to toggle)"
    >
      {#if themeStore.mode === 'light'}
        <Sun size={11} class="text-amber-500" />
        <span class="capitalize font-medium">Light</span>
      {:else if themeStore.mode === 'dark'}
        <Moon size={11} class="text-indigo-400" />
        <span class="capitalize font-medium">Dark</span>
      {:else}
        <Monitor size={11} class="text-slate-500 dark:text-slate-400" />
        <span class="font-medium">Auto ({themeStore.resolvedTheme})</span>
      {/if}
    </button>

    <span class="text-indigo-600 dark:text-indigo-400 font-mono font-bold">FugDB v0.1.0</span>
  </div>
</footer>
