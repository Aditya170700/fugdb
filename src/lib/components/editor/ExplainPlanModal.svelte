<script lang="ts">
  import { explainStore } from '$lib/state/explain.svelte';
  import ExplainTreeNode from './ExplainTreeNode.svelte';
  import { 
    X, 
    Search, 
    Sparkles, 
    Copy, 
    Check, 
    Layers, 
    FileText, 
    RefreshCw, 
    AlertTriangle, 
    ZoomIn, 
    ZoomOut, 
    Maximize2, 
    ArrowRight, 
    Database, 
    Key, 
    Timer, 
    Activity, 
    Cpu, 
    HardDrive, 
    ShieldCheck, 
    Flame,
    CheckCircle2,
    Code2,
    Share2,
    Info
  } from 'lucide-svelte';

  let zoomLevel = $state(1);
  let copiedRaw = $state(false);
  let copiedAiSql = $state(false);
  let copiedIndexDdl = $state(false);
  let rawSearchTerm = $state('');

  function handleZoom(delta: number) {
    zoomLevel = Math.max(0.4, Math.min(2.0, parseFloat((zoomLevel + delta).toFixed(2))));
  }

  function resetZoom() {
    zoomLevel = 1;
  }

  async function copyToClipboard(text: string, type: 'raw' | 'sql' | 'ddl') {
    try {
      await navigator.clipboard.writeText(text);
      if (type === 'raw') {
        copiedRaw = true;
        setTimeout(() => copiedRaw = false, 2000);
      } else if (type === 'sql') {
        copiedAiSql = true;
        setTimeout(() => copiedAiSql = false, 2000);
      } else if (type === 'ddl') {
        copiedIndexDdl = true;
        setTimeout(() => copiedIndexDdl = false, 2000);
      }
    } catch (e) {
      console.error('Failed to copy to clipboard', e);
    }
  }

  const selectedNode = $derived(explainStore.selectedNode);
  const parsedTree = $derived(explainStore.parsedTree);
  const explainResult = $derived(explainStore.explainResult);
  const optimization = $derived(explainStore.optimizationResult);

  const filteredRawPlan = $derived.by(() => {
    if (!explainResult?.rawPlan) return '';
    if (!rawSearchTerm.trim()) return explainResult.rawPlan;
    return explainResult.rawPlan
      .split('\n')
      .filter(line => line.toLowerCase().includes(rawSearchTerm.toLowerCase()))
      .join('\n');
  });
</script>

