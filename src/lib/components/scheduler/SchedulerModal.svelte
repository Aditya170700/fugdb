<script lang="ts">
  import { onMount } from "svelte";
  import {
    CalendarClock,
    Clock,
    PlayCircle,
    RefreshCw,
    X,
    Plus,
    Search,
    Trash2,
    Edit2,
    Folder,
    FileSpreadsheet,
    HardDrive,
    CheckCircle2,
    XCircle,
    AlertTriangle,
    FileText,
    ExternalLink,
    Copy,
    Check,
    SlidersHorizontal,
    Database,
    Zap,
    Archive,
  } from "lucide-svelte";
  import { schedulerStore } from "$lib/state/scheduler.svelte";
  import { connectionStore } from "$lib/state/connection.svelte";
  import type { ScheduleJob, ScheduleLog } from "$lib/api/types";
  import ScheduleEditorModal from "./ScheduleEditorModal.svelte";

  const stats = $derived(schedulerStore.stats);
  const jobs = $derived(schedulerStore.filteredJobs);
  const logs = $derived(schedulerStore.logs);

  let copiedPath = $state<string | null>(null);
  let logSearch = $state("");
  let logFilterJobId = $state<string>("all");
  let logFilterStatus = $state<"all" | "success" | "failed">("all");

  const filteredLogs = $derived.by(() => {
    let list = logs;
    if (logFilterJobId !== "all") {
      list = list.filter((l) => l.job_id === logFilterJobId);
    }
    if (logFilterStatus !== "all") {
      list = list.filter((l) => l.status === logFilterStatus);
    }
    const q = logSearch.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (l) =>
          l.job_name.toLowerCase().includes(q) ||
          (l.file_path && l.file_path.toLowerCase().includes(q)) ||
          (l.error_message && l.error_message.toLowerCase().includes(q)),
      );
    }
    return list;
  });

  onMount(() => {
    schedulerStore.init();
  });

  function formatTimestamp(ts?: number): string {
    if (!ts) return "Never";
    const date = new Date(ts);
    return date.toLocaleString();
  }

  function formatDuration(ms?: number): string {
    if (ms === undefined || ms === null) return "—";
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }

  function formatFileSize(bytes?: number): string {
    if (!bytes) return "0 B";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function formatNextRunCountdown(ts?: number): string {
    if (!ts) return "Paused";
    const diffMs = ts - Date.now();
    if (diffMs <= 0) return "Due now";
    const mins = Math.floor(diffMs / 60000);
    if (mins < 60) return `in ${mins}m`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `in ${hrs}h ${mins % 60}m`;
    const days = Math.floor(hrs / 24);
    return `in ${days}d ${hrs % 24}h`;
  }

  function handleCopyPath(path: string) {
    navigator.clipboard.writeText(path);
    copiedPath = path;
    setTimeout(() => {
      copiedPath = null;
    }, 1500);
  }

  async function handleDeleteJob(job: ScheduleJob) {
    if (
      !confirm(
        `Are you sure you want to delete automation schedule:\n"${job.name}"?`,
      )
    ) {
      return;
    }
    try {
      await schedulerStore.deleteJob(job.id);
    } catch (err: any) {
      alert(`Failed to delete job: ${err?.message || err}`);
    }
  }

  async function handleRunNow(jobId: string) {
    try {
      await schedulerStore.runNow(jobId);
    } catch (err: any) {
      alert(`Manual execution failed: ${err?.message || err}`);
    }
  }

  async function handleClearLogs() {
    if (
      !confirm(
        "Are you sure you want to clear all scheduler execution history logs?",
      )
    ) {
      return;
    }
    try {
      await schedulerStore.clearLogs(
        logFilterJobId === "all" ? undefined : logFilterJobId,
      );
    } catch (err: any) {
      alert(`Failed to clear logs: ${err?.message || err}`);
    }
  }

  function getConnectionName(connId: string): string {
    return (
      connectionStore.connections.find((c) => c.id === connId)?.name ||
      "Default"
    );
  }
</script>

{#if schedulerStore.isModalOpen}
  <!-- Modal Backdrop -->
  <div
    class="fixed inset-0 z-50 bg-black/60 dark:bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <!-- Modal Window -->
    <div
      class="w-full max-w-6xl h-[90vh] bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-150"
    >
      <!-- 1. Top Header -->
      <div
        class="px-5 py-3.5 border-b border-slate-200 dark:border-zinc-800 bg-surface-900 dark:bg-zinc-900/90 flex items-center justify-between shrink-0"
      >
        <div class="flex items-center gap-3">
          <div
            class="p-2 rounded-xl bg-indigo-50 dark:bg-indigo-600/20 text-indigo-600 dark:text-indigo-400 border border-indigo-200 dark:border-indigo-500/30"
          >
            <CalendarClock class="w-5 h-5" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-base font-bold text-slate-900 dark:text-zinc-100">
                Scheduled Query Automations & Local Backups
              </h2>
              <span
                class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 border border-indigo-500/20"
              >
                {stats.active} Active / {stats.total} Total
              </span>
            </div>
            <p class="text-xs text-slate-500 dark:text-zinc-400 mt-0.5">
              Automate recurring SQL queries, data export tasks, and full
              database backups locally.
            </p>
          </div>
        </div>

        <!-- Header Actions -->
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => schedulerStore.openNewJob("query_export")}
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-xl shadow-xs transition-colors cursor-pointer"
          >
            <Plus class="w-3.5 h-3.5" />
            <span>Query Export</span>
          </button>

          <button
            type="button"
            onclick={() => schedulerStore.openNewJob("database_backup")}
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-emerald-600 hover:bg-emerald-500 text-white font-semibold rounded-xl shadow-xs transition-colors cursor-pointer"
          >
            <Plus class="w-3.5 h-3.5" />
            <span>DB Backup</span>
          </button>

          <button
            type="button"
            onclick={() => schedulerStore.loadAll()}
            class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-zinc-400 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-lg transition-colors cursor-pointer"
            title="Refresh Schedules & Logs"
          >
            <RefreshCw
              class="w-4 h-4 {schedulerStore.isLoading
                ? 'animate-spin text-indigo-600 dark:text-indigo-400'
                : ''}"
            />
          </button>

          <button
            type="button"
            onclick={() => schedulerStore.closeModal()}
            class="p-1.5 text-slate-400 hover:text-slate-700 dark:text-zinc-400 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-lg transition-colors cursor-pointer ml-1"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- 2. Stats Bar Cards -->
      <div
        class="grid grid-cols-2 md:grid-cols-4 gap-3 p-4 bg-surface-900/60 dark:bg-zinc-900/40 border-b border-slate-200 dark:border-zinc-800 shrink-0"
      >
        <!-- Stat 1: Total Jobs -->
        <div
          class="p-3 bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex items-center justify-between shadow-xs"
        >
          <div class="flex flex-col">
            <span
              class="text-[11px] font-medium text-slate-500 dark:text-zinc-400"
              >Total Automations</span
            >
            <span class="text-lg font-bold text-slate-900 dark:text-zinc-100"
              >{stats.total}</span
            >
          </div>
          <div
            class="p-2 rounded-lg bg-indigo-500/10 text-indigo-600 dark:text-indigo-400"
          >
            <CalendarClock class="w-4 h-4" />
          </div>
        </div>

        <!-- Stat 2: Active Runners -->
        <div
          class="p-3 bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex items-center justify-between shadow-xs"
        >
          <div class="flex flex-col">
            <span
              class="text-[11px] font-medium text-slate-500 dark:text-zinc-400"
              >Active Timers</span
            >
            <span
              class="text-lg font-bold text-emerald-600 dark:text-emerald-400"
              >{stats.active}</span
            >
          </div>
          <div
            class="p-2 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
          >
            <Zap class="w-4 h-4" />
          </div>
        </div>

        <!-- Stat 3: Query Exports -->
        <div
          class="p-3 bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex items-center justify-between shadow-xs"
        >
          <div class="flex flex-col">
            <span
              class="text-[11px] font-medium text-slate-500 dark:text-zinc-400"
              >Query Exports</span
            >
            <span class="text-lg font-bold text-sky-600 dark:text-cyan-400"
              >{stats.queryExports}</span
            >
          </div>
          <div
            class="p-2 rounded-lg bg-sky-500/10 text-sky-600 dark:text-cyan-400"
          >
            <FileSpreadsheet class="w-4 h-4" />
          </div>
        </div>

        <!-- Stat 4: Database Backups -->
        <div
          class="p-3 bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-xl flex items-center justify-between shadow-xs"
        >
          <div class="flex flex-col">
            <span
              class="text-[11px] font-medium text-slate-500 dark:text-zinc-400"
              >SQL Backups</span
            >
            <span class="text-lg font-bold text-purple-600 dark:text-purple-400"
              >{stats.backups}</span
            >
          </div>
          <div
            class="p-2 rounded-lg bg-purple-500/10 text-purple-600 dark:text-purple-400"
          >
            <HardDrive class="w-4 h-4" />
          </div>
        </div>
      </div>

      <!-- 3. Navigation Sub-Tabs & Filters -->
      <div
        class="px-4 py-2.5 bg-surface-900/90 dark:bg-zinc-900/70 border-b border-slate-200 dark:border-zinc-800 flex items-center justify-between gap-3 shrink-0 flex-wrap"
      >
        <!-- Sub-Tabs Switcher -->
        <div
          class="flex items-center gap-1 bg-surface-950 dark:bg-zinc-950 p-0.5 rounded-xl border border-slate-200 dark:border-zinc-800 shadow-xs"
        >
          <button
            type="button"
            onclick={() => (schedulerStore.activeTab = "dashboard")}
            class="flex items-center gap-1.5 px-3 py-1 text-xs rounded-lg font-semibold transition-all cursor-pointer {schedulerStore.activeTab ===
            'dashboard'
              ? 'bg-surface-800 dark:bg-zinc-800 text-indigo-700 dark:text-indigo-300 shadow-xs'
              : 'text-slate-600 dark:text-zinc-400 hover:text-slate-900 dark:hover:text-zinc-200'}"
          >
            <CalendarClock class="w-3.5 h-3.5" />
            <span>Automations & Backups ({jobs.length})</span>
          </button>

          <button
            type="button"
            onclick={() => (schedulerStore.activeTab = "logs")}
            class="flex items-center gap-1.5 px-3 py-1 text-xs rounded-lg font-semibold transition-all cursor-pointer {schedulerStore.activeTab ===
            'logs'
              ? 'bg-surface-800 dark:bg-zinc-800 text-indigo-700 dark:text-indigo-300 shadow-xs'
              : 'text-slate-600 dark:text-zinc-400 hover:text-slate-900 dark:hover:text-zinc-200'}"
          >
            <FileText class="w-3.5 h-3.5" />
            <span>Execution Logs ({logs.length})</span>
          </button>
        </div>

        <!-- Filter & Search Controls -->
        {#if schedulerStore.activeTab === "dashboard"}
          <div class="flex items-center gap-2 flex-1 max-w-lg justify-end">
            <!-- Filter Chips -->
            <div class="hidden sm:flex items-center gap-1">
              {#each [{ id: "all", label: "All" }, { id: "query_export", label: "Exports" }, { id: "database_backup", label: "Backups" }, { id: "active", label: "Active" }, { id: "paused", label: "Paused" }] as f}
                <button
                  type="button"
                  onclick={() => (schedulerStore.filterType = f.id as any)}
                  class="px-2 py-0.5 text-[11px] rounded-md font-medium transition-colors cursor-pointer {schedulerStore.filterType ===
                  f.id
                    ? 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-500/30'
                    : 'text-slate-500 dark:text-zinc-400 hover:bg-surface-800 dark:hover:bg-zinc-800'}"
                >
                  {f.label}
                </button>
              {/each}
            </div>

            <!-- Search Input -->
            <div class="relative w-48 sm:w-60">
              <Search
                class="w-3.5 h-3.5 absolute left-2.5 top-2 text-slate-400 dark:text-zinc-500 pointer-events-none"
              />
              <input
                type="text"
                bind:value={schedulerStore.searchQuery}
                placeholder="Search jobs..."
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg pl-8 pr-3 py-1 text-xs text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 shadow-xs"
              />
            </div>
          </div>
        {:else}
          <!-- Logs Filter Controls -->
          <div class="flex items-center gap-2 flex-1 max-w-lg justify-end">
            <select
              bind:value={logFilterJobId}
              class="bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-2.5 py-1 text-xs text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500"
            >
              <option value="all">All Automation Jobs</option>
              {#each schedulerStore.jobs as j (j.id)}
                <option value={j.id}>{j.name}</option>
              {/each}
            </select>

            <select
              bind:value={logFilterStatus}
              class="bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg px-2.5 py-1 text-xs text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500"
            >
              <option value="all">All Statuses</option>
              <option value="success">Success</option>
              <option value="failed">Failed</option>
            </select>

            <button
              type="button"
              onclick={handleClearLogs}
              class="flex items-center gap-1 px-2.5 py-1 text-xs text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-lg border border-rose-200 dark:border-rose-900/30 transition-colors cursor-pointer"
              title="Clear log history"
            >
              <Trash2 class="w-3 h-3" />
              <span>Clear</span>
            </button>
          </div>
        {/if}
      </div>

      <!-- 4. Main Body Content -->
      <div class="flex-1 overflow-y-auto p-4 custom-scrollbar">
        {#if schedulerStore.activeTab === "dashboard"}
          {#if jobs.length === 0}
            <!-- Empty State -->
            <div
              class="flex flex-col items-center justify-center py-20 px-4 text-center text-slate-500 dark:text-zinc-500 gap-3"
            >
              <div
                class="p-4 bg-surface-900 dark:bg-zinc-900 rounded-2xl border border-slate-200 dark:border-zinc-800 shadow-xs"
              >
                <CalendarClock class="w-10 h-10 text-indigo-500/80 stroke-1" />
              </div>
              <div>
                <p class="text-sm font-bold text-slate-800 dark:text-zinc-200">
                  No Scheduled Automations Found
                </p>
                <p
                  class="text-xs text-slate-500 dark:text-zinc-400 mt-1 max-w-md"
                >
                  Create your first automated query export or scheduled database
                  backup to run tasks periodically in the background.
                </p>
              </div>
              <div class="flex items-center gap-2 mt-2">
                <button
                  type="button"
                  onclick={() => schedulerStore.openNewJob("query_export")}
                  class="flex items-center gap-1.5 px-3.5 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-xl shadow-xs transition-colors cursor-pointer"
                >
                  <Plus class="w-3.5 h-3.5" />
                  <span>Create Query Export</span>
                </button>
                <button
                  type="button"
                  onclick={() => schedulerStore.openNewJob("database_backup")}
                  class="flex items-center gap-1.5 px-3.5 py-1.5 text-xs bg-surface-900 hover:bg-surface-800 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-200 font-semibold rounded-xl border border-slate-200 dark:border-zinc-700 transition-colors cursor-pointer shadow-xs"
                >
                  <Plus class="w-3.5 h-3.5" />
                  <span>Create Database Backup</span>
                </button>
              </div>
            </div>
          {:else}
            <!-- Jobs Cards Grid -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
              {#each jobs as job (job.id)}
                {@const isRunning = !!schedulerStore.runningJobIds[job.id]}

                <div
                  class="p-4 bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-2xl flex flex-col justify-between gap-3 shadow-xs hover:border-indigo-300 dark:hover:border-zinc-700 transition-all"
                >
                  <!-- Top Row: Icon, Title, Type Badge & Enabled Toggle -->
                  <div class="flex items-start justify-between gap-3">
                    <div class="flex items-start gap-3 min-w-0">
                      <div
                        class="p-2.5 rounded-xl shrink-0 mt-0.5 {job.job_type ===
                        'query_export'
                          ? 'bg-indigo-50 dark:bg-indigo-600/20 text-indigo-600 dark:text-indigo-400 border border-indigo-200 dark:border-indigo-500/30'
                          : 'bg-emerald-50 dark:bg-emerald-600/20 text-emerald-600 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-500/30'}"
                      >
                        {#if job.job_type === "query_export"}
                          <FileSpreadsheet class="w-4 h-4" />
                        {:else}
                          <HardDrive class="w-4 h-4" />
                        {/if}
                      </div>

                      <div class="flex flex-col min-w-0">
                        <div class="flex items-center gap-2 flex-wrap">
                          <h4
                            class="font-bold text-xs text-slate-900 dark:text-zinc-100 truncate"
                            title={job.name}
                          >
                            {job.name}
                          </h4>
                          <span
                            class="text-[9px] px-1.5 py-0.2 rounded font-mono uppercase font-bold {job.job_type ===
                            'query_export'
                              ? 'bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 border border-indigo-500/20'
                              : 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/20'}"
                          >
                            {job.job_type === "query_export"
                              ? "Export"
                              : "Backup"}
                          </span>
                        </div>
                        {#if job.description}
                          <p
                            class="text-[11px] text-slate-500 dark:text-zinc-400 truncate mt-0.5"
                          >
                            {job.description}
                          </p>
                        {/if}
                      </div>
                    </div>

                    <!-- Enabled Toggle Switch -->
                    <button
                      type="button"
                      onclick={() =>
                        schedulerStore.toggleEnabled(job.id, !job.enabled)}
                      class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {job.enabled
                        ? 'bg-emerald-600'
                        : 'bg-slate-300 dark:bg-zinc-700'}"
                      role="switch"
                      aria-checked={job.enabled}
                      title={job.enabled
                        ? "Automation is Active (Click to Pause)"
                        : "Automation is Paused (Click to Activate)"}
                    >
                      <span
                        class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-md ring-0 transition duration-200 ease-in-out {job.enabled
                          ? 'translate-x-4'
                          : 'translate-x-0'}"
                      ></span>
                    </button>
                  </div>

                  <!-- Details Badges (Connection, Database, Schedule, Next Run) -->
                  <div
                    class="grid grid-cols-2 gap-2 text-xs bg-surface-950 dark:bg-zinc-950 p-2.5 rounded-xl border border-slate-200/80 dark:border-zinc-800/80"
                  >
                    <div
                      class="flex items-center gap-1.5 text-slate-600 dark:text-zinc-400 truncate"
                    >
                      <Database class="w-3.5 h-3.5 text-slate-400 shrink-0" />
                      <span class="font-mono text-[11px] truncate"
                        >{getConnectionName(job.connection_id)} / {job.database ||
                          "default"}</span
                      >
                    </div>

                    <div
                      class="flex items-center gap-1.5 text-slate-600 dark:text-zinc-400 truncate"
                    >
                      <Clock class="w-3.5 h-3.5 text-amber-500 shrink-0" />
                      <span
                        class="font-mono text-[11px] truncate font-medium text-slate-800 dark:text-zinc-200"
                        >{job.frequency_display}</span
                      >
                    </div>

                    <div
                      class="flex items-center gap-1.5 text-slate-500 dark:text-zinc-400 col-span-2 pt-0.5 border-t border-slate-200 dark:border-zinc-800/60"
                    >
                      <span class="text-[10px]">Next Run:</span>
                      <span
                        class="font-mono text-[11px] font-semibold {job.enabled
                          ? 'text-indigo-600 dark:text-indigo-400'
                          : 'text-slate-400 dark:text-zinc-600'}"
                      >
                        {job.enabled
                          ? formatNextRunCountdown(job.next_run_at)
                          : "Paused"}
                      </span>
                      {#if job.last_run_at}
                        <span
                          class="text-[10px] text-slate-400 ml-auto font-mono"
                        >
                          Last: {formatTimestamp(job.last_run_at)} ({job.last_run_status ||
                            "ok"})
                        </span>
                      {/if}
                    </div>
                  </div>

                  <!-- Last Run Status Bar & Action Buttons -->
                  <div
                    class="flex items-center justify-between gap-2 pt-1 border-t border-slate-200 dark:border-zinc-800"
                  >
                    <!-- Last Run Details / File Size -->
                    <div class="flex items-center gap-1.5 text-[11px] min-w-0">
                      {#if job.last_run_status === "success"}
                        <span
                          class="flex items-center gap-1 text-emerald-600 dark:text-emerald-400 font-semibold shrink-0"
                        >
                          <CheckCircle2 class="w-3.5 h-3.5" />
                          <span>{job.last_run_rows || 0} rows</span>
                        </span>
                        {#if job.last_run_bytes}
                          <span
                            class="text-slate-400 dark:text-zinc-600 font-mono"
                            >• {formatFileSize(job.last_run_bytes)}</span
                          >
                        {/if}
                      {:else if job.last_run_status === "failed"}
                        <span
                          class="flex items-center gap-1 text-rose-600 dark:text-rose-400 font-semibold shrink-0"
                          title={job.last_run_error}
                        >
                          <XCircle class="w-3.5 h-3.5" />
                          <span class="truncate max-w-[120px]">Failed</span>
                        </span>
                      {:else}
                        <span
                          class="text-slate-400 dark:text-zinc-600 text-[11px]"
                          >Ready to run</span
                        >
                      {/if}
                    </div>

                    <!-- Action Buttons -->
                    <div class="flex items-center gap-1 shrink-0">
                      <!-- Run Now -->
                      <button
                        type="button"
                        onclick={() => handleRunNow(job.id)}
                        disabled={isRunning}
                        class="flex items-center gap-1 px-2.5 py-1 text-xs bg-indigo-50 hover:bg-indigo-100 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-indigo-700 dark:text-indigo-300 rounded-lg border border-indigo-200 dark:border-zinc-700 transition-colors cursor-pointer font-medium disabled:opacity-40 shadow-xs"
                        title="Execute this job immediately"
                      >
                        <PlayCircle
                          class="w-3.5 h-3.5 {isRunning ? 'animate-spin' : ''}"
                        />
                        <span>{isRunning ? "Running..." : "Run Now"}</span>
                      </button>

                      <!-- Open Output Folder -->
                      {#if job.last_run_file || job.query_config?.target_dir || job.backup_config?.target_dir}
                        <button
                          type="button"
                          onclick={() =>
                            schedulerStore.openFolder(
                              job.last_run_file ||
                                job.query_config?.target_dir ||
                                job.backup_config?.target_dir ||
                                "",
                            )}
                          class="p-1 text-slate-500 hover:text-slate-800 dark:text-zinc-400 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-lg transition-colors cursor-pointer"
                          title="Open Output Folder in Finder / Explorer"
                        >
                          <Folder class="w-3.5 h-3.5" />
                        </button>
                      {/if}

                      <!-- Edit -->
                      <button
                        type="button"
                        onclick={() => schedulerStore.openEditJob(job)}
                        class="p-1 text-slate-500 hover:text-slate-800 dark:text-zinc-400 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-lg transition-colors cursor-pointer"
                        title="Edit Schedule Configuration"
                      >
                        <Edit2 class="w-3.5 h-3.5" />
                      </button>

                      <!-- Delete -->
                      <button
                        type="button"
                        onclick={() => handleDeleteJob(job)}
                        class="p-1 text-rose-500 hover:text-rose-700 dark:text-rose-400 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-lg transition-colors cursor-pointer"
                        title="Delete Schedule"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        {:else}
          <!-- Execution Logs Tab Table -->
          <div class="space-y-3">
            <div
              class="border border-slate-200 dark:border-zinc-800 rounded-2xl overflow-hidden bg-surface-900 dark:bg-zinc-900 shadow-xs"
            >
              <table class="w-full text-left text-xs border-collapse">
                <thead>
                  <tr
                    class="bg-surface-900 dark:bg-zinc-900/80 border-b border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 font-mono text-[11px]"
                  >
                    <th class="p-3 font-semibold w-24">Status</th>
                    <th class="p-3 font-semibold">Job Name</th>
                    <th class="p-3 font-semibold">Started At</th>
                    <th class="p-3 font-semibold w-20">Duration</th>
                    <th class="p-3 font-semibold w-24">Rows / Size</th>
                    <th class="p-3 font-semibold">Output File</th>
                  </tr>
                </thead>
                <tbody
                  class="divide-y divide-slate-200 dark:divide-zinc-800/60 font-mono text-xs"
                >
                  {#if filteredLogs.length === 0}
                    <tr>
                      <td
                        colspan="6"
                        class="p-8 text-center text-slate-500 dark:text-zinc-500 font-sans"
                      >
                        No execution logs matching filter.
                      </td>
                    </tr>
                  {:else}
                    {#each filteredLogs as log (log.id)}
                      <tr
                        class="hover:bg-surface-800/50 dark:hover:bg-zinc-800/30 transition-colors"
                      >
                        <!-- Status Badge -->
                        <td class="p-3 align-top">
                          {#if log.status === "success"}
                            <span
                              class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30"
                            >
                              <CheckCircle2 class="w-3 h-3" />
                              <span>OK</span>
                            </span>
                          {:else if log.status === "failed"}
                            <span
                              class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-rose-500/15 text-rose-700 dark:text-rose-300 border border-rose-500/30"
                            >
                              <XCircle class="w-3 h-3" />
                              <span>FAIL</span>
                            </span>
                          {:else}
                            <span
                              class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30"
                            >
                              <Clock class="w-3 h-3 animate-spin" />
                              <span>RUN</span>
                            </span>
                          {/if}
                        </td>

                        <!-- Job Name & Error snippet -->
                        <td class="p-3 align-top">
                          <span
                            class="font-bold text-slate-900 dark:text-zinc-100"
                            >{log.job_name}</span
                          >
                          {#if log.error_message}
                            <div
                              class="mt-1 p-2 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900/40 rounded-lg text-rose-700 dark:text-rose-300 text-[11px] leading-relaxed break-all"
                            >
                              {log.error_message}
                            </div>
                          {/if}
                        </td>

                        <!-- Timestamp -->
                        <td
                          class="p-3 align-top text-slate-500 dark:text-zinc-400"
                        >
                          {formatTimestamp(log.started_at)}
                        </td>

                        <!-- Duration -->
                        <td
                          class="p-3 align-top text-slate-700 dark:text-zinc-300 font-semibold"
                        >
                          {formatDuration(log.duration_ms)}
                        </td>

                        <!-- Rows / Size -->
                        <td class="p-3 align-top">
                          <div class="flex flex-col">
                            <span class="text-slate-800 dark:text-zinc-200"
                              >{log.rows_processed !== undefined
                                ? `${log.rows_processed} rows`
                                : "—"}</span
                            >
                            <span
                              class="text-[10px] text-slate-400 dark:text-zinc-500"
                              >{formatFileSize(log.bytes_written)}</span
                            >
                          </div>
                        </td>

                        <!-- File Path -->
                        <td class="p-3 align-top">
                          {#if log.file_path}
                            <div class="flex items-center gap-1.5 max-w-sm">
                              <span
                                class="truncate text-slate-600 dark:text-zinc-400 text-[11px]"
                                title={log.file_path}
                              >
                                {log.file_path}
                              </span>
                              <button
                                type="button"
                                onclick={() =>
                                  schedulerStore.openFolder(log.file_path!)}
                                class="p-1 text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-400 rounded transition-colors cursor-pointer"
                                title="Open in Finder / File Explorer"
                              >
                                <Folder class="w-3 h-3" />
                              </button>
                              <button
                                type="button"
                                onclick={() => handleCopyPath(log.file_path!)}
                                class="p-1 text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-400 rounded transition-colors cursor-pointer"
                                title="Copy file path"
                              >
                                {#if copiedPath === log.file_path}
                                  <Check class="w-3 h-3 text-emerald-500" />
                                {:else}
                                  <Copy class="w-3 h-3" />
                                {/if}
                              </button>
                            </div>
                          {:else}
                            <span class="text-slate-400 dark:text-zinc-600"
                              >—</span
                            >
                          {/if}
                        </td>
                      </tr>
                    {/each}
                  {/if}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </div>

      <!-- 5. Bottom Status Bar -->
      <div
        class="px-5 py-2.5 bg-surface-900 dark:bg-zinc-900 border-t border-slate-200 dark:border-zinc-800 flex items-center justify-between text-xs text-slate-500 dark:text-zinc-400 font-mono shrink-0"
      >
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"
          ></span>
          <span>Background Automation Engine Active</span>
        </div>
        <button
          type="button"
          onclick={() => schedulerStore.closeModal()}
          class="px-3 py-1 bg-surface-800 hover:bg-surface-700 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-300 font-semibold rounded-lg border border-slate-300 dark:border-zinc-700 transition-colors cursor-pointer shadow-xs"
        >
          Close (Esc)
        </button>
      </div>
    </div>
  </div>

  <!-- Create / Edit Drawer Modal -->
  <ScheduleEditorModal />
{/if}
