import { api } from '../api/client';
import type { ScheduleJob, ScheduleLog, ScheduleJobType } from '../api/types';
import { isTauri } from '../utils/environment';

class SchedulerStore {
  jobs = $state<ScheduleJob[]>([]);
  logs = $state<ScheduleLog[]>([]);
  isLoading = $state(false);
  isModalOpen = $state(false);
  activeTab = $state<'dashboard' | 'logs'>('dashboard');
  searchQuery = $state('');
  filterType = $state<'all' | 'query_export' | 'database_backup' | 'active' | 'paused'>('all');
  runningJobIds = $state<Record<string, boolean>>({});
  
  // Editor drawer/modal state
  isEditingJob = $state(false);
  editingJob = $state<ScheduleJob | null>(null);
  defaultBackupDir = $state('');

  private hasSetupListeners = false;

  filteredJobs = $derived.by(() => {
    let result = this.jobs;
    const q = this.searchQuery.trim().toLowerCase();
    if (q) {
      result = result.filter(j => 
        j.name.toLowerCase().includes(q) || 
        (j.description && j.description.toLowerCase().includes(q)) ||
        (j.database && j.database.toLowerCase().includes(q)) ||
        j.frequency_display.toLowerCase().includes(q)
      );
    }

    if (this.filterType === 'query_export') {
      result = result.filter(j => j.job_type === 'query_export');
    } else if (this.filterType === 'database_backup') {
      result = result.filter(j => j.job_type === 'database_backup');
    } else if (this.filterType === 'active') {
      result = result.filter(j => j.enabled);
    } else if (this.filterType === 'paused') {
      result = result.filter(j => !j.enabled);
    }

    return result;
  });

  stats = $derived.by(() => {
    const total = this.jobs.length;
    const active = this.jobs.filter(j => j.enabled).length;
    const queryExports = this.jobs.filter(j => j.job_type === 'query_export').length;
    const backups = this.jobs.filter(j => j.job_type === 'database_backup').length;
    const successRuns = this.logs.filter(l => l.status === 'success').length;
    const failedRuns = this.logs.filter(l => l.status === 'failed').length;

    return {
      total,
      active,
      paused: total - active,
      queryExports,
      backups,
      successRuns,
      failedRuns
    };
  });

  async init() {
    this.setupEventListeners();
    await this.loadAll();
  }

  async setupEventListeners() {
    if (this.hasSetupListeners || !isTauri) return;
    this.hasSetupListeners = true;

    try {
      const { listen } = await import('@tauri-apps/api/event');

      await listen<{ job_id: string; job_name: string }>('scheduler:job-started', (event) => {
        this.runningJobIds[event.payload.job_id] = true;
      });

      await listen<{ job_id: string; job_name: string; output_file: string }>('scheduler:job-completed', (event) => {
        delete this.runningJobIds[event.payload.job_id];
        this.loadAll();
      });

      await listen<{ job_id: string; job_name: string; error: string }>('scheduler:job-failed', (event) => {
        delete this.runningJobIds[event.payload.job_id];
        this.loadAll();
      });
    } catch (e) {
      console.warn('Failed to attach scheduler event listeners:', e);
    }
  }

  async loadAll() {
    this.isLoading = true;
    try {
      const [jobsData, logsData, defaultDir] = await Promise.all([
        api.listSchedules(),
        api.getScheduleLogs(),
        api.getDefaultBackupDirectory().catch(() => '')
      ]);
      this.jobs = jobsData;
      this.logs = logsData;
      this.defaultBackupDir = defaultDir;
    } catch (err) {
      console.error('Failed to load scheduler data:', err);
    } finally {
      this.isLoading = false;
    }
  }

  async loadLogs(jobId?: string) {
    try {
      this.logs = await api.getScheduleLogs(jobId);
    } catch (err) {
      console.error('Failed to load logs:', err);
    }
  }

  get isOpen() {
    return this.isModalOpen;
  }