{#if explainStore.isOpen}
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4 animate-in fade-in duration-200"
    role="dialog"
    aria-modal="true"
    aria-labelledby="explain-plan-title"
  >
    <div 
      class="w-full max-w-6xl h-[88vh] bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-150"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 bg-surface-900 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between gap-4 shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-indigo-500/10 text-indigo-500 border border-indigo-500/20 flex items-center justify-center font-bold">
            <Activity size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 id="explain-plan-title" class="text-sm font-bold text-slate-900 dark:text-white">
                Query EXPLAIN Execution Plan
              </h2>
              <span class="px-2 py-0.5 rounded text-[10.5px] font-mono font-semibold bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20">
                {explainResult?.dialect || explainStore.driver.toUpperCase()}
              </span>
              {#if explainStore.isAnalyze}
                <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30">
                  EXPLAIN ANALYZE
                </span>
              {:else}
                <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-slate-500/15 text-slate-600 dark:text-slate-400 border border-slate-500/30">
                  ESTIMATED PLAN
                </span>
              {/if}
            </div>
            <p class="text-[11px] text-slate-500 dark:text-slate-400 truncate max-w-xl font-mono">
              {explainStore.sql}
            </p>
          </div>
        </div>

        <!-- Header Actions -->
        <div class="flex items-center gap-2">
          <!-- Toggle Analyze vs Plan only -->
          <div class="flex items-center bg-surface-950 p-0.5 rounded-lg border border-slate-200 dark:border-slate-800 text-xs">
            <button
              type="button"
              onclick={() => explainStore.runExplain(true)}
              disabled={explainStore.isLoading}
              class="px-2.5 py-1 rounded-md text-[11px] font-semibold transition-colors cursor-pointer {explainStore.isAnalyze ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-500 hover:text-slate-300'}"
            >
              Analyze (Run)
            </button>
            <button
              type="button"
              onclick={() => explainStore.runExplain(false)}
              disabled={explainStore.isLoading}
              class="px-2.5 py-1 rounded-md text-[11px] font-semibold transition-colors cursor-pointer {!explainStore.isAnalyze ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-500 hover:text-slate-300'}"
            >
              Plan Only
            </button>
          </div>

          <!-- Re-run Button -->
          <button
            type="button"
            onclick={() => explainStore.runExplain()}
            disabled={explainStore.isLoading}
            class="flex items-center gap-1.5 px-2.5 py-1.5 bg-surface-800 hover:bg-surface-700 text-slate-200 rounded-lg text-xs font-semibold border border-slate-700 transition-colors cursor-pointer"
            title="Re-run explain plan"
          >
            <RefreshCw size={13} class={explainStore.isLoading ? 'animate-spin' : ''} />
            <span>Refresh</span>
          </button>

          <!-- AI Optimize CTA Button -->
          <button
            type="button"
            onclick={() => explainStore.runAiOptimize()}
            disabled={explainStore.isOptimizing || explainStore.isLoading}
            class="flex items-center gap-1.5 px-3 py-1.5 bg-gradient-to-r from-violet-600 to-indigo-600 hover:from-violet-500 hover:to-indigo-500 text-white rounded-lg text-xs font-bold shadow-sm transition-all cursor-pointer"
            title="Diagnose bottlenecks and get optimized query + indexes with AI"
          >
            <Sparkles size={13} class={explainStore.isOptimizing ? 'animate-spin' : 'animate-pulse'} />
            <span>{explainStore.isOptimizing ? 'Optimizing...' : '✨ AI Optimize'}</span>
          </button>

          <!-- Close Modal -->
          <button
            type="button"
            onclick={() => explainStore.close()}
            class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-1.5 rounded-lg hover:bg-surface-800 transition-colors cursor-pointer"
            title="Close (Esc)"
          >
            <X size={18} />
          </button>
        </div>
      </div>

      <!-- Top Metric Summary Cards -->
      {#if parsedTree}
        <div class="px-5 py-2.5 bg-surface-900/60 border-b border-slate-200 dark:border-slate-800 grid grid-cols-2 md:grid-cols-4 gap-3 text-xs shrink-0">
          <!-- Total Cost -->
          <div class="flex items-center gap-2.5 px-3 py-2 bg-white dark:bg-surface-950 rounded-xl border border-slate-200 dark:border-slate-800">
            <div class="w-7 h-7 rounded-lg bg-indigo-500/10 text-indigo-500 flex items-center justify-center font-bold shrink-0">
              <Cpu size={15} />
            </div>
            <div class="min-w-0">
              <span class="text-[10px] text-slate-400 uppercase tracking-wider block font-medium">Estimated Cost</span>
              <span class="text-xs font-bold font-mono text-slate-800 dark:text-slate-100">
                {parsedTree.maxCost.toLocaleString()}
              </span>
            </div>
          </div>

          <!-- Execution Duration -->
          <div class="flex items-center gap-2.5 px-3 py-2 bg-white dark:bg-surface-950 rounded-xl border border-slate-200 dark:border-slate-800">
            <div class="w-7 h-7 rounded-lg bg-emerald-500/10 text-emerald-500 flex items-center justify-center font-bold shrink-0">
              <Timer size={15} />
            </div>
            <div class="min-w-0">
              <span class="text-[10px] text-slate-400 uppercase tracking-wider block font-medium">Execution Time</span>
              <span class="text-xs font-bold font-mono text-emerald-600 dark:text-emerald-400">
                {parsedTree.totalExecutionTime !== undefined ? `${parsedTree.totalExecutionTime.toFixed(2)} ms` : '—'}
              </span>
              {#if parsedTree.totalPlanningTime !== undefined}
                <span class="text-[10px] text-slate-400 font-mono ml-1">
                  (plan {parsedTree.totalPlanningTime.toFixed(2)}ms)
                </span>
              {/if}
            </div>
          </div>

          <!-- Bottlenecks Warning -->
          <div class="flex items-center gap-2.5 px-3 py-2 bg-white dark:bg-surface-950 rounded-xl border border-slate-200 dark:border-slate-800">
            <div class="w-7 h-7 rounded-lg {parsedTree.bottlenecksCount > 0 ? 'bg-amber-500/15 text-amber-500' : 'bg-emerald-500/10 text-emerald-500'} flex items-center justify-center font-bold shrink-0">
              {#if parsedTree.bottlenecksCount > 0}
                <AlertTriangle size={15} />
              {:else}
                <ShieldCheck size={15} />
              {/if}
            </div>
            <div class="min-w-0">
              <span class="text-[10px] text-slate-400 uppercase tracking-wider block font-medium">Bottlenecks</span>
              <span class="text-xs font-bold {parsedTree.bottlenecksCount > 0 ? 'text-amber-500' : 'text-emerald-500'}">
                {parsedTree.bottlenecksCount > 0 ? `${parsedTree.bottlenecksCount} Potential Issue${parsedTree.bottlenecksCount > 1 ? 's' : ''}` : 'Clean Plan (No Alert)'}
              </span>
            </div>
          </div>

          <!-- Scans Summary -->
          <div class="flex items-center gap-2.5 px-3 py-2 bg-white dark:bg-surface-950 rounded-xl border border-slate-200 dark:border-slate-800">
            <div class="w-7 h-7 rounded-lg bg-sky-500/10 text-sky-500 flex items-center justify-center font-bold shrink-0">
              <Database size={15} />
            </div>
            <div class="min-w-0 flex items-center gap-2 font-mono text-[11px]">
              <div class="flex flex-col">
                <span class="text-[10px] text-slate-400 uppercase tracking-wider font-sans font-medium">Access Types</span>
                <div class="flex items-center gap-1.5 font-bold">
                  <span class="text-rose-500" title="Sequential full table scans">{parsedTree.seqScansCount} Seq</span>
                  <span class="text-slate-400">/</span>
                  <span class="text-emerald-500" title="Index scans">{parsedTree.indexScansCount} Index</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Tab Navigation -->
      <div class="px-5 bg-surface-900 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-1">
          <button
            type="button"
            onclick={() => explainStore.activeTab = 'visual'}
            class="flex items-center gap-1.5 px-3.5 py-2.5 text-xs font-semibold border-b-2 transition-colors cursor-pointer {explainStore.activeTab === 'visual' ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400 bg-surface-800/40' : 'border-transparent text-slate-400 hover:text-slate-200'}"
          >
            <Layers size={14} />
            <span>Visual Execution Tree</span>
          </button>

          <button
            type="button"
            onclick={() => explainStore.activeTab = 'raw'}
            class="flex items-center gap-1.5 px-3.5 py-2.5 text-xs font-semibold border-b-2 transition-colors cursor-pointer {explainStore.activeTab === 'raw' ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400 bg-surface-800/40' : 'border-transparent text-slate-400 hover:text-slate-200'}"
          >
            <Code2 size={14} />
            <span>Raw Output ({explainResult?.jsonPlan ? 'JSON' : 'Text'})</span>
          </button>

          <button
            type="button"
            onclick={() => explainStore.activeTab = 'ai'}
            class="flex items-center gap-1.5 px-3.5 py-2.5 text-xs font-semibold border-b-2 transition-colors cursor-pointer {explainStore.activeTab === 'ai' ? 'border-violet-500 text-violet-600 dark:text-violet-400 bg-violet-500/10' : 'border-transparent text-slate-400 hover:text-violet-300'}"
          >
            <Sparkles size={14} class="text-violet-400" />
            <span>AI Optimization Studio</span>
            {#if optimization}
              <span class="w-2 h-2 rounded-full bg-violet-500 animate-pulse"></span>
            {/if}
          </button>
        </div>

        <!-- Right Side Tab Controls -->
        {#if explainStore.activeTab === 'visual'}
          <div class="flex items-center gap-1 text-xs">
            <button
              type="button"
              onclick={() => handleZoom(-0.1)}
              class="p-1.5 text-slate-400 hover:text-slate-200 rounded hover:bg-surface-800 transition-colors cursor-pointer"
              title="Zoom out"
            >
              <ZoomOut size={14} />
            </button>
            <span class="text-[11px] font-mono text-slate-400 px-1 min-w-[36px] text-center">
              {Math.round(zoomLevel * 100)}%
            </span>
            <button
              type="button"
              onclick={() => handleZoom(0.1)}
              class="p-1.5 text-slate-400 hover:text-slate-200 rounded hover:bg-surface-800 transition-colors cursor-pointer"
              title="Zoom in"
            >
              <ZoomIn size={14} />
            </button>
            <button
              type="button"
              onclick={resetZoom}
              class="p-1.5 text-slate-400 hover:text-slate-200 rounded hover:bg-surface-800 transition-colors cursor-pointer ml-1"
              title="Reset Zoom"
            >
              <Maximize2 size={13} />
            </button>
          </div>
        {/if}
      </div>

      <!-- Tab 1: Visual Execution Tree -->
      {#if explainStore.activeTab === 'visual'}
        <div class="flex-1 flex overflow-hidden relative">
          <!-- Tree Canvas -->
          <div class="flex-1 overflow-auto p-8 flex justify-center items-start bg-slate-950/30">
            {#if explainStore.isLoading}
              <div class="m-auto flex flex-col items-center gap-3 text-slate-400">
                <RefreshCw size={24} class="animate-spin text-indigo-500" />
                <span class="text-xs font-medium">Executing EXPLAIN query on database...</span>
              </div>
            {:else if explainStore.error}
              <div class="m-auto max-w-md p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-xs text-rose-300 flex flex-col gap-2">
                <div class="flex items-center gap-2 font-bold text-rose-400">
                  <AlertTriangle size={16} />
                  <span>Failed to execute EXPLAIN query</span>
                </div>
                <p class="font-mono text-[11px] select-text">{explainStore.error}</p>
              </div>
            {:else if parsedTree}
              <div 
                class="transition-transform origin-top duration-75" 
                style="transform: scale({zoomLevel});"
              >
                <ExplainTreeNode node={parsedTree.root} maxCost={parsedTree.maxCost} />
              </div>
            {:else}
              <div class="m-auto text-xs text-slate-500">
                No explain plan data available.
              </div>
            {/if}
          </div>

          <!-- Right Side Selected Node Inspector -->
          {#if selectedNode}
            <aside class="w-80 border-l border-slate-200 dark:border-slate-800 bg-surface-900 flex flex-col shrink-0 overflow-hidden animate-in slide-in-from-right duration-150 text-xs">
              <div class="p-3.5 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between gap-2 bg-surface-950">
                <div class="flex items-center gap-2 min-w-0">
                  <span class="font-bold text-xs text-slate-100 truncate">
                    {selectedNode.nodeType}
                  </span>
                </div>
                <button
                  type="button"
                  onclick={() => explainStore.selectedNode = null}
                  class="text-slate-400 hover:text-slate-200 p-1 rounded hover:bg-surface-800 cursor-pointer"
                >
                  <X size={14} />
                </button>
              </div>

              <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-4">
                <!-- Bottleneck Alert -->
                {#if selectedNode.isBottleneck}
                  <div class="p-2.5 bg-rose-500/10 border border-rose-500/30 rounded-xl flex flex-col gap-1 text-rose-300">
                    <div class="flex items-center gap-1.5 font-bold text-rose-400 text-xs">
                      <AlertTriangle size={14} />
                      <span>Bottleneck Detected</span>
                    </div>
                    <p class="text-[11px] leading-relaxed font-sans">{selectedNode.warningMessage}</p>
                  </div>
                {/if}

                <!-- General Attributes -->
                <div class="flex flex-col gap-2">
                  <span class="text-[10px] text-slate-400 font-bold uppercase tracking-wider">Node Details</span>
                  
                  <div class="bg-surface-950 rounded-xl border border-slate-800 divide-y divide-slate-800/80 font-mono text-[11px]">
                    {#if selectedNode.relationName}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Table / Relation</span>
                        <span class="font-bold text-slate-200">{selectedNode.relationName}</span>
                      </div>
                    {/if}

                    {#if selectedNode.alias}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Alias</span>
                        <span class="text-slate-200">{selectedNode.alias}</span>
                      </div>
                    {/if}

                    {#if selectedNode.indexName}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Index Name</span>
                        <span class="text-indigo-400 font-bold">{selectedNode.indexName}</span>
                      </div>
                    {/if}

                    {#if selectedNode.joinType}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Join Type</span>
                        <span class="text-violet-400">{selectedNode.joinType}</span>
                      </div>
                    {/if}

                    {#if selectedNode.totalCost !== undefined}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Total Cost</span>
                        <span class="text-slate-200 font-bold">{selectedNode.totalCost.toFixed(2)}</span>
                      </div>
                    {/if}

                    {#if selectedNode.startupCost !== undefined}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Startup Cost</span>
                        <span class="text-slate-300">{selectedNode.startupCost.toFixed(2)}</span>
                      </div>
                    {/if}

                    {#if selectedNode.actualTotalTime !== undefined}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Actual Total Time</span>
                        <span class="text-emerald-400 font-bold">{selectedNode.actualTotalTime.toFixed(3)} ms</span>
                      </div>
                    {/if}

                    {#if selectedNode.actualStartupTime !== undefined}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Startup Time</span>
                        <span class="text-slate-300">{selectedNode.actualStartupTime.toFixed(3)} ms</span>
                      </div>
                    {/if}

                    {#if selectedNode.actualRows !== undefined}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Actual Rows</span>
                        <span class="text-emerald-400 font-bold">{selectedNode.actualRows.toLocaleString()}</span>
                      </div>
                    {/if}

                    {#if selectedNode.planRows !== undefined}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Estimated Rows</span>
                        <span class="text-slate-400">{selectedNode.planRows.toLocaleString()}</span>
                      </div>
                    {/if}

                    {#if selectedNode.actualLoops !== undefined}
                      <div class="p-2 flex items-center justify-between">
                        <span class="text-slate-400 font-sans">Loops</span>
                        <span class="text-slate-300">{selectedNode.actualLoops}</span>
                      </div>
                    {/if}
                  </div>
                </div>

                <!-- Buffer I/O Stats if present -->
                {#if selectedNode.sharedHitBlocks !== undefined || selectedNode.sharedReadBlocks !== undefined}
                  <div class="flex flex-col gap-2">
                    <span class="text-[10px] text-slate-400 font-bold uppercase tracking-wider">Buffer & I/O</span>
                    <div class="bg-surface-950 rounded-xl border border-slate-800 p-2.5 flex flex-col gap-1.5 font-mono text-[11px]">
                      {#if selectedNode.sharedHitBlocks !== undefined}
                        <div class="flex justify-between">
                          <span class="text-slate-400 font-sans">Shared Cache Hits</span>
                          <span class="text-emerald-400 font-bold">{selectedNode.sharedHitBlocks} blocks</span>
                        </div>
                      {/if}
                      {#if selectedNode.sharedReadBlocks !== undefined}
                        <div class="flex justify-between">
                          <span class="text-slate-400 font-sans">Disk Reads</span>
                          <span class="text-amber-400 font-bold">{selectedNode.sharedReadBlocks} blocks</span>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}

                <!-- Predicate / Conditions -->
                {#if selectedNode.filter || selectedNode.indexCond || selectedNode.hashCond}
                  <div class="flex flex-col gap-2">
                    <span class="text-[10px] text-slate-400 font-bold uppercase tracking-wider">Predicates & Filters</span>
                    <div class="bg-surface-950 rounded-xl border border-slate-800 p-2.5 flex flex-col gap-2 text-[11px] font-mono">
                      {#if selectedNode.filter}
                        <div>
                          <span class="text-[10px] text-indigo-400 font-sans block">Filter:</span>
                          <span class="text-slate-200 select-text break-all">{selectedNode.filter}</span>
                        </div>
                      {/if}
                      {#if selectedNode.indexCond}
                        <div>
                          <span class="text-[10px] text-emerald-400 font-sans block">Index Condition:</span>
                          <span class="text-slate-200 select-text break-all">{selectedNode.indexCond}</span>
                        </div>
                      {/if}
                      {#if selectedNode.hashCond}
                        <div>
                          <span class="text-[10px] text-violet-400 font-sans block">Hash Condition:</span>
                          <span class="text-slate-200 select-text break-all">{selectedNode.hashCond}</span>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}

                <!-- Raw Node JSON -->
                {#if selectedNode.rawDetails}
                  <details class="group">
                    <summary class="text-[10px] text-slate-400 font-bold uppercase tracking-wider cursor-pointer hover:text-slate-200 py-1">
                      Raw Node JSON
                    </summary>
                    <pre class="mt-2 p-2 bg-surface-950 rounded-lg border border-slate-800 font-mono text-[10px] text-slate-300 overflow-x-auto select-text whitespace-pre-wrap">{JSON.stringify(selectedNode.rawDetails, null, 2)}</pre>
                  </details>
                {/if}
              </div>
            </aside>
          {/if}
        </div>
      {/if}

      <!-- Tab 2: Raw Plan View -->
      {#if explainStore.activeTab === 'raw'}
        <div class="flex-1 flex flex-col overflow-hidden p-4 gap-3 bg-surface-950">
          <div class="flex items-center justify-between gap-3">
            <div class="relative flex-1 max-w-md">
              <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
              <input
                type="text"
                bind:value={rawSearchTerm}
                placeholder="Search raw plan output..."
                class="w-full pl-9 pr-3 py-1.5 bg-surface-900 border border-slate-800 rounded-lg text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-indigo-500"
              />
            </div>

            <button
              type="button"
              onclick={() => copyToClipboard(explainResult?.rawPlan || '', 'raw')}
              class="flex items-center gap-1.5 px-3 py-1.5 bg-surface-800 hover:bg-surface-700 text-slate-200 rounded-lg text-xs font-semibold border border-slate-700 transition-colors cursor-pointer"
            >
              {#if copiedRaw}
                <Check size={13} class="text-emerald-400" />
                <span class="text-emerald-400">Copied!</span>
              {:else}
                <Copy size={13} />
                <span>Copy Plan</span>
              {/if}
            </button>
          </div>

          <div class="flex-1 overflow-auto rounded-xl border border-slate-800 bg-surface-900/90 p-4 font-mono text-xs text-slate-200 select-text whitespace-pre leading-relaxed">
            {filteredRawPlan || 'No plan output returned.'}
          </div>
        </div>
      {/if}

      <!-- Tab 3: AI Optimization Studio -->
      {#if explainStore.activeTab === 'ai'}
        <div class="flex-1 flex flex-col overflow-y-auto p-6 bg-surface-950">
          {#if explainStore.isOptimizing}
            <div class="m-auto flex flex-col items-center gap-3 text-slate-400 py-12">
              <Sparkles size={32} class="animate-spin text-violet-500" />
              <div class="text-center">
                <h3 class="text-sm font-bold text-slate-200">Analyzing Execution Plan with AI Copilot...</h3>
                <p class="text-xs text-slate-500 mt-1">Diagnosing Sequential Scans, join algorithms, and evaluating indexing strategies</p>
              </div>
            </div>
          {:else if optimization}
            <div class="max-w-4xl mx-auto w-full flex flex-col gap-6">
              <!-- AI Header Summary -->
              <div class="p-4 bg-gradient-to-r from-violet-950/40 via-surface-900 to-indigo-950/40 border border-violet-500/30 rounded-2xl flex items-start justify-between gap-4">
                <div class="flex items-start gap-3">
                  <div class="w-9 h-9 rounded-xl bg-violet-500/20 text-violet-400 flex items-center justify-center shrink-0 border border-violet-500/30">
                    <Sparkles size={20} />
                  </div>
                  <div>
                    <h3 class="text-sm font-bold text-white flex items-center gap-2">
                      AI Performance Optimization Diagnosis
                    </h3>
                    <div class="flex items-center gap-2 text-xs text-slate-400 mt-0.5">
                      <span>Model: <b class="text-violet-400 font-mono">{optimization.modelUsed}</b></span>
                      <span>•</span>
                      <span>Latency: <b class="text-slate-300 font-mono">{optimization.executionTimeMs.toFixed(0)} ms</b></span>
                      <span>•</span>
                      <span>Dialect: <b class="text-slate-300">{optimization.dialect}</b></span>
                    </div>
                  </div>
                </div>

                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    onclick={() => explainStore.runAiOptimize()}
                    class="flex items-center gap-1.5 px-3 py-1.5 bg-surface-800 hover:bg-surface-700 text-slate-200 rounded-lg text-xs font-semibold border border-slate-700 transition-colors cursor-pointer"
                  >
                    <RefreshCw size={13} />
                    <span>Re-evaluate</span>
                  </button>
                </div>
              </div>

              <!-- Bottleneck & Strategy Explanation -->
              <div class="p-5 bg-surface-900 rounded-2xl border border-slate-800 flex flex-col gap-3">
                <div class="flex items-center gap-2 text-xs font-bold text-slate-200 uppercase tracking-wider">
                  <Activity size={14} class="text-violet-400" />
                  <span>Key Findings & Optimization Strategy</span>
                </div>
                <div class="text-xs text-slate-300 leading-relaxed space-y-2 select-text font-sans whitespace-pre-line">
                  {optimization.explanation}
                </div>
              </div>

              <!-- Optimized SQL Query -->
              <div class="p-5 bg-surface-900 rounded-2xl border border-slate-800 flex flex-col gap-3">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2 text-xs font-bold text-slate-200 uppercase tracking-wider">
                    <Code2 size={14} class="text-emerald-400" />
                    <span>Optimized Query & Index Recommendations</span>
                  </div>

                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => copyToClipboard(optimization.sql, 'sql')}
                      class="flex items-center gap-1.5 px-2.5 py-1 bg-surface-800 hover:bg-surface-700 text-slate-200 rounded-md text-xs font-medium border border-slate-700 transition-colors cursor-pointer"
                    >
                      {#if copiedAiSql}
                        <Check size={12} class="text-emerald-400" />
                        <span class="text-emerald-400">Copied</span>
                      {:else}
                        <Copy size={12} />
                        <span>Copy</span>
                      {/if}
                    </button>

                    <button
                      type="button"
                      onclick={() => explainStore.applyOptimizedSql(optimization.sql)}
                      class="flex items-center gap-1.5 px-3 py-1 bg-emerald-600 hover:bg-emerald-500 text-white rounded-md text-xs font-bold shadow-sm transition-colors cursor-pointer"
                    >
                      <CheckCircle2 size={13} />
                      <span>Apply to Editor</span>
                    </button>
                  </div>
                </div>

                <div class="p-4 bg-surface-950 rounded-xl border border-slate-800/90 font-mono text-xs text-emerald-300 overflow-x-auto select-text whitespace-pre leading-relaxed">
                  {optimization.sql}
                </div>
              </div>
            </div>
          {:else}
            <!-- Empty AI State with CTA -->
            <div class="m-auto max-w-md flex flex-col items-center text-center gap-4 py-12">
              <div class="w-14 h-14 rounded-2xl bg-violet-500/10 text-violet-400 flex items-center justify-center border border-violet-500/20">
                <Sparkles size={28} />
              </div>
              <div>
                <h3 class="text-base font-bold text-slate-100">AI Performance Optimizer Studio</h3>
                <p class="text-xs text-slate-400 mt-1.5 leading-relaxed">
                  Let AI analyze your execution tree, uncover costly Sequential Scans, recommend compound indexes (`CREATE INDEX ...`), and rewrite queries for maximum throughput.
                </p>
              </div>
              <button
                type="button"
                onclick={() => explainStore.runAiOptimize()}
                class="flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-violet-600 to-indigo-600 hover:from-violet-500 hover:to-indigo-500 text-white rounded-xl text-xs font-bold shadow-md transition-all cursor-pointer"
              >
                <Sparkles size={14} class="animate-pulse" />
                <span>Run AI Optimization Diagnosis</span>
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}
