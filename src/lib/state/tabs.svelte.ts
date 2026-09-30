import type { QueryResult } from '../api/types';
import { api } from '../api/client';
import { connectionStore } from './connection.svelte';

export interface TabItem {
  id: string;
  title: string;
  type: 'sql' | 'table_grid' | 'erd' | 'scratchpad';
  connectionId: string;
  sql?: string;
  tableName?: string;
  queryResult?: QueryResult;
  isExecuting?: boolean;
  isModified?: boolean;
}

export class TabsStore {
  tabs = $state<TabItem[]>([
    {
      id: 'tab-1',
      title: 'Query 1 (users)',
      type: 'sql',
      connectionId: 'conn-local-pg',
      sql: 'SELECT id, name, email, role, is_active, created_at \nFROM users \nORDER BY id ASC \nLIMIT 50;',
    }
  ]);

  activeTabId = $state<string>('tab-1');

  activeTab = $derived(
    this.tabs.find(t => t.id === this.activeTabId)
  );

  openNewSqlTab(initialSql?: string, title?: string) {
    const newId = `tab-${Date.now()}`;
    const newTab: TabItem = {
      id: newId,
      title: title || `Query ${this.tabs.filter(t => t.type === 'sql').length + 1}`,
      type: 'sql',
      connectionId: connectionStore.activeConnectionId,
      sql: initialSql || 'SELECT * FROM users LIMIT 100;',
    };
    this.tabs.push(newTab);
    this.activeTabId = newId;
  }

  openTableGridTab(tableName: string) {
    const existing = this.tabs.find(t => t.type === 'table_grid' && t.tableName === tableName && t.connectionId === connectionStore.activeConnectionId);
    if (existing) {
      this.activeTabId = existing.id;
      return;
    }

    const newId = `tab-grid-${Date.now()}`;
    const newTab: TabItem = {
      id: newId,
      title: `Table: ${tableName}`,
      type: 'table_grid',
      connectionId: connectionStore.activeConnectionId,
      tableName,
      sql: `SELECT * FROM ${tableName} LIMIT 100;`,
    };
    this.tabs.push(newTab);
    this.activeTabId = newId;
    this.runTabQuery(newId);
  }

  closeTab(id: string) {
    const idx = this.tabs.findIndex(t => t.id === id);
    if (idx === -1) return;
    this.tabs.splice(idx, 1);
    if (this.activeTabId === id && this.tabs.length > 0) {
      this.activeTabId = this.tabs[Math.max(0, idx - 1)].id;
    }
  }

  async runTabQuery(tabId: string) {
    const tab = this.tabs.find(t => t.id === tabId);
    if (!tab || !tab.sql) return;

    tab.isExecuting = true;
    try {
      const result = await api.executeQuery(tab.connectionId, tab.sql);
      tab.queryResult = result;
    } catch (err) {
      console.error('Execution error:', err);
    } finally {
      tab.isExecuting = false;
    }
  }
}

export const tabsStore = new TabsStore();
