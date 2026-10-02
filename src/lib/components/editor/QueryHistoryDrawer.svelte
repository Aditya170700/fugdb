<script lang="ts">
  import { 
    History, 
    Star, 
    Search, 
    X, 
    CheckCircle2, 
    XCircle, 
    Copy, 
    Check, 
    Trash2, 
    Play, 
    CornerDownLeft, 
    Database, 
    Filter,
    Clock,
    Sparkles,
    Tag,
    Edit3
  } from 'lucide-svelte';
  import { historyStore, type QueryHistoryItem } from '$lib/state/history.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import CustomSelect from '$lib/components/ui/CustomSelect.svelte';

  let copiedId = $state<string | null>(null);
  let editingFavoriteId = $state<string | null>(null);
  let editingLabelValue = $state<string>('');

  function startEditingLabel(item: QueryHistoryItem) {
    editingFavoriteId = item.id;
    editingLabelValue = item.favoriteLabel || '';
  }

  function saveFavoriteLabel(id: string) {
    historyStore.setFavoriteLabel(id, editingLabelValue.trim());
    editingFavoriteId = null;
    editingLabelValue = '';
  }

  function formatTime(timestamp: number): string {
    const diff = Date.now() - timestamp;
    if (diff < 60_000) return 'Just now';
    if (diff < 3600_000) return `${Math.floor(diff / 60_000)}m ago`;
    if (diff < 86400_000) return `${Math.floor(diff / 3600_000)}h ago`;
    return new Date(timestamp).toLocaleDateString(undefined, { 
      month: 'short', 
      day: 'numeric', 
      hour: '2-digit', 
      minute: '2-digit' 
    });
  }

  async function copyToClipboard(item: QueryHistoryItem) {
    try {
      await navigator.clipboard.writeText(item.sql);
      copiedId = item.id;
      setTimeout(() => {
        if (copiedId === item.id) copiedId = null;
      }, 1500);
    } catch (err) {
      console.error('Failed to copy SQL:', err);
    }
  }

  function openInNewTab(item: QueryHistoryItem) {
    tabsStore.openNewSqlTab(item.sql, item.favoriteLabel || `Query (History)`);
    historyStore.isOpen = false;
  }

  function insertIntoCurrentTab(item: QueryHistoryItem) {
    const activeTab = tabsStore.activeTab;
    if (activeTab) {
      if (activeTab.type === 'sql') {
        activeTab.sql = item.sql;
      } else {
        tabsStore.openNewSqlTab(item.sql);
      }
    } else {
      tabsStore.openNewSqlTab(item.sql);
    }
    historyStore.isOpen = false;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (editingFavoriteId) {
        editingFavoriteId = null;
      } else if (historyStore.isOpen) {
        historyStore.isOpen = false;
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if historyStore.isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex justify-end animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <!-- Drawer Container -->
    <div 
      class="w-full max-w-xl h-full bg-surface-900 border-l border-slate-200 dark:border-slate-800 shadow-2xl flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 animate-in slide-in-from-right duration-200"
    >
      <!-- Header -->
      <div class="p-4 border-b border-slate-200 dark:border-slate-800 flex flex-col gap-3 bg-surface-950/50">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2.5">
            <div class="p-2 bg-indigo-500/15 text-indigo-600 dark:text-indigo-400 rounded-lg border border-indigo-500/20">
              <History size={18} />
            </div>
            <div>
              <h2 class="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
                <span>Query History & Favorites</span>
                <span class="text-[11px] font-normal text-slate-500 dark:text-slate-400">
                  ({historyStore.items.length})
                </span>
              </h2>
              <p class="text-[11px] text-slate-500 dark:text-slate-400">
                Persistent execution log & pinned queries
              </p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            {#if historyStore.items.length > 0}
              <button 
                type="button"
                onclick={() => {
                  if (confirm('Clear non-favorite query history?')) {
                    historyStore.clearHistory();
                  }
                }}
                class="px-2 py-1 text-[11px] text-slate-500 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/10 rounded transition-colors cursor-pointer"
                title="Clear non-starred history"
              >
                Clear
              </button>
            {/if}
            <button 
              type="button"
              onclick={() => historyStore.isOpen = false}
              class="p-1.5 text-slate-400 hover:text-slate-900 dark:hover:text-white rounded-lg hover:bg-surface-800 transition-colors cursor-pointer"
              title="Close (Esc)"
            >
              <X size={16} />
            </button>
          </div>
        </div>

        <!-- Search Input -->
        <div class="relative">
          <Search size={13} class="absolute left-3 top-2.5 text-slate-400 dark:text-slate-500" />
          <input 
            type="text" 
            bind:value={historyStore.searchQuery}
            placeholder="Search queries by SQL, database, or tag..."
            class="w-full bg-surface-900 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-700/80 text-xs rounded-lg pl-8.5 pr-3 py-2 focus:outline-none focus:border-indigo-500 placeholder:text-slate-400 dark:placeholder:text-slate-500 shadow-xs"
          />
          {#if historyStore.searchQuery}
            <button 
              type="button"
              onclick={() => historyStore.searchQuery = ''}
              class="absolute right-2.5 top-2.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
            >
              <X size={12} />
            </button>
          {/if}
        </div>

        <!-- Filter Tabs & Connection Filter -->
        <div class="flex items-center justify-between gap-2 overflow-x-auto text-xs pb-0.5">
          <div class="flex items-center gap-1.5 shrink-0">
            <button 
              type="button"
              onclick={() => historyStore.filterStatus = 'all'}
              class="px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {historyStore.filterStatus === 'all' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:bg-surface-800'}"
            >
              All ({historyStore.items.length})
            </button>

            <button 
              type="button"
              onclick={() => historyStore.filterStatus = 'favorite'}
              class="flex items-center gap-1 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {historyStore.filterStatus === 'favorite' ? 'bg-amber-500 text-slate-950 font-bold shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:bg-surface-800'}"
            >
              <Star size={11} class={historyStore.filterStatus === 'favorite' ? 'fill-slate-950' : 'text-amber-500'} />
              <span>Favorites ({historyStore.favoriteCount})</span>
            </button>

            <button 
              type="button"
              onclick={() => historyStore.filterStatus = 'success'}
              class="flex items-center gap-1 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {historyStore.filterStatus === 'success' ? 'bg-emerald-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:bg-surface-800'}"
            >
              <CheckCircle2 size={11} />
              <span>Success ({historyStore.successCount})</span>
            </button>

            <button 
              type="button"
              onclick={() => historyStore.filterStatus = 'error'}
              class="flex items-center gap-1 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {historyStore.filterStatus === 'error' ? 'bg-rose-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:bg-surface-800'}"
            >
              <XCircle size={11} />
              <span>Errors ({historyStore.errorCount})</span>
            </button>
          </div>

          <!-- Connection filter select -->
          <div class="w-40 shrink-0">
            <CustomSelect
              size="sm"
              bind:value={historyStore.selectedConnectionFilter}
              options={[
                { value: 'all', label: 'All Connections' },
                ...connectionStore.connections.map(conn => ({
                  value: conn.id,
                  label: conn.name
                }))
              ]}
            />
          </div>
        </div>
      </div>

      <!-- History List Body -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        {#if historyStore.filteredItems.length === 0}
          <div class="h-64 flex flex-col items-center justify-center text-center p-6 text-slate-400 dark:text-slate-500 gap-2">
            <Clock size={32} class="stroke-1 opacity-60" />
            {#if historyStore.searchQuery || historyStore.filterStatus !== 'all' || historyStore.selectedConnectionFilter !== 'all'}
              <p class="text-xs font-semibold">No queries match your current filter.</p>
              <button 
                type="button"
                onclick={() => { historyStore.searchQuery = ''; historyStore.filterStatus = 'all'; historyStore.selectedConnectionFilter = 'all'; }}
                class="text-xs text-indigo-500 hover:underline cursor-pointer"
              >
                Reset filters
              </button>
            {:else}
              <p class="text-xs font-semibold">No query history recorded yet.</p>
              <p class="text-[11px] max-w-xs text-slate-500">
                Queries executed via <kbd class="px-1 py-0.5 bg-surface-800 rounded font-mono text-[10px]">Cmd+Enter</kbd> will automatically appear here.
              </p>
            {/if}
          </div>
        {:else}
          {#each historyStore.filteredItems as item (item.id)}
            <div 
              class="group relative bg-surface-950 border border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 rounded-xl p-3.5 flex flex-col gap-2.5 shadow-xs hover:shadow-md transition-all"
            >
              <!-- Card Top Header -->
              <div class="flex items-center justify-between gap-2 text-[11px]">
                <div class="flex items-center gap-2 truncate">
                  {#if item.status === 'success'}
                    <span class="flex items-center gap-1 font-bold text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-1.5 py-0.5 rounded text-[10px] font-mono">
                      <Check size={10} class="stroke-3" />
                      <span>{item.durationMs.toFixed(0)}ms</span>
                      {#if item.rowCount != null}
                        <span>• {item.rowCount} rows</span>
                      {/if}
                    </span>
                  {:else}
                    <span class="flex items-center gap-1 font-bold text-rose-600 dark:text-rose-400 bg-rose-500/10 border border-rose-500/20 px-1.5 py-0.5 rounded text-[10px] font-mono">
                      <X size={10} class="stroke-3" />
                      <span>Failed</span>
                    </span>
                  {/if}

                  <span class="text-slate-500 dark:text-slate-400 font-mono truncate text-[10.5px]">
                    {item.connectionName} • {item.database}
                  </span>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                  <span class="text-[10px] text-slate-400 dark:text-slate-500 font-mono">
                    {formatTime(item.executedAt)}
                  </span>

                  <!-- Favorite Star Button -->
                  <button 
                    type="button"
                    onclick={() => historyStore.toggleFavorite(item.id)}
                    class="p-1 rounded hover:bg-surface-800 transition-colors cursor-pointer {item.isFavorite ? 'text-amber-500' : 'text-slate-400 dark:text-slate-500 hover:text-amber-400'}"
                    title={item.isFavorite ? "Remove from Favorites" : "Pin to Favorites"}
                  >
                    <Star size={14} class={item.isFavorite ? 'fill-amber-500' : ''} />
                  </button>
                </div>
              </div>

              <!-- Favorite Custom Label Tag or Editor -->
              {#if editingFavoriteId === item.id}
                <div class="flex items-center gap-1.5 bg-surface-900 border border-indigo-500/50 rounded-lg p-1.5">
                  <Tag size={12} class="text-indigo-400 shrink-0" />
                  <input
                    type="text"
                    bind:value={editingLabelValue}
                    placeholder="Enter custom query name (e.g. Monthly Active Users)..."
                    class="flex-1 bg-transparent text-xs text-slate-100 focus:outline-none placeholder:text-slate-500"
                    onkeydown={(e) => {
                      if (e.key === 'Enter') saveFavoriteLabel(item.id);
                      if (e.key === 'Escape') editingFavoriteId = null;
                    }}
                  />
                  <button
                    type="button"
                    onclick={() => saveFavoriteLabel(item.id)}
                    class="px-2 py-0.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-[10.5px] font-semibold"
                  >
                    Save
                  </button>
                  <button
                    type="button"
                    onclick={() => editingFavoriteId = null}
                    class="px-1.5 py-0.5 text-slate-400 hover:text-slate-200 text-[10.5px]"
                  >
                    Cancel
                  </button>
                </div>
              {:else if item.favoriteLabel}
                <div class="flex items-center justify-between text-xs bg-amber-500/10 border border-amber-500/20 text-amber-600 dark:text-amber-400 rounded-md px-2 py-1">
                  <div class="flex items-center gap-1.5 font-bold truncate">
                    <Star size={11} class="fill-amber-500 shrink-0" />
                    <span class="truncate">{item.favoriteLabel}</span>
                  </div>
                  <button
                    type="button"
                    onclick={() => startEditingLabel(item)}
                    class="p-0.5 hover:text-amber-300 opacity-70 hover:opacity-100"
                    title="Edit Name"
                  >
                    <Edit3 size={11} />
                  </button>
                </div>
              {:else if item.isFavorite}
                <div class="flex items-center">
                  <button
                    type="button"
                    onclick={() => startEditingLabel(item)}
                    class="flex items-center gap-1 text-[10.5px] text-amber-500/80 hover:text-amber-400 hover:underline cursor-pointer"
                  >
                    <Tag size={10} />
                    <span>+ Add query label / name</span>
                  </button>
                </div>
              {/if}

              <!-- SQL Query Snippet Box -->
              <div class="bg-surface-900 rounded-lg p-2.5 font-mono text-xs text-slate-800 dark:text-slate-200 overflow-x-auto whitespace-pre-wrap break-all max-h-32 select-text border border-slate-200/60 dark:border-slate-800/80">
                {item.sql}
              </div>

              <!-- Error Message Preview if failed -->
              {#if item.errorMessage}
                <div class="text-[10.5px] text-rose-600 dark:text-rose-400 bg-rose-500/5 border border-rose-500/20 rounded p-1.5 font-mono truncate">
                  {item.errorMessage}
                </div>
              {/if}

              <!-- Actions Toolbar -->
              <div class="flex items-center justify-between pt-1 border-t border-slate-200/50 dark:border-slate-800/60 text-xs">
                <div class="flex items-center gap-1.5">
                  <button 
                    type="button"
                    onclick={() => insertIntoCurrentTab(item)}
                    class="flex items-center gap-1 px-2 py-1 rounded text-[11px] font-semibold text-slate-700 dark:text-slate-300 hover:bg-surface-800 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors cursor-pointer"
                    title="Insert into active SQL tab"
                  >
                    <CornerDownLeft size={12} />
                    <span>Use Query</span>
                  </button>

                  <button 
                    type="button"
                    onclick={() => openInNewTab(item)}
                    class="flex items-center gap-1 px-2 py-1 rounded text-[11px] font-semibold text-slate-700 dark:text-slate-300 hover:bg-surface-800 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors cursor-pointer"
                    title="Open in new SQL query tab"
                  >
                    <Play size={11} class="fill-current" />
                    <span>Open in Tab</span>
                  </button>
                </div>

                <div class="flex items-center gap-1">
                  <button 
                    type="button"
                    onclick={() => copyToClipboard(item)}
                    class="flex items-center gap-1 px-2 py-1 rounded text-[11px] font-semibold text-slate-500 hover:text-slate-900 dark:hover:text-white hover:bg-surface-800 transition-colors cursor-pointer"
                    title="Copy SQL to Clipboard"
                  >
                    {#if copiedId === item.id}
                      <Check size={12} class="text-emerald-500" />
                      <span class="text-emerald-500">Copied</span>
                    {:else}
                      <Copy size={12} />
                      <span>Copy</span>
                    {/if}
                  </button>

                  <button 
                    type="button"
                    onclick={() => historyStore.removeEntry(item.id)}
                    class="p-1 rounded text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-surface-800 transition-colors cursor-pointer"
                    title="Delete item"
                  >
                    <Trash2 size={12} />
                  </button>
                </div>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