  get isEditorOpen() {
    return this.isEditingJob;
  }

  openModal(tab: 'dashboard' | 'logs' = 'dashboard') {
    this.activeTab = tab;
    this.isModalOpen = true;
    this.loadAll();
  }

  closeModal() {
    this.isModalOpen = false;
    this.isEditingJob = false;
    this.editingJob = null;
  }

  toggle(tab: 'dashboard' | 'logs' = 'dashboard') {
    if (this.isModalOpen) {
      this.closeModal();
    } else {
      this.openModal(tab);
    }
  }

  openNewJob(initialType: ScheduleJobType = 'query_export', connectionId: string = 'default', database?: string) {
    const defaultDir = this.defaultBackupDir || '/tmp';
    const newJob: ScheduleJob = {
      id: `job_${Date.now()}`,
      name: initialType === 'query_export' ? 'New Scheduled Query Export' : 'New Automated Database Backup',
      description: '',
      enabled: true,
      job_type: initialType,
      connection_id: connectionId,
      database: database || '',
      cron_expression: 'interval:1h',
      frequency_display: 'Every 1 hour',
      query_config: initialType === 'query_export' ? {
        sql: 'SELECT * FROM users LIMIT 500;',
        format: 'csv',
        target_dir: defaultDir,
        filename_pattern: '{name}_{timestamp}.csv'
      } : undefined,
      backup_config: initialType === 'database_backup' ? {
        tables: [],
        include_schema: true,
        include_data: true,
        target_dir: defaultDir,
        filename_pattern: 'backup_{db}_{timestamp}.sql'
      } : undefined,
      created_at: Date.now(),
      updated_at: Date.now()
    };

    this.editingJob = newJob;
    this.isEditingJob = true;
  }

  openEditJob(job: ScheduleJob) {
    this.editingJob = JSON.parse(JSON.stringify(job));
    this.isEditingJob = true;
  }

  closeEditJob() {
    this.isEditingJob = false;
    this.editingJob = null;
  }

  async saveJob(job: ScheduleJob): Promise<ScheduleJob> {
    try {
      const saved = await api.createOrUpdateSchedule(job);
      await this.loadAll();
      this.closeEditJob();
      return saved;
    } catch (err) {
      console.error('Failed to save schedule job:', err);
      throw err;
    }
  }

  async deleteJob(id: string) {
    try {
      await api.deleteSchedule(id);
      this.jobs = this.jobs.filter(j => j.id !== id);
      this.logs = this.logs.filter(l => l.job_id !== id);
    } catch (err) {
      console.error('Failed to delete schedule job:', err);
      throw err;
    }
  }

  async toggleEnabled(id: string, enabled: boolean) {
    try {
      const updated = await api.toggleScheduleEnabled(id, enabled);
      const idx = this.jobs.findIndex(j => j.id === id);
      if (idx >= 0) {
        this.jobs[idx] = updated;
      }
    } catch (err) {
      console.error('Failed to toggle schedule enabled status:', err);
      throw err;
    }
  }

  async runNow(id: string): Promise<ScheduleLog> {
    this.runningJobIds[id] = true;
    try {
      const log = await api.runScheduleNow(id);
      await this.loadAll();
      return log;
    } catch (err) {
      console.error('Failed to run schedule now:', err);
      throw err;
    } finally {
      delete this.runningJobIds[id];
    }
  }

  async openFolder(path: string) {
    try {
      await api.openScheduleOutputFolder(path);
    } catch (err) {
      console.error('Failed to open folder:', err);
    }
  }

  async clearLogs(jobId?: string) {
    try {
      await api.clearScheduleLogs(jobId);
      if (jobId) {
        this.logs = this.logs.filter(l => l.job_id !== jobId);
      } else {
        this.logs = [];
      }
    } catch (err) {
      console.error('Failed to clear logs:', err);
      throw err;
    }
  }
}

export const schedulerStore = new SchedulerStore();
