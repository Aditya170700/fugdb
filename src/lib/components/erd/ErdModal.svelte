<script lang="ts">
  import { Network, X, Download, ZoomIn, ZoomOut } from 'lucide-svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  const mockTables = [
    {
      name: 'users',
      columns: [
        { name: 'id', type: 'INT', isPk: true },
        { name: 'name', type: 'VARCHAR' },
        { name: 'email', type: 'VARCHAR' },
        { name: 'role', type: 'VARCHAR' }
      ]
    },
    {
      name: 'orders',
      columns: [
        { name: 'id', type: 'INT', isPk: true },
        { name: 'user_id', type: 'INT', isFk: true },
        { name: 'order_number', type: 'VARCHAR' },
        { name: 'total_amount', type: 'DECIMAL' }
      ]
    },
    {
      name: 'order_items',
      columns: [
        { name: 'id', type: 'INT', isPk: true },
        { name: 'order_id', type: 'INT', isFk: true },
        { name: 'product_id', type: 'INT', isFk: true },
        { name: 'quantity', type: 'INT' }
      ]
    }
  ];
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-6">
    <div class="bg-surface-900 border border-slate-700 w-full max-w-4xl h-[600px] rounded-xl shadow-2xl flex flex-col overflow-hidden">
      <!-- Topbar -->
      <div class="px-5 py-3 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Network size={18} class="text-indigo-400" />
          <h3 class="font-bold text-sm text-slate-100">Interactive Entity Relationship Diagram (ERD)</h3>
        </div>
        <div class="flex items-center gap-1.5">
          <button class="p-1.5 text-slate-400 hover:text-slate-100 rounded-lg bg-surface-800 hover:bg-surface-700 transition-colors" title="Zoom In"><ZoomIn size={14} /></button>
          <button class="p-1.5 text-slate-400 hover:text-slate-100 rounded-lg bg-surface-800 hover:bg-surface-700 transition-colors" title="Zoom Out"><ZoomOut size={14} /></button>
          <button onclick={onClose} class="p-1.5 text-slate-400 hover:text-slate-100 rounded-lg hover:bg-surface-800 transition-colors" title="Close"><X size={16} /></button>
        </div>
      </div>

      <!-- Canvas Area -->
      <div class="flex-1 bg-surface-950 p-8 overflow-auto flex items-center justify-around relative">
        {#each mockTables as tbl}
          <div class="w-56 bg-surface-900 border border-slate-700 rounded-lg shadow-xl overflow-hidden">
            <div class="bg-indigo-950/60 border-b border-slate-800 px-3 py-2 font-mono text-xs font-bold text-indigo-300">
              {tbl.name}
            </div>
            <div class="divide-y divide-slate-800/60 p-1 text-[11px] font-mono">
              {#each tbl.columns as col}
                <div class="px-2 py-1.5 flex items-center justify-between">
                  <span class="text-slate-300 font-medium {col.isPk ? 'text-amber-400' : ''}">{col.name}</span>
                  <div class="flex items-center gap-1">
                    {#if col.isPk}<span class="text-[9px] bg-amber-500/20 text-amber-300 px-1 rounded">PK</span>{/if}
                    {#if col.isFk}<span class="text-[9px] bg-sky-500/20 text-sky-300 px-1 rounded">FK</span>{/if}
                    <span class="text-slate-500 text-[10px]">{col.type}</span>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
