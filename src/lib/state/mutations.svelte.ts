import type { ColumnMetadata, DriverType, QueryResult } from '../api/types';
import { api } from '../api/client';
import { connectionStore } from './connection.svelte';
import { tabsStore } from './tabs.svelte';

export interface CellChange {
  original: any;
  current: any;
  columnIdx: number;
}

export interface InsertedRow {
  tempId: string;
  values: Record<string, any>;
}

export interface VisualDiffItem {
  id: string;
  type: 'update' | 'insert' | 'delete';
  rowIdentifier: string;
  rowKey: string | number;
  tableName: string;
  changes?: {
    columnName: string;
    oldValue: any;
    newValue: any;
  }[];
  insertedValues?: Record<string, any>;
  deletedValues?: Record<string, any>;
}

export class TabMutationState {
  tabId: string;
  // Map of rowKey -> (Map of colName -> CellChange)
  modifiedCells = $state<Record<string, Record<string, CellChange>>>({});
  // Inserted rows
  insertedRows = $state<InsertedRow[]>([]);
  // Deleted row keys
  deletedRowKeys = $state<string[]>([]);
  // Store original row snapshot for deleted rows to build WHERE clause
  deletedRowsOriginals = $state<Record<string, Record<string, any>>>({});

  isCommitting = $state(false);
  errorMessage = $state<string | null>(null);
  successMessage = $state<string | null>(null);
  isDrawerOpen = $state(false);
  activeDrawerTab = $state<'diff' | 'sql'>('diff');

  constructor(tabId: string) {
    this.tabId = tabId;
  }

  get modifiedRowCount(): number {
    return Object.keys(this.modifiedCells).length;
  }

  get insertedRowCount(): number {
    return this.insertedRows.length;
  }

  get deletedRowCount(): number {
    return this.deletedRowKeys.length;
  }

  get totalChangesCount(): number {
    return this.modifiedRowCount + this.insertedRowCount + this.deletedRowCount;
  }

  get hasChanges(): boolean {
    return this.totalChangesCount > 0;
  }

  // Set cell modification
  setCell(rowKey: string, columnName: string, columnIdx: number, originalValue: any, newValue: any) {
    // If value is identical to original, remove change
    if (this.areValuesEqual(originalValue, newValue)) {
      this.revertCell(rowKey, columnName);
      return;
    }

    if (!this.modifiedCells[rowKey]) {
      this.modifiedCells[rowKey] = {};
    }

    this.modifiedCells[rowKey][columnName] = {
      original: originalValue,
      current: newValue,
      columnIdx
    };

    // Auto open drawer when first change is staged
    if (!this.isDrawerOpen) {
      this.isDrawerOpen = true;
    }
  }

  revertCell(rowKey: string, columnName: string) {
    if (this.modifiedCells[rowKey]) {
      delete this.modifiedCells[rowKey][columnName];
      if (Object.keys(this.modifiedCells[rowKey]).length === 0) {
        delete this.modifiedCells[rowKey];
      }
    }
  }

  revertRow(rowKey: string) {
    if (this.modifiedCells[rowKey]) {
      delete this.modifiedCells[rowKey];
    }
    if (this.deletedRowKeys.includes(rowKey)) {
      this.unmarkRowDeleted(rowKey);
    }
  }

  // Insert row
  addInsertedRow(columns: ColumnMetadata[]): string {
    const tempId = `new_row_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`;
    const values: Record<string, any> = {};
    
    for (const col of columns) {
      // Don't prefill autoincrement primary key if name is id
      if (col.isPrimaryKey || col.name.toLowerCase() === 'id') {
        values[col.name] = null;
      } else if (col.dataType.includes('bool')) {
        values[col.name] = false;
      } else {
        values[col.name] = null;
      }
    }

    this.insertedRows.push({ tempId, values });
    this.isDrawerOpen = true;
    return tempId;
  }

  updateInsertedRowCell(tempId: string, columnName: string, newValue: any) {
    const row = this.insertedRows.find(r => r.tempId === tempId);
    if (row) {
      row.values[columnName] = newValue;
    }
  }

  removeInsertedRow(tempId: string) {
    this.insertedRows = this.insertedRows.filter(r => r.tempId !== tempId);
  }

  // Delete row
  markRowDeleted(rowKey: string, rowValues: Record<string, any>) {
    if (!this.deletedRowKeys.includes(rowKey)) {
      this.deletedRowKeys.push(rowKey);
      this.deletedRowsOriginals[rowKey] = { ...rowValues };
      // Also clean up any cell modifications on this row
      if (this.modifiedCells[rowKey]) {
        delete this.modifiedCells[rowKey];
      }
    }
    this.isDrawerOpen = true;
  }

