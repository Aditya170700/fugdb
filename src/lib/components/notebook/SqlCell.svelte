<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorState, Compartment } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { Chart, registerables } from 'chart.js';
  import type { SqlNotebookCell, NotebookChartConfig } from '$lib/api/types';
  import { notebookStore } from '$lib/state/notebook.svelte';
  import { themeStore } from '$lib/state/theme.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import CustomSelect, { type SelectOption } from '$lib/components/ui/CustomSelect.svelte';
  import { createSqlLanguageSupport } from '../editor/sqlCompletion';
  import { 
    Play, 
    RefreshCw, 
    Trash2, 
    ArrowUp, 
    ArrowDown, 
    Copy, 
    Database, 
    Table, 
    BarChart3, 
    Code2, 
    Check, 
    AlertCircle, 
    ChevronDown, 
    ChevronUp,
    Sparkles,
    Layers,
    PieChart,
    LineChart
  } from 'lucide-svelte';

  // Register Chart.js components
  Chart.register(...registerables);

  const chartTypeOptions: SelectOption[] = [
    { value: 'bar', label: 'Bar Chart', badge: 'BAR' },
    { value: 'line', label: 'Line Chart', badge: 'LINE' },
    { value: 'area', label: 'Area Chart', badge: 'AREA' },
    { value: 'pie', label: 'Pie Chart', badge: 'PIE' },
    { value: 'doughnut', label: 'Doughnut Chart', badge: 'DONUT' }
  ];

  let { 
    notebookId, 
    cell, 
    index, 
    totalCells,
    defaultConnectionId 
  }: { 
    notebookId: string; 
    cell: SqlNotebookCell; 
    index: number; 
    totalCells: number; 
    defaultConnectionId?: string;
  } = $props();

  let editorContainer = $state<HTMLDivElement | null>(null);
  let view: EditorView | undefined = $state();
  let canvasRef = $state<HTMLCanvasElement | null>(null);
  let chartInstance: Chart | null = null;
  let copiedJson = $state(false);

  const themeCompartment = new Compartment();
  const languageCompartment = new Compartment();

  const activeConn = $derived(
    connectionStore.connections.find(c => c.id === (cell.connectionId || defaultConnectionId || connectionStore.activeConnectionId))
  );

  const isDark = $derived(themeStore.resolvedTheme === 'dark');
  const result = $derived(cell.result);

  const columnOptions = $derived<SelectOption[]>(
    (result?.columns || []).map(col => ({
      value: col.name,
      label: col.name,
      badge: col.dataType || undefined,
      badgeColor: 'bg-slate-200 dark:bg-slate-800 text-slate-600 dark:text-slate-400 font-mono text-[9.5px]'
    }))
  );

  const lightSyntaxHighlight = HighlightStyle.define([
    { tag: tags.keyword, color: '#4338ca', fontWeight: '700' },
    { tag: tags.operatorKeyword, color: '#4338ca', fontWeight: '700' },
    { tag: tags.typeName, color: '#1d4ed8', fontWeight: '600' },
    { tag: tags.string, color: '#047857' },
    { tag: tags.number, color: '#b45309', fontWeight: '600' },
    { tag: tags.comment, color: '#64748b', fontStyle: 'italic' },
    { tag: tags.operator, color: '#4338ca' },
    { tag: tags.punctuation, color: '#334155' },
    { tag: tags.variableName, color: '#0f172a' },
    { tag: tags.propertyName, color: '#0f172a' },
  ]);

  function getEditorCustomTheme(dark: boolean) {
    return EditorView.theme({
      '&': { 
        fontSize: '12.5px', 
        backgroundColor: dark ? '#030712' : '#ffffff',
        color: dark ? '#f8fafc' : '#0f172a',
        borderRadius: '0.5rem',
      },
      '.cm-scroller': { 
        padding: '6px 0',
        fontFamily: 'JetBrains Mono, monospace',
        minHeight: '60px',
        maxHeight: '260px',
      },
      '.cm-gutters': { 
        backgroundColor: dark ? '#080d1a' : '#f8fafc', 
        borderRight: dark ? '1px solid #1e293b' : '1px solid #e2e8f0', 
        color: dark ? '#475569' : '#64748b' 
      },
      '.cm-activeLineGutter': { 
        backgroundColor: dark ? '#1e293b' : '#e2e8f0' 
      },
      '.cm-activeLine': { 
        backgroundColor: dark ? '#1e293b30' : '#f1f5f9' 
      },
      '.cm-selectionBackground, ::selection': {
        backgroundColor: dark ? '#3b82f640' : '#bfdbfe !important'
      }
    });
  }

  function initEditor() {
    if (!editorContainer) return;
    if (view) view.destroy();

    const sqlLangSupport = createSqlLanguageSupport(
      activeConn?.driver || 'postgres',
      connectionStore.activeSchemaTree || undefined
    );

    const shiftEnterKeymap = keymap.of([
      {
        key: 'Shift-Enter',
        run: () => {
          runQuery();
          return true;
        }
      }
    ]);

    const state = EditorState.create({
      doc: cell.sql,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        history(),
        closeBrackets(),
        autocompletion(),
        shiftEnterKeymap,
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...historyKeymap,
          ...completionKeymap,
        ]),
        languageCompartment.of(sqlLangSupport),
        themeCompartment.of([
          isDark ? oneDark : syntaxHighlighting(lightSyntaxHighlight),
          getEditorCustomTheme(isDark),
        ]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const newDoc = update.state.doc.toString();
            notebookStore.updateSqlContent(notebookId, cell.id, newDoc);
          }
        })
      ]
    });

    view = new EditorView({
      state,
      parent: editorContainer
    });
  }

  $effect(() => {
    if (view) {
      view.dispatch({
        effects: themeCompartment.reconfigure([
          isDark ? oneDark : syntaxHighlighting(lightSyntaxHighlight),
          getEditorCustomTheme(isDark)
        ])
      });
    }
  });

  onMount(() => {
    initEditor();
  });

  onDestroy(() => {
    if (view) view.destroy();
    if (chartInstance) chartInstance.destroy();
  });

  function runQuery() {
    notebookStore.runCell(notebookId, cell.id);
  }

  // Chart Rendering
  function renderChart() {
    if (!canvasRef || !result || !result.columns || result.columns.length === 0 || !result.rows) return;
    if (chartInstance) {
      chartInstance.destroy();
      chartInstance = null;
    }

    const cfg = cell.chartConfig || {
      type: 'bar',
      xAxisColumn: result.columns[0]?.name || '',
      yAxisColumns: result.columns[1] ? [result.columns[1].name] : [result.columns[0]?.name || ''],
      title: 'Metrics'
    };

    const xCol = cfg.xAxisColumn || result.columns[0]?.name || '';
    const yCols = cfg.yAxisColumns && cfg.yAxisColumns.length > 0 
      ? cfg.yAxisColumns 
      : [result.columns[1]?.name || result.columns[0]?.name || ''];

    const xColIdx = Math.max(0, result.columns.findIndex(c => c.name === xCol));
    const labels = result.rows.map(r => String(r[xColIdx] ?? ''));
    const palette = ['#6366f1', '#10b981', '#f59e0b', '#ec4899', '#8b5cf6', '#06b6d4', '#3b82f6'];

    const datasets = yCols.map((colName, cIdx) => {
      const colIdx = Math.max(0, result.columns.findIndex(c => c.name === colName));
      const color = palette[cIdx % palette.length];
      const isPie = cfg.type === 'pie' || cfg.type === 'doughnut';
      return {
        label: colName,
        data: result.rows.map(r => {
          const val = r[colIdx];
          return typeof val === 'number' ? val : parseFloat(String(val)) || 0;
        }),
        backgroundColor: isPie ? palette : `${color}33`,
        borderColor: color,
        borderWidth: 2,
        fill: cfg.type === 'area',
        tension: 0.35
      };
    });

    try {
      chartInstance = new Chart(canvasRef, {
        type: cfg.type === 'area' ? 'line' : cfg.type,
        data: {
          labels,
          datasets
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          animation: { duration: 350 },
          plugins: {
            legend: {
              position: 'top',
              labels: {
                color: isDark ? '#94a3b8' : '#475569',
                font: { family: 'inherit', size: 11 }
              }
            },
            title: {
              display: !!cfg.title,
              text: cfg.title || '',
              color: isDark ? '#f8fafc' : '#0f172a',
              font: { weight: 'bold', size: 13 }
            }
          },
          scales: cfg.type === 'pie' || cfg.type === 'doughnut' ? {} : {
            x: {
              ticks: { color: isDark ? '#64748b' : '#94a3b8', font: { size: 10 } },
              grid: { color: isDark ? '#1e293b' : '#f1f5f9' }
            },
            y: {
              ticks: { color: isDark ? '#64748b' : '#94a3b8', font: { size: 10 } },
              grid: { color: isDark ? '#1e293b' : '#f1f5f9' }
            }
          }
        }
      });
    } catch (e) {
      console.error('Failed to render notebook chart:', e);
    }
  }

  $effect(() => {
    if (cell.displayMode === 'chart' && result && canvasRef) {
      renderChart();
    }
  });

  function copyJsonOutput() {
    if (!result) return;
    navigator.clipboard.writeText(JSON.stringify(result.rows, null, 2));
    copiedJson = true;
    setTimeout(() => copiedJson = false, 1500);
  }
