<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Sparkles,
    X,
    Play,
    Send,
    Code,
    Copy,
    Check,
    RefreshCw,
    Settings,
    History,
    ShieldCheck,
    Table as TableIcon,
    Database,
    ArrowRight,
    CornerDownLeft,
    CheckCircle2,
    AlertCircle,
    Info,
    Layers
  } from 'lucide-svelte';
  import { aiStore } from '$lib/state/ai.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import type { AiSqlResponse } from '$lib/api/types';
  import CustomSelect from '$lib/components/ui/CustomSelect.svelte';

  let userPrompt = $state<string>('');
  let selectedConnectionId = $state<string>('');
  let isCopied = $state<boolean>(false);
  let copyTimeout: any = null;
  let showHistory = $state<boolean>(false);
  let selectedTables = $state<string[]>([]);
  let isAllTablesSelected = $state<boolean>(true);

  const activeConn = $derived(connectionStore.activeConnection);
  const connections = $derived(connectionStore.connections);
  const currentSchema = $derived(
    selectedConnectionId ? connectionStore.schemas[selectedConnectionId] : undefined
  );
  const tables = $derived(currentSchema?.tables || []);

  $effect(() => {
    if (activeConn && !selectedConnectionId) {
      selectedConnectionId = activeConn.id;
    }
  });

  $effect(() => {
    if (selectedConnectionId && !connectionStore.schemas[selectedConnectionId]) {
      connectionStore.loadSchema(selectedConnectionId);
    }
  });

  const quickPrompts = [
    'Top 10 users with the highest order volume',
    'Find orders placed in the last 30 days grouped by status',
    'List all users who have not made any purchases yet',
    'Calculate average order total by country or city',
    'Find duplicate emails across all records',
    'Count new customer registrations by month',
  ];

  async function handleGenerate() {
    if (!userPrompt.trim() || aiStore.isGenerating) return;
    const tablesScope = isAllTablesSelected ? undefined : selectedTables;
    await aiStore.generateSql(userPrompt, tablesScope);
  }

  function handleInsertToEditor() {
    if (!aiStore.lastResponse?.sql) return;
    const activeTab = tabsStore.activeTab;
    if (activeTab && activeTab.type === 'sql') {
      activeTab.sql = aiStore.lastResponse.sql;
      aiStore.closeDrawer();
    } else {
      // Create new query tab with generated SQL
      tabsStore.openNewSqlTab(aiStore.lastResponse.sql, `AI: ${userPrompt.slice(0, 20)}...`);
      aiStore.closeDrawer();
    }
  }

  async function handleRunQuery() {
    if (!aiStore.lastResponse?.sql) return;
    tabsStore.openNewSqlTab(
      aiStore.lastResponse.sql,
      `AI: ${userPrompt.slice(0, 15)}...`
    );
    aiStore.closeDrawer();
    // Execute query in new tab
    if (tabsStore.activeTabId) {
      setTimeout(() => {
        tabsStore.runTabQuery(tabsStore.activeTabId);
      }, 100);
    }
  }

  async function copySql() {
    if (!aiStore.lastResponse?.sql) return;
    try {
      await navigator.clipboard.writeText(aiStore.lastResponse.sql);
      isCopied = true;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => isCopied = false, 2000);
    } catch (e) {
      console.error('Failed to copy SQL:', e);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      handleGenerate();
    }
  }

  function toggleTableSelection(tableName: string) {
    if (selectedTables.includes(tableName)) {
      selectedTables = selectedTables.filter(t => t !== tableName);
    } else {
      selectedTables = [...selectedTables, tableName];
    }
  }
</script>

