import { api } from '../api/client';
import { connectionStore } from './connection.svelte';
import type { ServerProcess, ServerHealthStats } from '../api/types';

export type ProcessFilterStatus = 'all' | 'active' | 'blocked' | 'slow_3s' | 'idle';

export class MonitorStore {
  isOpen = $state<boolean>(false);
  isLoading = $state<boolean>(false);
  isOperating = $state<boolean>(false);
  stats = $state<ServerHealthStats | null>(null);
  selectedProcess = $state<ServerProcess | null>(null);
  killConfirmTarget = $state<ServerProcess | null>(null);

  filterText = $state<string>('');
  filterStatus = $state<ProcessFilterStatus>('all');
  autoRefreshInterval = $state<number>(3); // 0 = off, 2, 3, 5, 10 seconds
  lastUpdated = $state<number | null>(null);

  errorMessage = $state<string | null>(null);
  successMessage = $state<string | null>(null);

  private timer: any = null;

  open() {
    this.isOpen = true;
    this.errorMessage = null;
    this.successMessage = null;
    this.fetchProcesses();
    this.setupTimer();
  }

  close() {
    this.isOpen = false;
    this.selectedProcess = null;
    this.killConfirmTarget = null;
    this.clearTimer();
  }

  toggle() {
    if (this.isOpen) {
      this.close();
    } else {
      this.open();
    }
  }

  setAutoRefresh(intervalSec: number) {
    this.autoRefreshInterval = intervalSec;
    this.setupTimer();
  }

  private clearTimer() {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  private setupTimer() {
    this.clearTimer();
    if (this.autoRefreshInterval > 0 && this.isOpen) {
      this.timer = setInterval(() => {
        if (this.isOpen && !this.isOperating) {
          this.fetchProcesses(true);
        }
      }, this.autoRefreshInterval * 1000);
    }
  }

  async fetchProcesses(isBackground = false) {
    const connId = connectionStore.activeConnectionId;
    if (!connId) return;
    const driver = connectionStore.activeConnection?.driver;

    if (!isBackground) {
      this.isLoading = true;
    }

    try {
      const data = await api.getServerProcesses(connId, driver);
      this.stats = data;
      this.lastUpdated = Date.now();
      this.errorMessage = null;

      // Keep selected process in sync
      if (this.selectedProcess) {
        const updated = data.processes.find(p => p.pid === this.selectedProcess?.pid);
        if (updated) {
          this.selectedProcess = updated;
        }
      }
    } catch (err: any) {
      console.error('Failed to fetch server processes:', err);
      this.errorMessage = err?.toString() || 'Failed to fetch server processes';
    } finally {
      this.isLoading = false;
    }
  }

  async killProcess(process: ServerProcess) {
    const connId = connectionStore.activeConnectionId;
    if (!connId) return;
    const driver = connectionStore.activeConnection?.driver;

    this.isOperating = true;
    try {
      await api.killServerProcess(connId, process.pid, driver);
      this.showToast('success', `✓ Process PID ${process.pid} terminated successfully.`);
      this.killConfirmTarget = null;
      if (this.selectedProcess?.pid === process.pid) {
        this.selectedProcess = null;
      }
      await this.fetchProcesses();
    } catch (err: any) {
      console.error('Failed to kill process:', err);
      this.showToast('error', `Failed to terminate PID ${process.pid}: ${err?.toString() || err}`);
    } finally {
      this.isOperating = false;
    }
  }

  async cancelQuery(process: ServerProcess) {
    const connId = connectionStore.activeConnectionId;
    if (!connId) return;
    const driver = connectionStore.activeConnection?.driver;

    this.isOperating = true;
    try {
      await api.cancelServerQuery(connId, process.pid, driver);
      this.showToast('success', `✓ Query running on PID ${process.pid} cancelled.`);
      await this.fetchProcesses();
    } catch (err: any) {
      console.error('Failed to cancel query:', err);
      this.showToast('error', `Failed to cancel query on PID ${process.pid}: ${err?.toString() || err}`);
    } finally {
      this.isOperating = false;
    }
  }

  showToast(type: 'success' | 'error', message: string) {
    if (type === 'success') {
      this.successMessage = message;
      setTimeout(() => {
        if (this.successMessage === message) this.successMessage = null;
      }, 3500);
    } else {
      this.errorMessage = message;
      setTimeout(() => {
        if (this.errorMessage === message) this.errorMessage = null;
      }, 4500);
    }
  }

  get filteredProcesses(): ServerProcess[] {
    if (!this.stats) return [];
    let list = this.stats.processes;

    // Filter by status pill
    if (this.filterStatus === 'active') {
      list = list.filter(p => {
        const s = p.state.toLowerCase();
        return s.includes('active') || s.includes('run') || s.includes('exec') || s.includes('query');
      });
    } else if (this.filterStatus === 'blocked') {
      list = list.filter(p => Boolean(p.blockedBy) || p.state.toLowerCase().includes('wait') || (p.waitEvent && p.waitEvent.toLowerCase().includes('lock')));
    } else if (this.filterStatus === 'slow_3s') {
      list = list.filter(p => p.durationSeconds >= 3.0);
    } else if (this.filterStatus === 'idle') {
      list = list.filter(p => {
        const s = p.state.toLowerCase();
        return s.includes('idle') || s.includes('sleep');
      });
    }

    // Filter by text search (PID, user, db, query, client)
    if (this.filterText.trim()) {
      const q = this.filterText.toLowerCase().trim();
      list = list.filter(p => 
        p.pid.toLowerCase().includes(q) ||
        p.user.toLowerCase().includes(q) ||
        p.database.toLowerCase().includes(q) ||
        (p.query && p.query.toLowerCase().includes(q)) ||
        (p.clientAddr && p.clientAddr.toLowerCase().includes(q)) ||
        (p.applicationName && p.applicationName.toLowerCase().includes(q))
      );
    }

    return list;
  }
}

export const monitorStore = new MonitorStore();
