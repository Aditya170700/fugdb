<script lang="ts">
  import { 
    X, 
    Plus, 
    Save, 
    Clock, 
    Database, 
    HardDrive, 
    Folder, 
    FileText, 
    Calendar, 
    Sparkles, 
    Check, 
    FileSpreadsheet, 
    FileCode, 
    Table as TableIcon,
    AlertCircle
  } from 'lucide-svelte';
  import { schedulerStore } from '$lib/state/scheduler.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import type { ScheduleJob, ScheduleJobType } from '$lib/api/types';
  import CustomSelect from '../ui/CustomSelect.svelte';
  import { isTauri } from '$lib/utils/environment';

  let job = $derived(schedulerStore.editingJob);

  let jobName = $state('');
  let jobDescription = $state('');
  let jobType = $state<ScheduleJobType>('query_export');
  let connectionId = $state('');
  let database = $state('');
  let cronExpression = $state('interval:1h');
  let frequencyDisplay = $state('Every 1 hour');
  let enabled = $state(true);

  // Query config
  let querySql = $state('SELECT * FROM users LIMIT 500;');
  let exportFormat = $state('csv');
  let targetDir = $state('');
  let filenamePattern = $state('{name}_{timestamp}.csv');

  // Backup config
  let backupTables = $state<string[]>([]);
  let backupAllTables = $state(true);
  let includeSchema = $state(true);
  let includeData = $state(true);
  let backupTargetDir = $state('');
  let backupFilenamePattern = $state('backup_{db}_{timestamp}.sql');

  // Custom schedule state
  let schedulePreset = $state('interval:1h');
  let dailyTime = $state('02:00');
  let weeklyDay = $state('sun');
  let weeklyTime = $state('03:00');
  let customCron = $state('0 2 * * *');
  let tableSearch = $state('');

  let isSaving = $state(false);

  // Sync state with editingJob
  $effect(() => {
    if (job) {
      jobName = job.name || '';
      jobDescription = job.description || '';
      jobType = job.job_type || 'query_export';
      connectionId = job.connection_id || connectionStore.activeConnectionId || connectionStore.connections[0]?.id || 'default';
      database = job.database || connectionStore.activeConnection?.database || '';
      cronExpression = job.cron_expression || 'interval:1h';
      frequencyDisplay = job.frequency_display || 'Every 1 hour';
      enabled = job.enabled ?? true;

      // Detect preset
      if (cronExpression.startsWith('daily:')) {
        schedulePreset = 'daily';
        dailyTime = cronExpression.replace('daily:', '');
      } else if (cronExpression.startsWith('weekly:')) {
        schedulePreset = 'weekly';
        const parts = cronExpression.replace('weekly:', '').split(':');
        weeklyDay = parts[0] || 'sun';
        weeklyTime = `${parts[1] || '03'}:${parts[2] || '00'}`;
      } else if (cronExpression.startsWith('interval:')) {
        schedulePreset = cronExpression;
      } else {
        schedulePreset = 'custom';
        customCron = cronExpression;
      }

      if (job.query_config) {
        querySql = job.query_config.sql || 'SELECT * FROM users LIMIT 500;';
        exportFormat = job.query_config.format || 'csv';
        targetDir = job.query_config.target_dir || schedulerStore.defaultBackupDir || '';
        filenamePattern = job.query_config.filename_pattern || '{name}_{timestamp}.csv';
      } else {
        targetDir = schedulerStore.defaultBackupDir || '';
      }

      if (job.backup_config) {
        backupTables = job.backup_config.tables || [];
        backupAllTables = (job.backup_config.tables || []).length === 0;
        includeSchema = job.backup_config.include_schema ?? true;
        includeData = job.backup_config.include_data ?? true;
        backupTargetDir = job.backup_config.target_dir || schedulerStore.defaultBackupDir || '';
        backupFilenamePattern = job.backup_config.filename_pattern || 'backup_{db}_{timestamp}.sql';
      } else {
        backupTargetDir = schedulerStore.defaultBackupDir || '';
      }
    }
  });

  const connectionOptions = $derived(
    connectionStore.connections.map(c => ({
      value: c.id,
      label: `${c.name} (${c.driver.toUpperCase()})`
    }))
  );

  const formatOptions = [
    { value: 'csv', label: 'CSV (Comma Separated Values)' },
    { value: 'json', label: 'JSON Array (Pretty Formatted)' },
    { value: 'xlsx', label: 'Excel Workbook (.xlsx)' },
    { value: 'tsv', label: 'TSV (Tab Separated Values)' },
    { value: 'ndjson', label: 'NDJSON (Newline Delimited JSON)' }
  ];

  const availableTables = $derived(
    (connectionStore.activeSchemaTree?.tables || []).map(t => t.name)
  );

  const filteredAvailableTables = $derived(
    availableTables.filter(t => !tableSearch || t.toLowerCase().includes(tableSearch.toLowerCase()))
  );

  function handleSchedulePresetChange(preset: string) {
    schedulePreset = preset;
    if (preset === 'interval:15m') {
      cronExpression = 'interval:15m';
      frequencyDisplay = 'Every 15 minutes';
    } else if (preset === 'interval:30m') {
      cronExpression = 'interval:30m';
      frequencyDisplay = 'Every 30 minutes';
    } else if (preset === 'interval:1h') {
      cronExpression = 'interval:1h';
      frequencyDisplay = 'Every 1 hour';
    } else if (preset === 'interval:6h') {
      cronExpression = 'interval:6h';
      frequencyDisplay = 'Every 6 hours';
    } else if (preset === 'interval:12h') {
      cronExpression = 'interval:12h';
      frequencyDisplay = 'Every 12 hours';
    } else if (preset === 'daily') {
      cronExpression = `daily:${dailyTime}`;
      frequencyDisplay = `Daily at ${dailyTime}`;
    } else if (preset === 'weekly') {
      const dayNames: Record<string, string> = { mon: 'Monday', tue: 'Tuesday', wed: 'Wednesday', thu: 'Thursday', fri: 'Friday', sat: 'Saturday', sun: 'Sunday' };
      cronExpression = `weekly:${weeklyDay}:${weeklyTime}`;
      frequencyDisplay = `Weekly on ${dayNames[weeklyDay] || 'Sunday'} at ${weeklyTime}`;
    } else if (preset === 'custom') {
      cronExpression = customCron;
      frequencyDisplay = `Custom Cron: ${customCron}`;
    }
  }

  function updateDailyCron() {
    cronExpression = `daily:${dailyTime}`;
    frequencyDisplay = `Daily at ${dailyTime}`;
  }

  function updateWeeklyCron() {
    const dayNames: Record<string, string> = { mon: 'Monday', tue: 'Tuesday', wed: 'Wednesday', thu: 'Thursday', fri: 'Friday', sat: 'Saturday', sun: 'Sunday' };
    cronExpression = `weekly:${weeklyDay}:${weeklyTime}`;
    frequencyDisplay = `Weekly on ${dayNames[weeklyDay] || 'Sunday'} at ${weeklyTime}`;
  }

  function updateCustomCron() {
    cronExpression = customCron;
    frequencyDisplay = `Custom Cron: ${customCron}`;
  }

  async function handleBrowseFolder(target: 'query' | 'backup') {
    if (isTauri) {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const selected = await open({
          directory: true,
          multiple: false,
          title: 'Select Destination Folder for Automated Backups'
        });
        if (selected && typeof selected === 'string') {
          if (target === 'query') {
            targetDir = selected;
          } else {
            backupTargetDir = selected;
          }
        }
      } catch (err) {
        console.error('Failed to open directory picker:', err);
      }
    } else {
      const path = prompt('Enter destination directory path:', target === 'query' ? targetDir : backupTargetDir);
      if (path) {
        if (target === 'query') targetDir = path;
        else backupTargetDir = path;
      }
    }
  }

  function toggleTableSelection(tableName: string) {
    if (backupTables.includes(tableName)) {
      backupTables = backupTables.filter(t => t !== tableName);
    } else {
      backupTables = [...backupTables, tableName];
    }
  }

  async function handleSave() {
    if (!jobName.trim()) {
      alert('Please provide a name for this scheduled automation.');
      return;
    }

    if (jobType === 'query_export' && !querySql.trim()) {
      alert('Please enter the SQL query to execute.');
      return;
    }

    if (jobType === 'query_export' && !targetDir.trim()) {
      alert('Please specify the destination directory.');
      return;
    }

    if (jobType === 'database_backup' && !backupTargetDir.trim()) {
      alert('Please specify the destination backup directory.');
      return;
    }

    isSaving = true;
    try {
      const updatedJob: ScheduleJob = {
        id: job?.id || `job_${Date.now()}`,
        name: jobName.trim(),
        description: jobDescription.trim() || undefined,
        enabled,
        job_type: jobType,
        connection_id: connectionId,
        database: database.trim() || undefined,
        cron_expression: cronExpression,
        frequency_display: frequencyDisplay,
        query_config: jobType === 'query_export' ? {
          sql: querySql.trim(),
          format: exportFormat,
          target_dir: targetDir.trim(),
          filename_pattern: filenamePattern.trim() || '{name}_{timestamp}.csv'
        } : undefined,
        backup_config: jobType === 'database_backup' ? {
          tables: backupAllTables ? [] : backupTables,
          include_schema: includeSchema,
          include_data: includeData,
          target_dir: backupTargetDir.trim(),
          filename_pattern: backupFilenamePattern.trim() || 'backup_{db}_{timestamp}.sql'
        } : undefined,
        created_at: job?.created_at || Date.now(),
        updated_at: Date.now(),
        last_run_at: job?.last_run_at,
        last_run_status: job?.last_run_status,
        last_run_duration_ms: job?.last_run_duration_ms,
        last_run_error: job?.last_run_error,
        last_run_file: job?.last_run_file,
        last_run_rows: job?.last_run_rows,
        last_run_bytes: job?.last_run_bytes,
        next_run_at: job?.next_run_at
      };

      await schedulerStore.saveJob(updatedJob);
    } catch (err: any) {
      alert(`Failed to save automation: ${err?.message || err}`);
    } finally {
      isSaving = false;
    }
  }
