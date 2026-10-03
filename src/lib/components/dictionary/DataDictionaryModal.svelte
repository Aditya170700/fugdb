<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    BookOpen, 
    X, 
    Copy, 
    Check, 
    Download, 
    Printer, 
    RefreshCw, 
    FileText, 
    Code, 
    Eye, 
    Search,
    Database,
    Sparkles,
    CheckCircle2,
    ExternalLink
  } from 'lucide-svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { api } from '$lib/api/client';
  import CustomSelect from '$lib/components/ui/CustomSelect.svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  // State
  let selectedConnectionId = $state<string>('');
  let activeTab = $state<'html' | 'markdown' | 'json'>('html');
  let isLoading = $state<boolean>(false);
  let isPrinting = $state<boolean>(false);
  let searchQuery = $state<string>('');
  let isCopied = $state<boolean>(false);
  let copyTimeout: any = null;

  // Generated contents
  let htmlContent = $state<string>('');
  let markdownContent = $state<string>('');
  let jsonContent = $state<string>('');
  let errorMessage = $state<string | null>(null);

  const connections = $derived(connectionStore.connections);
  const activeConn = $derived(connectionStore.activeConnection);

  $effect(() => {
    if (isOpen && activeConn && !selectedConnectionId) {
      selectedConnectionId = activeConn.id;
    }
  });

  $effect(() => {
    if (isOpen && selectedConnectionId) {
      loadDocumentation(selectedConnectionId);
    }
  });

  async function loadDocumentation(connId: string) {
    if (!connId) return;
    isLoading = true;
    errorMessage = null;

    try {
      // Ensure schema is loaded
      await connectionStore.loadSchema(connId);

      const [html, md, json] = await Promise.all([
        api.generateDataDictionary(connId, 'html'),
        api.generateDataDictionary(connId, 'markdown'),
        api.generateDataDictionary(connId, 'json'),
      ]);

      htmlContent = html;
      markdownContent = md;
      jsonContent = json;
    } catch (err: any) {
      console.error('Failed to generate data dictionary:', err);
      errorMessage = typeof err === 'string' ? err : err?.message || String(err);
    } finally {
      isLoading = false;
    }
  }

  async function copyToClipboard() {
    let contentToCopy = '';
    if (activeTab === 'markdown') contentToCopy = markdownContent;
    else if (activeTab === 'json') contentToCopy = jsonContent;
    else contentToCopy = htmlContent;

    try {
      await navigator.clipboard.writeText(contentToCopy);
      isCopied = true;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        isCopied = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  async function handleExportFile() {
    const conn = connections.find(c => c.id === selectedConnectionId);
    const dbName = conn?.name?.replace(/[^a-zA-Z0-9_-]/g, '_').toLowerCase() || 'database';

    let ext = 'html';
    let filterName = 'HTML Documentation (*.html)';
    let format: 'html' | 'markdown' | 'json' = 'html';

    if (activeTab === 'markdown') {
      ext = 'md';
      filterName = 'Markdown Document (*.md)';
      format = 'markdown';
    } else if (activeTab === 'json') {
      ext = 'json';
      filterName = 'JSON Schema Definition (*.json)';
      format = 'json';
    }

    const defaultName = `data_dictionary_${dbName}.${ext}`;
    const selected = await api.pickSaveFile(defaultName, [{ name: filterName, extensions: [ext] }]);

    if (selected) {
      try {
        await api.exportDataDictionaryFile(selectedConnectionId, format, selected);
      } catch (err: any) {
        console.error('Failed to save file:', err);
      }
    }
  }

  async function handlePrint() {
    if (!selectedConnectionId || isPrinting) return;
    isPrinting = true;
    try {
      await api.printDataDictionary(selectedConnectionId);
    } catch (err) {
      console.warn('Native print command failed, falling back to iframe print:', err);
      const iframe = document.getElementById('dict-preview-iframe') as HTMLIFrameElement;
      if (iframe && iframe.contentWindow) {
        iframe.contentWindow.focus();
        iframe.contentWindow.print();
      } else {
        window.print();
      }
    } finally {
      setTimeout(() => {
        isPrinting = false;
      }, 1500);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div 
    class="fixed inset-0 bg-slate-950/60 dark:bg-black/80 backdrop-blur-xs z-50 flex items-center justify-center p-3 sm:p-6 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <div 
      class="bg-white dark:bg-surface-900 border border-slate-200 dark:border-slate-800 w-full max-w-5xl h-[92vh] rounded-2xl shadow-2xl flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 animate-in zoom-in-95 duration-200"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-slate-50/90 dark:bg-surface-950/80">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-gradient-to-tr from-amber-500 to-orange-500 text-white rounded-xl shadow-md shadow-amber-500/20 shrink-0">
            <BookOpen size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="font-bold text-sm text-slate-900 dark:text-white">
                1-Click Data Dictionary & Schema Documentation
              </h3>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-400 font-bold border border-amber-500/30">
                HTML • Markdown • JSON
              </span>
            </div>
            <p class="text-[11px] text-slate-600 dark:text-slate-400">
              Interactive entity documentation, column definitions, keys, constraints, and printable exports
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <!-- Database Selector Dropdown -->
          <div class="w-56">
            <CustomSelect
              id="dict-conn-select"
              size="sm"
              bind:value={selectedConnectionId}
              options={connections.map(c => ({
                value: c.id,
                label: `${c.name} (${c.driver})`,
                badge: c.environment
              }))}
            />
          </div>

          <button 
            type="button"
            onclick={() => loadDocumentation(selectedConnectionId)}
            disabled={isLoading}
            class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-white rounded-lg hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer"
            title="Refresh Documentation"
          >
            <RefreshCw size={15} class={isLoading ? 'animate-spin text-amber-500' : ''} />
          </button>

          <button 
            type="button"
            onclick={onClose} 
            class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-white rounded-lg hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer" 
            title="Close (Esc)"
          >
            <X size={16} />
          </button>
        </div>
      </div>

      <!-- Action Bar & Tabs -->
      <div class="flex items-center justify-between border-b border-slate-200 dark:border-slate-800 bg-slate-100/70 dark:bg-surface-950/50 px-5 py-2">
        <!-- Mode Tabs -->
        <div class="flex items-center gap-1.5 text-xs font-semibold">
          <button 
            type="button"
            onclick={() => activeTab = 'html'}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all cursor-pointer {activeTab === 'html' ? 'bg-white dark:bg-surface-800 text-indigo-600 dark:text-indigo-400 border-slate-300 dark:border-slate-700 shadow-xs font-bold' : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            <Eye size={13} />
            <span>Interactive View (HTML)</span>
          </button>

          <button 
            type="button"
            onclick={() => activeTab = 'markdown'}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all cursor-pointer {activeTab === 'markdown' ? 'bg-white dark:bg-surface-800 text-indigo-600 dark:text-indigo-400 border-slate-300 dark:border-slate-700 shadow-xs font-bold' : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            <FileText size={13} />
            <span>Markdown (.md)</span>
          </button>

          <button 
            type="button"
            onclick={() => activeTab = 'json'}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all cursor-pointer {activeTab === 'json' ? 'bg-white dark:bg-surface-800 text-indigo-600 dark:text-indigo-400 border-slate-300 dark:border-slate-700 shadow-xs font-bold' : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            <Code size={13} />
            <span>JSON Schema</span>
          </button>
        </div>

        <!-- Export & Copy Actions -->
        <div class="flex items-center gap-2">
          {#if activeTab === 'html'}
            <button
              type="button"
              onclick={handlePrint}
              disabled={isPrinting || isLoading}
              class="px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white bg-white dark:bg-surface-800 border border-slate-200 dark:border-slate-700 rounded-lg flex items-center gap-1.5 cursor-pointer shadow-xs hover:bg-slate-50 dark:hover:bg-surface-700 transition-colors disabled:opacity-50"
              title="Open Print Dialog / Save as PDF"
            >
              {#if isPrinting}
                <RefreshCw size={13} class="animate-spin text-indigo-500" />
                <span>Opening PDF...</span>
              {:else}
                <Printer size={13} class="text-slate-500 dark:text-slate-400" />
                <span>Print / PDF</span>
              {/if}
            </button>
          {/if}

          <button
            type="button"
            onclick={copyToClipboard}
            class="px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white bg-white dark:bg-surface-800 border border-slate-200 dark:border-slate-700 rounded-lg flex items-center gap-1.5 cursor-pointer shadow-xs hover:bg-slate-50 dark:hover:bg-surface-700 transition-colors"
          >
            {#if isCopied}
              <Check size={13} class="text-emerald-600 dark:text-emerald-400" />
              <span class="text-emerald-600 dark:text-emerald-400 font-bold">Copied!</span>
            {:else}
              <Copy size={13} class="text-slate-500 dark:text-slate-400" />
              <span>Copy {activeTab === 'markdown' ? 'Markdown' : activeTab === 'json' ? 'JSON' : 'HTML'}</span>
            {/if}
          </button>

          <button
            type="button"
            onclick={handleExportFile}
            class="px-3.5 py-1.5 text-xs font-semibold text-white bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-500 hover:to-orange-500 rounded-lg flex items-center gap-1.5 cursor-pointer shadow-md shadow-amber-600/20 transition-all"
          >
            <Download size={13} />
            <span>Save .{activeTab === 'markdown' ? 'md' : activeTab === 'json' ? 'json' : 'html'}</span>
          </button>
        </div>
      </div>

      <!-- Modal Body (Preview Area) -->
      <div class="flex-1 overflow-hidden relative bg-slate-50/50 dark:bg-surface-950/60 min-h-0">
        {#if isLoading}
          <div class="absolute inset-0 bg-white/70 dark:bg-surface-900/70 backdrop-blur-xs flex flex-col items-center justify-center gap-3 z-20">
            <RefreshCw size={28} class="animate-spin text-amber-500" />
            <span class="text-xs font-semibold text-slate-700 dark:text-slate-300">Compiling Schema Documentation...</span>
          </div>
        {/if}

        {#if errorMessage}
          <div class="p-6">
            <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-700 dark:text-rose-300 text-xs">
              <strong class="block mb-1 font-bold">Error generating documentation:</strong>
              <span class="font-mono">{errorMessage}</span>
            </div>
          </div>
        {:else if activeTab === 'html'}
          <!-- HTML Sandbox Iframe for isolated styling & search -->
          <iframe 
            id="dict-preview-iframe"
            title="Data Dictionary Preview"
            srcdoc={htmlContent}
            class="w-full h-full border-0 bg-transparent"
          ></iframe>
        {:else if activeTab === 'markdown'}
          <!-- Markdown Monospace Code View -->
          <div class="w-full h-full overflow-y-auto p-4 font-mono text-xs">
            <pre class="bg-white dark:bg-surface-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 text-slate-800 dark:text-slate-200 overflow-x-auto whitespace-pre leading-relaxed select-text shadow-xs">{markdownContent}</pre>
          </div>
        {:else}
          <!-- JSON Schema View -->
          <div class="w-full h-full overflow-y-auto p-4 font-mono text-xs">
            <pre class="bg-white dark:bg-surface-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 text-indigo-700 dark:text-indigo-300 overflow-x-auto whitespace-pre leading-relaxed select-text shadow-xs">{jsonContent}</pre>
          </div>
        {/if}
      </div>

      <!-- Footer Info Bar -->
      <div class="px-5 py-2.5 border-t border-slate-200 dark:border-slate-800 bg-slate-50/90 dark:bg-surface-950/70 flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400">
        <div class="flex items-center gap-3">
          <span class="flex items-center gap-1 text-emerald-600 dark:text-emerald-400 font-semibold">
            <CheckCircle2 size={13} />
            Schema synced
          </span>
          <span>•</span>
          <span>Supports print-to-PDF, markdown documentation, and developer wikis</span>
        </div>

        <button 
          type="button"
          onclick={onClose} 
          class="px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white rounded-md hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