{#if aiStore.isDrawerOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 bg-slate-950/40 dark:bg-black/60 backdrop-blur-xs z-40 transition-opacity animate-in fade-in duration-150"
    onclick={() => aiStore.closeDrawer()}
    role="presentation"
  ></div>

  <!-- Sliding Drawer -->
  <div
    class="fixed top-0 right-0 h-full w-full max-w-xl bg-white dark:bg-surface-900 border-l border-slate-200 dark:border-slate-800 shadow-2xl z-50 flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 animate-in slide-in-from-right duration-200"
    role="dialog"
    aria-modal="true"
  >
    <!-- Drawer Header -->
    <div class="px-5 py-3.5 border-b border-slate-200 dark:border-slate-800 bg-slate-50/90 dark:bg-surface-950/80 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-gradient-to-tr from-violet-600 to-indigo-600 text-white rounded-xl shadow-md shadow-violet-500/20 shrink-0">
          <Sparkles size={18} />
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h3 class="font-bold text-sm text-slate-900 dark:text-white">
              NL-to-SQL AI Copilot
            </h3>
            <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-violet-500/10 text-violet-700 dark:text-violet-300 font-bold border border-violet-500/30 flex items-center gap-1">
              {aiStore.provider === 'ollama' ? '🦙 Ollama Local' : `✨ ${aiStore.provider}`}
            </span>
          </div>
          <p class="text-[11px] text-slate-600 dark:text-slate-400">
            Ask in plain English or Indonesian — AI writes optimized, schema-aware SQL
          </p>
        </div>
      </div>

      <div class="flex items-center gap-1.5">
        <button
          type="button"
          onclick={() => showHistory = !showHistory}
          class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-white rounded-lg hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer {showHistory ? 'text-violet-600 dark:text-violet-400 bg-violet-50 dark:bg-violet-950/40' : ''}"
          title="Prompt History"
        >
          <History size={16} />
        </button>

        <button
          type="button"
          onclick={() => aiStore.openSettings()}
          class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-white rounded-lg hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer"
          title="AI Settings"
        >
          <Settings size={16} />
        </button>

        <button
          type="button"
          onclick={() => aiStore.closeDrawer()}
          class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-white rounded-lg hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer"
          title="Close (Esc)"
        >
          <X size={16} />
        </button>
      </div>
    </div>

    <!-- Active Model & Context Bar -->
    <div class="px-5 py-2 border-b border-slate-200 dark:border-slate-800 bg-slate-100/60 dark:bg-surface-950/40 flex items-center justify-between text-xs flex-wrap gap-2">
      <div class="flex items-center gap-2">
        <Database size={13} class="text-violet-600 dark:text-violet-400" />
        <span class="font-semibold text-slate-700 dark:text-slate-300">Target:</span>
        <span class="font-mono text-[11px] text-slate-900 dark:text-slate-100 font-bold">
          {activeConn?.name || 'No Active DB'}
        </span>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-[10px] text-slate-500">Model:</span>
        <span class="font-mono text-[10px] bg-white dark:bg-surface-800 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-700 font-semibold text-slate-700 dark:text-slate-300">
          {aiStore.model}
        </span>
      </div>
    </div>

    <!-- Drawer Body -->
    <div class="flex-1 overflow-y-auto p-5 space-y-4 text-xs bg-slate-50/40 dark:bg-surface-950/30">
      {#if showHistory}
        <!-- Prompt History View -->
        <div class="space-y-3 animate-in fade-in duration-150">
          <div class="flex items-center justify-between">
            <span class="font-bold text-xs text-slate-900 dark:text-slate-200 flex items-center gap-1.5">
              <History size={13} />
              Recent AI Queries ({aiStore.promptHistory.length})
            </span>
            <button
              type="button"
              onclick={() => showHistory = false}
              class="text-[11px] text-violet-600 dark:text-violet-400 hover:underline cursor-pointer font-semibold"
            >
              Back to Prompt
            </button>
          </div>

          {#if aiStore.promptHistory.length > 0}
            <div class="space-y-2">
              {#each aiStore.promptHistory as item}
                <button
                  type="button"
                  onclick={() => { userPrompt = item.prompt; showHistory = false; handleGenerate(); }}
                  class="w-full text-left p-3 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-surface-900 hover:border-violet-400 dark:hover:border-violet-600 transition-all cursor-pointer shadow-xs space-y-1.5 group"
                >
                  <div class="font-semibold text-slate-900 dark:text-slate-100 text-xs group-hover:text-violet-600 dark:group-hover:text-violet-400">
                    "{item.prompt}"
                  </div>
                  <pre class="font-mono text-[10px] text-slate-500 dark:text-slate-400 bg-slate-50 dark:bg-surface-950 p-1.5 rounded truncate">{item.sql}</pre>
                </button>
              {/each}
            </div>
          {:else}
            <div class="text-center py-12 text-slate-500">
              No prompt history yet. Start asking questions below!
            </div>
          {/if}
        </div>
      {:else}
        <!-- 1. Natural Language Prompt Input Card -->
        <div class="bg-white dark:bg-surface-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 shadow-xs space-y-3">
          <label for="ai-prompt-input" class="font-bold text-slate-900 dark:text-slate-200 text-xs block">
            What data would you like to retrieve or analyze?
          </label>

          <div class="relative">
            <textarea
              id="ai-prompt-input"
              bind:value={userPrompt}
              onkeydown={handleKeyDown}
              rows="3"
              placeholder="e.g. Tampilkan 10 customer dengan total belanjaan terbanyak di tahun 2026..."
              class="w-full bg-slate-50 dark:bg-surface-950 border border-slate-200 dark:border-slate-700 rounded-lg p-3 text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:border-violet-500 resize-none placeholder:text-slate-400 dark:placeholder:text-slate-500"
            ></textarea>
          </div>

          <div class="flex items-center justify-between flex-wrap gap-2">
            <div class="text-[10px] text-slate-500 flex items-center gap-1">
              <span>Press</span>
              <kbd class="px-1.5 py-0.5 bg-slate-100 dark:bg-surface-800 border border-slate-200 dark:border-slate-700 rounded font-mono font-bold">⌘/Ctrl + Enter</kbd>
              <span>to generate</span>
            </div>

            <button
              type="button"
              onclick={handleGenerate}
              disabled={!userPrompt.trim() || aiStore.isGenerating}
              class="px-4 py-1.5 bg-gradient-to-r from-violet-600 to-indigo-600 hover:from-violet-500 hover:to-indigo-500 text-white rounded-lg font-semibold text-xs flex items-center gap-1.5 cursor-pointer shadow-md shadow-violet-600/20 disabled:opacity-50 transition-all"
            >
              {#if aiStore.isGenerating}
                <RefreshCw size={13} class="animate-spin" />
                <span>Thinking & Writing SQL...</span>
              {:else}
                <Sparkles size={13} />
                <span>Generate SQL</span>
              {/if}
            </button>
          </div>
        </div>

        <!-- Quick Prompt Ideas Pills -->
        <div>
          <span class="text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400 block mb-1.5">
            Quick Prompt Ideas:
          </span>
          <div class="flex flex-wrap gap-1.5">
            {#each quickPrompts as prompt}
              <button
                type="button"
                onclick={() => { userPrompt = prompt; handleGenerate(); }}
                class="px-2.5 py-1 bg-white dark:bg-surface-900 hover:bg-violet-50 dark:hover:bg-violet-950/40 border border-slate-200 dark:border-slate-800 hover:border-violet-400 dark:hover:border-violet-600 text-[11px] text-slate-700 dark:text-slate-300 rounded-lg transition-colors cursor-pointer shadow-2xs text-left"
              >
                "{prompt}"
              </button>
            {/each}
          </div>
        </div>

        <!-- Error Alert if failed -->
        {#if aiStore.errorMessage}
          <div class="p-3.5 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-700 dark:text-rose-300 text-xs flex items-start gap-2.5 animate-in fade-in duration-150 overflow-hidden">
            <AlertCircle size={16} class="shrink-0 mt-0.5 text-rose-600" />
            <div class="space-y-1 min-w-0 flex-1">
              <strong class="font-bold block">Generation Failed:</strong>
              <p class="font-mono text-[11px] leading-relaxed break-words break-all whitespace-pre-wrap">{aiStore.errorMessage}</p>
              {#if aiStore.provider === 'ollama'}
                <p class="text-[10px] text-rose-600 dark:text-rose-400 mt-1">
                  Tip: Ensure Ollama is running (`ollama serve` or app active) with model `{aiStore.model}` installed.
                </p>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Generated SQL Result Card -->
        {#if aiStore.lastResponse}
          <div class="bg-white dark:bg-surface-900 rounded-xl border border-slate-200 dark:border-slate-800 shadow-xs overflow-hidden space-y-0 animate-in zoom-in-95 duration-200">
            <!-- Result Header -->
            <div class="px-4 py-2.5 border-b border-slate-200 dark:border-slate-800 bg-slate-50/80 dark:bg-surface-950/80 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Code size={14} class="text-violet-600 dark:text-violet-400" />
                <span class="font-bold text-xs text-slate-900 dark:text-slate-100">Generated SQL Query</span>
                <span class="text-[9px] font-mono px-1.5 py-0.2 rounded bg-violet-500/15 text-violet-700 dark:text-violet-300 font-bold">
                  {aiStore.lastResponse.dialect}
                </span>
              </div>

              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  onclick={copySql}
                  class="px-2.5 py-1 rounded bg-white dark:bg-surface-800 border border-slate-200 dark:border-slate-700 hover:bg-slate-100 text-slate-700 dark:text-slate-300 text-[11px] font-semibold flex items-center gap-1 cursor-pointer shadow-2xs transition-colors"
                >
                  {#if isCopied}
                    <Check size={12} class="text-emerald-500" />
                    <span class="text-emerald-500 font-bold">Copied</span>
                  {:else}
                    <Copy size={12} />
                    <span>Copy</span>
                  {/if}
                </button>
              </div>
            </div>

            <!-- SQL Monospace Block -->
            <div class="p-3 bg-slate-950 text-violet-300 font-mono text-xs overflow-x-auto select-text leading-relaxed">
              <pre>{aiStore.lastResponse.sql}</pre>
            </div>

            <!-- Explanation & Details -->
            {#if aiStore.lastResponse.explanation}
              <div class="p-3.5 border-t border-slate-200 dark:border-slate-800/80 bg-slate-50/50 dark:bg-surface-950/40 space-y-2">
                <div class="font-bold text-[11px] text-slate-800 dark:text-slate-200 flex items-center gap-1.5">
                  <Info size={13} class="text-violet-500" />
                  <span>Query Explanation:</span>
                </div>
                <div class="text-[11px] text-slate-600 dark:text-slate-300 whitespace-pre-line leading-relaxed select-text">
                  {aiStore.lastResponse.explanation}
                </div>

                {#if aiStore.lastResponse.tablesUsed.length > 0}
                  <div class="flex items-center gap-1.5 pt-1 flex-wrap">
                    <span class="text-[10px] text-slate-500">Tables involved:</span>
                    {#each aiStore.lastResponse.tablesUsed as tbl}
                      <span class="px-1.5 py-0.2 rounded bg-slate-200 dark:bg-surface-800 font-mono text-[10px] text-slate-700 dark:text-slate-300 font-semibold">
                        {tbl}
                      </span>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            <!-- Action Bar -->
            <div class="p-3 border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-surface-900 flex items-center justify-between gap-2">
              <span class="text-[10px] text-slate-400 font-mono">
                {aiStore.lastResponse.executionTimeMs.toFixed(1)}ms • {aiStore.lastResponse.modelUsed}
              </span>

              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={handleInsertToEditor}
                  class="px-3 py-1.5 rounded-lg border border-slate-200 dark:border-slate-700 bg-slate-100 hover:bg-slate-200 dark:bg-surface-800 dark:hover:bg-surface-700 text-slate-800 dark:text-slate-200 text-xs font-semibold flex items-center gap-1.5 cursor-pointer shadow-xs transition-colors"
                >
                  <Code size={13} />
                  <span>Insert to Editor</span>
                </button>

                <button
                  type="button"
                  onclick={handleRunQuery}
                  class="px-3.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold flex items-center gap-1.5 cursor-pointer shadow-md shadow-emerald-600/20 transition-all"
                >
                  <Play size={13} fill="currentColor" />
                  <span>Run Query Now</span>
                </button>
              </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </div>
{/if}
