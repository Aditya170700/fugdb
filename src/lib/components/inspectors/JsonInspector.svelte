<script lang="ts">
  import { 
    Code, 
    Copy, 
    Check, 
    ChevronRight, 
    ChevronDown, 
    Search, 
    Sparkles, 
    Minimize2, 
    CheckCircle2, 
    AlertCircle,
    ListTree,
    FileCode
  } from 'lucide-svelte';

  let { 
    value, 
    onChange 
  }: { 
    value: any; 
    onChange: (newVal: any) => void; 
  } = $props();

  let mode = $state<'tree' | 'raw'>('tree');
  let rawText = $state<string>('');
  let parseError = $state<string | null>(null);
  let parsedObject = $state<any>(null);
  let searchQuery = $state<string>('');
  let copied = $state<boolean>(false);
  let expandedPaths = $state<Set<string>>(new Set(['root']));

  // Initialize rawText and parsedObject from incoming value
  $effect(() => {
    try {
      if (typeof value === 'object' && value !== null) {
        parsedObject = value;
        rawText = JSON.stringify(value, null, 2);
        parseError = null;
      } else if (typeof value === 'string') {
        const parsed = JSON.parse(value);
        parsedObject = parsed;
        rawText = JSON.stringify(parsed, null, 2);
        parseError = null;
      } else {
        parsedObject = value;
        rawText = JSON.stringify(value, null, 2);
        parseError = null;
      }
    } catch (err: any) {
      parsedObject = null;
      rawText = String(value ?? '');
      parseError = err?.message || 'Invalid JSON syntax';
      mode = 'raw';
    }
  });

  function handleRawChange(text: string) {
    rawText = text;
    try {
      if (text.trim() === '') {
        parsedObject = null;
        parseError = null;
        onChange(null);
        return;
      }
      const parsed = JSON.parse(text);
      parsedObject = parsed;
      parseError = null;
      onChange(parsed);
    } catch (err: any) {
      parseError = err?.message || 'Invalid JSON syntax';
    }
  }

  function formatBeautify(spaces = 2) {
    try {
      if (parsedObject !== null) {
        rawText = JSON.stringify(parsedObject, null, spaces);
        parseError = null;
      }
    } catch {}
  }

  function formatMinify() {
    try {
      if (parsedObject !== null) {
        rawText = JSON.stringify(parsedObject);
        parseError = null;
      }
    } catch {}
  }

  function copyJson() {
    navigator.clipboard.writeText(rawText);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }

  function toggleExpand(path: string) {
    const next = new Set(expandedPaths);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedPaths = next;
  }

  function expandAll() {
    const all = new Set<string>();
    function collect(obj: any, path: string) {
      all.add(path);
      if (obj && typeof obj === 'object') {
        for (const k of Object.keys(obj)) {
          collect(obj[k], `${path}.${k}`);
        }
      }
    }
    collect(parsedObject, 'root');
    expandedPaths = all;
  }

  function collapseAll() {
    expandedPaths = new Set(['root']);
  }
</script>

