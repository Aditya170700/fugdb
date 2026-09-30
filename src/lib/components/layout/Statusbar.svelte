<script lang="ts">
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { CheckCircle2, Clock, Layers, Cpu } from 'lucide-svelte';

  const activeTab = $derived(tabsStore.activeTab);
  const result = $derived(activeTab?.queryResult);
</script>

<footer class="h-6 border-t border-slate-800 bg-surface-950 text-slate-400 text-[11px] px-3 flex items-center justify-between select-none z-20">
  <div class="flex items-center gap-4">
    <div class="flex items-center gap-1.5 text-emerald-400">
      <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
      <span class="font-medium">Ready</span>
    </div>

    {#if result}
      <div class="flex items-center gap-1">
        <Layers size={12} class="text-slate-500" />
        <span>{result.rows.length} rows</span>
      </div>

      <div class="flex items-center gap-1">
        <Clock size={12} class="text-slate-500" />
        <span>{result.executionTimeMs.toFixed(1)} ms</span>
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-4">
    <div class="flex items-center gap-1 text-slate-500">
      <Cpu size={12} />
      <span>Memory: ~34 MB RAM</span>
    </div>
    <span class="text-indigo-400 font-mono">FugDB v0.1.0-alpha</span>
  </div>
</footer>
