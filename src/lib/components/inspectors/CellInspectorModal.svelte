<script lang="ts">
  import { 
    X, 
    Sparkles, 
    Save, 
    Clock, 
    Code, 
    Binary, 
    FileText, 
    Image as ImageIcon,
    Database,
    Tag,
    Undo,
    Check
  } from 'lucide-svelte';
  import { inspectorStore, type InspectorTab } from '$lib/state/inspector.svelte';
  import JsonInspector from './JsonInspector.svelte';
  import DateTimeInspector from './DateTimeInspector.svelte';
  import BinaryHashInspector from './BinaryHashInspector.svelte';
  import BlobTextInspector from './BlobTextInspector.svelte';

  let draftValue = $state<any>(null);
  let isModified = $state<boolean>(false);

  $effect(() => {
    if (inspectorStore.isOpen && inspectorStore.target) {
      draftValue = inspectorStore.target.value;
      isModified = false;
    }
  });

  function handleValueChange(newVal: any) {
    draftValue = newVal;
    isModified = JSON.stringify(newVal) !== JSON.stringify(inspectorStore.target?.value);
  }

  function handleApply() {
    if (inspectorStore.target?.onApply) {
      inspectorStore.target.onApply(draftValue);
    }
    inspectorStore.close();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && inspectorStore.isOpen) {
      inspectorStore.close();
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter' && inspectorStore.isOpen) {
      e.preventDefault();
      handleApply();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if inspectorStore.isOpen && inspectorStore.target}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-black/65 backdrop-blur-xs flex items-center justify-center p-4 sm:p-6 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <!-- Modal Container -->
    <div 
      class="w-full max-w-3xl h-[82vh] bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 animate-in zoom-in-95 duration-200"
    >
      <!-- Top Modal Header -->
      <div class="p-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-surface-950/70">
        <div class="flex items-center gap-2.5 truncate">
          <div class="p-2 bg-indigo-500/15 text-indigo-600 dark:text-indigo-400 rounded-lg border border-indigo-500/20 shrink-0">
            <Sparkles size={18} />
          </div>
          <div class="truncate">
            <div class="flex items-center gap-2 truncate">
              <h2 class="text-sm font-bold text-slate-900 dark:text-white truncate font-mono">
                {inspectorStore.target.columnName}
              </h2>
              {#if inspectorStore.target.dataType}
                <span class="text-[10px] uppercase font-mono px-1.5 py-0.2 rounded-full bg-surface-800 text-slate-400 border border-slate-700">
                  {inspectorStore.target.dataType}
                </span>
              {/if}
              {#if isModified}
                <span class="text-[10px] font-bold text-amber-500 bg-amber-500/10 border border-amber-500/30 px-1.5 py-0.5 rounded animate-pulse">
                  Modified
                </span>
              {/if}
            </div>
            <p class="text-[11px] text-slate-500 dark:text-slate-400 font-sans truncate">
              Table: <span class="text-indigo-400 font-mono">{inspectorStore.target.tableName || 'Active Result'}</span>
            </p>
          </div>
        </div>

        <!-- Tab Bar Selector inside header -->
        <div class="flex items-center gap-1 bg-surface-950 p-1 rounded-xl border border-slate-200 dark:border-slate-800 shrink-0">
          <button
            type="button"
            onclick={() => inspectorStore.activeTab = 'json'}
            class="flex items-center gap-1 px-2.5 py-1 rounded-lg font-semibold text-xs transition-colors cursor-pointer {inspectorStore.activeTab === 'json' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-400 hover:text-white'}"
            title="JSON Tree & Formatter"
          >
            <Code size={13} />
            <span>JSON</span>
          </button>

          <button
            type="button"
            onclick={() => inspectorStore.activeTab = 'datetime'}
            class="flex items-center gap-1 px-2.5 py-1 rounded-lg font-semibold text-xs transition-colors cursor-pointer {inspectorStore.activeTab === 'datetime' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-400 hover:text-white'}"
            title="Date & Timezone Converter"
          >
            <Clock size={13} />
            <span>Date & Time</span>
          </button>

          <button
            type="button"
            onclick={() => inspectorStore.activeTab = 'binary'}
            class="flex items-center gap-1 px-2.5 py-1 rounded-lg font-semibold text-xs transition-colors cursor-pointer {inspectorStore.activeTab === 'binary' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-400 hover:text-white'}"
            title="UUID / Base64 / Hex"
          >
            <Binary size={13} />
            <span>UUID & Hex</span>
          </button>

          <button
            type="button"
            onclick={() => inspectorStore.activeTab = 'text'}
            class="flex items-center gap-1 px-2.5 py-1 rounded-lg font-semibold text-xs transition-colors cursor-pointer {inspectorStore.activeTab === 'text' || inspectorStore.activeTab === 'image' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-400 hover:text-white'}"
            title="Text, Markdown & Media"
          >
            <FileText size={13} />
            <span>Text / Blob</span>
          </button>
        </div>

        <button
          type="button"
          onclick={() => inspectorStore.close()}
          class="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-surface-800 transition-colors cursor-pointer ml-2"
          title="Close (Esc)"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Inspector Body Switcher -->
      <div class="flex-1 flex flex-col overflow-hidden bg-surface-950">
        {#if inspectorStore.activeTab === 'json'}
          <JsonInspector value={draftValue} onChange={handleValueChange} />
        {:else if inspectorStore.activeTab === 'datetime'}
          <DateTimeInspector value={draftValue} onChange={handleValueChange} />
        {:else if inspectorStore.activeTab === 'binary'}
          <BinaryHashInspector value={draftValue} onChange={handleValueChange} />
        {:else}
          <BlobTextInspector value={draftValue} onChange={handleValueChange} />
        {/if}
      </div>

      <!-- Bottom Actions Footer -->
      <div class="p-3 border-t border-slate-200 dark:border-slate-800 bg-surface-900 flex items-center justify-between text-xs">
        <div class="flex items-center gap-2 text-slate-500 text-[11px] font-mono">
          <span>Tip: Press <kbd class="px-1.5 py-0.5 bg-surface-800 rounded text-slate-300 font-sans text-[10px]">Cmd+Enter</kbd> to Apply</span>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => inspectorStore.close()}
            class="px-3 py-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-surface-800 font-semibold transition-colors cursor-pointer"
          >
            Cancel
          </button>

          <button
            type="button"
            onclick={handleApply}
            class="flex items-center gap-1.5 px-3.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white font-bold rounded-lg shadow-sm transition-colors cursor-pointer"
            title="Apply changes to cell mutation buffer (Cmd+Enter)"
          >
            <Save size={13} />
            <span>Apply to Cell</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
