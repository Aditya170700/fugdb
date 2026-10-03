<script lang="ts">
  import { onMount } from 'svelte';
  import { Server, Cpu, HardDrive, Users, Clock, Database, RefreshCw, Search } from 'lucide-svelte';
  import { redisStore } from '../../state/redis.svelte';

  let { connectionId }: { connectionId: string } = $props();

  const redisState = $derived(redisStore.getState(connectionId));
  let filterText = $state('');

  onMount(() => {
    redisStore.loadServerInfo(connectionId);
  });

  function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${mins}m`;
  }
</script>

<div class="h-full flex flex-col bg-surface-950 dark:bg-zinc-950 text-slate-800 dark:text-zinc-200 overflow-y-auto custom-scrollbar p-5 space-y-6">
  <!-- Header & Refresh -->
  <div class="flex items-center justify-between border-b border-slate-200 dark:border-zinc-800 pb-4">
    <div>
      <h2 class="text-base font-semibold text-slate-900 dark:text-zinc-100 flex items-center gap-2">
        <Server class="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
        Redis Server Telemetry & Metrics
      </h2>
      <p class="text-xs text-slate-500 dark:text-zinc-400 mt-0.5">Real-time node statistics, memory consumption, and runtime metrics.</p>
    </div>

    <button
      type="button"
      onclick={() => redisStore.loadServerInfo(connectionId)}
      class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-surface-900 hover:bg-surface-800 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-200 rounded border border-slate-200 dark:border-zinc-700 transition-colors cursor-pointer shadow-xs"
    >
      <RefreshCw class="w-3.5 h-3.5" />
      <span>Refresh Metrics</span>
    </button>
  </div>

  {#if redisState.serverInfo}
    {@const info = redisState.serverInfo}

    <!-- Stat Cards Grid -->
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
      <!-- 1. Version -->
      <div class="p-3 bg-surface-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex flex-col gap-1 shadow-xs">
        <span class="text-[11px] text-slate-500 dark:text-zinc-400 font-medium">Redis Version</span>
        <span class="text-sm font-bold text-indigo-600 dark:text-indigo-400 font-mono">{info.version}</span>
        <span class="text-[10px] text-slate-400 dark:text-zinc-500 truncate">{info.os}</span>
      </div>

      <!-- 2. Used Memory -->
      <div class="p-3 bg-surface-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex flex-col gap-1 shadow-xs">
        <span class="text-[11px] text-slate-500 dark:text-zinc-400 font-medium flex items-center gap-1">
          <HardDrive class="w-3 h-3 text-cyan-600 dark:text-cyan-400" />
          Used Memory
        </span>
        <span class="text-sm font-bold text-cyan-700 dark:text-cyan-400 font-mono">{info.usedMemoryHuman}</span>
        <span class="text-[10px] text-slate-400 dark:text-zinc-500">Peak: {info.usedMemoryPeakHuman}</span>
      </div>

      <!-- 3. Connected Clients -->
      <div class="p-3 bg-surface-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex flex-col gap-1 shadow-xs">
        <span class="text-[11px] text-slate-500 dark:text-zinc-400 font-medium flex items-center gap-1">
          <Users class="w-3 h-3 text-emerald-600 dark:text-emerald-400" />
          Clients
        </span>
        <span class="text-sm font-bold text-emerald-700 dark:text-emerald-400 font-mono">{info.connectedClients}</span>
        <span class="text-[10px] text-slate-400 dark:text-zinc-500">Active connections</span>
      </div>

      <!-- 4. Uptime -->
      <div class="p-3 bg-surface-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex flex-col gap-1 shadow-xs">
        <span class="text-[11px] text-slate-500 dark:text-zinc-400 font-medium flex items-center gap-1">
          <Clock class="w-3 h-3 text-amber-500 dark:text-amber-400" />
          Uptime
        </span>
        <span class="text-sm font-bold text-amber-700 dark:text-amber-400 font-mono">{formatUptime(info.uptimeSeconds)}</span>
        <span class="text-[10px] text-slate-400 dark:text-zinc-500">{info.uptimeSeconds.toLocaleString()}s total</span>
      </div>

      <!-- 5. Total Keys -->
      <div class="p-3 bg-surface-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex flex-col gap-1 shadow-xs">
        <span class="text-[11px] text-slate-500 dark:text-zinc-400 font-medium flex items-center gap-1">
          <Database class="w-3 h-3 text-rose-600 dark:text-rose-400" />
          Total Keys
        </span>
        <span class="text-sm font-bold text-rose-700 dark:text-rose-400 font-mono">{info.totalKeys}</span>
        <span class="text-[10px] text-slate-400 dark:text-zinc-500">Across current DB</span>
      </div>

      <!-- 6. Ops Per Sec -->
      <div class="p-3 bg-surface-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex flex-col gap-1 shadow-xs">
        <span class="text-[11px] text-slate-500 dark:text-zinc-400 font-medium flex items-center gap-1">
          <Cpu class="w-3 h-3 text-purple-600 dark:text-purple-400" />
          Throughput
        </span>
        <span class="text-sm font-bold text-purple-700 dark:text-purple-400 font-mono">
          {info.rawInfo['instantaneous_ops_per_sec'] || '0'}
        </span>
        <span class="text-[10px] text-slate-400 dark:text-zinc-500">ops / sec</span>
      </div>
    </div>

    <!-- Raw Info Key-Value Table -->
    <div class="space-y-3 pt-2">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-semibold text-slate-700 dark:text-zinc-300 uppercase tracking-wider">All Redis Server Properties</h3>
        <div class="relative w-64">
          <Search class="w-3.5 h-3.5 absolute left-2.5 top-2 text-slate-400 dark:text-zinc-500 pointer-events-none" />
          <input
            type="text"
            bind:value={filterText}
            placeholder="Filter property name or value..."
            class="w-full bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded pl-8 pr-3 py-1 text-xs text-slate-900 dark:text-zinc-200 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 shadow-xs"
          />
        </div>
      </div>

      <div class="border border-slate-200 dark:border-zinc-800 rounded-xl overflow-hidden bg-surface-900/40 dark:bg-zinc-900/40 shadow-xs">
        <table class="w-full text-left text-xs border-collapse">
          <thead>
            <tr class="bg-surface-900 dark:bg-zinc-900/80 border-b border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 font-mono text-[11px]">
              <th class="p-2.5 font-medium w-1/3">Property</th>
              <th class="p-2.5 font-medium">Value</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-200 dark:divide-zinc-800/60 font-mono">
            {#each Object.entries(info.rawInfo).filter(([k, v]) => !filterText || k.toLowerCase().includes(filterText.toLowerCase()) || v.toLowerCase().includes(filterText.toLowerCase())) as [key, val] (key)}
              <tr class="hover:bg-surface-800/50 dark:hover:bg-zinc-800/30 transition-colors">
                <td class="p-2.5 text-slate-500 dark:text-zinc-400 font-medium">{key}</td>
                <td class="p-2.5 text-slate-900 dark:text-zinc-200 select-all">{val}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {:else}
    <div class="flex flex-col items-center justify-center py-20 gap-2 text-slate-500 dark:text-zinc-500">
      <RefreshCw class="w-6 h-6 animate-spin text-indigo-600 dark:text-indigo-400" />
      <span class="text-xs">Fetching Redis server telemetry...</span>
    </div>
  {/if}
</div>
