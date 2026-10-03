import type { FugpadDocument, NotebookCell, MarkdownNotebookCell, SqlNotebookCell, NotebookChartConfig } from '../api/types';
import { api } from '../api/client';
import { connectionStore } from './connection.svelte';
import { generateStandaloneHtmlReport, generateMarkdownExport } from '../utils/notebookExport';

export class NotebookStore {
  // Map of notebookId -> FugpadDocument
  docs = $state<Record<string, FugpadDocument>>({});

  getNotebook(id: string): FugpadDocument | undefined {
    return this.docs[id];
  }

  createDefaultTemplate(title?: string, connId?: string): FugpadDocument {
    const id = `pad-${Date.now()}`;
    const defaultConn = connId || connectionStore.activeConnectionId || '';
    const conn = connectionStore.connections.find(c => c.id === defaultConn);
    const isMssql = conn?.driver === 'mssql';

    const sampleSql = isMssql
      ? `SELECT TOP 10 
    name as item_name, 
    type_desc as category, 
    create_date as created_at 
FROM sys.objects 
WHERE type IN ('U', 'V')
ORDER BY create_date DESC;`
      : `SELECT 
    'Category ' || (id % 5 + 1) as category,
    COUNT(*) as total_records,
    ROUND(AVG(id * 12.5), 2) as avg_revenue
FROM (
    SELECT 1 as id UNION SELECT 2 UNION SELECT 3 UNION SELECT 4 UNION SELECT 5 
    UNION SELECT 6 UNION SELECT 7 UNION SELECT 8 UNION SELECT 9 UNION SELECT 10
) sample_data
GROUP BY category
ORDER BY avg_revenue DESC;`;

    const template: FugpadDocument = {
      version: 1,
      id,
      title: title || 'Data Analysis & Metrics Scratchpad',
      description: 'Interactive SQL notebook combining Markdown narrative, multi-query execution, and dynamic chart visualizations.',
      defaultConnectionId: defaultConn,
      createdAt: Date.now(),
      updatedAt: Date.now(),
      cells: [
        {
          id: `cell-md-1`,
          type: 'markdown',
          content: `# 📊 Analytics & KPI Executive Summary\n\nWelcome to your **Interactive SQL Notebook** (` + '`.fugpad`' + `)! Use this scratchpad to query live data, document findings with markdown, and visualize metric charts side-by-side.\n\n### ⚡ Quick Tips:\n- **Shift+Enter**: Execute active SQL cell\n- **Double Click** markdown to edit raw text\n- Switch outputs between **Grid 📋**, **Chart 📈**, and **JSON 🔣**\n- Export to self-contained **HTML Reports** or **.fugpad** documents for team sharing.`
        },
        {
          id: `cell-sql-1`,
          type: 'sql',
          sql: sampleSql,
          displayMode: 'chart',
          chartConfig: {
            type: 'bar',
            xAxisColumn: 'category',
            yAxisColumns: ['avg_revenue', 'total_records'],
            title: 'Revenue & Volume by Category'
          },
          collapsed: false
        },
        {
          id: `cell-md-2`,
          type: 'markdown',
          content: `### 🔍 Detailed Record Breakdown\nBelow is the raw query output for granular inspection.`
        },
        {
          id: `cell-sql-2`,
          type: 'sql',
          sql: isMssql ? `SELECT TOP 20 name, object_id, type_desc FROM sys.tables;` : `SELECT id, 'User_' || id as username, (id * 100) as balance FROM (SELECT 1 as id UNION SELECT 2 UNION SELECT 3 UNION SELECT 4 UNION SELECT 5) t;`,
          displayMode: 'grid',
          collapsed: false
        }
      ]
    };

    return template;
  }

  createNotebook(title?: string, connectionId?: string, customDoc?: FugpadDocument): string {
    const doc = customDoc || this.createDefaultTemplate(title, connectionId);
    this.docs[doc.id] = doc;
    return doc.id;
  }

  updateTitle(notebookId: string, title: string) {
    const doc = this.docs[notebookId];
    if (doc) {
      doc.title = title;
      doc.updatedAt = Date.now();
    }
  }

  updateDescription(notebookId: string, desc: string) {
    const doc = this.docs[notebookId];
    if (doc) {
      doc.description = desc;
      doc.updatedAt = Date.now();
    }
  }

  setConnection(notebookId: string, connectionId: string) {
    const doc = this.docs[notebookId];
    if (doc) {
      doc.defaultConnectionId = connectionId;
      doc.updatedAt = Date.now();
    }
  }

