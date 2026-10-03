<script lang="ts">
  import { 
    Sparkles, 
    X, 
    Check, 
    Copy, 
    Play, 
    AlertCircle, 
    RefreshCw, 
    Database, 
    Settings, 
    Wrench,
    ArrowRight,
    CheckCircle2,
    Code,
    Cpu,
    ExternalLink
  } from 'lucide-svelte';
  import { aiStore } from '$lib/state/ai.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';

  const fixTarget = $derived(aiStore.fixTarget);
  const isOpen = $derived(aiStore.isFixModalOpen && fixTarget !== null);
  const activeConn = $derived(connectionStore.activeConnection);

  let copied = $state(false);
  let showFailedQuery = $state(false);

  function handleCopy(sql: string) {
    navigator.clipboard.writeText(sql);
    copied = true;
    setTimeout(() => { copied = false; }, 2000);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      aiStore.closeFixModal();
    } else if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      if (fixTarget?.fixedResponse?.sql && !fixTarget.isFixing) {
        e.preventDefault();
        aiStore.applyFix(fixTarget.tabId, true);
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen && fixTarget}
  <!-- Backdrop Container -->
  <div 
    class="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <!-- Click outside backdrop blocker -->
    <button 
      type="button"
      aria-label="Close modal backdrop"
      onclick={() => aiStore.closeFixModal()}
      class="fixed inset-0 bg-transparent border-0 cursor-default p-0 m-0 z-0"
    ></button>

    <!-- Modal Dialog Window -->
    <div 
      class="relative z-1 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-150"
    >
      <!-- Header -->
      <div class="px-5 py-4 border-b border-slate-200 dark:border-slate-800 bg-surface-950/50 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-violet-600 to-indigo-600 flex items-center justify-center text-white shadow-md shadow-violet-500/20">
            <Wrench size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-bold text-slate-900 dark:text-white">Fix Query with AI Copilot</h3>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-violet-500/10 text-violet-600 dark:text-violet-400 border border-violet-500/20 font-mono">
                {activeConn?.driver?.toUpperCase() || 'SQL'}
              </span>
            </div>
            <p class="text-[11px] text-slate-500 dark:text-slate-400">
              Auto-diagnosing schema constraints and dialect syntax with <span class="font-medium text-slate-700 dark:text-slate-300 capitalize">{aiStore.provider}</span> ({aiStore.model})
            </p>
          </div>
        </div>

        <div class="flex items-center gap-1.5">
          <button
            type="button"
            onclick={() => {
              aiStore.openSettings();
            }}
            class="p-1.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-surface-800 rounded-lg transition-colors cursor-pointer"
            title="Configure AI Provider & API Keys"
          >
            <Settings size={16} />
          </button>
          <button
            type="button"
            onclick={() => aiStore.closeFixModal()}
            class="p-1.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-surface-800 rounded-lg transition-colors cursor-pointer"
            title="Close (Esc)"
          >
            <X size={17} />
          </button>
        </div>
      </div>

      <!-- Modal Body (Scrollable) -->
      <div class="p-5 overflow-y-auto space-y-4 flex-1">
        <!-- 1. Database Error Summary Card -->
        <div class="bg-rose-500/10 border border-rose-500/30 rounded-xl p-3.5 space-y-2">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1.5 text-rose-600 dark:text-rose-400 font-bold text-xs uppercase tracking-wider">
              <AlertCircle size={14} />
              <span>Database Execution Error</span>
            </div>
            <button
              type="button"
              onclick={() => showFailedQuery = !showFailedQuery}
              class="text-[11px] text-rose-700 dark:text-rose-300 hover:underline font-semibold cursor-pointer"
            >
              {showFailedQuery ? 'Hide Failed SQL' : 'View Failed SQL'}
            </button>
          </div>

          <p class="font-mono text-rose-950 dark:text-rose-200 text-xs bg-surface-950/80 p-2.5 rounded-lg border border-rose-500/20 break-words font-medium select-text leading-relaxed">
            {fixTarget.errorMessage}
          </p>

          {#if showFailedQuery}
            <div class="pt-1 animate-in fade-in duration-100">
              <span class="text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-wider font-bold block mb-1">Failed Query:</span>
              <pre class="bg-surface-950 p-2.5 rounded-lg border border-slate-300 dark:border-slate-800 text-slate-600 dark:text-slate-300 font-mono text-xs overflow-x-auto select-text whitespace-pre-wrap">{fixTarget.failedSql}</pre>
            </div>
          {/if}
        </div>

        <!-- 2. AI Diagnosis & Fixed Query Section -->
        {#if fixTarget.isFixing}
          <!-- Loading State -->
          <div class="p-8 border border-slate-200 dark:border-slate-800/80 rounded-xl bg-surface-950/40 flex flex-col items-center justify-center text-center space-y-3">
            <div class="relative flex items-center justify-center">
              <div class="w-12 h-12 rounded-2xl bg-violet-500/10 dark:bg-violet-500/20 flex items-center justify-center text-violet-600 dark:text-violet-400">
                <Sparkles size={22} class="animate-pulse" />
              </div>
              <RefreshCw size={44} class="animate-spin text-violet-500 absolute inset-0 m-auto" />
            </div>

            <div class="space-y-1">
              <h4 class="text-xs font-bold text-slate-800 dark:text-slate-200">
                AI Copilot is analyzing query error...
              </h4>
              <p class="text-[11px] text-slate-500 dark:text-slate-400 max-w-sm">
                Inspecting table schemas, foreign key relationships, and <span class="capitalize font-semibold">{activeConn?.driver || 'SQL'}</span> dialect syntax rules.
              </p>
            </div>
          </div>

        {:else if fixTarget.fixError}
          <!-- AI Fix Error State -->
          <div class="bg-amber-500/10 border border-amber-500/30 rounded-xl p-4 space-y-3">
            <div class="flex items-center gap-2 text-amber-700 dark:text-amber-400 font-bold text-xs">
              <AlertCircle size={15} />
              <span>Could not generate fix from AI provider</span>
            </div>
            <p class="text-xs text-amber-900 dark:text-amber-200 font-mono bg-surface-950/60 p-2.5 rounded-lg border border-amber-500/20 break-words">
              {fixTarget.fixError}
            </p>

            <div class="flex items-center gap-2 pt-1">
              <button
                type="button"
                onclick={() => aiStore.retryFix()}
                class="px-3 py-1.5 bg-amber-600 hover:bg-amber-500 text-white font-bold rounded-lg text-xs transition-colors flex items-center gap-1.5 cursor-pointer shadow-xs"
              >
                <RefreshCw size={12} />
                <span>Retry Fix</span>
              </button>
              <button
                type="button"
                onclick={() => aiStore.openSettings()}
                class="px-3 py-1.5 bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 font-semibold rounded-lg text-xs transition-colors flex items-center gap-1.5 cursor-pointer border border-slate-200 dark:border-slate-700"
              >
                <Settings size={12} />
                <span>Check AI API Key / Settings</span>
              </button>
            </div>
          </div>

        {:else if fixTarget.fixedResponse}
          <!-- AI Fix Success State -->
          <div class="space-y-3.5 animate-in fade-in duration-200">
            <!-- Root Cause & Explanation Card -->
            <div class="bg-violet-500/10 dark:bg-violet-950/30 border border-violet-500/30 rounded-xl p-3.5 space-y-1.5">
              <div class="flex items-center gap-1.5 text-violet-700 dark:text-violet-400 font-bold text-xs">
                <Sparkles size={13} />
                <span>Root Cause & Solution</span>
              </div>
              <p class="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-line leading-relaxed">
                {fixTarget.fixedResponse.explanation}
              </p>
            </div>

            <!-- Fixed SQL Code Block -->
            <div class="border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden bg-surface-950">
              <div class="px-3.5 py-2 bg-surface-900 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between text-xs">
                <div class="flex items-center gap-2">
                  <Code size={13} class="text-emerald-500" />
                  <span class="font-bold text-slate-800 dark:text-slate-200 text-xs">Corrected SQL Query</span>
                  <span class="px-1.5 py-0.2 rounded text-[10px] font-mono font-bold bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30">
                    Fixed
                  </span>
                </div>

                <button
                  type="button"
                  onclick={() => handleCopy(fixTarget.fixedResponse?.sql || '')}
                  class="flex items-center gap-1 text-[11px] font-semibold text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-400 cursor-pointer"
                  title="Copy fixed SQL"
                >
                  {#if copied}
                    <Check size={12} class="text-emerald-500" />
                    <span class="text-emerald-500">Copied!</span>
                  {:else}
                    <Copy size={12} />
                    <span>Copy</span>
                  {/if}
                </button>
              </div>

              <div class="p-3 font-mono text-xs text-slate-800 dark:text-emerald-300 overflow-x-auto max-h-56 select-text whitespace-pre-wrap leading-relaxed">
                {fixTarget.fixedResponse.sql}
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3.5 border-t border-slate-200 dark:border-slate-800 bg-surface-950/70 flex items-center justify-between shrink-0">
        <span class="text-[11px] text-slate-500 dark:text-slate-400 font-mono">
          Press <kbd class="px-1.5 py-0.5 rounded bg-surface-800 text-slate-300 font-mono">⌘↵</kbd> to Apply & Run
        </span>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => aiStore.closeFixModal()}
            class="px-3.5 py-1.5 text-xs font-semibold text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white hover:bg-surface-800 rounded-lg transition-colors cursor-pointer"
          >
            Cancel
          </button>

          <button
            type="button"
            disabled={!fixTarget?.fixedResponse?.sql || fixTarget.isFixing}
            onclick={() => aiStore.applyFix(fixTarget.tabId, false)}
            class="px-3.5 py-1.5 bg-surface-800 hover:bg-surface-700 text-slate-800 dark:text-slate-200 font-semibold rounded-lg text-xs transition-colors border border-slate-300 dark:border-slate-700 disabled:opacity-40 cursor-pointer"
            title="Replace query in editor without running"
          >
            <span>Apply to Editor</span>
          </button>

          <button
            type="button"
            disabled={!fixTarget?.fixedResponse?.sql || fixTarget.isFixing}
            onclick={() => aiStore.applyFix(fixTarget.tabId, true)}
            class="px-4 py-1.5 bg-gradient-to-r from-violet-600 to-indigo-600 hover:from-violet-500 hover:to-indigo-500 active:from-violet-700 active:to-indigo-700 text-white font-bold rounded-lg text-xs shadow-md shadow-indigo-500/20 transition-all flex items-center gap-1.5 disabled:opacity-40 cursor-pointer"
            title="Replace query and execute immediately (Cmd+Enter)"
          >
            <Play size={12} class="fill-current" />
            <span>Apply & Run</span>
            <span class="text-[10px] opacity-75 font-mono">⌘↵</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
