import { api } from '../api/client';
import { connectionStore } from './connection.svelte';

export interface SessionTransactionState {
  autoCommit: boolean;
  inTransaction: boolean;
  uncommittedCount: number;
  startedAt?: number;
}

const STORAGE_KEY = 'fugdb_autocommit_prefs';

const DEFAULT_STATE: Readonly<SessionTransactionState> = {
  autoCommit: true,
  inTransaction: false,
  uncommittedCount: 0,
};

export class SessionStore {
  // Map of connectionId -> Transaction State
  states = $state<Record<string, SessionTransactionState>>({});
  isOperating = $state<boolean>(false);
  notification = $state<{ type: 'success' | 'error' | 'info'; message: string; timestamp: number } | null>(null);

  constructor() {
    this.loadPrefs();
  }

  private loadPrefs() {
    if (typeof window === 'undefined') return;
    try {
      const data = localStorage.getItem(STORAGE_KEY);
      if (data) {
        const parsed = JSON.parse(data);
        if (typeof parsed === 'object' && parsed !== null) {
          for (const [connId, autoCommit] of Object.entries(parsed)) {
            this.ensureState(connId).autoCommit = Boolean(autoCommit);
          }
        }
      }
    } catch {}
  }

  private savePrefs() {
    if (typeof window === 'undefined') return;
    try {
      const prefs: Record<string, boolean> = {};
      for (const [connId, s] of Object.entries(this.states)) {
        prefs[connId] = s.autoCommit;
      }
      localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
    } catch {}
  }

  private ensureState(connectionId?: string): SessionTransactionState {
    const id = connectionId || connectionStore.activeConnectionId || 'default';
    if (!this.states[id]) {
      this.states[id] = {
        autoCommit: true,
        inTransaction: false,
        uncommittedCount: 0
      };
    }
    return this.states[id];
  }

  getState(connectionId?: string): SessionTransactionState {
    const id = connectionId || connectionStore.activeConnectionId || 'default';
    return this.states[id] ?? DEFAULT_STATE;
  }

  isAutoCommit(connectionId?: string): boolean {
    return this.getState(connectionId).autoCommit;
  }

  isInTransaction(connectionId?: string): boolean {
    return this.getState(connectionId).inTransaction;
  }

  getUncommittedCount(connectionId?: string): number {
    return this.getState(connectionId).uncommittedCount;
  }

  toggleAutoCommit(connectionId?: string) {
    const state = this.ensureState(connectionId);
    state.autoCommit = !state.autoCommit;
    this.savePrefs();

    this.showNotification(
      'info',
      state.autoCommit 
        ? '⚡ Auto-commit enabled (Changes commit immediately)' 
        : '🔒 Manual Transaction enabled (Explicit COMMIT / ROLLBACK required)'
    );
  }

  async begin(connectionId?: string, driver?: string) {
    const id = connectionId || connectionStore.activeConnectionId;
    if (!id) return;
    const d = driver || connectionStore.activeConnection?.driver;

    this.isOperating = true;
    try {
      await api.beginTransaction(id, d);
      const state = this.ensureState(id);
      state.inTransaction = true;
      state.uncommittedCount = 0;
      state.startedAt = Date.now();
      this.showNotification('info', '🔒 Transaction started (BEGIN). Execute queries and commit when ready.');
    } catch (err: any) {
      console.error('Failed to begin transaction:', err);
      this.showNotification('error', `Failed to start transaction: ${err?.toString() || err}`);
    } finally {
      this.isOperating = false;
    }
  }

  async commit(connectionId?: string, driver?: string) {
    const id = connectionId || connectionStore.activeConnectionId;
    if (!id) return;
    const d = driver || connectionStore.activeConnection?.driver;

    this.isOperating = true;
    try {
      await api.commitTransaction(id, d);
      const state = this.ensureState(id);
      const count = state.uncommittedCount;
      state.inTransaction = false;
      state.uncommittedCount = 0;
      state.startedAt = undefined;
      this.showNotification('success', `✓ Transaction committed successfully (${count} statement${count === 1 ? '' : 's'}).`);
    } catch (err: any) {
      console.error('Failed to commit transaction:', err);
      this.showNotification('error', `Failed to commit transaction: ${err?.toString() || err}`);
    } finally {
      this.isOperating = false;
    }
  }

  async rollback(connectionId?: string, driver?: string) {
    const id = connectionId || connectionStore.activeConnectionId;
    if (!id) return;
    const d = driver || connectionStore.activeConnection?.driver;

    this.isOperating = true;
    try {
      await api.rollbackTransaction(id, d);
      const state = this.ensureState(id);
      const count = state.uncommittedCount;
      state.inTransaction = false;
      state.uncommittedCount = 0;
      state.startedAt = undefined;
      this.showNotification('info', `↩ Transaction rolled back (${count} statement${count === 1 ? '' : 's'} reverted).`);
    } catch (err: any) {
      console.error('Failed to rollback transaction:', err);
      this.showNotification('error', `Failed to rollback transaction: ${err?.toString() || err}`);
    } finally {
      this.isOperating = false;
    }
  }

  trackQuery(connectionId: string, sql: string) {
    const state = this.ensureState(connectionId);
    const cleaned = sql.trim().toUpperCase();

    // Check explicit transaction SQL commands
    if (cleaned.startsWith('BEGIN') || cleaned.startsWith('START TRANSACTION')) {
      state.inTransaction = true;
      state.uncommittedCount = 0;
      state.startedAt = Date.now();
      return;
    }

    if (cleaned.startsWith('COMMIT')) {
      state.inTransaction = false;
      state.uncommittedCount = 0;
      state.startedAt = undefined;
      return;
    }

    if (cleaned.startsWith('ROLLBACK')) {
      state.inTransaction = false;
      state.uncommittedCount = 0;
      state.startedAt = undefined;
      return;
    }

    // In manual mode, track state modifications
    if (!state.autoCommit) {
      state.inTransaction = true;
      if (!state.startedAt) state.startedAt = Date.now();
      const isMutation = /^(INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|REPLACE)\b/i.test(sql.trim());
      if (isMutation) {
        state.uncommittedCount += 1;
      }
    }
  }

  private showNotification(type: 'success' | 'error' | 'info', message: string) {
    this.notification = { type, message, timestamp: Date.now() };
    setTimeout(() => {
      if (this.notification && Date.now() - this.notification.timestamp >= 3500) {
        this.notification = null;
      }
    }, 4000);
  }
}

export const sessionStore = new SessionStore();
