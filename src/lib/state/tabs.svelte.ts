import type { QueryResult } from '../api/types';
import { api } from '../api/client';
import { connectionStore } from './connection.svelte';
import { assessSqlRisk } from '../utils/safetyGuard';
import { safetyStore } from './safety.svelte';
import { historyStore } from './history.svelte';
import { sessionStore } from './session.svelte';
import { notebookStore } from './notebook.svelte';

export interface TabItem {
  id: string;
  title: string;
  type: 'sql' | 'table_grid' | 'erd' | 'scratchpad' | 'notebook' | 'redis';
  connectionId: string;
  sql?: string;
  tableName?: string;
  queryResult?: QueryResult;
  errorMessage?: string;
  isExecuting?: boolean;
  isModified?: boolean;
}

export class TabsStore {
  tabs = $state<TabItem[]>([]);
  activeTabId = $state<string>('');

  activeTab = $derived(
    this.tabs.find(t => t.id === this.activeTabId)
  );

  openRedisTab(connectionId: string, title?: string) {
    const existing = this.tabs.find(t => t.type === 'redis' && t.connectionId === connectionId);
    if (existing) {
      this.activeTabId = existing.id;
      return;
    }

    const conn = connectionStore.connections.find(c => c.id === connectionId);
    const tabTitle = title || `Redis: ${conn?.name || 'Server'}`;
    const newId = `tab-redis-${Date.now()}`;
    const newTab: TabItem = {
      id: newId,
      title: tabTitle,
      type: 'redis',
      connectionId
    };
    this.tabs.push(newTab);
    this.activeTabId = newId;
  }

  openNewNotebookTab(title?: string, initialDocId?: string) {
    const newId = initialDocId || `tab-pad-${Date.now()}`;
    const defaultTitle = title || `Notebook ${this.tabs.filter(t => t.type === 'notebook').length + 1}`;
    notebookStore.getOrCreate(newId, defaultTitle, connectionStore.activeConnectionId);

    const newTab: TabItem = {
      id: newId,
      title: defaultTitle,
      type: 'notebook',
      connectionId: connectionStore.activeConnectionId,
    };
    this.tabs.push(newTab);
    this.activeTabId = newId;
  }

  openNewSqlTab(initialSql?: string, title?: string) {
    const conn = connectionStore.activeConnection;
    const isMssql = conn?.driver === 'mssql';
    const defaultSql = isMssql ? 'SELECT TOP (100) * FROM sys.tables;' : 'SELECT * FROM users LIMIT 100;';

    const newId = `tab-${Date.now()}`;
    const newTab: TabItem = {
      id: newId,
      title: title || `Query ${this.tabs.filter(t => t.type === 'sql').length + 1}`,
      type: 'sql',
      connectionId: connectionStore.activeConnectionId,
      sql: initialSql || defaultSql,
    };
    this.tabs.push(newTab);
    this.activeTabId = newId;
  }

  openTableGridTab(tableName: string, schema?: string) {
    const existing = this.tabs.find(t => t.type === 'table_grid' && t.tableName === tableName && t.connectionId === connectionStore.activeConnectionId);
    if (existing) {
      this.activeTabId = existing.id;
      return;
    }

    const conn = connectionStore.activeConnection;
    const driver = conn?.driver;

    let sql = '';
    if (driver === 'mssql') {
      const target = schema ? `[${schema}].[${tableName}]` : `[${tableName}]`;
      sql = `SELECT TOP (100) * FROM ${target};`;
    } else if (driver === 'mysql') {
      sql = `SELECT * FROM \`${tableName}\` LIMIT 100;`;
    } else {
      const target = schema && schema !== 'public' ? `"${schema}"."${tableName}"` : `"${tableName}"`;
      sql = `SELECT * FROM ${target} LIMIT 100;`;
    }

    const newId = `tab-grid-${Date.now()}`;
    const newTab: TabItem = {
      id: newId,
      title: `Table: ${tableName}`,
      type: 'table_grid',
      connectionId: connectionStore.activeConnectionId,
      tableName,
      sql,
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

  updateTabSql(tabId: string, newSql: string) {
    const tab = this.tabs.find(t => t.id === tabId);
    if (tab) {
      tab.sql = newSql;
      tab.errorMessage = undefined;
    }
  }

  async runTabQuery(tabId: string) {
    const tab = this.tabs.find(t => t.id === tabId);
    if (!tab || !tab.sql) return;

    const conn = connectionStore.connections.find(c => c.id === tab.connectionId);
    const environment = conn?.environment || 'dev';
    const databaseName = conn?.database || connectionStore.activeSchemaTree?.currentDatabase || 'master';

    // 1. Intercept dangerous/destructive queries on Production database
    if (environment === 'production' && tab.sql.trim()) {
      const risk = assessSqlRisk(tab.sql);
      if (risk.isDangerous) {
        const allowed = await safetyStore.requestConfirmation({
          connectionId: tab.connectionId,
          connectionName: conn?.name || 'Production Database',
          databaseName,
          environment,
          sql: tab.sql,
          risk
        });

        if (!allowed) {
          // User aborted execution
          return;
        }
      }
    }

    const start = Date.now();
    tab.isExecuting = true;
    tab.errorMessage = undefined;
    try {
      const result = await api.executeQuery(tab.connectionId, tab.sql);
      tab.queryResult = result;

      // Track session transaction state
      sessionStore.trackQuery(tab.connectionId, tab.sql);

      // Log successful execution into persistent history
      historyStore.addEntry({
        sql: tab.sql,
        connectionId: tab.connectionId,
        connectionName: conn?.name || 'Database',
        database: databaseName,
        executedAt: start,
        durationMs: result.executionTimeMs || (Date.now() - start),
        status: 'success',
        rowCount: result.affectedRows
      });
    } catch (err: any) {
      console.error('Execution error:', err);
      const errMsg = typeof err === 'string' ? err : err?.message || JSON.stringify(err);
      tab.errorMessage = errMsg;
      tab.queryResult = undefined;

      // Log failed execution into persistent history
      historyStore.addEntry({
        sql: tab.sql,
        connectionId: tab.connectionId,
        connectionName: conn?.name || 'Database',
        database: databaseName,
        executedAt: start,
        durationMs: Date.now() - start,
        status: 'error',
        errorMessage: errMsg
      });
    } finally {
      tab.isExecuting = false;
    }
  }
}

export const tabsStore = new TabsStore();