  addCell(notebookId: string, type: 'markdown' | 'sql', afterCellId?: string): string {
    const doc = this.docs[notebookId];
    if (!doc) return '';

    const newId = `cell-${type}-${Date.now()}`;
    const newCell: NotebookCell = type === 'markdown'
      ? {
          id: newId,
          type: 'markdown',
          content: '### New Section\nWrite notes, queries explanation, or findings here...',
          isEditing: true
        }
      : {
          id: newId,
          type: 'sql',
          sql: 'SELECT * FROM users LIMIT 10;',
          displayMode: 'grid',
          collapsed: false
        };

    if (afterCellId) {
      const idx = doc.cells.findIndex(c => c.id === afterCellId);
      if (idx !== -1) {
        doc.cells.splice(idx + 1, 0, newCell);
      } else {
        doc.cells.push(newCell);
      }
    } else {
      doc.cells.push(newCell);
    }

    doc.updatedAt = Date.now();
    return newId;
  }

  deleteCell(notebookId: string, cellId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const idx = doc.cells.findIndex(c => c.id === cellId);
    if (idx !== -1) {
      doc.cells.splice(idx, 1);
      doc.updatedAt = Date.now();
    }
  }

  moveCell(notebookId: string, cellId: string, direction: 'up' | 'down') {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const idx = doc.cells.findIndex(c => c.id === cellId);
    if (idx === -1) return;

    const targetIdx = direction === 'up' ? idx - 1 : idx + 1;
    if (targetIdx < 0 || targetIdx >= doc.cells.length) return;

    const [moved] = doc.cells.splice(idx, 1);
    doc.cells.splice(targetIdx, 0, moved);
    doc.updatedAt = Date.now();
  }

  duplicateCell(notebookId: string, cellId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const idx = doc.cells.findIndex(c => c.id === cellId);
    if (idx === -1) return;

    const orig = doc.cells[idx];
    const copy: NotebookCell = JSON.parse(JSON.stringify(orig));
    copy.id = `cell-${copy.type}-${Date.now()}`;
    doc.cells.splice(idx + 1, 0, copy);
    doc.updatedAt = Date.now();
  }

  updateMarkdownContent(notebookId: string, cellId: string, content: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const cell = doc.cells.find(c => c.id === cellId);
    if (cell && cell.type === 'markdown') {
      (cell as MarkdownNotebookCell).content = content;
      doc.updatedAt = Date.now();
    }
  }

  updateSqlContent(notebookId: string, cellId: string, sql: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const cell = doc.cells.find(c => c.id === cellId);
    if (cell && cell.type === 'sql') {
      (cell as SqlNotebookCell).sql = sql;
      doc.updatedAt = Date.now();
    }
  }

  setCellDisplayMode(notebookId: string, cellId: string, mode: 'grid' | 'chart' | 'json') {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const cell = doc.cells.find(c => c.id === cellId);
    if (cell && cell.type === 'sql') {
      const sqlCell = cell as SqlNotebookCell;
      sqlCell.displayMode = mode;
      if (mode === 'chart' && !sqlCell.chartConfig && sqlCell.result?.columns && sqlCell.result.columns.length > 0) {
        const cols = sqlCell.result.columns;
        sqlCell.chartConfig = {
          type: 'bar',
          xAxisColumn: cols[0]?.name || '',
          yAxisColumns: cols[1] ? [cols[1].name] : [cols[0]?.name || ''],
          title: 'Query Data Chart'
        };
      }
      doc.updatedAt = Date.now();
    }
  }

  updateChartConfig(notebookId: string, cellId: string, config: Partial<NotebookChartConfig>) {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const cell = doc.cells.find(c => c.id === cellId);
    if (cell && cell.type === 'sql') {
      const sqlCell = cell as SqlNotebookCell;
      if (!sqlCell.chartConfig) {
        sqlCell.chartConfig = {
          type: 'bar',
          xAxisColumn: '',
          yAxisColumns: [],
          title: 'Visualization'
        };
      }
      sqlCell.chartConfig = { ...sqlCell.chartConfig, ...config };
      doc.updatedAt = Date.now();
    }
  }

  clearCellOutput(notebookId: string, cellId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const cell = doc.cells.find(c => c.id === cellId);
    if (cell && cell.type === 'sql') {
      const s = cell as SqlNotebookCell;
      s.result = undefined;
      s.errorMessage = undefined;
      doc.updatedAt = Date.now();
    }
  }

