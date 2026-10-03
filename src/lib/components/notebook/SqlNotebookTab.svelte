<script lang="ts">
  import { onMount } from 'svelte';
  import { notebookStore } from '$lib/state/notebook.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import MarkdownCell from './MarkdownCell.svelte';
  import SqlCell from './SqlCell.svelte';
  import { 
    BookText, 
    Play, 
    Plus, 
    FileCode2, 
    FileText, 
    Download, 
    Upload, 
    Share2, 
    Globe, 
    Database, 
    Sparkles, 
    Check, 
    Edit2,
    ChevronDown,
    Save,
    Printer,
    FileSpreadsheet,
    Layers
  } from 'lucide-svelte';

  let { tabId }: { tabId: string } = $props();

  const currentTab = $derived(tabsStore.tabs.find(t => t.id === tabId));
  const connections = $derived(connectionStore.connections);

  // Initialize or get document
  let docId = $state<string>('');
  
  onMount(() => {
    if (!docId) {
      // Check if doc exists for this tab or create new
      const existingDoc = notebookStore.getNotebook(tabId);
      if (existingDoc) {
        docId = tabId;
      } else {
        docId = notebookStore.createNotebook(currentTab?.title, currentTab?.connectionId);
      }
    }
  });

  const doc = $derived(docId ? notebookStore.getNotebook(docId) : undefined);
  const activeConn = $derived(
    connections.find(c => c.id === (doc?.defaultConnectionId || connectionStore.activeConnectionId))
  );

  let isEditingTitle = $state(false);
  let titleInput = $state('');
  let isExportDropdownOpen = $state(false);
  let isRunningAll = $state(false);

  function focusAction(node: HTMLInputElement) {
    node.focus();
  }

  function startEditTitle() {
    titleInput = doc?.title || 'Notebook';
    isEditingTitle = true;
  }

  function saveTitle() {
    if (docId && titleInput.trim()) {
      notebookStore.updateTitle(docId, titleInput.trim());
      if (currentTab) {
        currentTab.title = titleInput.trim();
      }
    }
    isEditingTitle = false;
  }

  async function handleRunAll() {
    if (!docId) return;
    isRunningAll = true;
    try {
      await notebookStore.runAllCells(docId);
    } finally {
      isRunningAll = false;
    }
  }

  function handleAddSql() {
    if (docId) {
      notebookStore.addCell(docId, 'sql');
    }
  }

  function handleAddMarkdown() {
    if (docId) {
      notebookStore.addCell(docId, 'markdown');
    }
  }

  async function handleSaveFugpad() {
    if (docId) {
      isExportDropdownOpen = false;
      await notebookStore.saveAsFugpadFile(docId);
    }
  }

  async function handleExportHtml() {
    if (docId) {
      isExportDropdownOpen = false;
      await notebookStore.exportHtmlReport(docId);
    }
  }

  async function handleExportMarkdown() {
    if (docId) {
      isExportDropdownOpen = false;
      await notebookStore.exportMarkdownReport(docId);
    }
  }

  async function handleOpenFile() {
    const loadedDocId = await notebookStore.openFugpadFile();
    if (loadedDocId) {
      docId = loadedDocId;
      const newDoc = notebookStore.getNotebook(loadedDocId);
      if (newDoc && currentTab) {
        currentTab.title = newDoc.title;
      }
    }
  }
</script>

