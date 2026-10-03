<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    BarChart3,
    LineChart,
    PieChart,
    TrendingUp,
    Activity,
    Download,
    Copy,
    Check,
    X,
    Maximize2,
    Minimize2,
    Layers,
    ArrowUpDown,
    Palette,
    SlidersHorizontal,
    Table,
    Sparkles,
    Hash,
    Sigma
  } from 'lucide-svelte';
  import type { QueryResult } from '$lib/api/types';
  import { themeStore } from '$lib/state/theme.svelte';
  import {
    Chart,
    BarController,
    BarElement,
    LineController,
    LineElement,
    PointElement,
    PieController,
    DoughnutController,
    ScatterController,
    ArcElement,
    CategoryScale,
    LinearScale,
    TimeScale,
    Tooltip,
    Legend,
    Filler
  } from 'chart.js';

  // Register Chart.js modules
  Chart.register(
    BarController,
    BarElement,
    LineController,
    LineElement,
    PointElement,
    PieController,
    DoughnutController,
    ScatterController,
    ArcElement,
    CategoryScale,
    LinearScale,
    TimeScale,
    Tooltip,
    Legend,
    Filler
  );

  let {
    isOpen = $bindable(false),
    result,
    title = 'Query Result Data',
    sql = ''
  }: {
    isOpen: boolean;
    result?: QueryResult;
    title?: string;
    sql?: string;
  } = $props();

  type ChartType = 'bar' | 'horizontal-bar' | 'line' | 'area' | 'pie' | 'doughnut' | 'scatter';

  let selectedChartType = $state<ChartType>('bar');
  let selectedXCol = $state<string>('');
  let selectedYCols = $state<string[]>([]);
  let aggregation = $state<'sum' | 'avg' | 'count' | 'min' | 'max' | 'none'>('none');
  let sortOrder = $state<'none' | 'val-desc' | 'val-asc' | 'x-asc' | 'x-desc'>('none');
  let topNLimit = $state<number>(50);
  let selectedPalette = $state<'indigo' | 'emerald' | 'sunset' | 'cyber'>('indigo');
  let isFullscreen = $state(false);
  let isCopying = $state(false);
  let copySuccess = $state(false);

  let canvasRef: HTMLCanvasElement | null = $state(null);
  let chartInstance: Chart | null = null;

  const palettes: Record<string, string[]> = {
    indigo: ['#6366f1', '#8b5cf6', '#a855f7', '#3b82f6', '#06b6d4', '#10b981', '#f59e0b', '#ec4899'],
    emerald: ['#10b981', '#06b6d4', '#14b8a6', '#3b82f6', '#84cc16', '#22c55e', '#0ea5e9', '#6366f1'],
    sunset: ['#f97316', '#f43f5e', '#fbbf24', '#e11d48', '#c026d3', '#ea580c', '#d97706', '#8b5cf6'],
    cyber: ['#06b6d4', '#ec4899', '#a855f7', '#eab308', '#22c55e', '#3b82f6', '#f97316', '#6366f1']
  };

  // Inspect column types & identify numeric vs category columns
  const columnAnalysis = $derived.by(() => {
    if (!result?.columns || !result?.rows) return { numericCols: [], categoryCols: [] };

    const numericCols: string[] = [];
    const categoryCols: string[] = [];

    result.columns.forEach((col, idx) => {
      const typeStr = (col.dataType || '').toLowerCase();
      const isExplicitNumeric = 
        typeStr.includes('int') || 
        typeStr.includes('float') || 
        typeStr.includes('double') || 
        typeStr.includes('decimal') || 
        typeStr.includes('numeric') || 
        typeStr.includes('real') || 
        typeStr.includes('number') || 
        typeStr.includes('money') || 
        typeStr.includes('serial');

      // Sample first 10 rows to verify values
      const sampleVals = result.rows.slice(0, 15).map(r => r[idx]).filter(v => v !== null && v !== undefined);
      const isSampleNumeric = sampleVals.length > 0 && sampleVals.every(v => !isNaN(Number(v)) && typeof v !== 'boolean');

      if (isExplicitNumeric || isSampleNumeric) {
        numericCols.push(col.name);
      } else {
        categoryCols.push(col.name);
      }
    });

    return { numericCols, categoryCols };
  });

  // Auto-initialize default axes when modal opens or result changes
  $effect(() => {
    if (isOpen && result && result.columns.length > 0) {
      const { numericCols, categoryCols } = columnAnalysis;
      
      // Auto-pick best X column (preferred category/date column, or fallback to first column)
      if (!selectedXCol || !result.columns.some(c => c.name === selectedXCol)) {
        selectedXCol = categoryCols[0] || result.columns[0].name;
      }

      // Auto-pick best Y column(s) (first numeric column, or fallback)
      if (selectedYCols.length === 0 || !selectedYCols.some(y => result.columns.some(c => c.name === y))) {
        if (numericCols.length > 0) {
          // If first numeric is an ID and there are other metrics, pick the metric
          const nonIdNumeric = numericCols.find(c => !c.toLowerCase().endsWith('id'));
          selectedYCols = [nonIdNumeric || numericCols[0]];
        } else {
          selectedYCols = [result.columns[Math.min(1, result.columns.length - 1)].name];
        }
      }
    }
  });

  // Process & Aggregate Chart Data
  const chartData = $derived.by(() => {
    if (!result?.columns || !result?.rows || !selectedXCol || selectedYCols.length === 0) {
      return { labels: [], datasets: [], stats: { count: 0, sum: 0, avg: 0, min: 0, max: 0 } };
    }

    const xIdx = result.columns.findIndex(c => c.name === selectedXCol);
    if (xIdx === -1) return { labels: [], datasets: [], stats: { count: 0, sum: 0, avg: 0, min: 0, max: 0 } };

    const yIndices = selectedYCols.map(y => result.columns.findIndex(c => c.name === y)).filter(i => i !== -1);
    if (yIndices.length === 0) return { labels: [], datasets: [], stats: { count: 0, sum: 0, avg: 0, min: 0, max: 0 } };

    // Group & Aggregate Data
    const rawGroups = new Map<string, number[]>();

    for (const row of result.rows) {
      const rawX = row[xIdx];
      const label = rawX === null || rawX === undefined ? '(NULL)' : String(rawX);

      if (!rawGroups.has(label)) {
        rawGroups.set(label, yIndices.map(() => 0));
      }

      const currentVals = rawGroups.get(label)!;
      yIndices.forEach((yIdx, i) => {
        const rawY = row[yIdx];
        const numY = Number(rawY);
        const val = isNaN(numY) ? 0 : numY;

        if (aggregation === 'none' || aggregation === 'sum' || aggregation === 'count') {
          currentVals[i] += aggregation === 'count' ? 1 : val;
        } else if (aggregation === 'min') {
          currentVals[i] = rawGroups.get(label)![i] === 0 ? val : Math.min(currentVals[i], val);
        } else if (aggregation === 'max') {
          currentVals[i] = Math.max(currentVals[i], val);
        } else if (aggregation === 'avg') {
          currentVals[i] += val; // will divide by count below if needed
        }
      });
    }

    let entries = Array.from(rawGroups.entries()).map(([label, vals]) => ({ label, vals }));

    // Apply Sorting
    if (sortOrder === 'val-desc') {
      entries.sort((a, b) => (b.vals[0] || 0) - (a.vals[0] || 0));
    } else if (sortOrder === 'val-asc') {
      entries.sort((a, b) => (a.vals[0] || 0) - (b.vals[0] || 0));
    } else if (sortOrder === 'x-asc') {
      entries.sort((a, b) => a.label.localeCompare(b.label, undefined, { numeric: true }));
    } else if (sortOrder === 'x-desc') {
      entries.sort((a, b) => b.label.localeCompare(a.label, undefined, { numeric: true }));
    }

    // Apply Limit
    if (topNLimit > 0 && entries.length > topNLimit) {
      entries = entries.slice(0, topNLimit);
    }

    const labels = entries.map(e => e.label);
    const colorList = palettes[selectedPalette] || palettes.indigo;

    const datasets = yIndices.map((yIdx, i) => {
      const colName = result.columns[yIdx]?.name || `Metric ${i + 1}`;
      const color = colorList[i % colorList.length];
      const data = entries.map(e => e.vals[i]);

      const isPieOrDonut = selectedChartType === 'pie' || selectedChartType === 'doughnut';

      return {
        label: colName,
        data,
        backgroundColor: isPieOrDonut
          ? colorList.slice(0, entries.length)
          : selectedChartType === 'area'
            ? `${color}33`
            : `${color}cc`,
        borderColor: isPieOrDonut
          ? (themeStore.resolvedTheme === 'dark' ? '#0f172a' : '#ffffff')
          : color,
        borderWidth: selectedChartType === 'line' || selectedChartType === 'area' ? 2.5 : 1.5,
        fill: selectedChartType === 'area',
        tension: selectedChartType === 'line' || selectedChartType === 'area' ? 0.35 : 0,
        pointRadius: entries.length > 50 ? 0 : 3.5,
        pointHoverRadius: 6,
        borderRadius: selectedChartType === 'bar' || selectedChartType === 'horizontal-bar' ? 4 : 0
      };
    });

    // Compute Overall Metric Stats for Top KPIs
    const allValues = entries.flatMap(e => e.vals[0] || 0);
    const count = allValues.length;
    const sum = allValues.reduce((a, b) => a + b, 0);
    const avg = count > 0 ? sum / count : 0;
    const min = count > 0 ? Math.min(...allValues) : 0;
    const max = count > 0 ? Math.max(...allValues) : 0;

    return {
      labels,
      datasets,
      stats: { count, sum, avg, min, max }
    };
  });

  // Re-render Chart.js on Canvas
  function renderChart() {
    if (!canvasRef) return;

    if (chartInstance) {
      chartInstance.destroy();
      chartInstance = null;
    }

    const isDark = themeStore.resolvedTheme === 'dark';
    const textColor = isDark ? '#94a3b8' : '#475569';
    const gridColor = isDark ? '#1e293b80' : '#e2e8f080';

    const { labels, datasets } = chartData;

    const baseType = selectedChartType === 'horizontal-bar' 
      ? 'bar' 
      : selectedChartType === 'area' 
        ? 'line' 
        : selectedChartType;

    const isHorizontal = selectedChartType === 'horizontal-bar';
    const isPieOrDonut = selectedChartType === 'pie' || selectedChartType === 'doughnut';

    try {
      chartInstance = new Chart(canvasRef, {
        type: baseType as any,
        data: {
          labels,
          datasets
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          indexAxis: isHorizontal ? 'y' : 'x',
          animation: {
            duration: 400
          },
          plugins: {
            legend: {
              display: isPieOrDonut || datasets.length > 1,
              position: isPieOrDonut ? 'right' : 'top',
              labels: {
                color: isDark ? '#e2e8f0' : '#1e293b',
                font: {
                  family: 'Inter, sans-serif',
                  size: 11
                },
                boxWidth: 12,
                padding: 12
              }
            },
            tooltip: {
              backgroundColor: isDark ? '#0f172a' : '#ffffff',
              titleColor: isDark ? '#f8fafc' : '#0f172a',
              bodyColor: isDark ? '#cbd5e1' : '#334155',
              borderColor: isDark ? '#334155' : '#cbd5e1',
              borderWidth: 1,
              padding: 10,
              boxPadding: 4,
              usePointStyle: true,
              callbacks: {
                label: function (context: any) {
                  let label = context.dataset.label || '';
                  if (label) label += ': ';
                  if (context.parsed.y !== null && context.parsed.y !== undefined) {
                    label += Number(context.parsed.y).toLocaleString();
                  } else if (context.parsed.x !== null && context.parsed.x !== undefined && isHorizontal) {
                    label += Number(context.parsed.x).toLocaleString();
                  } else if (context.raw !== null && context.raw !== undefined) {
                    label += Number(context.raw).toLocaleString();
                  }
                  return label;
                }
              }
            }
          },
          scales: isPieOrDonut ? {} : {
            x: {
              grid: {
                color: gridColor
              },
              ticks: {
                color: textColor,
                font: {
                  family: 'JetBrains Mono, monospace',
                  size: 10
                },
                maxRotation: 45,
                autoSkip: true,
                maxTicksLimit: isHorizontal ? 10 : 25
              }
            },
            y: {
              grid: {
                color: gridColor
              },
              ticks: {
                color: textColor,
                font: {
                  family: 'JetBrains Mono, monospace',
                  size: 10
                },
                callback: function (val: any) {
                  return Number(val).toLocaleString();
                }
              }
            }
          }
        }
      });
    } catch (e) {
      console.error('Failed to instantiate Chart.js:', e);
    }
  }

  // Effect to update chart when config or data changes
  $effect(() => {
    if (isOpen && canvasRef && chartData) {
      // Small timeout to ensure DOM container size is ready
      const t = setTimeout(() => {
        renderChart();
      }, 50);
      return () => clearTimeout(t);
    }
  });

  onDestroy(() => {
    if (chartInstance) {
      chartInstance.destroy();
      chartInstance = null;
    }
  });

  function toggleYColumn(col: string) {
    if (selectedYCols.includes(col)) {
      if (selectedYCols.length > 1) {
        selectedYCols = selectedYCols.filter(c => c !== col);
      }
    } else {
      selectedYCols = [...selectedYCols, col];
    }
  }

  // Export Chart as PNG Image
  function downloadPng() {
    if (!canvasRef) return;
    const isDark = themeStore.resolvedTheme === 'dark';
    
    // Create high-res offscreen canvas with background
    const offscreen = document.createElement('canvas');
    offscreen.width = canvasRef.width;
    offscreen.height = canvasRef.height;
    const ctx = offscreen.getContext('2d');
    if (!ctx) return;

    // Background color
    ctx.fillStyle = isDark ? '#020617' : '#ffffff';
    ctx.fillRect(0, 0, offscreen.width, offscreen.height);
    ctx.drawImage(canvasRef, 0, 0);

    const dataUrl = offscreen.toDataURL('image/png');
    const a = document.createElement('a');
    a.href = dataUrl;
    a.download = `${title.toLowerCase().replace(/[^a-z0-9]/g, '_')}_chart.png`;
    a.click();
  }

  // Copy Chart Image to Clipboard
  async function copyChartToClipboard() {
    if (!canvasRef) return;
    isCopying = true;
    try {
      const isDark = themeStore.resolvedTheme === 'dark';
      const offscreen = document.createElement('canvas');
      offscreen.width = canvasRef.width;
      offscreen.height = canvasRef.height;
      const ctx = offscreen.getContext('2d');
      if (ctx) {
        ctx.fillStyle = isDark ? '#020617' : '#ffffff';
        ctx.fillRect(0, 0, offscreen.width, offscreen.height);
        ctx.drawImage(canvasRef, 0, 0);

        offscreen.toBlob(async (blob) => {
          if (blob) {
            await navigator.clipboard.write([
              new ClipboardItem({ 'image/png': blob })
            ]);
            copySuccess = true;
            setTimeout(() => { copySuccess = false; }, 2000);
          }
        }, 'image/png');
      }
    } catch (e) {
      console.error('Failed to copy chart to clipboard:', e);
    } finally {
      isCopying = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      isOpen = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen && result}
  <!-- Modal Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-black/70 backdrop-blur-xs flex items-center justify-center p-3 sm:p-6 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <!-- Background Click Blocker -->
    <button 
      type="button"
      aria-label="Close modal backdrop"
      onclick={() => isOpen = false}
      class="fixed inset-0 bg-transparent border-0 cursor-default p-0 m-0 z-0"
    ></button>

    <!-- Modal Dialog Frame -->
    <div 
      class="relative z-1 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 transition-all duration-200 {isFullscreen ? 'w-full h-full' : 'w-[1120px] max-w-[96vw] h-[740px] max-h-[92vh]'}"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-slate-200 dark:border-slate-800 bg-surface-950/60 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center text-white shadow-md shadow-indigo-500/20">
            <BarChart3 size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-bold text-slate-900 dark:text-white">Instant Data Charting & Visualization</h3>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20 font-mono">
                {title}
              </span>
            </div>
            <p class="text-[11px] text-slate-500 dark:text-slate-400">
              Interactive visual analytics with auto-aggregated series, multi-chart types, and retina image export
            </p>
          </div>
        </div>

        <!-- Header Actions -->
        <div class="flex items-center gap-2">
          <!-- Copy Image Button -->
          <button
            type="button"
            onclick={copyChartToClipboard}
            disabled={isCopying}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 border border-slate-300 dark:border-slate-700 transition-colors cursor-pointer shadow-xs"
            title="Copy high-res chart image to clipboard"
          >
            {#if copySuccess}
              <Check size={13} class="text-emerald-500" />
              <span class="text-emerald-500 font-bold">Copied Image!</span>
            {:else}
              <Copy size={13} />
              <span>Copy Image</span>
            {/if}
          </button>

          <!-- Download PNG Button -->
          <button
            type="button"
            onclick={downloadPng}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold bg-indigo-600 hover:bg-indigo-500 text-white shadow-md shadow-indigo-500/20 transition-all cursor-pointer"
            title="Export chart as PNG image file"
          >
            <Download size={13} />
            <span>Export PNG</span>
          </button>

          <div class="h-4 w-px bg-slate-200 dark:border-slate-800 mx-1"></div>

          <!-- Fullscreen Toggle -->
          <button
            type="button"
            onclick={() => isFullscreen = !isFullscreen}
            class="p-1.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-surface-800 rounded-lg transition-colors cursor-pointer"
            title={isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
          >
            {#if isFullscreen}
              <Minimize2 size={16} />
            {:else}
              <Maximize2 size={16} />
            {/if}
          </button>

          <!-- Close Modal -->
          <button
            type="button"
            onclick={() => isOpen = false}
            class="p-1.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-surface-800 rounded-lg transition-colors cursor-pointer"
            title="Close (Esc)"
          >
            <X size={17} />
          </button>
        </div>
      </div>

      <!-- Main Layout: Left Config Sidebar + Right Canvas Chart -->
      <div class="flex-1 flex overflow-hidden">
        <!-- 1. Left Control Panel -->
        <aside class="w-72 sm:w-80 border-r border-slate-200 dark:border-slate-800/80 bg-surface-950/40 p-4 overflow-y-auto space-y-4 shrink-0 text-xs">
          <!-- Chart Type Selector -->
          <div class="space-y-1.5">
            <span class="font-bold text-[11px] text-slate-500 dark:text-slate-400 uppercase tracking-wider block">
              Chart Type
            </span>
            <div class="grid grid-cols-2 gap-1.5">
              <button
                type="button"
                onclick={() => selectedChartType = 'bar'}
                class="flex items-center gap-2 p-2 rounded-xl border transition-all text-left cursor-pointer {selectedChartType === 'bar' ? 'bg-indigo-600/15 border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold shadow-xs' : 'bg-surface-900 border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 text-slate-700 dark:text-slate-300'}"
              >
                <BarChart3 size={15} class="text-indigo-500 shrink-0" />
                <span>Bar Chart</span>
              </button>

              <button
                type="button"
                onclick={() => selectedChartType = 'horizontal-bar'}
                class="flex items-center gap-2 p-2 rounded-xl border transition-all text-left cursor-pointer {selectedChartType === 'horizontal-bar' ? 'bg-indigo-600/15 border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold shadow-xs' : 'bg-surface-900 border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 text-slate-700 dark:text-slate-300'}"
              >
                <SlidersHorizontal size={15} class="text-indigo-500 shrink-0 rotate-90" />
                <span>Horiz Bar</span>
              </button>

              <button
                type="button"
                onclick={() => selectedChartType = 'line'}
                class="flex items-center gap-2 p-2 rounded-xl border transition-all text-left cursor-pointer {selectedChartType === 'line' ? 'bg-indigo-600/15 border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold shadow-xs' : 'bg-surface-900 border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 text-slate-700 dark:text-slate-300'}"
              >
                <LineChart size={15} class="text-violet-500 shrink-0" />
                <span>Line Trend</span>
              </button>

              <button
                type="button"
                onclick={() => selectedChartType = 'area'}
                class="flex items-center gap-2 p-2 rounded-xl border transition-all text-left cursor-pointer {selectedChartType === 'area' ? 'bg-indigo-600/15 border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold shadow-xs' : 'bg-surface-900 border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 text-slate-700 dark:text-slate-300'}"
              >
                <TrendingUp size={15} class="text-teal-500 shrink-0" />
                <span>Area Filled</span>
              </button>

              <button
                type="button"
                onclick={() => selectedChartType = 'pie'}
                class="flex items-center gap-2 p-2 rounded-xl border transition-all text-left cursor-pointer {selectedChartType === 'pie' ? 'bg-indigo-600/15 border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold shadow-xs' : 'bg-surface-900 border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 text-slate-700 dark:text-slate-300'}"
              >
                <PieChart size={15} class="text-pink-500 shrink-0" />
                <span>Pie Share</span>
              </button>

              <button
                type="button"
                onclick={() => selectedChartType = 'doughnut'}
                class="flex items-center gap-2 p-2 rounded-xl border transition-all text-left cursor-pointer {selectedChartType === 'doughnut' ? 'bg-indigo-600/15 border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold shadow-xs' : 'bg-surface-900 border-slate-200 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 text-slate-700 dark:text-slate-300'}"
              >
                <Activity size={15} class="text-amber-500 shrink-0" />
                <span>Donut</span>
              </button>
            </div>
          </div>

          <!-- X-Axis / Dimension -->
          <div class="space-y-1.5">
            <span class="font-bold text-[11px] text-slate-500 dark:text-slate-400 uppercase tracking-wider block">
              X-Axis / Category Dimension
            </span>
            <select
              bind:value={selectedXCol}
              class="w-full bg-surface-900 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-800 rounded-xl px-3 py-2 text-xs focus:outline-none focus:border-indigo-500 font-mono"
            >
              {#each result.columns as col}
                <option value={col.name}>{col.name} ({col.dataType || 'text'})</option>
              {/each}
            </select>
          </div>

          <!-- Y-Axis / Metric Series Selection -->
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="font-bold text-[11px] text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                Y-Axis / Metric Values
              </span>
              <span class="text-[10px] text-slate-400 font-mono">
                {selectedYCols.length} selected
              </span>
            </div>

            <div class="max-h-36 overflow-y-auto border border-slate-200 dark:border-slate-800 rounded-xl bg-surface-900 p-1.5 space-y-1">
              {#each result.columns as col}
                {@const isSelected = selectedYCols.includes(col.name)}
                {@const isNumeric = columnAnalysis.numericCols.includes(col.name)}
                <button
                  type="button"
                  onclick={() => toggleYColumn(col.name)}
                  class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-colors cursor-pointer {isSelected ? 'bg-indigo-600/15 text-indigo-600 dark:text-indigo-400 font-bold' : 'hover:bg-surface-800 text-slate-700 dark:text-slate-300'}"
                >
                  <span class="font-mono truncate">{col.name}</span>
                  {#if isNumeric}
                    <span class="text-[9.5px] px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20 font-mono">
                      #num
                    </span>
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <!-- Aggregation Mode -->
          <div class="space-y-1.5">
            <span class="font-bold text-[11px] text-slate-500 dark:text-slate-400 uppercase tracking-wider block">
              Aggregation
            </span>
            <select
              bind:value={aggregation}
              class="w-full bg-surface-900 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-800 rounded-xl px-3 py-1.5 text-xs focus:outline-none focus:border-indigo-500"
            >
              <option value="none">None (Plot Raw Rows)</option>
              <option value="sum">Sum (Group & Add)</option>
              <option value="avg">Average</option>
              <option value="count">Count of Records</option>
              <option value="max">Maximum Value</option>
              <option value="min">Minimum Value</option>
            </select>
          </div>

          <!-- Sorting & Limits -->
          <div class="grid grid-cols-2 gap-2">
            <div class="space-y-1">
              <span class="font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-wider block">
                Sort
              </span>
              <select
                bind:value={sortOrder}
                class="w-full bg-surface-900 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-800 rounded-lg px-2 py-1.5 text-xs focus:outline-none"
              >
                <option value="none">Original</option>
                <option value="val-desc">Top Values (High-Low)</option>
                <option value="val-asc">Lowest (Low-High)</option>
                <option value="x-asc">A → Z / Date Asc</option>
                <option value="x-desc">Z → A / Date Desc</option>
              </select>
            </div>

            <div class="space-y-1">
              <span class="font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-wider block">
                Max Rows
              </span>
              <select
                bind:value={topNLimit}
                class="w-full bg-surface-900 text-slate-900 dark:text-slate-100 border border-slate-200 dark:border-slate-800 rounded-lg px-2 py-1.5 text-xs focus:outline-none font-mono"
              >
                <option value={10}>Top 10</option>
                <option value={25}>Top 25</option>
                <option value={50}>Top 50</option>
                <option value={100}>Top 100</option>
                <option value={500}>Top 500</option>
                <option value={0}>All Rows</option>
              </select>
            </div>
          </div>

          <!-- Color Palette Picker -->
          <div class="space-y-1.5">
            <span class="font-bold text-[11px] text-slate-500 dark:text-slate-400 uppercase tracking-wider block">
              Color Palette
            </span>
            <div class="grid grid-cols-2 gap-1.5">
              {#each Object.keys(palettes) as palKey}
                <button
                  type="button"
                  onclick={() => selectedPalette = palKey as any}
                  class="p-2 rounded-xl border flex flex-col gap-1.5 cursor-pointer transition-all {selectedPalette === palKey ? 'border-indigo-500 bg-indigo-500/10' : 'border-slate-200 dark:border-slate-800 bg-surface-900'}"
                >
                  <span class="font-semibold text-[10.5px] capitalize text-slate-800 dark:text-slate-200">{palKey}</span>
                  <div class="flex items-center gap-1">
                    {#each palettes[palKey].slice(0, 4) as col}
                      <span class="w-3 h-3 rounded-full" style="background-color: {col}"></span>
                    {/each}
                  </div>
                </button>
              {/each}
            </div>
          </div>
        </aside>

        <!-- 2. Right Main Chart View & KPI Cards -->
        <main class="flex-1 flex flex-col p-5 bg-surface-950 overflow-hidden space-y-4">
          <!-- Top KPI Stat Metric Cards -->
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 shrink-0">
            <!-- Data Points plotted -->
            <div class="bg-surface-900/80 border border-slate-200 dark:border-slate-800/80 rounded-xl p-3 flex flex-col justify-between shadow-xs">
              <span class="text-[10.5px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">Data Points</span>
              <div class="flex items-baseline gap-1 mt-1">
                <span class="text-base font-bold font-mono text-slate-900 dark:text-slate-100">{chartData.stats.count}</span>
                <span class="text-[10px] text-slate-400">plotted</span>
              </div>
            </div>

            <!-- Total / Sum -->
            <div class="bg-surface-900/80 border border-slate-200 dark:border-slate-800/80 rounded-xl p-3 flex flex-col justify-between shadow-xs">
              <span class="text-[10.5px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">Total Sum</span>
              <div class="flex items-baseline gap-1 mt-1 truncate">
                <span class="text-base font-bold font-mono text-indigo-600 dark:text-indigo-400 truncate">
                  {chartData.stats.sum.toLocaleString(undefined, { maximumFractionDigits: 1 })}
                </span>
              </div>
            </div>

            <!-- Average -->
            <div class="bg-surface-900/80 border border-slate-200 dark:border-slate-800/80 rounded-xl p-3 flex flex-col justify-between shadow-xs">
              <span class="text-[10.5px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">Average</span>
              <div class="flex items-baseline gap-1 mt-1 truncate">
                <span class="text-base font-bold font-mono text-emerald-600 dark:text-emerald-400 truncate">
                  {chartData.stats.avg.toLocaleString(undefined, { maximumFractionDigits: 2 })}
                </span>
              </div>
            </div>

            <!-- Max / Peak -->
            <div class="bg-surface-900/80 border border-slate-200 dark:border-slate-800/80 rounded-xl p-3 flex flex-col justify-between shadow-xs">
              <span class="text-[10.5px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">Peak / Max</span>
              <div class="flex items-baseline gap-1 mt-1 truncate">
                <span class="text-base font-bold font-mono text-violet-600 dark:text-violet-400 truncate">
                  {chartData.stats.max.toLocaleString(undefined, { maximumFractionDigits: 1 })}
                </span>
              </div>
            </div>
          </div>

          <!-- Chart Canvas Container -->
          <div class="flex-1 min-h-[300px] border border-slate-200 dark:border-slate-800/80 rounded-2xl bg-surface-900/60 p-4 relative flex items-center justify-center overflow-hidden shadow-inner">
            <canvas bind:this={canvasRef} class="w-full h-full"></canvas>
          </div>
        </main>
      </div>
    </div>
  </div>
{/if}