</script>

{#if schedulerStore.isEditingJob && job}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-black/60 dark:bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <!-- Modal Window -->
    <div class="bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-2xl shadow-2xl w-full max-w-3xl max-h-[92vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-150">
      
      <!-- Top Header -->
      <div class="p-4 border-b border-slate-200 dark:border-zinc-800 flex items-center justify-between bg-surface-900/95 dark:bg-zinc-900/95 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-indigo-50 dark:bg-indigo-600/20 text-indigo-600 dark:text-indigo-400 rounded-xl border border-indigo-200 dark:border-indigo-500/30">
            <Clock class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-slate-900 dark:text-zinc-100">
              {job.created_at === job.updated_at ? 'Create Scheduled Automation' : 'Edit Automation'}
            </h3>
            <p class="text-[11px] text-slate-500 dark:text-zinc-400">Configure background timers, query exports, or automated database backups</p>
          </div>
        </div>

        <button
          type="button"
          onclick={() => schedulerStore.closeEditJob()}
          class="p-1.5 text-slate-400 hover:text-slate-700 dark:text-zinc-400 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-lg transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Scrollable Form Body -->
      <div class="flex-1 overflow-y-auto p-5 space-y-5 custom-scrollbar text-xs">
        
        <!-- 1. Job Type Selector Tabs -->
        <div class="space-y-1.5">
          <span class="font-semibold text-slate-700 dark:text-zinc-300">Automation Category</span>
          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              onclick={() => (jobType = 'query_export')}
              class="flex items-start gap-3 p-3 rounded-xl border text-left transition-all cursor-pointer {jobType === 'query_export' ? 'bg-indigo-50/80 dark:bg-indigo-600/15 border-indigo-500 text-indigo-900 dark:text-indigo-200 shadow-xs' : 'bg-surface-950 dark:bg-zinc-950 border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 hover:border-slate-300 dark:hover:border-zinc-700'}"
            >
              <div class="p-2 rounded-lg bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 mt-0.5">
                <FileSpreadsheet class="w-4 h-4" />
              </div>
              <div class="flex flex-col min-w-0">
                <span class="font-bold text-xs text-slate-900 dark:text-zinc-100">Scheduled Query Export</span>
                <span class="text-[11px] text-slate-500 dark:text-zinc-400 mt-0.5 leading-relaxed">Run a custom SQL query periodically and export result sets (CSV, JSON, Excel) to disk.</span>
              </div>
            </button>

            <button
              type="button"
              onclick={() => (jobType = 'database_backup')}
              class="flex items-start gap-3 p-3 rounded-xl border text-left transition-all cursor-pointer {jobType === 'database_backup' ? 'bg-emerald-50/80 dark:bg-emerald-600/15 border-emerald-500 text-emerald-900 dark:text-emerald-200 shadow-xs' : 'bg-surface-950 dark:bg-zinc-950 border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 hover:border-slate-300 dark:hover:border-zinc-700'}"
            >
              <div class="p-2 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 mt-0.5">
                <HardDrive class="w-4 h-4" />
              </div>
              <div class="flex flex-col min-w-0">
                <span class="font-bold text-xs text-slate-900 dark:text-zinc-100">Automated Database Backup</span>
                <span class="text-[11px] text-slate-500 dark:text-zinc-400 mt-0.5 leading-relaxed">Generate complete SQL schema DDL and table data dumps directly to a backup directory.</span>
              </div>
            </button>
          </div>
        </div>

        <!-- 2. General Information -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <label for="sched-job-name" class="font-semibold text-slate-700 dark:text-zinc-300">Job Name *</label>
            <input
              id="sched-job-name"
              type="text"
              bind:value={jobName}
              placeholder="e.g. Daily Revenue CSV Export or Weekly DB Backup"
              class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-xs text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 shadow-xs"
            />
          </div>

          <div class="space-y-1.5">
            <label for="sched-job-desc" class="font-semibold text-slate-700 dark:text-zinc-300">Description (Optional)</label>
            <input
              id="sched-job-desc"
              type="text"
              bind:value={jobDescription}
              placeholder="e.g. Exported for accounting team every morning"
              class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-xs text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 shadow-xs"
            />
          </div>
        </div>

        <!-- 3. Target Database Connection -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <span class="font-semibold text-slate-700 dark:text-zinc-300">Target Connection</span>
            <CustomSelect
              value={connectionId}
              options={connectionOptions}
              onchange={(val) => (connectionId = val)}
            />
          </div>

          <div class="space-y-1.5">
            <label for="sched-job-db" class="font-semibold text-slate-700 dark:text-zinc-300">Database Name</label>
            <input
              id="sched-job-db"
              type="text"
              bind:value={database}
              placeholder="e.g. postgres or ecommerce"
              class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-xs text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 shadow-xs"
            />
          </div>
        </div>

        <!-- 4. Schedule Frequency Presets -->
        <div class="p-4 bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-xl space-y-3 shadow-xs">
          <div class="flex items-center justify-between">
            <span class="font-bold text-xs text-slate-900 dark:text-zinc-100 flex items-center gap-1.5">
              <Clock class="w-3.5 h-3.5 text-amber-500" />
              Execution Schedule Frequency
            </span>
            <span class="text-[11px] font-mono text-indigo-600 dark:text-indigo-400 font-semibold">
              {frequencyDisplay}
            </span>
          </div>

          <!-- Preset Buttons Grid -->
          <div class="flex flex-wrap gap-1.5">
            {#each [
              { id: 'interval:15m', label: 'Every 15m' },
              { id: 'interval:30m', label: 'Every 30m' },
              { id: 'interval:1h', label: 'Every 1 Hour' },
              { id: 'interval:6h', label: 'Every 6 Hours' },
              { id: 'interval:12h', label: 'Every 12 Hours' },
              { id: 'daily', label: 'Daily at Time' },
              { id: 'weekly', label: 'Weekly on Day' },
              { id: 'custom', label: 'Custom Cron' }
            ] as preset}
              <button
                type="button"
                onclick={() => handleSchedulePresetChange(preset.id)}
                class="px-2.5 py-1 text-xs rounded-lg font-medium transition-colors cursor-pointer {schedulePreset === preset.id ? 'bg-indigo-600 text-white shadow-xs' : 'bg-surface-900 dark:bg-zinc-900 text-slate-700 dark:text-zinc-300 hover:bg-surface-800 dark:hover:bg-zinc-800 border border-slate-200 dark:border-zinc-800'}"
              >
                {preset.label}
              </button>
            {/each}
          </div>

          <!-- Specific Timing Details -->
          {#if schedulePreset === 'daily'}
            <div class="flex items-center gap-3 pt-1 animate-in fade-in duration-100">
              <span class="text-slate-600 dark:text-zinc-400 font-medium">Run every day at:</span>
              <input
                type="time"
                bind:value={dailyTime}
                oninput={updateDailyCron}
                class="bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-lg px-2.5 py-1 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500"
              />
              <span class="text-[11px] text-slate-400 dark:text-zinc-500">(Local system time)</span>
            </div>
          {:else if schedulePreset === 'weekly'}
            <div class="flex items-center gap-3 pt-1 flex-wrap animate-in fade-in duration-100">
              <span class="text-slate-600 dark:text-zinc-400 font-medium">Run every week on:</span>
              <select
                bind:value={weeklyDay}
                onchange={updateWeeklyCron}
                class="bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-lg px-2.5 py-1 text-xs font-medium text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500"
              >
                <option value="mon">Monday</option>
                <option value="tue">Tuesday</option>
                <option value="wed">Wednesday</option>
                <option value="thu">Thursday</option>
                <option value="fri">Friday</option>
                <option value="sat">Saturday</option>
                <option value="sun">Sunday</option>
              </select>
              <span class="text-slate-600 dark:text-zinc-400 font-medium">at:</span>
              <input
                type="time"
                bind:value={weeklyTime}
                oninput={updateWeeklyCron}
                class="bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-lg px-2.5 py-1 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500"
              />
            </div>
          {:else if schedulePreset === 'custom'}
            <div class="space-y-1.5 pt-1 animate-in fade-in duration-100">
              <label for="sched-cron-input" class="text-slate-600 dark:text-zinc-400 font-medium">5-Field Cron Expression (Min Hour Dom Month Dow):</label>
              <input
                id="sched-cron-input"
                type="text"
                bind:value={customCron}
                oninput={updateCustomCron}
                placeholder="0 2 * * * (At 02:00 every day)"
                class="w-full bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-1.5 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500"
              />
            </div>
          {/if}
        </div>

        <!-- 5. TYPE SPECIFIC CONFIGURATION -->
        {#if jobType === 'query_export'}
          <!-- Query SQL & Export Format -->
          <div class="space-y-3 border-t border-slate-200 dark:border-zinc-800 pt-3">
            <div class="space-y-1.5">
              <label for="sched-sql-query" class="font-semibold text-slate-700 dark:text-zinc-300 flex items-center justify-between">
                <span>SQL Query to Execute *</span>
                <span class="text-[10px] text-slate-400 font-mono">SELECT statements recommended</span>
              </label>
              <textarea
                id="sched-sql-query"
                bind:value={querySql}
                placeholder="SELECT id, name, email, created_at FROM users WHERE created_at >= NOW() - INTERVAL '1 day';"
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg p-3 text-xs font-mono text-slate-900 dark:text-zinc-100 resize-none h-28 focus:outline-none focus:border-indigo-500 shadow-inner leading-relaxed"
                spellcheck="false"
              ></textarea>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <span class="font-semibold text-slate-700 dark:text-zinc-300">Export File Format</span>
                <CustomSelect
                  value={exportFormat}
                  options={formatOptions}
                  onchange={(val) => (exportFormat = val)}
                />
              </div>

              <div class="space-y-1.5">
                <label for="sched-filename-pattern" class="font-semibold text-slate-700 dark:text-zinc-300">Filename Pattern</label>
                <input
                  id="sched-filename-pattern"
                  type="text"
                  bind:value={filenamePattern}
                  placeholder="{`{name}_{timestamp}.${exportFormat}`}"
                  class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 shadow-xs"
                />
              </div>
            </div>

            <!-- Destination Directory Picker -->
            <div class="space-y-1.5">
              <label for="sched-target-dir" class="font-semibold text-slate-700 dark:text-zinc-300">Output Folder Path *</label>
              <div class="flex items-center gap-2">
                <input
                  id="sched-target-dir"
                  type="text"
                  bind:value={targetDir}
                  placeholder="/Users/username/Downloads/Reports"
                  class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
                />
                <button
                  type="button"
                  onclick={() => handleBrowseFolder('query')}
                  class="flex items-center gap-1.5 px-3 py-2 text-xs bg-surface-800 hover:bg-surface-700 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-200 rounded-lg border border-slate-300 dark:border-zinc-700 transition-colors cursor-pointer shrink-0 font-medium shadow-xs"
                >
                  <Folder class="w-3.5 h-3.5 text-indigo-500" />
                  <span>Browse...</span>
                </button>
              </div>
            </div>
          </div>
        {:else}
          <!-- Database Backup Config -->
          <div class="space-y-4 border-t border-slate-200 dark:border-zinc-800 pt-3">
            <!-- Schema / Data Options -->
            <div class="p-3 bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-xl flex items-center justify-around">
              <label class="flex items-center gap-2 text-slate-800 dark:text-zinc-200 font-semibold cursor-pointer">
                <input type="checkbox" bind:checked={includeSchema} class="rounded text-emerald-600 focus:ring-emerald-500" />
                <span>Include Table Schemas (DDL)</span>
              </label>

              <div class="h-4 w-[1px] bg-slate-200 dark:bg-zinc-800"></div>

              <label class="flex items-center gap-2 text-slate-800 dark:text-zinc-200 font-semibold cursor-pointer">
                <input type="checkbox" bind:checked={includeData} class="rounded text-emerald-600 focus:ring-emerald-500" />
                <span>Include Table Records (INSERTs)</span>
              </label>
            </div>

            <!-- Table Selection -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <span class="font-semibold text-slate-700 dark:text-zinc-300">Tables to Back Up</span>
                <label class="flex items-center gap-1.5 text-xs text-indigo-600 dark:text-indigo-400 font-semibold cursor-pointer">
                  <input type="checkbox" bind:checked={backupAllTables} class="rounded text-indigo-600" />
                  <span>Back up all tables ({availableTables.length || 'All'})</span>
                </label>
              </div>

              {#if !backupAllTables}
                <div class="p-3 bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-xl space-y-2">
                  <input
                    type="text"
                    bind:value={tableSearch}
                    placeholder="Search tables..."
                    class="w-full bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded px-2.5 py-1 text-xs text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500"
                  />
                  <div class="max-h-36 overflow-y-auto space-y-1 custom-scrollbar pr-1">
                    {#each filteredAvailableTables as t (t)}
                      <label class="flex items-center gap-2 px-2 py-1 rounded hover:bg-surface-800 dark:hover:bg-zinc-800 cursor-pointer text-slate-800 dark:text-zinc-200 font-mono text-[11px]">
                        <input
                          type="checkbox"
                          checked={backupTables.includes(t)}
                          onchange={() => toggleTableSelection(t)}
                          class="rounded text-indigo-600"
                        />
                        <TableIcon class="w-3 h-3 text-slate-400" />
                        <span>{t}</span>
                      </label>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>

            <!-- Backup Destination Folder -->
            <div class="space-y-1.5">
              <label for="sched-backup-target-dir" class="font-semibold text-slate-700 dark:text-zinc-300">Backup Directory *</label>
              <div class="flex items-center gap-2">
                <input
                  id="sched-backup-target-dir"
                  type="text"
                  bind:value={backupTargetDir}
                  placeholder="/Users/username/Backups"
                  class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-emerald-500 shadow-xs"
                />
                <button
                  type="button"
                  onclick={() => handleBrowseFolder('backup')}
                  class="flex items-center gap-1.5 px-3 py-2 text-xs bg-surface-800 hover:bg-surface-700 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-200 rounded-lg border border-slate-300 dark:border-zinc-700 transition-colors cursor-pointer shrink-0 font-medium shadow-xs"
                >
                  <Folder class="w-3.5 h-3.5 text-emerald-500" />
                  <span>Browse...</span>
                </button>
              </div>
            </div>

            <div class="space-y-1.5">
              <label for="sched-backup-filename" class="font-semibold text-slate-700 dark:text-zinc-300">Backup Filename Pattern</label>
              <input
                id="sched-backup-filename"
                type="text"
                bind:value={backupFilenamePattern}
                placeholder="backup_{'{db}'}_{'{timestamp}'}.sql"
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-emerald-500 shadow-xs"
              />
            </div>
          </div>
        {/if}

        <!-- Active Toggle -->
        <div class="p-3 bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-xl flex items-center justify-between shadow-xs">
          <div class="flex flex-col">
            <span class="font-semibold text-slate-800 dark:text-zinc-200">Enable Schedule Immediately</span>
            <span class="text-[11px] text-slate-500 dark:text-zinc-400">If enabled, FugDB will automatically run this job based on the configured timer.</span>
          </div>
          <button
            type="button"
            onclick={() => (enabled = !enabled)}
            class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {enabled ? 'bg-emerald-600' : 'bg-slate-300 dark:bg-zinc-700'}"
            role="switch"
            aria-checked={enabled}
            aria-label="Toggle schedule enabled"
          >
            <span class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out {enabled ? 'translate-x-4' : 'translate-x-0'}"></span>
          </button>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="p-4 border-t border-slate-200 dark:border-zinc-800 bg-surface-900/95 dark:bg-zinc-900/95 flex items-center justify-between shrink-0">
        <span class="text-[11px] text-slate-400 dark:text-zinc-500 font-mono">Tokens: {'{name}, {db}, {timestamp}, {date}'}</span>
        
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => schedulerStore.closeEditJob()}
            class="px-4 py-1.5 text-xs text-slate-600 dark:text-zinc-400 hover:text-slate-900 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-xl transition-colors cursor-pointer font-medium"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={handleSave}
            disabled={isSaving || !jobName.trim()}
            class="flex items-center gap-1.5 px-5 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 disabled:hover:bg-indigo-600 text-white font-semibold rounded-xl shadow-xs transition-colors cursor-pointer"
          >
            <Save class="w-3.5 h-3.5" />
            <span>{isSaving ? 'Saving Automation...' : 'Save Automation'}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