<div class="flex-1 flex flex-col h-full bg-slate-100/70 dark:bg-surface-950 overflow-hidden select-text">
  
  <!-- Notebook Canvas Top Header Toolbar -->
  <header class="px-5 py-2.5 bg-white dark:bg-surface-900 border-b border-slate-200 dark:border-slate-800 flex flex-wrap items-center justify-between gap-3 shrink-0 shadow-2xs z-20">
    
    <!-- Left: Title & Tag -->
    <div class="flex items-center gap-3 min-w-0">
      <div class="p-2 rounded-lg bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20 shrink-0">
        <BookText size={18} />
      </div>

      <div class="flex items-center gap-2 min-w-0">
        {#if isEditingTitle}
          <input
            type="text"
            bind:value={titleInput}
            onblur={saveTitle}
            onkeydown={(e) => { if (e.key === 'Enter') saveTitle(); else if (e.key === 'Escape') isEditingTitle = false; }}
            class="px-2 py-0.5 text-sm font-bold bg-surface-950 border border-indigo-500 rounded text-slate-900 dark:text-slate-100 focus:outline-none"
            use:focusAction
          />
        {:else}
          <button
            type="button"
            onclick={startEditTitle}
            class="text-sm font-bold text-slate-900 dark:text-slate-100 truncate hover:text-indigo-600 dark:hover:text-indigo-400 flex items-center gap-1.5 transition-colors cursor-pointer text-left"
            title="Click to rename notebook"
          >
            <span>{doc?.title || 'Interactive SQL Notebook'}</span>
            <Edit2 size={11} class="opacity-50" />
          </button>
        {/if}

        <span class="px-1.5 py-0.2 rounded text-[10px] font-mono font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30 uppercase tracking-wider shrink-0">
          .fugpad
        </span>
      </div>
    </div>

    <!-- Center: Connection Picker -->
    <div class="flex items-center gap-2">
      <span class="text-xs text-slate-500 font-medium hidden md:inline">Database:</span>
      <select
        value={doc?.defaultConnectionId || connectionStore.activeConnectionId}
        onchange={(e) => {
          if (docId) notebookStore.setConnection(docId, (e.target as HTMLSelectElement).value);
        }}
        class="px-2.5 py-1 text-xs font-semibold bg-surface-950 border border-slate-300 dark:border-slate-800 rounded-lg text-slate-800 dark:text-slate-200 cursor-pointer max-w-[200px] truncate"
      >
        {#each connections as conn (conn.id)}
          <option value={conn.id}>{conn.name} ({conn.driver})</option>
        {/each}
      </select>
    </div>

    <!-- Right Actions: Run All, Add Cells, Export Dropdown -->
    <div class="flex items-center gap-2 shrink-0">
      
      <!-- Run All Cells Button -->
      <button
        type="button"
        disabled={isRunningAll}
        onclick={handleRunAll}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:bg-slate-300 dark:disabled:bg-slate-800 text-white font-bold text-xs shadow-xs transition-all active:scale-95 cursor-pointer"
        title="Execute all SQL cells sequentially (Cmd+Shift+Enter)"
      >
        <Play size={12} class={isRunningAll ? 'animate-spin' : 'fill-white'} />
        <span>{isRunningAll ? 'Running All...' : 'Run All'}</span>
      </button>

      <div class="w-[1px] h-4 bg-slate-200 dark:bg-slate-800 mx-0.5"></div>

      <!-- + SQL Cell -->
      <button
        type="button"
        onclick={handleAddSql}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-slate-800 dark:text-slate-200 border border-slate-300 dark:border-slate-700 text-xs font-semibold transition-colors cursor-pointer"
        title="Add new SQL query block"
      >
        <Plus size={13} class="text-indigo-500" />
        <span>+ SQL</span>
      </button>

      <!-- + Markdown Cell -->
      <button
        type="button"
        onclick={handleAddMarkdown}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-slate-800 dark:text-slate-200 border border-slate-300 dark:border-slate-700 text-xs font-semibold transition-colors cursor-pointer"
        title="Add new Markdown note block"
      >
        <Plus size={13} class="text-emerald-500" />
        <span>+ Text</span>
      </button>

      <!-- Open .fugpad File -->
      <button
        type="button"
        onclick={handleOpenFile}
        class="p-1.5 text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-surface-800 rounded-lg border border-slate-300 dark:border-slate-700 transition-colors cursor-pointer"
        title="Open .fugpad file from disk"
      >
        <Upload size={14} />
      </button>

      <!-- Export Menu Dropdown -->
      <div class="relative">
        <button
          type="button"
          onclick={() => isExportDropdownOpen = !isExportDropdownOpen}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-slate-800 dark:text-slate-200 border border-slate-300 dark:border-slate-700 text-xs font-semibold transition-colors cursor-pointer"
        >
          <Download size={13} />
          <span>Export</span>
          <ChevronDown size={12} class="opacity-70" />
        </button>

        {#if isExportDropdownOpen}
          <!-- Dropdown Menu -->
          <div 
            class="absolute right-0 top-full mt-1.5 w-56 bg-white dark:bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl shadow-xl py-1.5 z-50 text-xs animate-in fade-in zoom-in-95 duration-100"
          >
            <button
              type="button"
              onclick={handleSaveFugpad}
              class="w-full px-3 py-2 text-left hover:bg-surface-800 flex items-center gap-2.5 text-slate-800 dark:text-slate-200 transition-colors cursor-pointer"
            >
              <Save size={14} class="text-indigo-500" />
              <div>
                <div class="font-bold">Save as .fugpad</div>
                <div class="text-[10.5px] text-slate-400">Native JSON format</div>
              </div>
            </button>

            <button
              type="button"
              onclick={handleExportHtml}
              class="w-full px-3 py-2 text-left hover:bg-surface-800 flex items-center gap-2.5 text-slate-800 dark:text-slate-200 transition-colors cursor-pointer"
            >
              <Globe size={14} class="text-emerald-500" />
              <div>
                <div class="font-bold">Standalone HTML Report</div>
                <div class="text-[10.5px] text-slate-400">Interactive charts & tables</div>
              </div>
            </button>

            <button
              type="button"
              onclick={handleExportMarkdown}
              class="w-full px-3 py-2 text-left hover:bg-surface-800 flex items-center gap-2.5 text-slate-800 dark:text-slate-200 transition-colors cursor-pointer"
            >
              <FileText size={14} class="text-amber-500" />
              <div>
                <div class="font-bold">Markdown (.md)</div>
                <div class="text-[10.5px] text-slate-400">Wiki / Documentation</div>
              </div>
            </button>
          </div>
        {/if}
      </div>
    </div>
  </header>

  <!-- Main Scrollable Canvas Content -->
  <main class="flex-1 overflow-y-auto p-6 md:p-8">
    <div class="max-w-4xl mx-auto">
      
      {#if doc && doc.cells.length > 0}
        <!-- List of Cells -->
        {#each doc.cells as cell, idx (cell.id)}
          {#if cell.type === 'markdown'}
            <MarkdownCell 
              notebookId={doc.id} 
              cell={cell as any} 
              index={idx} 
              totalCells={doc.cells.length} 
            />
          {:else}
            <SqlCell 
              notebookId={doc.id} 
              cell={cell as any} 
              index={idx} 
              totalCells={doc.cells.length}
              defaultConnectionId={doc.defaultConnectionId}
            />
          {/if}
        {/each}

        <!-- Bottom Append Toolbar -->
        <div class="mt-6 pt-4 border-t border-dashed border-slate-300 dark:border-slate-800 flex items-center justify-center gap-3">
          <button
            type="button"
            onclick={handleAddSql}
            class="flex items-center gap-2 px-4 py-2 rounded-xl bg-indigo-600/10 hover:bg-indigo-600/20 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30 text-xs font-bold transition-all active:scale-95 cursor-pointer shadow-2xs"
          >
            <Plus size={14} />
            <span>Add SQL Query Block</span>
          </button>

          <button
            type="button"
            onclick={handleAddMarkdown}
            class="flex items-center gap-2 px-4 py-2 rounded-xl bg-emerald-600/10 hover:bg-emerald-600/20 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30 text-xs font-bold transition-all active:scale-95 cursor-pointer shadow-2xs"
          >
            <Plus size={14} />
            <span>Add Markdown Text Block</span>
          </button>
        </div>

      {:else}
        <!-- Empty State -->
        <div class="p-12 text-center border-2 border-dashed border-slate-300 dark:border-slate-800 rounded-2xl my-8">
          <BookText size={48} class="text-slate-400 mx-auto mb-3 opacity-60" />
          <h3 class="text-base font-bold text-slate-800 dark:text-slate-200">Empty Notebook</h3>
          <p class="text-xs text-slate-500 mt-1 max-w-sm mx-auto">Add a SQL query cell or Markdown text cell to start analyzing and documenting your database.</p>
          
          <div class="mt-5 flex items-center justify-center gap-3">
            <button
              type="button"
              onclick={handleAddSql}
              class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-bold transition-colors cursor-pointer"
            >
              + Add First SQL Cell
            </button>
            <button
              type="button"
              onclick={handleAddMarkdown}
              class="px-4 py-2 bg-surface-800 hover:bg-surface-700 text-slate-800 dark:text-slate-200 rounded-lg text-xs font-bold border border-slate-300 dark:border-slate-700 transition-colors cursor-pointer"
            >
              + Add Markdown Cell
            </button>
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>