  unmarkRowDeleted(rowKey: string) {
    this.deletedRowKeys = this.deletedRowKeys.filter(k => k !== rowKey);
    delete this.deletedRowsOriginals[rowKey];
  }

  toggleRowDeleted(rowKey: string, rowValues: Record<string, any>) {
    if (this.deletedRowKeys.includes(rowKey)) {
      this.unmarkRowDeleted(rowKey);
    } else {
      this.markRowDeleted(rowKey, rowValues);
    }
  }

  // Clear all staged changes
  clearAll() {
    this.modifiedCells = {};
    this.insertedRows = [];
    this.deletedRowKeys = [];
    this.deletedRowsOriginals = {};
    this.errorMessage = null;
    this.successMessage = null;
  }

  private areValuesEqual(a: any, b: any): boolean {
    if (a === b) return true;
    if ((a === null || a === undefined || a === '') && (b === null || b === undefined || b === '')) return true;
    if (typeof a === 'number' && typeof b === 'string') return a.toString() === b;
    if (typeof a === 'boolean' && typeof b === 'string') return a.toString() === b.toLowerCase();
    return false;
  }

  // Extract table name from tab or SQL
  resolveTableName(tabSql?: string, tabTableName?: string): string {
    if (tabTableName && tabTableName.trim()) return tabTableName.trim();
    if (!tabSql) return 'table_name';

    // Regex match "FROM [schema.]table"
    const match = tabSql.match(/\bFROM\s+([`"']?[\w.]+[`"']?)/i);
    if (match && match[1]) {
      return match[1].replace(/[`"']/g, '').split('.').pop() || 'table_name';
    }
    return 'table_name';
  }

  // Quote identifier based on driver
  private quoteIdentifier(name: string, driver: DriverType): string {
    if (driver === 'mysql') return `\`${name}\``;
    if (driver === 'mssql') {
      if (name.includes('.')) {
        return name.split('.').map(part => `[${part.replace(/[\[\]]/g, '')}]`).join('.');
      }
      return `[${name.replace(/[\[\]]/g, '')}]`;
    }
    return `"${name}"`;
  }

  // Format SQL value literal
  private formatSqlValue(val: any, driver: DriverType): string {
    if (val === null || val === undefined || val === '') return 'NULL';
    if (typeof val === 'number') return isNaN(val) ? 'NULL' : val.toString();
    if (typeof val === 'boolean') {
      if (driver === 'mssql') return val ? '1' : '0';
      return val ? 'TRUE' : 'FALSE';
    }
    if (typeof val === 'object') {
      const jsonStr = JSON.stringify(val).replace(/'/g, "''");
      return `'${jsonStr}'`;
    }
    const str = String(val).replace(/'/g, "''");
    return `'${str}'`;
  }

  // Generate SQL batch script
  generateSql(
    driver: DriverType = 'postgres',
    columns: ColumnMetadata[] = [],
    tableName?: string,
    rawRows: any[][] = []
  ): string {
    const table = tableName || 'table_name';
    const qTable = this.quoteIdentifier(table, driver);
    const statements: string[] = [];

    // Find primary key column(s)
    const pkCols = columns.filter(c => c.isPrimaryKey || c.name.toLowerCase() === 'id');

    // 1. UPDATE Statements
    for (const [rowKey, colChanges] of Object.entries(this.modifiedCells)) {
      const setClauses: string[] = [];
      for (const [colName, change] of Object.entries(colChanges)) {
        const qCol = this.quoteIdentifier(colName, driver);
        const valLiteral = this.formatSqlValue(change.current, driver);
        setClauses.push(`${qCol} = ${valLiteral}`);
      }

      if (setClauses.length === 0) continue;

      // Build WHERE clause
      const whereClauses: string[] = [];
      if (pkCols.length > 0) {
        for (const pk of pkCols) {
          const qPk = this.quoteIdentifier(pk.name, driver);
          // If rowKey is the PK value
          const pkVal = colChanges[pk.name]?.original !== undefined 
            ? colChanges[pk.name].original 
            : rowKey;
          whereClauses.push(`${qPk} = ${this.formatSqlValue(pkVal, driver)}`);
        }
      } else {
        // Fallback: use original values of all columns
        const colNames = Object.keys(colChanges);
        for (const colName of colNames) {
          const qCol = this.quoteIdentifier(colName, driver);
          const orig = colChanges[colName].original;
          if (orig === null || orig === undefined) {
            whereClauses.push(`${qCol} IS NULL`);
          } else {
            whereClauses.push(`${qCol} = ${this.formatSqlValue(orig, driver)}`);
          }
        }
      }

      const whereStr = whereClauses.length > 0 ? whereClauses.join(' AND ') : '1 = 1';
      statements.push(`UPDATE ${qTable} SET ${setClauses.join(', ')} WHERE ${whereStr};`);
    }

    // 2. INSERT Statements
    for (const row of this.insertedRows) {
      const colsToInsert: string[] = [];
      const valuesToInsert: string[] = [];

      for (const [colName, val] of Object.entries(row.values)) {
        // Skip NULL primary keys (likely autoincrement / serial)
        const isPk = pkCols.some(pk => pk.name === colName);
        if (isPk && (val === null || val === undefined || val === '')) {
          continue;
        }

        colsToInsert.push(this.quoteIdentifier(colName, driver));
        valuesToInsert.push(this.formatSqlValue(val, driver));
      }

      if (colsToInsert.length > 0) {
        statements.push(
          `INSERT INTO ${qTable} (${colsToInsert.join(', ')}) VALUES (${valuesToInsert.join(', ')});`
        );
      }
    }

    // 3. DELETE Statements
    for (const rowKey of this.deletedRowKeys) {
      const origValues = this.deletedRowsOriginals[rowKey] || {};
      const whereClauses: string[] = [];

      if (pkCols.length > 0) {
        for (const pk of pkCols) {
          const qPk = this.quoteIdentifier(pk.name, driver);
          const val = origValues[pk.name] !== undefined ? origValues[pk.name] : rowKey;
          whereClauses.push(`${qPk} = ${this.formatSqlValue(val, driver)}`);
        }
      } else {
        for (const [colName, val] of Object.entries(origValues)) {
          const qCol = this.quoteIdentifier(colName, driver);
          if (val === null || val === undefined) {
            whereClauses.push(`${qCol} IS NULL`);
          } else {
            whereClauses.push(`${qCol} = ${this.formatSqlValue(val, driver)}`);
          }
        }
      }

      const whereStr = whereClauses.length > 0 ? whereClauses.join(' AND ') : '1 = 1';
      statements.push(`DELETE FROM ${qTable} WHERE ${whereStr};`);
    }

    return statements.join('\n');
  }

  // Generate structured visual diff items
  generateVisualDiffList(tableName: string): VisualDiffItem[] {
    const list: VisualDiffItem[] = [];

    // 1. Updates
    for (const [rowKey, colChanges] of Object.entries(this.modifiedCells)) {
      const changes = Object.entries(colChanges).map(([colName, change]) => ({
        columnName: colName,
        oldValue: change.original,
        newValue: change.current
      }));

      list.push({
        id: `update_${rowKey}`,
        type: 'update',
        rowIdentifier: `Row ${rowKey}`,
        rowKey,
        tableName,
        changes
      });
    }

    // 2. Inserts
    for (const row of this.insertedRows) {
      list.push({
        id: row.tempId,
        type: 'insert',
        rowIdentifier: `New Row (${row.tempId.slice(-4)})`,
        rowKey: row.tempId,
        tableName,
        insertedValues: row.values
      });
    }

    // 3. Deletes
    for (const rowKey of this.deletedRowKeys) {
      list.push({
        id: `delete_${rowKey}`,
        type: 'delete',
        rowIdentifier: `Row ${rowKey}`,
        rowKey,
        tableName,
        deletedValues: this.deletedRowsOriginals[rowKey]
      });
    }

    return list;
  }

  // Commit all staged mutations
  async commit(
    connectionId: string,
    driver: DriverType,
    columns: ColumnMetadata[],
    tableName: string
  ): Promise<boolean> {
    if (!this.hasChanges) return true;

    const sqlScript = this.generateSql(driver, columns, tableName);
    if (!sqlScript.trim()) {
      this.clearAll();
      return true;
    }

    this.isCommitting = true;
    this.errorMessage = null;
    this.successMessage = null;

    try {
      // Execute batch SQL against DB
      await api.executeQuery(connectionId, sqlScript);
      
      const count = this.totalChangesCount;
      this.clearAll();
      this.successMessage = `Successfully applied ${count} mutation${count > 1 ? 's' : ''} to ${tableName}.`;
      
      // Auto-refresh tab query
      await tabsStore.runTabQuery(this.tabId);

      setTimeout(() => {
        this.successMessage = null;
      }, 4000);

      return true;
    } catch (err: any) {
      console.error('Commit mutations error:', err);
      this.errorMessage = typeof err === 'string' ? err : err?.message || JSON.stringify(err);
      return false;
    } finally {
      this.isCommitting = false;
    }
  }
}

class MutationsStore {
  private tabStates = new Map<string, TabMutationState>();

  getTabState(tabId: string): TabMutationState {
    let state = this.tabStates.get(tabId);
    if (!state) {
      state = new TabMutationState(tabId);
      this.tabStates.set(tabId, state);
    }
    return state;
  }
}

export const mutationStore = new MutationsStore();
