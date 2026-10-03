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

<div class="h-full flex flex-col bg-zinc-950 text-zinc-200 overflow-y-auto custom-scrollbar p-5 space-y-6">
  <!-- Header & Refresh -->
  <div class="flex items-center justify-between border-b border-zinc-800 pb-4">
    <div>
      <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
        <Server class="w-5 h-5 text-indigo-400" />
        Redis Server Telemetry & Metrics
      </h2>
      <p class="text-xs text-zinc-500 mt-0.5">Real-time node statistics, memory consumption, and runtime metrics.</p>
    </div>

    <button
      type="button"
      onclick={() => redisStore.loadServerInfo(connectionId)}
      class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded border border-zinc-700 transition-colors cursor-pointer"
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
      <div class="p-3 bg-zinc-900/80 border border-zinc-800 rounded-lg flex flex-col gap-1">
        <span class="text-[11px] text-zinc-500 font-medium">Redis Version</span>
        <span class="text-sm font-bold text-indigo-400 font-mono">{info.version}</span>
        <span class="text-[10px] text-zinc-600 truncate">{info.os}</span>
      </div>

      <!-- 2. Used Memory -->
      <div class="p-3 bg-zinc-900/80 border border-zinc-800 rounded-lg flex flex-col gap-1">
        <span class="text-[11px] text-zinc-500 font-medium flex items-center gap-1">
          <HardDrive class="w-3 h-3 text-cyan-400" />
          Used Memory
        </span>
        <span class="text-sm font-bold text-cyan-400 font-mono">{info.usedMemoryHuman}</span>
        <span class="text-[10px] text-zinc-600">Peak: {info.usedMemoryPeakHuman}</span>
      </div>

      <!-- 3. Connected Clients -->
      <div class="p-3 bg-zinc-900/80 border border-zinc-800 rounded-lg flex flex-col gap-1">
        <span class="text-[11px] text-zinc-500 font-medium flex items-center gap-1">
          <Users class="w-3 h-3 text-emerald-400" />
          Clients
        </span>
        <span class="text-sm font-bold text-emerald-400 font-mono">{info.connectedClients}</span>
        <span class="text-[10px] text-zinc-600">Active connections</span>
      </div>

      <!-- 4. Uptime -->
      <div class="p-3 bg-zinc-900/80 border border-zinc-800 rounded-lg flex flex-col gap-1">
        <span class="text-[11px] text-zinc-500 font-medium flex items-center gap-1">
          <Clock class="w-3 h-3 text-amber-400" />
          Uptime
        </span>
        <span class="text-sm font-bold text-amber-400 font-mono">{formatUptime(info.uptimeSeconds)}</span>
        <span class="text-[10px] text-zinc-600">{info.uptimeSeconds.toLocaleString()}s total</span>
      </div>

      <!-- 5. Total Keys -->
      <div class="p-3 bg-zinc-900/80 border border-zinc-800 rounded-lg flex flex-col gap-1">
        <span class="text-[11px] text-zinc-500 font-medium flex items-center gap-1">
          <Database class="w-3 h-3 text-rose-400" />
          Total Keys
        </span>
        <span class="text-sm font-bold text-rose-400 font-mono">{info.totalKeys}</span>
        <span class="text-[10px] text-zinc-600">Across current DB</span>
      </div>

      <!-- 6. Ops Per Sec -->
      <div class="p-3 bg-zinc-900/80 border border-zinc-800 rounded-lg flex flex-col gap-1">
        <span class="text-[11px] text-zinc-500 font-medium flex items-center gap-1">
          <Cpu class="w-3 h-3 text-purple-400" />
          Throughput
        </span>
        <span class="text-sm font-bold text-purple-400 font-mono">
          {info.rawInfo['instantaneous_ops_per_sec'] || '0'}
        </span>
        <span class="text-[10px] text-zinc-600">ops / sec</span>
      </div>
    </div>

    <!-- Raw Info Key-Value Table -->
    <div class="space-y-3 pt-2">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-semibold text-zinc-300 uppercase tracking-wider">All Redis Server Properties</h3>
        <div class="relative w-64">
          <Search class="w-3.5 h-3.5 absolute left-2.5 top-2 text-zinc-500 pointer-events-none" />
          <input
            type="text"
            bind:value={filterText}
            placeholder="Filter property name or value..."
            class="w-full bg-zinc-900 border border-zinc-800 rounded pl-8 pr-3 py-1 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-indigo-500"
          />
        </div>
      </div>

      <div class="border border-zinc-800 rounded-lg overflow-hidden bg-zinc-900/40">
        <table class="w-full text-left text-xs border-collapse">
          <thead>
            <tr class="bg-zinc-900/80 border-b border-zinc-800 text-zinc-400 font-mono text-[11px]">
              <th class="p-2.5 font-medium w-1/3">Property</th>
              <th class="p-2.5 font-medium">Value</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-zinc-800/60 font-mono">
            {#each Object.entries(info.rawInfo).filter(([k, v]) => !filterText || k.toLowerCase().includes(filterText.toLowerCase()) || v.toLowerCase().includes(filterText.toLowerCase())) as [key, val] (key)}
              <tr class="hover:bg-zinc-800/30 transition-colors">
                <td class="p-2.5 text-zinc-400 font-medium">{key}</td>
                <td class="p-2.5 text-zinc-200 select-all">{val}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {:else}
    <div class="flex flex-col items-center justify-center py-20 gap-2 text-zinc-500">
      <RefreshCw class="w-6 h-6 animate-spin text-indigo-400" />
      <span class="text-xs">Fetching Redis server telemetry...</span>
    </div>
  {/if}
</div>
