<script lang="ts">
  import { onMount } from 'svelte';
  import { writable } from 'svelte/store';
  import { toPng, toSvg } from 'html-to-image';
  import { 
    SvelteFlow, 
    Background, 
    Controls, 
    MiniMap, 
    Panel,
    type Node,
    type Edge,
    BackgroundVariant
  } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';

  import { 
    Network, 
    X, 
    Download, 
    ZoomIn, 
    ZoomOut, 
    Maximize2, 
    RefreshCw, 
    Search, 
    Sparkles, 
    Layers, 
    Database, 
    FileSpreadsheet, 
    FileCode, 
    FileImage,
    Image,
    Check,
    Share2,
    Sliders
  } from 'lucide-svelte';

  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { api } from '$lib/api/client';
  import type { SchemaTree, RelationEdge } from '$lib/api/types';
  import ErdTableNode from './ErdTableNode.svelte';
  import { generateErdElements, type ErdNodeData } from './erdLayout';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  const nodeTypes: Record<string, any> = {
    tableNode: ErdTableNode as any
  };

  const nodes = writable<Node<ErdNodeData>[]>([]);
  const edges = writable<Edge[]>([]);
  let rawNodes: Node<ErdNodeData>[] = [];
  let flowContainer = $state<HTMLElement | null>(null);
  let isLoading = $state<boolean>(false);
  let searchQuery = $state<string>('');
  let isExportDropdownOpen = $state<boolean>(false);
  let copiedExport = $state<boolean>(false);

  const activeConn = $derived(connectionStore.activeConnection);
  const activeSchema = $derived(connectionStore.activeSchemaTree);

  // Load ERD data when modal is opened
  $effect(() => {
    if (isOpen && activeConn) {
      loadErdData();
    }
  });

  async function loadErdData() {
    if (!activeConn) return;
    isLoading = true;
    try {
      // 1. Get schema tree (tables + columns)
      let schemaTree = activeSchema;
      if (!schemaTree || schemaTree.tables.length === 0) {
        schemaTree = await api.fetchSchema(activeConn.id);
        connectionStore.activeSchemaTree = schemaTree;
      }

      // 2. Get foreign key relations
      let relations: RelationEdge[] = schemaTree.relations || [];
      if (!relations || relations.length === 0) {
        relations = await api.generateErdMetadata(activeConn.id);
      }

      // 3. Layout nodes and edges
      const layout = generateErdElements(schemaTree.tables, relations, {
        onOpenTable: (tableName, schema) => {
          tabsStore.openTableGridTab(tableName, schema);
          onClose();
        },
        onQueryTable: (tableName, schema) => {
          const sql = `SELECT * FROM "${schema || 'public'}"."${tableName}" LIMIT 100;`;
          tabsStore.openNewSqlTab(sql, `Query ${tableName}`);
          onClose();
        }
      });

      rawNodes = layout.nodes;
      applySearchFilter();
      edges.set(layout.edges);
    } catch (err) {
      console.error('Failed to load ERD metadata:', err);
    } finally {
      isLoading = false;
    }
  }

  function handleAutoArrange() {
    if (!activeSchema) return;
    const relations = activeSchema.relations || [];
    const layout = generateErdElements(activeSchema.tables, relations, {
      onOpenTable: (tableName, schema) => {
        tabsStore.openTableGridTab(tableName, schema);
        onClose();
      },
      onQueryTable: (tableName, schema) => {
        const sql = `SELECT * FROM "${schema || 'public'}"."${tableName}" LIMIT 100;`;
        tabsStore.openNewSqlTab(sql, `Query ${tableName}`);
        onClose();
      }
    });
    rawNodes = layout.nodes;
    applySearchFilter();
    edges.set(layout.edges);
  }

  function applySearchFilter() {
    const q = searchQuery.trim().toLowerCase();
    if (!q) {
      nodes.set(rawNodes.map(n => ({ ...n, style: '' })));
    } else {
      nodes.set(
        rawNodes.map(n => {
          const match = n.data.name.toLowerCase().includes(q) || 
                        n.data.columns.some(c => c.name.toLowerCase().includes(q));
          return {
            ...n,
            style: match ? 'opacity: 1; transform: scale(1.02);' : 'opacity: 0.2;'
          };
        })
      );
    }
  }

  $effect(() => {
    // Re-filter whenever searchQuery changes
    if (rawNodes.length > 0) {
      applySearchFilter();
    }
  });

  async function exportAsPng() {
    if (!flowContainer) return;
    try {
      const dataUrl = await toPng(flowContainer, {
        backgroundColor: '#090d16',
        filter: (domNode: HTMLElement) => {
          if (!domNode.classList) return true;
          return !domNode.classList.contains('svelte-flow__minimap') && !domNode.classList.contains('svelte-flow__controls');
        }
      });
      const a = document.createElement('a');
      a.href = dataUrl;
      a.download = `${activeSchema?.currentDatabase || 'database'}_erd.png`;
      a.click();
    } catch (err) {
      console.error('Failed to export ERD to PNG:', err);
    } finally {
      isExportDropdownOpen = false;
    }
  }

  async function exportAsSvg() {
    if (!flowContainer) return;
    try {
      const dataUrl = await toSvg(flowContainer, {
        backgroundColor: '#090d16',
        filter: (domNode: HTMLElement) => {
          if (!domNode.classList) return true;
          return !domNode.classList.contains('svelte-flow__minimap') && !domNode.classList.contains('svelte-flow__controls');
        }
      });
      const a = document.createElement('a');
      a.href = dataUrl;
      a.download = `${activeSchema?.currentDatabase || 'database'}_erd.svg`;
      a.click();
    } catch (err) {
      console.error('Failed to export ERD to SVG:', err);
    } finally {
      isExportDropdownOpen = false;
    }
  }

  function exportAsJson() {
    if (!activeSchema) return;
    const data = {
      database: activeSchema.currentDatabase,
      driver: activeConn?.driver,
      tables: activeSchema.tables,
      relations: activeSchema.relations || []
    };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${activeSchema.currentDatabase || 'database'}_erd_schema.json`;
    a.click();
    URL.revokeObjectURL(url);
    isExportDropdownOpen = false;
  }

  function exportAsSqlDdl() {
    if (!activeSchema) return;
    let ddl = `-- Schema Export for ${activeSchema.currentDatabase}\n-- Generated by FugDB ERD Visualizer\n\n`;
    activeSchema.tables.forEach(t => {
      ddl += `CREATE TABLE "${t.schema}"."${t.name}" (\n`;
      const cols = (t.columns || []).map(c => {
        let line = `  "${c.name}" ${c.dataType}`;
        if (c.isPrimaryKey) line += ' PRIMARY KEY';
        if (!c.nullable && !c.isPrimaryKey) line += ' NOT NULL';
        return line;
      });
      ddl += cols.join(',\n');
      ddl += '\n);\n\n';
    });

    const blob = new Blob([ddl], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${activeSchema.currentDatabase || 'database'}_ddl.sql`;
    a.click();
    URL.revokeObjectURL(url);
    isExportDropdownOpen = false;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 bg-black/75 backdrop-blur-xs z-50 flex items-center justify-center p-3 sm:p-6 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <!-- ERD Canvas Modal Container -->
    <div 
      class="bg-surface-900 border border-slate-200 dark:border-slate-800 w-full max-w-7xl h-[88vh] rounded-2xl shadow-2xl flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 animate-in zoom-in-95 duration-200"
    >
      <!-- Top Modal Toolbar -->
      <div class="px-4 py-3 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-surface-950/80 gap-3">
        <!-- Brand & Database Info -->
        <div class="flex items-center gap-3">
          <div class="p-2 bg-indigo-500/15 text-indigo-600 dark:text-indigo-400 rounded-lg border border-indigo-500/20 shrink-0">
            <Network size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="font-bold text-sm text-slate-900 dark:text-white">
                Entity Relationship Diagram (ERD)
              </h3>
              <span class="text-[11px] font-mono px-2 py-0.5 rounded-md bg-indigo-500/10 text-indigo-400 font-bold border border-indigo-500/30">
                {activeConn?.name || 'Local'} • {activeSchema?.currentDatabase || 'master'}
              </span>
            </div>
            <p class="text-[11px] text-slate-500 dark:text-slate-400">
              Interactive visual schema explorer & foreign key relationships
            </p>
          </div>
        </div>

        <!-- Center Search Filter -->
        <div class="relative w-48 sm:w-64">
          <Search size={12} class="absolute left-2.5 top-2 text-slate-400" />
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search tables or columns..."
            class="w-full bg-surface-900 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-700/80 rounded-lg pl-7 pr-6 py-1 text-xs focus:outline-none focus:border-indigo-500 placeholder:text-slate-500 shadow-xs"
          />
          {#if searchQuery}
            <button
              type="button"
              onclick={() => searchQuery = ''}
              class="absolute right-2 top-1.5 text-slate-400 hover:text-slate-200"
            >
              <X size={12} />
            </button>
          {/if}
        </div>

        <!-- Right Action Buttons -->
        <div class="flex items-center gap-2">
          <!-- Refresh / Re-layout Button -->
          <button
            type="button"
            onclick={handleAutoArrange}
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-surface-900 hover:bg-surface-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 transition-colors cursor-pointer shadow-xs"
            title="Auto arrange diagram layout"
          >
            <RefreshCw size={12} class={isLoading ? 'animate-spin text-indigo-400' : ''} />
            <span>Auto Layout</span>
          </button>

          <!-- Export Dropdown -->
          <div class="relative">
            <button
              type="button"
              onclick={() => isExportDropdownOpen = !isExportDropdownOpen}
              class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-surface-900 hover:bg-surface-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 transition-colors cursor-pointer shadow-xs"
              title="Export Schema Diagram"
            >
              <Download size={12} />
              <span>Export</span>
            </button>

            {#if isExportDropdownOpen}
              <div 
                class="fixed inset-0 z-40" 
                onclick={() => isExportDropdownOpen = false}
                role="presentation"
              ></div>

              <div class="absolute right-0 top-full mt-1.5 z-50 w-56 bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-xl shadow-2xl p-1 space-y-0.5 text-xs">
                <button
                  type="button"
                  onclick={exportAsPng}
                  class="w-full flex items-center gap-2 px-2.5 py-1.5 text-left rounded-lg hover:bg-surface-800 text-slate-800 dark:text-slate-200 transition-colors cursor-pointer"
                >
                  <Image size={13} class="text-sky-400" />
                  <span>Export Diagram (PNG Image)</span>
                </button>

                <button
                  type="button"
                  onclick={exportAsSvg}
                  class="w-full flex items-center gap-2 px-2.5 py-1.5 text-left rounded-lg hover:bg-surface-800 text-slate-800 dark:text-slate-200 transition-colors cursor-pointer"
                >
                  <FileImage size={13} class="text-amber-400" />
                  <span>Export Diagram (Vector SVG)</span>
                </button>

                <div class="my-1 border-t border-slate-200 dark:border-slate-800"></div>

                <button
                  type="button"
                  onclick={exportAsJson}
                  class="w-full flex items-center gap-2 px-2.5 py-1.5 text-left rounded-lg hover:bg-surface-800 text-slate-800 dark:text-slate-200 transition-colors cursor-pointer"
                >
                  <FileCode size={13} class="text-indigo-400" />
                  <span>Export JSON Schema</span>
                </button>

                <button
                  type="button"
                  onclick={exportAsSqlDdl}
                  class="w-full flex items-center gap-2 px-2.5 py-1.5 text-left rounded-lg hover:bg-surface-800 text-slate-800 dark:text-slate-200 transition-colors cursor-pointer"
                >
                  <FileSpreadsheet size={13} class="text-emerald-400" />
                  <span>Export SQL DDL Script</span>
                </button>
              </div>
            {/if}
          </div>

          <!-- Close Modal -->
          <button 
            type="button"
            onclick={onClose} 
            class="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-surface-800 transition-colors cursor-pointer ml-1" 
            title="Close (Esc)"
          >
            <X size={16} />
          </button>
        </div>
      </div>

      <!-- Main Interactive SvelteFlow Canvas -->
      <div 
        bind:this={flowContainer}
        class="flex-1 w-full h-full bg-surface-950 relative overflow-hidden"
      >
        {#if isLoading}
          <div class="absolute inset-0 z-20 flex flex-col items-center justify-center bg-surface-950/80 backdrop-blur-xs gap-3">
            <RefreshCw size={28} class="animate-spin text-indigo-500" />
            <p class="text-xs font-semibold text-slate-300">Extracting schema and building live ERD relations...</p>
          </div>
        {/if}

        {#if $nodes.length === 0 && !isLoading}
          <div class="absolute inset-0 flex flex-col items-center justify-center text-center p-6 text-slate-500 gap-2">
            <Database size={36} class="opacity-40" />
            <p class="text-sm font-semibold text-slate-300">No tables found in this database.</p>
            <p class="text-xs max-w-sm">Create tables or select a different active database connection to view its ERD.</p>
          </div>
        {:else}
          <SvelteFlow 
            {nodes} 
            {edges} 
            {nodeTypes} 
            fitView
            minZoom={0.15}
            maxZoom={2.5}
            defaultEdgeOptions={{
              type: 'smoothstep',
              animated: true
            }}
          >
            <!-- Background Grid Dots -->
            <Background variant={BackgroundVariant.Dots} gap={20} size={1} patternColor="#334155" />

            <!-- Controls (Zoom, Fit, Lock) -->
            <Controls showLock={false} position="bottom-right" />

            <!-- MiniMap in corner -->
            <MiniMap 
              position="bottom-left" 
              nodeColor="#4f46e5" 
              maskColor="rgba(15, 23, 42, 0.7)" 
              class="!bg-surface-900 !border-slate-800 !rounded-xl !overflow-hidden shadow-lg"
            />

            <!-- Top Left Stats Overlay Panel -->
            <Panel position="top-left" class="m-3">
              <div class="flex items-center gap-2 bg-surface-900/90 backdrop-blur-md border border-slate-200 dark:border-slate-800 rounded-xl px-3 py-1.5 shadow-lg text-xs font-mono">
                <span class="text-slate-800 dark:text-slate-200 font-bold">{$nodes.length} Tables</span>
                <span class="text-slate-500">•</span>
                <span class="text-indigo-600 dark:text-indigo-400 font-bold">{$edges.length} FK Relations</span>
              </div>
            </Panel>
          </SvelteFlow>
        {/if}
      </div>
    </div>
  </div>
{/if}
