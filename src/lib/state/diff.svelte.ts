import { api } from '../api/client';
import { connectionStore } from './connection.svelte';
import type { SchemaDiffResult, TableDiff, DiffAction } from '../api/types';

export class DiffStore {
  isOpen = $state<boolean>(false);
  sourceConnectionId = $state<string>('');
  targetConnectionId = $state<string>('');
  isComparing = $state<boolean>(false);
  isApplying = $state<boolean>(false);
  result = $state<SchemaDiffResult | null>(null);
  selectedTableDiff = $state<TableDiff | null>(null);

  activeTab = $state<'visual' | 'sql'>('visual');
  filterAction = $state<'all' | 'create' | 'alter' | 'drop' | 'identical'>('all');
  searchQuery = $state<string>('');

  errorMessage = $state<string | null>(null);
  successMessage = $state<string | null>(null);

  open(sourceId?: string, targetId?: string) {
    this.isOpen = true;
    this.errorMessage = null;
    this.successMessage = null;

    const conns = connectionStore.connections;
    this.sourceConnectionId = sourceId || connectionStore.activeConnectionId || (conns[0]?.id ?? '');
    this.targetConnectionId = targetId || (conns.find(c => c.id !== this.sourceConnectionId)?.id ?? this.sourceConnectionId);

    if (this.sourceConnectionId && this.targetConnectionId && this.sourceConnectionId !== this.targetConnectionId) {
      this.compare();
    }
  }

  close() {
    this.isOpen = false;
    this.errorMessage = null;
    this.successMessage = null;
  }

  toggle() {
    if (this.isOpen) {
      this.close();
    } else {
      this.open();
    }
  }

  swap() {
    const temp = this.sourceConnectionId;
    this.sourceConnectionId = this.targetConnectionId;
    this.targetConnectionId = temp;
    this.compare();
  }

  async compare() {
    if (!this.sourceConnectionId || !this.targetConnectionId) {
      this.errorMessage = 'Please select both Source and Target connections.';
      return;
    }

    if (this.sourceConnectionId === this.targetConnectionId) {
      this.errorMessage = 'Source and Target connections must be different.';
      return;
    }

    const srcConn = connectionStore.connections.find(c => c.id === this.sourceConnectionId);
    const tgtConn = connectionStore.connections.find(c => c.id === this.targetConnectionId);

    this.isComparing = true;
    this.errorMessage = null;
    this.successMessage = null;

    try {
      const data = await api.compareSchemas(
        this.sourceConnectionId,
        this.targetConnectionId,
        srcConn?.driver,
        tgtConn?.driver
      );
      this.result = data;
      this.selectedTableDiff = data.tableDiffs.find(t => t.action !== 'identical') || data.tableDiffs[0] || null;
    } catch (err: any) {
      console.error('Schema diff failed:', err);
      this.errorMessage = err?.toString() || 'Failed to compare schemas.';
    } finally {
      this.isComparing = false;
    }
  }

  async applyMigration() {
    if (!this.result || !this.result.fullMigrationSql.trim()) {
      this.errorMessage = 'No migration SQL to apply.';
      return;
    }

    this.isApplying = true;
    this.errorMessage = null;
    try {
      await api.applyMigrationScript(this.targetConnectionId, this.result.fullMigrationSql);
      this.showToast('success', `✓ Migration applied successfully to target database.`);
      // Refresh comparison
      await this.compare();
    } catch (err: any) {
      console.error('Failed to apply migration:', err);
      this.showToast('error', `Migration execution failed: ${err?.toString() || err}`);
    } finally {
      this.isApplying = false;
    }
  }

  showToast(type: 'success' | 'error', message: string) {
    if (type === 'success') {
      this.successMessage = message;
      setTimeout(() => {
        if (this.successMessage === message) this.successMessage = null;
      }, 4000);
    } else {
      this.errorMessage = message;
      setTimeout(() => {
        if (this.errorMessage === message) this.errorMessage = null;
      }, 5000);
    }
  }

  get filteredTableDiffs(): TableDiff[] {
    if (!this.result) return [];
    let list = this.result.tableDiffs;

    if (this.filterAction !== 'all') {
      list = list.filter(t => t.action === this.filterAction);
    }

    if (this.searchQuery.trim()) {
      const q = this.searchQuery.toLowerCase().trim();
      list = list.filter(t => 
        t.tableName.toLowerCase().includes(q) ||
        t.columns.some(c => c.name.toLowerCase().includes(q))
      );
    }

    return list;
  }
}

export const diffStore = new DiffStore();
