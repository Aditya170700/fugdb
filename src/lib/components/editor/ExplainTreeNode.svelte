<script lang="ts">
  import type { PlanNode } from '$lib/state/explain.svelte';
  import { explainStore } from '$lib/state/explain.svelte';
  import ExplainTreeNode from './ExplainTreeNode.svelte';
  import { 
    AlertTriangle, 
    Database, 
    Zap, 
    Layers, 
    Filter, 
    Key, 
    Timer, 
    HardDrive,
    ArrowDown,
    ChevronDown,
    ChevronRight
  } from 'lucide-svelte';

  let { 
    node, 
    depth = 0,
    maxCost = 100 
  }: { 
    node: PlanNode; 
    depth?: number;
    maxCost?: number;
  } = $props();

  let isCollapsed = $state(false);

  const isSelected = $derived(explainStore.selectedNode?.id === node.id);

  function getNodeBadgeStyle(nodeType: string) {
    const t = nodeType.toLowerCase();
    if (t.includes('seq scan') || t.includes('table scan') || t.includes('all')) {
      return {
        bg: 'bg-rose-500/15 border-rose-500/40 text-rose-600 dark:text-rose-400',
        badge: 'bg-rose-500 text-white',
        icon: 'text-rose-500',
      };
    }
    if (t.includes('index') || t.includes('search')) {
      return {
        bg: 'bg-emerald-500/15 border-emerald-500/40 text-emerald-600 dark:text-emerald-400',
        badge: 'bg-emerald-500 text-white',
        icon: 'text-emerald-500',
      };
    }
    if (t.includes('hash join') || t.includes('nested loop') || t.includes('merge join') || t.includes('join')) {
      return {
        bg: 'bg-violet-500/15 border-violet-500/40 text-violet-600 dark:text-violet-400',
        badge: 'bg-violet-500 text-white',
        icon: 'text-violet-500',
      };
    }
    if (t.includes('aggregate') || t.includes('group') || t.includes('count')) {
      return {
        bg: 'bg-sky-500/15 border-sky-500/40 text-sky-600 dark:text-sky-400',
        badge: 'bg-sky-500 text-white',
        icon: 'text-sky-500',
      };
    }
    if (t.includes('sort') || t.includes('limit') || t.includes('window')) {
      return {
        bg: 'bg-amber-500/15 border-amber-500/40 text-amber-600 dark:text-amber-400',
        badge: 'bg-amber-500 text-white',
        icon: 'text-amber-500',
      };
    }
    return {
      bg: 'bg-slate-500/15 border-slate-500/40 text-slate-700 dark:text-slate-300',
      badge: 'bg-slate-600 text-white',
      icon: 'text-slate-400',
    };
  }

  const badgeStyle = $derived(getNodeBadgeStyle(node.nodeType));
  const costPct = $derived(node.totalCost && maxCost ? Math.min(100, Math.round((node.totalCost / maxCost) * 100)) : (node.costPercent || 0));
</script>