  async runCell(notebookId: string, cellId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;
    const cell = doc.cells.find(c => c.id === cellId);
    if (!cell || cell.type !== 'sql') return;

    const sqlCell = cell as SqlNotebookCell;
    const connId = sqlCell.connectionId || doc.defaultConnectionId || connectionStore.activeConnectionId;

    if (!connId || !sqlCell.sql.trim()) return;

    sqlCell.isExecuting = true;
    sqlCell.errorMessage = undefined;
    const start = Date.now();

    try {
      const res = await api.executeQuery(connId, sqlCell.sql);
      sqlCell.result = res;
      sqlCell.executionDurationMs = res.executionTimeMs || (Date.now() - start);

      // Auto-configure chart config defaults if switched to chart or first run
      if (sqlCell.result.columns.length >= 2 && !sqlCell.chartConfig?.xAxisColumn) {
        sqlCell.chartConfig = {
          type: sqlCell.chartConfig?.type || 'bar',
          xAxisColumn: sqlCell.result.columns[0].name,
          yAxisColumns: [sqlCell.result.columns[1].name],
          title: sqlCell.chartConfig?.title || 'Data Metrics'
        };
      }
    } catch (err: any) {
      console.error('Notebook cell run error:', err);
      sqlCell.errorMessage = typeof err === 'string' ? err : err?.message || JSON.stringify(err);
      sqlCell.result = undefined;
    } finally {
      sqlCell.isExecuting = false;
      doc.updatedAt = Date.now();
    }
  }

  async runAllCells(notebookId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;

    const sqlCells = doc.cells.filter(c => c.type === 'sql');
    for (const cell of sqlCells) {
      await this.runCell(notebookId, cell.id);
    }
  }

  // Export & File I/O
  async saveAsFugpadFile(notebookId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;

    const jsonStr = JSON.stringify(doc, null, 2);
    const cleanTitle = doc.title.replace(/[^a-zA-Z0-9_-]/g, '_').toLowerCase();
    const defaultName = `${cleanTitle}.fugpad`;

    try {
      const targetPath = await api.saveFileDialog(defaultName, [
        { name: 'FugDB Notebook (*.fugpad)', extensions: ['fugpad', 'json'] }
      ]);

      if (targetPath) {
        await api.saveFugpadFile(targetPath, jsonStr);
      }
    } catch (err) {
      // Fallback download in browser
      const blob = new Blob([jsonStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = defaultName;
      a.click();
      URL.revokeObjectURL(url);
    }
  }

  async openFugpadFile(): Promise<string | null> {
    try {
      const selectedPath = await api.openFileDialog([
        { name: 'FugDB Notebook (*.fugpad)', extensions: ['fugpad', 'json'] }
      ]);

      if (selectedPath) {
        const content = await api.readFugpadFile(selectedPath);
        const parsed: FugpadDocument = JSON.parse(content);
        if (parsed && parsed.cells && Array.isArray(parsed.cells)) {
          parsed.id = `pad-${Date.now()}`;
          this.docs[parsed.id] = parsed;
          return parsed.id;
        }
      }
    } catch (e) {
      console.error('Error opening fugpad file:', e);
    }
    return null;
  }

  async exportHtmlReport(notebookId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;

    const conn = connectionStore.connections.find(c => c.id === doc.defaultConnectionId);
    const connName = conn?.name || 'Database Connection';
    const html = generateStandaloneHtmlReport(doc, connName);

    const cleanTitle = doc.title.replace(/[^a-zA-Z0-9_-]/g, '_').toLowerCase();
    const defaultName = `report_${cleanTitle}_${Date.now()}.html`;

    try {
      const targetPath = await api.saveFileDialog(defaultName, [
        { name: 'HTML Report (*.html)', extensions: ['html'] }
      ]);

      if (targetPath) {
        await api.exportNotebookHtmlFile(targetPath, html);
      }
    } catch (err) {
      const blob = new Blob([html], { type: 'text/html;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = defaultName;
      a.click();
      URL.revokeObjectURL(url);
    }
  }

  async exportMarkdownReport(notebookId: string) {
    const doc = this.docs[notebookId];
    if (!doc) return;

    const md = generateMarkdownExport(doc);
    const cleanTitle = doc.title.replace(/[^a-zA-Z0-9_-]/g, '_').toLowerCase();
    const defaultName = `${cleanTitle}.md`;

    try {
      const targetPath = await api.saveFileDialog(defaultName, [
        { name: 'Markdown (*.md)', extensions: ['md'] }
      ]);

      if (targetPath) {
        await api.saveFugpadFile(targetPath, md);
      }
    } catch (err) {
      const blob = new Blob([md], { type: 'text/markdown;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = defaultName;
      a.click();
      URL.revokeObjectURL(url);
    }
  }
}

export const notebookStore = new NotebookStore();