</script>

<div class="group relative rounded-xl border border-slate-200/80 dark:border-slate-800/80 bg-white dark:bg-surface-900/90 shadow-xs hover:border-indigo-500/50 dark:hover:border-indigo-500/40 transition-all overflow-hidden mb-4">
  
  <!-- Cell Header / Control Bar -->
  <div class="px-4 py-2 bg-slate-50/80 dark:bg-surface-950/80 border-b border-slate-200/80 dark:border-slate-800/80 flex flex-wrap items-center justify-between gap-2 text-xs">
    <div class="flex items-center gap-2">
      <!-- Run Button -->
      <button
        type="button"
        disabled={cell.isExecuting}
        onclick={runQuery}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:bg-slate-300 dark:disabled:bg-slate-800 text-white font-bold text-xs shadow-xs transition-all active:scale-95 cursor-pointer"
        title="Execute SQL Query (Shift+Enter)"
      >
        <Play size={12} class={cell.isExecuting ? 'animate-spin' : 'fill-white'} />
        <span>{cell.isExecuting ? 'Running...' : 'Run Cell'}</span>
      </button>

      <span class="flex items-center gap-1 px-2 py-0.5 rounded-md bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 font-mono font-bold text-[11px] border border-indigo-500/20">
        <span>SQL #{index + 1}</span>
      </span>

      <!-- Active Driver Indicator -->
      {#if activeConn}
        <span class="text-[11px] text-slate-500 font-mono hidden sm:inline truncate max-w-[140px]" title="Database: {activeConn.name}">
          • {activeConn.name}
        </span>
      {/if}

      <!-- Runtime badge -->
      {#if cell.executionDurationMs !== undefined}
        <span class="text-[10.5px] font-mono px-1.5 py-0.5 rounded bg-surface-800 text-slate-600 dark:text-slate-400 border border-slate-300 dark:border-slate-700">
          ⚡ {cell.executionDurationMs.toFixed(1)} ms
        </span>
      {/if}

      {#if result?.rows}
        <span class="text-[10.5px] font-mono px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/20">
          {result.rows.length} rows
        </span>
      {/if}
    </div>

    <!-- Right Controls: Display Mode & Cell Operations -->
    <div class="flex items-center gap-1">
      {#if result?.rows && result.rows.length > 0}
        <!-- Display Mode Switcher -->
        <div class="flex items-center bg-surface-800 p-0.5 rounded-lg border border-slate-300 dark:border-slate-700 mr-2">
          <button
            type="button"
            onclick={() => notebookStore.setCellDisplayMode(notebookId, cell.id, 'grid')}
            class="flex items-center gap-1 px-2 py-0.5 rounded text-[10.5px] font-semibold transition-all {cell.displayMode === 'grid' ? 'bg-white dark:bg-surface-700 text-indigo-600 dark:text-indigo-400 shadow-xs' : 'text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'}"
            title="Grid Data Table View"
          >
            <Table size={11} />
            <span>Grid</span>
          </button>

          <button
            type="button"
            onclick={() => notebookStore.setCellDisplayMode(notebookId, cell.id, 'chart')}
            class="flex items-center gap-1 px-2 py-0.5 rounded text-[10.5px] font-semibold transition-all {cell.displayMode === 'chart' ? 'bg-white dark:bg-surface-700 text-indigo-600 dark:text-indigo-400 shadow-xs' : 'text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'}"
            title="Chart Visualization View"
          >
            <BarChart3 size={11} />
            <span>Chart</span>
          </button>

          <button
            type="button"
            onclick={() => notebookStore.setCellDisplayMode(notebookId, cell.id, 'json')}
            class="flex items-center gap-1 px-2 py-0.5 rounded text-[10.5px] font-semibold transition-all {cell.displayMode === 'json' ? 'bg-white dark:bg-surface-700 text-indigo-600 dark:text-indigo-400 shadow-xs' : 'text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'}"
            title="JSON Format View"
          >
            <Code2 size={11} />
            <span>JSON</span>
          </button>
        </div>
      {/if}

      <!-- Clear Output -->
      {#if result || cell.errorMessage}
        <button
          type="button"
          onclick={() => notebookStore.clearCellOutput(notebookId, cell.id)}
          class="p-1 rounded text-slate-500 hover:text-amber-600 dark:hover:text-amber-400 hover:bg-amber-500/10 transition-colors"
          title="Clear Output"
        >
          <RefreshCw size={12} />
        </button>
      {/if}

      <div class="w-[1px] h-3.5 bg-slate-300 dark:bg-slate-700 mx-1"></div>

      <!-- Move Up / Down -->
      <button
        type="button"
        disabled={index === 0}
        onclick={() => notebookStore.moveCell(notebookId, cell.id, 'up')}
        class="p-1 rounded text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-surface-800 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        title="Move Cell Up"
      >
        <ArrowUp size={13} />
      </button>

      <button
        type="button"
        disabled={index === totalCells - 1}
        onclick={() => notebookStore.moveCell(notebookId, cell.id, 'down')}
        class="p-1 rounded text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-surface-800 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        title="Move Cell Down"
      >
        <ArrowDown size={13} />
      </button>

      <!-- Duplicate -->
      <button
        type="button"
        onclick={() => notebookStore.duplicateCell(notebookId, cell.id)}
        class="p-1 rounded text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-surface-800 transition-colors"
        title="Duplicate Cell"
      >
        <Copy size={13} />
      </button>

      <!-- Delete -->
      <button
        type="button"
        onclick={() => notebookStore.deleteCell(notebookId, cell.id)}
        class="p-1 rounded text-slate-500 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/15 transition-colors"
        title="Delete Cell"
      >
        <Trash2 size={13} />
      </button>
    </div>
  </div>

  <!-- SQL CodeMirror Editor Section -->
  <div class="p-3 bg-slate-50/50 dark:bg-surface-950/50">
    <div 
      bind:this={editorContainer} 
      class="border border-slate-300 dark:border-slate-800 rounded-lg overflow-hidden focus-within:ring-1 focus-within:ring-indigo-500 transition-all shadow-2xs"
    ></div>
    <div class="mt-1.5 flex items-center justify-between text-[10.5px] text-slate-400 font-mono">
      <span>Shortcut: <strong>Shift+Enter</strong> to execute cell</span>
    </div>
  </div>

  <!-- Execution Output / Result Section -->
  {#if cell.errorMessage}
    <!-- Error Alert -->
    <div class="p-3 mx-3 mb-3 bg-rose-500/10 border border-rose-500/30 rounded-lg text-xs text-rose-700 dark:text-rose-300 flex items-start gap-2">
      <AlertCircle size={15} class="shrink-0 mt-0.5 text-rose-500" />
      <div class="flex-1 font-mono break-all leading-relaxed">
        <strong>Query Error:</strong> {cell.errorMessage}
      </div>
    </div>
  {:else if result}
    <!-- Result Container -->
    <div class="border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-surface-950">
      
      {#if cell.displayMode === 'chart'}
        <!-- Chart Config Toolbar & Canvas -->
        <div class="p-4 flex flex-col gap-3">
          <!-- Chart Controls Bar -->
          <div class="flex flex-wrap items-center justify-between gap-3 p-2 bg-surface-900/90 rounded-lg border border-slate-200 dark:border-slate-800 text-xs">
            <div class="flex items-center gap-2">
              <span class="font-bold text-slate-700 dark:text-slate-300 shrink-0">Type:</span>
              <div class="w-36">
                <CustomSelect
                  value={cell.chartConfig?.type || 'bar'}
                  options={chartTypeOptions}
                  size="sm"
                  onchange={(val) => {
                    notebookStore.updateChartConfig(notebookId, cell.id, { type: val });
                    renderChart();
                  }}
                />
              </div>
            </div>

            <div class="flex items-center gap-2">
              <span class="font-bold text-slate-700 dark:text-slate-300 shrink-0">X-Axis:</span>
              <div class="w-44">
                <CustomSelect
                  value={cell.chartConfig?.xAxisColumn || result.columns[0]?.name}
                  options={columnOptions}
                  size="sm"
                  fontMono={true}
                  onchange={(val) => {
                    notebookStore.updateChartConfig(notebookId, cell.id, { xAxisColumn: val });
                    renderChart();
                  }}
                />
              </div>
            </div>

            <div class="flex items-center gap-2">
              <span class="font-bold text-slate-700 dark:text-slate-300 shrink-0">Y-Metric:</span>
              <div class="w-44">
                <CustomSelect
                  value={cell.chartConfig?.yAxisColumns?.[0] || result.columns[1]?.name || result.columns[0]?.name}
                  options={columnOptions}
                  size="sm"
                  fontMono={true}
                  onchange={(val) => {
                    notebookStore.updateChartConfig(notebookId, cell.id, { yAxisColumns: [val] });
                    renderChart();
                  }}
                />
              </div>
            </div>
          </div>

          <!-- Canvas Container -->
          <div class="relative h-64 w-full bg-surface-950 p-2 rounded-lg border border-slate-200 dark:border-slate-800">
            <canvas bind:this={canvasRef}></canvas>
          </div>
        </div>

      {:else if cell.displayMode === 'json'}
        <!-- JSON Formatted View -->
        <div class="p-3">
          <div class="flex items-center justify-between mb-1.5 text-xs text-slate-500 font-mono">
            <span>JSON Records ({result.rows.length})</span>
            <button
              type="button"
              onclick={copyJsonOutput}
              class="flex items-center gap-1 px-2 py-0.5 bg-surface-800 hover:bg-surface-700 rounded text-slate-700 dark:text-slate-300 border border-slate-300 dark:border-slate-700 transition-colors"
            >
              {#if copiedJson}
                <Check size={11} class="text-emerald-500" />
                <span>Copied</span>
              {:else}
                <Copy size={11} />
                <span>Copy JSON</span>
              {/if}
            </button>
          </div>
          <pre class="p-3 bg-surface-950 rounded-lg border border-slate-200 dark:border-slate-800 font-mono text-xs text-slate-800 dark:text-slate-200 max-h-60 overflow-auto whitespace-pre select-text leading-relaxed">{JSON.stringify(result.rows, null, 2)}</pre>
        </div>

      {:else}
        <!-- Tabular Grid View -->
        <div class="max-h-72 overflow-auto select-text">
          <table class="w-full text-left text-xs border-collapse font-mono">
            <thead class="sticky top-0 bg-slate-100 dark:bg-surface-900 border-b border-slate-200 dark:border-slate-800 text-[11px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-wider select-none z-10 font-sans">
              <tr>
                <th class="py-2 px-3 w-10 text-center text-slate-400">#</th>
                {#each result.columns as col (col.name)}
                  <th class="py-2 px-3 font-semibold">
                    <span>{col.name}</span>
                    {#if col.dataType}
                      <span class="text-[9.5px] font-normal text-slate-400 block lowercase">{col.dataType}</span>
                    {/if}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-200/80 dark:divide-slate-800/60">
              {#each result.rows.slice(0, 100) as row, rIdx (rIdx)}
                <tr class="hover:bg-slate-50 dark:hover:bg-surface-900/60 transition-colors">
                  <td class="py-1.5 px-3 text-center text-slate-400 text-[10.5px]">{rIdx + 1}</td>
                  {#each result.columns as col, colIdx (col.name)}
                    {@const val = row[colIdx]}
                    <td class="py-1.5 px-3 max-w-xs truncate text-slate-900 dark:text-slate-100">
                      {#if val === null || val === undefined}
                        <span class="text-slate-400 italic font-sans text-[10.5px]">NULL</span>
                      {:else if typeof val === 'boolean'}
                        <span class="px-1.5 py-0.2 rounded text-[10px] font-bold {val ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300' : 'bg-rose-500/15 text-rose-700 dark:text-rose-300'} font-sans">
                          {String(val)}
                        </span>
                      {:else}
                        <span>{String(val)}</span>
                      {/if}
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
          {#if result.rows.length > 100}
            <div class="p-2 text-center text-xs text-slate-500 bg-surface-900/40 border-t border-slate-200 dark:border-slate-800 font-sans">
              Showing first 100 of {result.rows.length} rows
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>
