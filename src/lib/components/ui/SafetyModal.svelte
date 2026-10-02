<script lang="ts">
  import { 
    ShieldAlert, 
    AlertTriangle, 
    X, 
    Flame, 
    CheckCircle2, 
    Database, 
    ArrowRight 
  } from 'lucide-svelte';
  import { safetyStore } from '$lib/state/safety.svelte';

  let confirmInput = $state('');

  // Target text user must type to confirm: database name, or "CONFIRM" if empty
  const requiredText = $derived(
    safetyStore.databaseName && safetyStore.databaseName.trim() 
      ? safetyStore.databaseName.trim() 
      : 'CONFIRM'
  );

  const isConfirmed = $derived(
    confirmInput.trim().toLowerCase() === requiredText.toLowerCase() ||
    confirmInput.trim().toUpperCase() === 'CONFIRM'
  );

  $effect(() => {
    if (safetyStore.isOpen) {
      confirmInput = '';
    }
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      safetyStore.cancel();
    }
    if (e.key === 'Enter' && isConfirmed) {
      safetyStore.confirm();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if safetyStore.isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-black/75 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <div 
      class="w-full max-w-xl bg-slate-900 border border-rose-500/40 rounded-xl shadow-2xl flex flex-col overflow-hidden text-slate-100 animate-in zoom-in-95 duration-150"
    >
      <!-- Header with Crimson Glow -->
      <div class="p-5 bg-rose-950/40 border-b border-rose-500/30 flex items-start gap-3.5">
        <div class="p-2.5 bg-rose-500/20 text-rose-400 rounded-lg shrink-0 border border-rose-500/30 shadow-inner">
          <ShieldAlert size={26} class="animate-pulse" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-xs font-black uppercase tracking-widest px-2 py-0.5 rounded bg-rose-500 text-white shadow-xs">
              {safetyStore.environment.toUpperCase()} GUARD
            </span>
            <span class="text-xs font-mono text-rose-300 font-semibold truncate">
              {safetyStore.connectionName}
            </span>
          </div>
          <h2 class="text-lg font-bold text-white mt-1">
            Destructive Query Intercepted
          </h2>
          <p class="text-xs text-rose-200/80 mt-0.5 leading-relaxed">
            You are attempting to execute a potentially destructive query on a protected environment.
          </p>
        </div>
        <button 
          type="button"
          onclick={() => safetyStore.cancel()}
          class="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition-colors cursor-pointer"
          title="Cancel"
        >
          <X size={18} />
        </button>
      </div>

      <!-- Body -->
      <div class="p-5 space-y-4 max-h-[65vh] overflow-y-auto">
        <!-- Detected Risks -->
        {#if safetyStore.risk && safetyStore.risk.reasons.length > 0}
          <div class="space-y-2">
            <span class="text-xs font-bold uppercase tracking-wider text-rose-400 flex items-center gap-1.5">
              <AlertTriangle size={14} />
              Detected Risk Factors ({safetyStore.risk.reasons.length})
            </span>
            <div class="bg-rose-950/30 border border-rose-500/20 rounded-lg p-3 space-y-1.5">
              {#each safetyStore.risk.reasons as reason}
                <div class="flex items-start gap-2 text-xs text-rose-200">
                  <Flame size={14} class="text-rose-400 shrink-0 mt-0.5" />
                  <span class="leading-relaxed font-medium">{reason}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- SQL Query Script Preview -->
        <div class="space-y-1.5">
          <div class="flex items-center justify-between text-xs text-slate-400">
            <span class="font-semibold uppercase tracking-wider text-[11px]">SQL Script Preview</span>
            <span class="font-mono text-[10px] text-slate-500">Database: {safetyStore.databaseName || 'current'}</span>
          </div>
          <div class="bg-slate-950 border border-slate-800 rounded-lg p-3 font-mono text-xs text-slate-200 max-h-36 overflow-y-auto whitespace-pre-wrap break-all select-text selection:bg-rose-500/40">
            {safetyStore.sql}
          </div>
        </div>

        <!-- Confirmation Input -->
        <div class="space-y-2 pt-2 border-t border-slate-800">
          <label for="safety-confirm-input" class="text-xs font-semibold text-slate-300 block">
            To proceed, type <code class="px-1.5 py-0.5 bg-slate-800 text-amber-300 rounded font-mono font-bold text-[11px] border border-slate-700">{requiredText}</code> below:
          </label>
          <div class="relative">
            <input 
              id="safety-confirm-input"
              type="text" 
              bind:value={confirmInput}
              placeholder="Type database name or CONFIRM..."
              autocomplete="off"
              class="w-full bg-slate-950 text-white font-mono text-xs rounded-lg px-3 py-2.5 border {isConfirmed ? 'border-emerald-500 focus:border-emerald-500' : 'border-slate-700 focus:border-rose-500'} focus:outline-none transition-colors"
            />
            {#if isConfirmed}
              <CheckCircle2 size={16} class="absolute right-3 top-3 text-emerald-400" />
            {/if}
          </div>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="p-4 bg-slate-950/80 border-t border-slate-800 flex items-center justify-between gap-3">
        <button 
          type="button"
          onclick={() => safetyStore.cancel()}
          class="px-4 py-2 rounded-lg text-xs font-semibold text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 transition-colors cursor-pointer"
        >
          Cancel (Esc)
        </button>

        <button 
          type="button"
          disabled={!isConfirmed}
          onclick={() => safetyStore.confirm()}
          class="px-4 py-2 rounded-lg text-xs font-bold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed {isConfirmed ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-lg shadow-rose-900/40' : 'bg-slate-800 text-slate-400'}"
        >
          <span>Execute on Production</span>
          <ArrowRight size={14} />
        </button>
      </div>
    </div>
  </div>
{/if}
