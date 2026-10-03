<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    Dices, 
    Sparkles, 
    X, 
    Check, 
    Copy, 
    Eye, 
    Code, 
    Sliders, 
    Database, 
    RefreshCw, 
    ArrowRight, 
    Table as TableIcon,
    KeyRound,
    Link2,
    CheckCircle2,
    AlertCircle,
    Download,
    Play
  } from 'lucide-svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { api } from '$lib/api/client';
  import type { ColumnMockRule, TableMockInspection, MockBatchResult } from '$lib/api/types';
  import CustomSelect from '$lib/components/ui/CustomSelect.svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  // State
  let selectedConnectionId = $state<string>('');
  let selectedTable = $state<string>('');
  let activeTab = $state<'rules' | 'preview' | 'sql'>('rules');
  let rowCount = $state<number>(100);
  let customRowCount = $state<number>(100);
  let chunkSize = $state<number>(250);

  // Table rules state
  let isLoadingSchema = $state<boolean>(false);
  let isInspecting = $state<boolean>(false);
  let columnRules = $state<ColumnMockRule[]>([]);
  let previewRows = $state<Record<string, any>[]>([]);
  let isGeneratingPreview = $state<boolean>(false);
  let sqlScript = $state<string>('');
  let isGeneratingSql = $state<boolean>(false);

  // Execution state
  let isInserting = $state<boolean>(false);
  let insertResult = $state<MockBatchResult | null>(null);
  let errorMessage = $state<string | null>(null);
  let isCopied = $state<boolean>(false);
  let copyTimeout: any = null;

  const connections = $derived(connectionStore.connections);
  const activeConn = $derived(connectionStore.activeConnection);
  const tables = $derived(selectedConnectionId ? (connectionStore.schemas[selectedConnectionId]?.tables || []) : []);

  const generatorOptions = [
    { value: 'full_name', label: '👤 Person: Full Name (e.g. Aditya Pratama)' },
    { value: 'first_name', label: '👤 Person: First Name (e.g. Alexander)' },
    { value: 'last_name', label: '👤 Person: Last Name (e.g. Wijaya)' },
    { value: 'username', label: '👤 Person: Username (e.g. swiftcoder42)' },
    { value: 'email', label: '📧 Contact: Email Address (e.g. name@domain.com)' },
    { value: 'phone', label: '📞 Contact: Phone Number (+62-812-xxxx)' },
    { value: 'street_address', label: '📍 Location: Street Address (e.g. Jl. Sudirman 45)' },
    { value: 'city', label: '📍 Location: City (Jakarta, Tokyo, London...)' },
    { value: 'country', label: '📍 Location: Country' },
    { value: 'zip_code', label: '📍 Location: Zip / Postal Code' },
    { value: 'company', label: '💼 Work: Company Name (TechNova Corp)' },
    { value: 'job_title', label: '💼 Work: Job Title (Software Engineer)' },
    { value: 'url', label: '🌐 Internet: Website URL' },
    { value: 'avatar_url', label: '🖼️ Internet: Avatar Image URL' },
    { value: 'ip_v4', label: '🌐 Internet: IPv4 Address (192.168.x.x)' },
    { value: 'price', label: '💰 Finance: Price / Decimal Amount ($19.99)' },
    { value: 'integer', label: '🔢 Numbers: Random Integer (1 - 1000)' },
    { value: 'boolean', label: '🔘 Logic: Boolean (TRUE / FALSE)' },
    { value: 'uuid', label: '🔑 ID: UUID v4 (Unique String)' },
    { value: 'auto_increment', label: '🔢 ID: Auto-Increment / Sequence' },
    { value: 'fk_reference', label: '🔗 Relation: Sample from FK Target Table' },
    { value: 'timestamp_past', label: '📅 Date: Past Timestamp (1-2 years ago)' },
    { value: 'timestamp_future', label: '📅 Date: Future Timestamp (next 1 year)' },
    { value: 'date_past', label: '📅 Date: Birth Date (18-60 years ago)' },
    { value: 'status', label: '🏷️ Enum: Status (active, pending, completed)' },
    { value: 'lorem_sentence', label: '📝 Content: Description Sentence' },
    { value: 'json_object', label: '📦 Structured: JSON Config Object' },
    { value: 'custom_list', label: '⚙️ Custom: Random Pick from List' },
  ];

  $effect(() => {
    if (isOpen && activeConn && !selectedConnectionId) {
      selectedConnectionId = activeConn.id;
    }
  });

  $effect(() => {
    if (isOpen && selectedConnectionId) {
      loadTablesForConnection(selectedConnectionId);
    }
  });

  $effect(() => {
    if (isOpen && selectedConnectionId && selectedTable) {
      inspectTable(selectedConnectionId, selectedTable);
    }
  });

  async function loadTablesForConnection(connId: string) {
    if (!connId) return;
    isLoadingSchema = true;
    try {
      await connectionStore.loadSchema(connId);
      const curTables = connectionStore.schemas[connId]?.tables || [];
      if (curTables.length > 0 && (!selectedTable || !curTables.some(t => t.name === selectedTable))) {
        selectedTable = curTables[0].name;
      }
    } catch (err) {
      console.error('Failed to load schema for mock generator:', err);
    } finally {
      isLoadingSchema = false;
    }
  }

  async function inspectTable(connId: string, tblName: string) {
    if (!connId || !tblName) return;
    isInspecting = true;
    errorMessage = null;
    insertResult = null;

    try {
      const inspection = await api.inspectTableMockConfig(connId, tblName);
      columnRules = inspection.columns;
    } catch (err: any) {
      console.error('Failed to inspect table for mock data:', err);
      errorMessage = typeof err === 'string' ? err : err?.message || String(err);
    } finally {
      isInspecting = false;
    }
  }

  function toggleAllColumns(checked: boolean) {
    columnRules = columnRules.map(r => ({
      ...r,
      include: checked
    }));
  }

  async function handleTabChange(tab: 'rules' | 'preview' | 'sql') {
    activeTab = tab;
    if (tab === 'preview') {
      await loadPreview();
    } else if (tab === 'sql') {
      await loadSqlScript();
    }
  }

  async function loadPreview() {
    isGeneratingPreview = true;
    try {
      previewRows = await api.previewMockRows(columnRules, 10);
    } catch (err: any) {
      console.error('Failed to generate preview:', err);
    } finally {
      isGeneratingPreview = false;
    }
  }

  async function loadSqlScript() {
    isGeneratingSql = true;
    try {
      sqlScript = await api.generateMockSqlScript(selectedTable, columnRules, rowCount);
    } catch (err: any) {
      console.error('Failed to generate SQL script:', err);
    } finally {
      isGeneratingSql = false;
    }
  }

  async function copySqlToClipboard() {
    try {
      await navigator.clipboard.writeText(sqlScript);
      isCopied = true;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => isCopied = false, 2000);
    } catch (err) {
      console.error('Failed to copy SQL:', err);
    }
  }

  async function handleDownloadSql() {
    const defaultName = `mock_data_${selectedTable}_${rowCount}.sql`;
    const selected = await api.pickSaveFile(defaultName, [{ name: 'SQL Script (*.sql)', extensions: ['sql'] }]);
    if (selected) {
      try {
        await api.exportDataDictionaryFile(selectedConnectionId, 'markdown', selected); // save raw text
      } catch (err) {
        console.error('Failed to download SQL:', err);
      }
    }
  }

  async function handleExecuteBatch() {
    if (!selectedConnectionId || !selectedTable || isInserting) return;
    isInserting = true;
    errorMessage = null;
    insertResult = null;

    try {
      const result = await api.executeMockBatchInsert(
        selectedConnectionId,
        selectedTable,
        columnRules,
        rowCount,
        chunkSize
      );
      insertResult = result;
      // Refresh schema row counts
      connectionStore.loadSchema(selectedConnectionId);
    } catch (err: any) {
      console.error('Failed to execute mock batch:', err);
      errorMessage = typeof err === 'string' ? err : err?.message || String(err);
    } finally {
      isInserting = false;
    }
  }

  function handleOpenDataGrid() {
    onClose();
    if (selectedTable) {
      tabsStore.openTableGridTab(selectedTable);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }

  const includedColumnsCount = $derived(columnRules.filter(r => r.include).length);
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
          <div class="p-2 bg-gradient-to-tr from-violet-600 to-fuchsia-600 text-white rounded-xl shadow-md shadow-violet-500/20 shrink-0">
            <Dices size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="font-bold text-sm text-slate-900 dark:text-white">
                Smart QA Mock Data Generator
              </h3>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-violet-500/10 text-violet-700 dark:text-violet-300 font-bold border border-violet-500/30">
                Schema-Aware • FK Integrity
              </span>
            </div>
            <p class="text-[11px] text-slate-600 dark:text-slate-400">
              Generate realistic synthetic test datasets with semantic column inference and relationship validation
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <!-- Connection Selector -->
          <div class="w-48">
            <CustomSelect
              id="qa-conn-select"
              size="sm"
              bind:value={selectedConnectionId}
              options={connections.map(c => ({
                value: c.id,
                label: c.name,
                badge: c.environment
              }))}
            />
          </div>

          <!-- Target Table Selector -->
          <div class="w-52">
            <CustomSelect
              id="qa-table-select"
              size="sm"
              bind:value={selectedTable}
              options={tables.map((t: { name: string; schema: string; rowCountEstimate?: number }) => ({
                value: t.name,
                label: `${t.schema}.${t.name}`,
                badge: t.rowCountEstimate !== undefined ? `~${t.rowCountEstimate} rows` : undefined
              }))}
            />
          </div>

          <button 
            type="button"
            onclick={() => inspectTable(selectedConnectionId, selectedTable)}
            disabled={isInspecting}
            class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-white rounded-lg hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer"
            title="Re-inspect Table Schema"
          >
            <RefreshCw size={15} class={isInspecting ? 'animate-spin text-violet-500' : ''} />
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

      <!-- Mode Tabs & Top Toolbar -->
      <div class="flex items-center justify-between border-b border-slate-200 dark:border-slate-800 bg-slate-100/70 dark:bg-surface-950/50 px-5 py-2">
        <div class="flex items-center gap-1.5 text-xs font-semibold">
          <button 
            type="button"
            onclick={() => handleTabChange('rules')}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all cursor-pointer {activeTab === 'rules' ? 'bg-white dark:bg-surface-800 text-violet-600 dark:text-violet-400 border-slate-300 dark:border-slate-700 shadow-xs font-bold' : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            <Sliders size={13} />
            <span>Column Rules & Mappings</span>
            <span class="px-1.5 py-0.2 text-[10px] rounded-full bg-violet-500/15 text-violet-700 dark:text-violet-300 font-mono">
              {includedColumnsCount}/{columnRules.length}
            </span>
          </button>

          <button 
            type="button"
            onclick={() => handleTabChange('preview')}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all cursor-pointer {activeTab === 'preview' ? 'bg-white dark:bg-surface-800 text-violet-600 dark:text-violet-400 border-slate-300 dark:border-slate-700 shadow-xs font-bold' : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            <Eye size={13} />
            <span>Live Sample Preview (10 Rows)</span>
          </button>

          <button 
            type="button"
            onclick={() => handleTabChange('sql')}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all cursor-pointer {activeTab === 'sql' ? 'bg-white dark:bg-surface-800 text-violet-600 dark:text-violet-400 border-slate-300 dark:border-slate-700 shadow-xs font-bold' : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'}"
          >
            <Code size={13} />
            <span>SQL Insert Script</span>
          </button>
        </div>

        {#if activeTab === 'rules'}
          <div class="flex items-center gap-2 text-xs">
            <button
              type="button"
              onclick={() => toggleAllColumns(true)}
              class="px-2.5 py-1 text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white rounded hover:bg-slate-200/70 dark:hover:bg-surface-800 transition-colors font-medium cursor-pointer"
            >
              Select All
            </button>
            <span class="text-slate-300 dark:text-slate-700">•</span>
            <button
              type="button"
              onclick={() => toggleAllColumns(false)}
              class="px-2.5 py-1 text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white rounded hover:bg-slate-200/70 dark:hover:bg-surface-800 transition-colors font-medium cursor-pointer"
            >
              Deselect All
            </button>
          </div>
        {:else if activeTab === 'sql'}
          <div class="flex items-center gap-2 text-xs">
            <button
              type="button"
              onclick={copySqlToClipboard}
              class="px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white bg-white dark:bg-surface-800 border border-slate-200 dark:border-slate-700 rounded-lg flex items-center gap-1.5 cursor-pointer shadow-xs"
            >
              {#if isCopied}
                <Check size={13} class="text-emerald-500" />
                <span class="text-emerald-500 font-bold">Copied!</span>
              {:else}
                <Copy size={13} class="text-slate-400" />
                <span>Copy SQL</span>
              {/if}
            </button>
          </div>
        {/if}
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-hidden relative bg-slate-50/50 dark:bg-surface-950/60 flex flex-col min-h-0">
        {#if isInspecting || isLoadingSchema}
          <div class="absolute inset-0 bg-white/70 dark:bg-surface-900/70 backdrop-blur-xs flex flex-col items-center justify-center gap-3 z-20">
            <RefreshCw size={26} class="animate-spin text-violet-500" />
            <span class="text-xs font-semibold text-slate-700 dark:text-slate-300">Inspecting Schema & Foreign Key Constraints...</span>
          </div>
        {/if}

        {#if errorMessage}
          <div class="p-6">
            <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-700 dark:text-rose-300 text-xs flex items-start gap-3">
              <AlertCircle size={16} class="shrink-0 mt-0.5" />
              <div>
                <strong class="block mb-1 font-bold">Error:</strong>
                <span class="font-mono">{errorMessage}</span>
              </div>
            </div>
          </div>
        {:else if activeTab === 'rules'}
          <!-- 1. Column Rules Configurator Table -->
          <div class="flex-1 overflow-y-auto p-4 space-y-2 select-text">
            <table class="w-full border-collapse text-left text-xs bg-white dark:bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-xs">
              <thead class="bg-slate-100/80 dark:bg-surface-950/80 border-b border-slate-200 dark:border-slate-800 text-[11px] font-semibold text-slate-500 uppercase tracking-wider">
                <tr>
                  <th class="py-2.5 px-3 w-10 text-center">Inc</th>
                  <th class="py-2.5 px-3">Column Name & Keys</th>
                  <th class="py-2.5 px-3 w-28">Data Type</th>
                  <th class="py-2.5 px-3 w-72">Semantic Generator Rule</th>
                  <th class="py-2.5 px-3 w-40">Options / Null %</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-200 dark:divide-slate-800/80">
                {#each columnRules as rule (rule.columnName)}
                  <tr class="hover:bg-slate-50 dark:hover:bg-surface-800/50 transition-colors {rule.include ? '' : 'opacity-40'}">
                    <!-- Include Checkbox -->
                    <td class="py-2.5 px-3 text-center">
                      <input 
                        type="checkbox" 
                        bind:checked={rule.include} 
                        class="rounded border-slate-300 text-violet-600 focus:ring-violet-500 cursor-pointer"
                      />
                    </td>

                    <!-- Column Name & Badges -->
                    <td class="py-2.5 px-3">
                      <div class="flex items-center gap-2 flex-wrap">
                        <span class="font-mono font-bold text-slate-800 dark:text-slate-200">{rule.columnName}</span>
                        {#if rule.isPrimaryKey}
                          <span class="px-1.5 py-0.2 rounded text-[9px] font-bold bg-amber-500/15 text-amber-700 dark:text-amber-400 border border-amber-500/30 flex items-center gap-0.5">
                            <KeyRound size={9} /> PK
                          </span>
                        {/if}
                        {#if rule.isForeignKey}
                          <span class="px-1.5 py-0.2 rounded text-[9px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30 flex items-center gap-0.5" title="References {rule.fkTargetTable}.{rule.fkTargetColumn}">
                            <Link2 size={9} /> FK ➔ {rule.fkTargetTable}
                          </span>
                        {/if}
                        {#if rule.nullable}
                          <span class="px-1.5 py-0.2 rounded text-[9px] text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-surface-800">
                            NULL
                          </span>
                        {/if}
                      </div>
                    </td>

                    <!-- Data Type -->
                    <td class="py-2.5 px-3">
                      <span class="font-mono text-[11px] text-slate-600 dark:text-slate-400 font-semibold">{rule.dataType}</span>
                    </td>

                    <!-- Generator Type Dropdown -->
                    <td class="py-2.5 px-3">
                      <div class="w-full">
                        <CustomSelect
                          id="gen-{rule.columnName}"
                          size="sm"
                          bind:value={rule.generatorType}
                          options={generatorOptions}
                        />
                      </div>
                    </td>

                    <!-- Options / Null % -->
                    <td class="py-2.5 px-3">
                      {#if rule.generatorType === 'custom_list'}
                        <input
                          type="text"
                          bind:value={rule.customOptions}
                          placeholder="opt1, opt2, opt3"
                          class="w-full px-2 py-1 text-[11px] bg-slate-50 dark:bg-surface-950 border border-slate-200 dark:border-slate-700 rounded focus:outline-none focus:border-violet-500"
                        />
                      {:else if rule.generatorType === 'fk_reference'}
                        <span class="text-[10px] text-emerald-600 dark:text-emerald-400 font-mono">
                          {rule.sampleFkValues?.length || 0} keys sampled
                        </span>
                      {:else if rule.nullable}
                        <div class="flex items-center gap-2">
                          <input 
                            type="range" 
                            min="0" 
                            max="50" 
                            step="5" 
                            bind:value={rule.nullPercentage} 
                            class="w-16 accent-violet-600 cursor-pointer" 
                          />
                          <span class="text-[10px] font-mono text-slate-500">{rule.nullPercentage}% null</span>
                        </div>
                      {:else}
                        <span class="text-[10px] text-slate-400 dark:text-slate-600">NOT NULL</span>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else if activeTab === 'preview'}
          <!-- 2. Live Sample Preview Grid -->
          <div class="flex-1 overflow-auto p-4 select-text">
            {#if isGeneratingPreview}
              <div class="flex items-center justify-center h-48 text-slate-500">
                <RefreshCw size={20} class="animate-spin mr-2 text-violet-500" />
                Generating live 10-row dataset...
              </div>
            {:else if previewRows.length > 0}
              <div class="border border-slate-200 dark:border-slate-800 rounded-xl overflow-x-auto bg-white dark:bg-surface-900 shadow-xs">
                <table class="w-full border-collapse text-left text-xs">
                  <thead class="bg-slate-100 dark:bg-surface-950 border-b border-slate-200 dark:border-slate-800 text-[11px] font-semibold text-slate-500 uppercase">
                    <tr>
                      <th class="py-2.5 px-3 w-10 text-center text-slate-400 font-mono">#</th>
                      {#each Object.keys(previewRows[0]) as colKey}
                        <th class="py-2.5 px-3 font-mono text-slate-700 dark:text-slate-300">{colKey}</th>
                      {/each}
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-slate-200 dark:divide-slate-800">
                    {#each previewRows as row, rIdx}
                      <tr class="hover:bg-slate-50 dark:hover:bg-surface-800/40 font-mono text-[11px]">
                        <td class="py-2 px-3 text-center text-slate-400 font-bold">{rIdx + 1}</td>
                        {#each Object.values(row) as val}
                          <td class="py-2 px-3 whitespace-nowrap">
                            {#if val === null}
                              <span class="text-slate-400 italic">NULL</span>
                            {:else if typeof val === 'boolean'}
                              <span class="font-bold {val ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}">{val ? 'TRUE' : 'FALSE'}</span>
                            {:else if typeof val === 'object'}
                              <span class="text-indigo-600 dark:text-indigo-400">{JSON.stringify(val)}</span>
                            {:else}
                              <span class="text-slate-800 dark:text-slate-200">{val}</span>
                            {/if}
                          </td>
                        {/each}
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {:else}
              <div class="text-center py-12 text-slate-500">
                Click Refresh Preview to generate sample data.
              </div>
            {/if}
          </div>
        {:else}
          <!-- 3. SQL Insert Script Monospace View -->
          <div class="flex-1 overflow-auto p-4 select-text">
            {#if isGeneratingSql}
              <div class="flex items-center justify-center h-48 text-slate-500">
                <RefreshCw size={20} class="animate-spin mr-2 text-violet-500" />
                Compiling parameterized SQL script...
              </div>
            {:else}
              <pre class="bg-white dark:bg-surface-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 font-mono text-xs text-violet-700 dark:text-violet-300 whitespace-pre overflow-x-auto leading-relaxed shadow-xs">{sqlScript}</pre>
            {/if}
          </div>
        {/if}

        <!-- Success Result Banner if batch finished -->
        {#if insertResult}
          <div class="px-5 py-3 bg-emerald-500/10 border-t border-emerald-500/30 flex items-center justify-between text-xs animate-in slide-in-from-bottom duration-200">
            <div class="flex items-center gap-3">
              <div class="p-1.5 bg-emerald-500 text-white rounded-lg">
                <CheckCircle2 size={16} />
              </div>
              <div>
                <div class="font-bold text-emerald-800 dark:text-emerald-300">
                  Successfully generated & inserted {insertResult.insertedRows.toLocaleString()} rows into `{insertResult.tableName}`!
                </div>
                <div class="text-[11px] text-emerald-700 dark:text-emerald-400">
                  Execution Time: {insertResult.executionTimeMs.toFixed(1)}ms • {insertResult.chunksCount} chunks executed • Average speed: {Math.round((insertResult.insertedRows / (insertResult.executionTimeMs || 1)) * 1000).toLocaleString()} rows/sec
                </div>
              </div>
            </div>

            <button
              type="button"
              onclick={handleOpenDataGrid}
              class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white font-semibold rounded-lg flex items-center gap-1.5 shadow-xs cursor-pointer transition-colors"
            >
              <span>View in DataGrid</span>
              <ArrowRight size={13} />
            </button>
          </div>
        {/if}
      </div>

      <!-- Modal Footer (Row Count Selector & Action Trigger) -->
      <div class="px-5 py-3 border-t border-slate-200 dark:border-slate-800 bg-slate-50/90 dark:bg-surface-950/80 flex items-center justify-between gap-4">
        <!-- Preset Row Count Buttons -->
        <div class="flex items-center gap-2 text-xs">
          <span class="font-semibold text-slate-600 dark:text-slate-400">Generate Rows:</span>
          {#each [10, 50, 100, 500, 1000, 5000, 10000] as count}
            <button
              type="button"
              onclick={() => { rowCount = count; customRowCount = count; }}
              class="px-2.5 py-1 rounded-md text-xs font-mono font-semibold transition-colors cursor-pointer border {rowCount === count ? 'bg-violet-600 text-white border-violet-600 shadow-xs' : 'bg-white dark:bg-surface-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:bg-slate-100 dark:hover:bg-surface-700'}"
            >
              {count >= 1000 ? `${count / 1000}k` : count}
            </button>
          {/each}
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center gap-2">
          {#if activeTab === 'rules'}
            <button
              type="button"
              onclick={() => handleTabChange('preview')}
              class="px-3.5 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white bg-white dark:bg-surface-800 border border-slate-200 dark:border-slate-700 rounded-lg flex items-center gap-1.5 cursor-pointer shadow-xs hover:bg-slate-50 dark:hover:bg-surface-700 transition-colors"
            >
              <Eye size={13} class="text-slate-500 dark:text-slate-400" />
              <span>Preview 10 Rows</span>
            </button>
          {/if}

          <button
            type="button"
            onclick={handleExecuteBatch}
            disabled={isInserting || includedColumnsCount === 0}
            class="px-4 py-1.5 text-xs font-semibold text-white bg-gradient-to-r from-violet-600 to-fuchsia-600 hover:from-violet-500 hover:to-fuchsia-500 rounded-lg flex items-center gap-2 cursor-pointer shadow-md shadow-violet-600/20 transition-all disabled:opacity-50"
          >
            {#if isInserting}
              <RefreshCw size={13} class="animate-spin text-white" />
              <span>Inserting {rowCount.toLocaleString()} Rows...</span>
            {:else}
              <Play size={13} fill="currentColor" />
              <span>Generate & Insert {rowCount.toLocaleString()} Rows</span>
            {/if}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