<!-- Header Toolbar -->
<div class="p-3 border-b border-slate-200 dark:border-slate-800 bg-surface-900/60 flex items-center justify-between text-xs gap-3">
  <!-- Mode Toggle Buttons -->
  <div class="flex items-center gap-1 bg-surface-950 p-0.5 rounded-lg border border-slate-200 dark:border-slate-800">
    <button
      type="button"
      onclick={() => mode = 'tree'}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'tree' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
    >
      <ListTree size={12} />
      <span>Interactive Tree</span>
    </button>
    <button
      type="button"
      onclick={() => mode = 'raw'}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'raw' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
    >
      <FileCode size={12} />
      <span>Raw Code & Edit</span>
    </button>
  </div>

  <!-- Center Actions & Search (Tree mode) or Format (Raw mode) -->
  <div class="flex items-center gap-2">
    {#if mode === 'tree'}
      <div class="relative w-48">
        <Search size={12} class="absolute left-2.5 top-2 text-slate-400 dark:text-slate-500" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter JSON keys..."
          class="w-full bg-surface-950 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-800 rounded-md pl-7 pr-2 py-1 text-[11px] focus:outline-none focus:border-indigo-500"
        />
      </div>

      <button
        type="button"
        onclick={expandAll}
        class="px-2 py-1 text-[11px] font-semibold text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded transition-colors"
      >
        Expand All
      </button>

      <button
        type="button"
        onclick={collapseAll}
        class="px-2 py-1 text-[11px] font-semibold text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded transition-colors"
      >
        Collapse
      </button>
    {:else}
      <button
        type="button"
        onclick={() => formatBeautify(2)}
        class="flex items-center gap-1 px-2 py-1 text-[11px] font-semibold text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded transition-colors"
        title="Format with 2 spaces"
      >
        <Sparkles size={11} class="text-indigo-500" />
        <span>Prettify</span>
      </button>

      <button
        type="button"
        onclick={formatMinify}
        class="flex items-center gap-1 px-2 py-1 text-[11px] font-semibold text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded transition-colors"
        title="Minify JSON into single line"
      >
        <Minimize2 size={11} />
        <span>Minify</span>
      </button>
    {/if}
  </div>

  <!-- Copy Button -->
  <button
    type="button"
    onclick={copyJson}
    class="flex items-center gap-1 px-2.5 py-1 text-slate-700 dark:text-slate-300 bg-surface-950 border border-slate-200 dark:border-slate-800 hover:bg-surface-800 rounded-md font-semibold text-[11px] shadow-xs transition-colors"
  >
    {#if copied}
      <Check size={12} class="text-emerald-500" />
      <span class="text-emerald-500">Copied</span>
    {:else}
      <Copy size={12} />
      <span>Copy JSON</span>
    {/if}
  </button>
</div>

<!-- Validation Status Banner if in Raw Mode -->
{#if mode === 'raw'}
  <div class="px-3 py-1.5 border-b border-slate-200 dark:border-slate-800 text-[11px] flex items-center justify-between font-mono {parseError ? 'bg-rose-500/10 text-rose-600 dark:text-rose-400' : 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400'}">
    <div class="flex items-center gap-1.5 truncate">
      {#if parseError}
        <AlertCircle size={13} class="shrink-0 text-rose-500" />
        <span class="truncate">{parseError}</span>
      {:else}
        <CheckCircle2 size={13} class="shrink-0 text-emerald-500" />
        <span>Valid JSON Syntax</span>
      {/if}
    </div>
    <span class="text-[10px] text-slate-500 font-sans">{rawText.length} characters</span>
  </div>
{/if}

<!-- Inspector Content Body -->
<div class="flex-1 overflow-auto p-4 bg-surface-950 font-mono text-xs select-text">
  {#if mode === 'tree'}
    {#if parseError}
      <div class="p-4 bg-rose-500/10 border border-rose-500/20 rounded-xl text-rose-400 text-xs space-y-2">
        <p class="font-bold">Cannot render tree — Invalid JSON syntax:</p>
        <p class="text-[11px] font-mono bg-surface-900 p-2 rounded">{parseError}</p>
        <button
          type="button"
          onclick={() => mode = 'raw'}
          class="px-2.5 py-1 bg-rose-600 text-white rounded text-xs font-semibold"
        >
          Switch to Raw Editor to fix
        </button>
      </div>
    {:else}
      <div class="space-y-1">
        {#snippet renderNode(nodeVal: any, nodeKey: string, currentPath: string, depth: number)}
          {@const isObject = typeof nodeVal === 'object' && nodeVal !== null}
          {@const isArray = Array.isArray(nodeVal)}
          {@const isExpanded = expandedPaths.has(currentPath)}
          {@const entries = isObject ? Object.entries(nodeVal) : []}
          {@const matchesSearch = !searchQuery || nodeKey.toLowerCase().includes(searchQuery.toLowerCase()) || String(nodeVal).toLowerCase().includes(searchQuery.toLowerCase())}

          {#if matchesSearch || isObject}
            <div style="padding-left: {depth * 16}px;" class="py-0.5 group">
              {#if isObject}
                <div 
                  role="button"
                  tabindex="0"
                  onclick={() => toggleExpand(currentPath)}
                  onkeydown={(e) => { if (e.key === 'Enter') toggleExpand(currentPath); }}
                  class="inline-flex items-center gap-1.5 text-slate-800 dark:text-slate-200 hover:text-indigo-400 cursor-pointer rounded px-1 -mx-1 hover:bg-surface-900/60"
                >
                  {#if isExpanded}
                    <ChevronDown size={13} class="text-slate-400" />
                  {:else}
                    <ChevronRight size={13} class="text-slate-400" />
                  {/if}

                  <span class="font-bold text-indigo-600 dark:text-indigo-400">{nodeKey}:</span>
                  <span class="text-[11px] text-slate-400 dark:text-slate-500">
                    {isArray ? `Array(${entries.length})` : `Object{${entries.length}}`}
                  </span>
                </div>

                {#if isExpanded}
                  <div class="border-l border-slate-200 dark:border-slate-800 ml-2 mt-0.5 space-y-0.5">
                    {#each entries as [k, v] (`${currentPath}.${k}`)}
                      {@render renderNode(v, k, `${currentPath}.${k}`, depth + 1)}
                    {/each}
                  </div>
                {/if}
              {:else}
                <div class="inline-flex items-baseline gap-2 py-0.5 px-1 rounded hover:bg-surface-900/60">
                  <span class="text-indigo-600 dark:text-indigo-400 font-semibold">{nodeKey}:</span>
                  
                  {#if nodeVal === null}
                    <span class="text-slate-400 italic">null</span>
                  {:else if typeof nodeVal === 'boolean'}
                    <span class={nodeVal ? 'text-emerald-500 font-bold' : 'text-rose-500 font-bold'}>{nodeVal ? 'true' : 'false'}</span>
                  {:else if typeof nodeVal === 'number'}
                    <span class="text-amber-500 font-semibold">{nodeVal}</span>
                  {:else if typeof nodeVal === 'string'}
                    <span class="text-emerald-600 dark:text-emerald-400 break-all font-sans">"{nodeVal}"</span>
                  {:else}
                    <span class="text-slate-300">{String(nodeVal)}</span>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        {/snippet}

        {@render renderNode(parsedObject, 'root', 'root', 0)}
      </div>
    {/if}
  {:else}
    <!-- Raw Textarea Editor -->
    <textarea
      value={rawText}
      oninput={(e) => handleRawChange((e.currentTarget as HTMLTextAreaElement).value)}
      class="w-full h-full min-h-[300px] bg-transparent text-slate-900 dark:text-slate-100 font-mono text-xs focus:outline-none resize-none leading-relaxed"
      placeholder="Paste or edit JSON here..."
      spellcheck="false"
    ></textarea>
  {/if}
</div>