<div class="flex flex-col items-center">
  <!-- Node Card Box -->
  <div 
    class="relative flex flex-col min-w-[280px] max-w-[340px] bg-white dark:bg-surface-900 rounded-xl border transition-all duration-150 cursor-pointer shadow-sm hover:shadow-md select-none group
      {isSelected 
        ? 'ring-2 ring-indigo-500 border-indigo-500 shadow-indigo-500/10 dark:shadow-indigo-500/20' 
        : node.isBottleneck 
          ? 'border-amber-500/60 dark:border-amber-500/40 hover:border-amber-500' 
          : 'border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700'
      }"
    onclick={() => explainStore.selectNode(node)}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === 'Enter' && explainStore.selectNode(node)}
  >
    <!-- Top Header: Node Type & Badges -->
    <div class="p-3 border-b border-slate-100 dark:border-slate-800/80 flex items-center justify-between gap-2 bg-slate-50/50 dark:bg-surface-950/40 rounded-t-xl">
      <div class="flex items-center gap-2 min-w-0">
        <span class="px-2 py-0.5 rounded text-[11px] font-bold uppercase tracking-wider truncate {badgeStyle.bg}">
          {node.nodeType}
        </span>
      </div>

      <div class="flex items-center gap-1.5 shrink-0">
        {#if node.isBottleneck}
          <div 
            class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-rose-500/20 text-rose-600 dark:text-rose-400 border border-rose-500/30 text-[10px] font-bold animate-pulse"
            title={node.warningMessage || 'Bottleneck detected'}
          >
            <AlertTriangle size={11} />
            <span>Bottleneck</span>
          </div>
        {/if}

        {#if node.children.length > 0}
          <button
            type="button"
            class="p-0.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 rounded hover:bg-slate-200/50 dark:hover:bg-slate-800 cursor-pointer"
            onclick={(e) => {
              e.stopPropagation();
              isCollapsed = !isCollapsed;
            }}
            title={isCollapsed ? 'Expand sub-tree' : 'Collapse sub-tree'}
          >
            {#if isCollapsed}
              <ChevronRight size={14} />
            {:else}
              <ChevronDown size={14} />
            {/if}
          </button>
        {/if}
      </div>
    </div>

    <!-- Body Information -->
    <div class="p-3 flex flex-col gap-2 text-xs">
      <!-- Target Relation & Index -->
      {#if node.relationName || node.indexName}
        <div class="flex items-center gap-1.5 text-slate-800 dark:text-slate-200 font-mono text-[11.5px] truncate">
          <Database size={13} class="shrink-0 text-slate-400" />
          <span class="font-bold truncate">{node.relationName || ''}</span>
          {#if node.alias && node.alias !== node.relationName}
            <span class="text-slate-400 text-[10px]">as {node.alias}</span>
          {/if}
          {#if node.indexName}
            <span class="text-indigo-500 dark:text-indigo-400 text-[10.5px] truncate flex items-center gap-0.5">
              <Key size={11} />
              {node.indexName}
            </span>
          {/if}
        </div>
      {/if}

      <!-- Cost & Duration Breakdown -->
      <div class="grid grid-cols-2 gap-2 pt-1 border-t border-slate-100 dark:border-slate-800/60 font-mono text-[11px]">
        <div>
          <span class="text-[10px] text-slate-400 uppercase tracking-wider block font-sans">Cost</span>
          <span class="font-semibold text-slate-700 dark:text-slate-300">
            {node.totalCost !== undefined ? node.totalCost.toFixed(2) : '—'}
          </span>
        </div>

        <div>
          <span class="text-[10px] text-slate-400 uppercase tracking-wider block font-sans">Time</span>
          <span class="font-semibold text-slate-700 dark:text-slate-300">
            {node.actualTotalTime !== undefined ? `${node.actualTotalTime.toFixed(2)} ms` : '—'}
          </span>
        </div>
      </div>

      <!-- Rows: Actual vs Estimated -->
      {#if node.actualRows !== undefined || node.planRows !== undefined}
        <div class="flex items-center justify-between text-[11px] font-mono bg-slate-100/70 dark:bg-surface-950/60 px-2 py-1 rounded">
          <span class="text-slate-400 text-[10.5px]">Rows:</span>
          <div class="flex items-center gap-1.5">
            {#if node.actualRows !== undefined}
              <span class="text-emerald-600 dark:text-emerald-400 font-bold" title="Actual Rows returned">
                {node.actualRows.toLocaleString()}
              </span>
            {/if}
            {#if node.planRows !== undefined}
              <span class="text-slate-400 text-[10px]" title="Planner estimated rows">
                (est. {node.planRows.toLocaleString()})
              </span>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Relative Cost Bar -->
      {#if costPct > 0}
        <div class="flex flex-col gap-0.5 pt-0.5">
          <div class="flex justify-between text-[10px] text-slate-400">
            <span>Query Cost Share</span>
            <span class="font-bold {costPct > 50 ? 'text-rose-500' : 'text-slate-500'}">{costPct}%</span>
          </div>
          <div class="w-full h-1.5 bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
            <div 
              class="h-full rounded-full transition-all {costPct > 50 ? 'bg-gradient-to-r from-amber-500 to-rose-500' : 'bg-gradient-to-r from-indigo-500 to-sky-500'}" 
              style="width: {costPct}%;"
            ></div>
          </div>
        </div>
      {/if}

      <!-- Filter / Condition Snippet -->
      {#if node.filter || node.hashCond || node.indexCond}
        <div class="text-[10px] font-mono text-slate-500 dark:text-slate-400 bg-slate-50 dark:bg-surface-950 px-1.5 py-1 rounded border border-slate-200/60 dark:border-slate-800 truncate" title={node.filter || node.hashCond || node.indexCond}>
          <span class="text-indigo-400 font-bold">Cond:</span> {node.filter || node.hashCond || node.indexCond}
        </div>
      {/if}
    </div>
  </div>

  <!-- Connector Line and Children Nodes -->
  {#if node.children.length > 0 && !isCollapsed}
    <div class="flex flex-col items-center">
      <!-- Vertical branch line down -->
      <div class="w-[2px] h-6 bg-slate-300 dark:bg-slate-700"></div>

      <!-- Horizontal connector bar if multiple children -->
      {#if node.children.length > 1}
        <div class="relative flex justify-center items-start">
          <div class="absolute top-0 left-1/2 -translate-x-1/2 w-[calc(100%-140px)] h-[2px] bg-slate-300 dark:bg-slate-700"></div>
          <div class="flex items-start gap-8 pt-4">
            {#each node.children as child (child.id)}
              <div class="flex flex-col items-center relative">
                <!-- Drop line from horizontal bar to child -->
                <div class="w-[2px] h-4 bg-slate-300 dark:bg-slate-700 -mt-4"></div>
                <ExplainTreeNode node={child} depth={depth + 1} {maxCost} />
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <!-- Single child -->
        <div class="flex flex-col items-center">
          <ExplainTreeNode node={node.children[0]} depth={depth + 1} {maxCost} />
        </div>
      {/if}
    </div>
  {/if}
</div>
