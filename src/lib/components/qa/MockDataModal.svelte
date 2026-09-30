<script lang="ts">
  import { Dices, Sparkles, X } from 'lucide-svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { api } from '$lib/api/client';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  let targetTable = $state('users');
  let rowCount = $state(50);
  let isGenerating = $state(false);
  let successMessage = $state<string | null>(null);

  async function handleGenerate() {
    isGenerating = true;
    successMessage = null;
    try {
      const inserted = await api.generateMockBatch(connectionStore.activeConnectionId, targetTable, rowCount);
      successMessage = `Successfully generated & inserted ${inserted} mock rows into ${targetTable}!`;
    } catch (err: any) {
      console.error(err);
    } finally {
      isGenerating = false;
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-surface-900 border border-slate-200 dark:border-slate-700 w-full max-w-lg rounded-xl shadow-2xl p-6 flex flex-col gap-4 text-xs">
      <div class="flex items-center justify-between border-b border-slate-200 dark:border-slate-800 pb-3">
        <div class="flex items-center gap-2">
          <div class="p-1.5 bg-violet-500/20 text-violet-600 dark:text-violet-400 rounded-lg">
            <Dices size={18} />
          </div>
          <div>
            <h3 class="font-bold text-sm text-slate-900 dark:text-slate-100">Smart Mock Data Generator (QA)</h3>
            <p class="text-[11px] text-slate-500 dark:text-slate-400">Generate schema-aware realistic seed data via fake-rs</p>
          </div>
        </div>
        <button onclick={onClose} class="text-slate-400 hover:text-slate-700 dark:hover:text-slate-100 hover:bg-surface-800 p-1.5 rounded-lg transition-colors" title="Close">
          <X size={16} />
        </button>
      </div>

      <div class="space-y-3">
        <div>
          <label for="mock-target-table" class="font-semibold text-slate-700 dark:text-slate-300 block mb-1">Target Table</label>
          <input id="mock-target-table" bind:value={targetTable} class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md p-2 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500" />
        </div>

        <div>
          <label for="mock-row-count" class="font-semibold text-slate-700 dark:text-slate-300 block mb-1">Number of Rows</label>
          <select id="mock-row-count" bind:value={rowCount} class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md p-2 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500">
            <option value={10}>10 rows (Quick test)</option>
            <option value={50}>50 rows (Standard)</option>
            <option value={500}>500 rows (Load test)</option>
            <option value={5000}>5,000 rows (Stress test)</option>
          </select>
        </div>

        {#if successMessage}
          <div class="p-3 bg-emerald-500/10 border border-emerald-500/30 text-emerald-800 dark:text-emerald-300 rounded-md">
            {successMessage}
          </div>
        {/if}
      </div>

      <div class="flex justify-end gap-2 pt-3 border-t border-slate-200 dark:border-slate-800">
        <button onclick={onClose} class="px-3 py-1.5 text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200">Cancel</button>
        <button 
          onclick={handleGenerate} 
          disabled={isGenerating}
          class="px-4 py-1.5 bg-violet-600 hover:bg-violet-500 text-white font-semibold rounded-md shadow-md shadow-violet-600/20 disabled:opacity-50"
        >
          {isGenerating ? 'Generating...' : 'Generate Batch'}
        </button>
      </div>
    </div>
  </div>
{/if}
