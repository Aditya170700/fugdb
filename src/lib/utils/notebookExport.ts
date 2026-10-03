import type { FugpadDocument, SqlNotebookCell, MarkdownNotebookCell } from '../api/types';

/**
 * Lightweight safe Markdown renderer for Fugpad notebook cells
 */
export function renderMarkdown(md: string): string {
  if (!md) return '';
  
  let html = md
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // Code blocks: ```sql ... ```
  html = html.replace(/```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g, (_m, lang, code) => {
    return `<div class="my-3 rounded-lg overflow-hidden border border-slate-200 dark:border-slate-800 bg-surface-950 font-mono text-xs">
      ${lang ? `<div class="px-3 py-1 bg-surface-900 border-b border-slate-200 dark:border-slate-800 text-[11px] font-bold text-slate-500 uppercase">${lang}</div>` : ''}
      <pre class="p-3 overflow-x-auto text-slate-800 dark:text-slate-200 leading-relaxed"><code>${code.trim()}</code></pre>
    </div>`;
  });

  // Markdown Tables: | col1 | col2 |
  html = html.replace(/((?:\|[^\n]+\|\r?\n)+)/g, (match) => {
    const lines = match.trim().split('\n').map(l => l.trim()).filter(Boolean);
    if (lines.length < 2) return match;
    
    const headers = lines[0].split('|').slice(1, -1).map(h => h.trim());
    const isDivider = lines[1].includes('---');
    const rowStartIndex = isDivider ? 2 : 1;

    let tableHtml = `<div class="overflow-x-auto my-3 rounded-lg border border-slate-200 dark:border-slate-800"><table class="w-full text-left text-xs border-collapse">`;
    tableHtml += `<thead class="bg-surface-900 border-b border-slate-200 dark:border-slate-800 text-slate-700 dark:text-slate-300 font-bold"><tr>`;
    headers.forEach(h => {
      tableHtml += `<th class="py-2 px-3">${h}</th>`;
    });
    tableHtml += `</tr></thead><tbody class="divide-y divide-slate-200/80 dark:divide-slate-800/60">`;

    for (let i = rowStartIndex; i < lines.length; i++) {
      const cells = lines[i].split('|').slice(1, -1).map(c => c.trim());
      tableHtml += `<tr class="hover:bg-surface-900/40">`;
      cells.forEach(c => {
        tableHtml += `<td class="py-2 px-3 text-slate-800 dark:text-slate-200">${c}</td>`;
      });
      tableHtml += `</tr>`;
    }

    tableHtml += `</tbody></table></div>`;
    return tableHtml;
  });

  // Inline code: `code`
  html = html.replace(/`([^`]+)`/g, '<code class="px-1.5 py-0.5 rounded bg-surface-800 border border-slate-300 dark:border-slate-700 text-indigo-600 dark:text-indigo-400 font-mono text-xs font-semibold">$1</code>');

  // Headings
  html = html.replace(/^#### (.*$)/gim, '<h4 class="text-xs font-bold text-slate-800 dark:text-slate-200 mt-3 mb-1 uppercase tracking-wider">$1</h4>');
  html = html.replace(/^### (.*$)/gim, '<h3 class="text-sm font-bold text-slate-900 dark:text-slate-100 mt-4 mb-1.5">$1</h3>');
  html = html.replace(/^## (.*$)/gim, '<h2 class="text-base font-bold text-slate-900 dark:text-slate-100 mt-5 mb-2 border-b border-slate-200 dark:border-slate-800 pb-1">$1</h2>');
  html = html.replace(/^# (.*$)/gim, '<h1 class="text-xl font-extrabold text-slate-900 dark:text-slate-100 mt-6 mb-3 border-b-2 border-indigo-500 pb-1.5">$1</h1>');

  // Blockquotes
  html = html.replace(/^\> (.*$)/gim, '<blockquote class="border-l-4 border-indigo-500 bg-indigo-50/50 dark:bg-indigo-950/20 px-3 py-2 rounded-r-lg my-2 text-xs italic text-slate-700 dark:text-slate-300">$1</blockquote>');

  // Checklists
  html = html.replace(/^\s*-\s*\[x\]\s*(.*$)/gim, '<li class="flex items-center gap-2 text-xs text-slate-700 dark:text-slate-300 my-1"><span class="w-4 h-4 rounded bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 flex items-center justify-center font-bold text-[10px]">✓</span><span>$1</span></li>');
  html = html.replace(/^\s*-\s*\[\s*\]\s*(.*$)/gim, '<li class="flex items-center gap-2 text-xs text-slate-700 dark:text-slate-300 my-1"><span class="w-4 h-4 rounded border border-slate-400 dark:border-slate-600 flex items-center justify-center text-[10px]"></span><span>$1</span></li>');

  // Unordered list items
  html = html.replace(/^\s*-\s+(.*$)/gim, '<li class="ml-4 list-disc text-xs text-slate-700 dark:text-slate-300 my-0.5">$1</li>');
  
  // Ordered list items
  html = html.replace(/^\s*([0-9]+)\.\s+(.*$)/gim, '<li class="ml-4 list-decimal text-xs text-slate-700 dark:text-slate-300 my-0.5"><span class="font-bold text-slate-900 dark:text-slate-100">$1.</span> $2</li>');

  // Bold & Italic
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong class="font-bold text-slate-900 dark:text-slate-100">$1</strong>');
  html = html.replace(/\*([^*]+)\*/g, '<em class="italic text-slate-700 dark:text-slate-300">$1</em>');

  // Horizontal rules
  html = html.replace(/^---$/gim, '<hr class="my-4 border-slate-200 dark:border-slate-800" />');

  // Paragraph breaks (preserve formatting)
  html = html.replace(/\n\n/g, '<div class="h-2"></div>');

  return html;
}

/**
 * Generate Standalone HTML Report with Chart.js visualization and interactive data tables
 */
export function generateStandaloneHtmlReport(doc: FugpadDocument, connectionName: string): string {
  const dateStr = new Date(doc.updatedAt || Date.now()).toLocaleString();
  
  const cellsHtml = doc.cells.map((cell, idx) => {
    if (cell.type === 'markdown') {
      const mdCell = cell as MarkdownNotebookCell;
      return `
        <div class="notebook-cell markdown-cell">
          <div class="cell-content">
            ${renderMarkdown(mdCell.content)}
          </div>
        </div>
      `;
    } else {
      const sqlCell = cell as SqlNotebookCell;
      const res = sqlCell.result;
      const chartCfg = sqlCell.chartConfig;
      const chartId = `chart_${sqlCell.id.replace(/[^a-zA-Z0-9]/g, '_')}_${idx}`;

      let resultHtml = '';
      if (sqlCell.errorMessage) {
        resultHtml = `
          <div class="error-banner">
            <strong>⚠️ Execution Error:</strong> ${sqlCell.errorMessage}
          </div>
        `;
      } else if (res && res.columns && res.columns.length > 0) {
        if (sqlCell.displayMode === 'chart' && chartCfg) {
          // Prepare chart data script
          const xColIdx = Math.max(0, res.columns.findIndex(c => c.name === chartCfg.xAxisColumn));
          const labels = res.rows.map(r => String(r[xColIdx] ?? ''));
          const yCols = chartCfg.yAxisColumns && chartCfg.yAxisColumns.length > 0 ? chartCfg.yAxisColumns : [res.columns[1]?.name || res.columns[0]?.name];
          const datasets = yCols.map((colName, cIdx) => {
            const colIdx = Math.max(0, res.columns.findIndex(c => c.name === colName));
            const colors = ['#6366f1', '#10b981', '#f59e0b', '#ec4899', '#8b5cf6', '#06b6d4'];
            const color = colors[cIdx % colors.length];
            return {
              label: colName,
              data: res.rows.map(r => {
                const val = r[colIdx];
                return typeof val === 'number' ? val : parseFloat(String(val)) || 0;
              }),
              backgroundColor: chartCfg.type === 'pie' || chartCfg.type === 'doughnut' ? colors : `${color}33`,
              borderColor: color,
              borderWidth: 2,
              fill: chartCfg.type === 'area',
              tension: 0.3
            };
          });

          resultHtml = `
            <div class="chart-container">
              <div class="chart-header">
                <strong>📊 ${chartCfg.title || 'Data Visualization'}</strong> (${chartCfg.type.toUpperCase()})
              </div>
              <div style="position: relative; height: 320px; width: 100%;">
                <canvas id="${chartId}"></canvas>
              </div>
              <script>
                window.addEventListener('DOMContentLoaded', () => {
                  const ctx = document.getElementById('${chartId}').getContext('2d');
                  new Chart(ctx, {
                    type: '${chartCfg.type === 'area' ? 'line' : chartCfg.type}',
                    data: {
                      labels: ${JSON.stringify(labels)},
                      datasets: ${JSON.stringify(datasets)}
                    },
                    options: {
                      responsive: true,
                      maintainAspectRatio: false,
                      plugins: {
                        legend: { position: 'top' },
                        title: { display: false }
                      }
                    }
                  });
                });
              </script>
            </div>
          `;
        } else {
          // Data Grid Table
          const maxRows = Math.min(res.rows.length, 250);
          let rowsHtml = '';
          for (let i = 0; i < maxRows; i++) {
            const row = res.rows[i];
            rowsHtml += '<tr>';
            rowsHtml += `<td class="row-num">${i + 1}</td>`;
            res.columns.forEach((_col, colIdx) => {
              const val = row[colIdx];
              const displayVal = val === null || val === undefined ? '<span class="null-badge">NULL</span>' : String(val);
              rowsHtml += `<td>${displayVal}</td>`;
            });
            rowsHtml += '</tr>';
          }

          resultHtml = `
            <div class="result-table-wrapper">
              <div class="result-meta">
                <span>Rows: <strong>${res.rows.length}</strong></span>
                <span>Execution: <strong>${(sqlCell.executionDurationMs || res.executionTimeMs || 0).toFixed(1)} ms</strong></span>
                ${res.rows.length > 250 ? '<span class="truncated-badge">Showing first 250 rows</span>' : ''}
              </div>
              <div class="table-scroll">
                <table>
                  <thead>
                    <tr>
                      <th class="row-num">#</th>
                      ${res.columns.map(c => `<th>${c.name} <span class="col-type">${c.dataType || ''}</span></th>`).join('')}
                    </tr>
                  </thead>
                  <tbody>
                    ${rowsHtml}
                  </tbody>
                </table>
              </div>
            </div>
          `;
        }
      }

      return `
        <div class="notebook-cell sql-cell">
          <div class="cell-sql-header">
            <span class="cell-badge">SQL</span>
            <span class="cell-conn">${connectionName}</span>
          </div>
          <pre class="sql-code"><code>${sqlCell.sql}</code></pre>
          ${resultHtml}
        </div>
      `;
    }
  }).join('\n');

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${doc.title} - FugDB Interactive Report</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    :root {
      --bg: #0f172a;
      --card-bg: #1e293b;
      --text: #f8fafc;
      --text-muted: #94a3b8;
      --border: #334155;
      --primary: #6366f1;
      --primary-light: #818cf8;
      --accent: #10b981;
      --danger: #ef4444;
    }
    @media (prefers-color-scheme: light) {
      :root {
        --bg: #f8fafc;
        --card-bg: #ffffff;
        --text: #0f172a;
        --text-muted: #64748b;
        --border: #e2e8f0;
      }
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background-color: var(--bg);
      color: var(--text);
      line-height: 1.6;
      padding: 2rem 1rem;
    }
    .container {
      max-width: 960px;
      margin: 0 auto;
    }
    header.report-header {
      border-bottom: 2px solid var(--primary);
      padding-bottom: 1.5rem;
      margin-bottom: 2rem;
    }
    .report-title {
      font-size: 1.85rem;
      font-weight: 800;
      color: var(--text);
      display: flex;
      align-items: center;
      gap: 0.75rem;
    }
    .report-meta {
      display: flex;
      flex-wrap: wrap;
      gap: 1.5rem;
      margin-top: 0.75rem;
      font-size: 0.85rem;
      color: var(--text-muted);
    }
    .badge {
      display: inline-block;
      padding: 0.2rem 0.6rem;
      border-radius: 9999px;
      background: rgba(99, 102, 241, 0.15);
      color: var(--primary-light);
      font-weight: 600;
      border: 1px solid rgba(99, 102, 241, 0.3);
    }
    .notebook-cell {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 1.25rem;
      margin-bottom: 1.5rem;
      box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    }
    .markdown-cell {
      border-left: 4px solid var(--accent);
    }
    .sql-cell {
      border-left: 4px solid var(--primary);
    }
    .cell-sql-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 0.75rem;
      font-size: 0.75rem;
      font-family: monospace;
    }
    .cell-badge {
      background: var(--primary);
      color: white;
      padding: 0.15rem 0.5rem;
      border-radius: 4px;
      font-weight: bold;
    }
    .cell-conn {
      color: var(--text-muted);
    }
    pre.sql-code {
      background: rgba(0, 0, 0, 0.25);
      border: 1px solid var(--border);
      padding: 1rem;
      border-radius: 8px;
      font-family: ui-monospace, "SF Mono", Monaco, Consolas, monospace;
      font-size: 0.85rem;
      overflow-x: auto;
      margin-bottom: 1rem;
      color: var(--primary-light);
    }
    .result-table-wrapper {
      margin-top: 1rem;
      border: 1px solid var(--border);
      border-radius: 8px;
      overflow: hidden;
    }
    .result-meta {
      padding: 0.5rem 0.75rem;
      background: rgba(0, 0, 0, 0.1);
      border-bottom: 1px solid var(--border);
      font-size: 0.75rem;
      display: flex;
      gap: 1rem;
      color: var(--text-muted);
    }
    .table-scroll {
      max-height: 400px;
      overflow: auto;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.8rem;
      font-family: monospace;
      text-align: left;
    }
    th, td {
      padding: 0.5rem 0.75rem;
      border-bottom: 1px solid var(--border);
      white-space: nowrap;
    }
    th {
      background: rgba(0, 0, 0, 0.15);
      font-weight: 700;
      position: sticky;
      top: 0;
      z-index: 2;
    }
    th .col-type {
      font-size: 0.65rem;
      opacity: 0.6;
      font-weight: normal;
    }
    td.row-num, th.row-num {
      width: 40px;
      color: var(--text-muted);
      text-align: center;
    }
    .null-badge {
      opacity: 0.5;
      font-style: italic;
    }
    .chart-container {
      margin-top: 1rem;
      padding: 1rem;
      background: rgba(0, 0, 0, 0.1);
      border-radius: 8px;
      border: 1px solid var(--border);
    }
    .chart-header {
      font-size: 0.85rem;
      margin-bottom: 0.75rem;
    }
    .error-banner {
      background: rgba(239, 68, 68, 0.15);
      border: 1px solid var(--danger);
      color: var(--danger);
      padding: 0.75rem 1rem;
      border-radius: 8px;
      font-size: 0.85rem;
      margin-top: 0.75rem;
    }
    footer {
      text-align: center;
      margin-top: 3rem;
      font-size: 0.8rem;
      color: var(--text-muted);
    }
    @media print {
      body { background: white; color: black; padding: 0; }
      .notebook-cell { page-break-inside: avoid; box-shadow: none; border: 1px solid #ccc; }
      pre.sql-code { background: #f5f5f5; color: black; }
    }
  </style>
</head>
<body>
  <div class="container">
    <header class="report-header">
      <div class="report-title">
        <span>📓</span>
        <span>${doc.title}</span>
        <span class="badge">.fugpad report</span>
      </div>
      <div class="report-meta">
        <span>Database: <strong>${connectionName}</strong></span>
        <span>Generated: <strong>${dateStr}</strong></span>
        <span>Cells: <strong>${doc.cells.length}</strong></span>
      </div>
      ${doc.description ? `<p style="margin-top: 0.75rem; font-size: 0.9rem; color: var(--text-muted);">${doc.description}</p>` : ''}
    </header>

    <main>
      ${cellsHtml}
    </main>

    <footer>
      Generated with <strong>FugDB</strong> • High-Performance Multi-Database GUI
    </footer>
  </div>
</body>
</html>`;
}

/**
 * Generate GitHub Flavored Markdown document from Notebook
 */
export function generateMarkdownExport(doc: FugpadDocument): string {
  let md = `# 📓 ${doc.title}\n\n`;
  if (doc.description) {
    md += `> ${doc.description}\n\n`;
  }
  md += `*Generated: ${new Date(doc.updatedAt || Date.now()).toLocaleString()}*\n\n---\n\n`;

  doc.cells.forEach((cell, idx) => {
    if (cell.type === 'markdown') {
      const m = cell as MarkdownNotebookCell;
      md += `${m.content.trim()}\n\n`;
    } else {
      const s = cell as SqlNotebookCell;
      md += `### Query #${idx + 1}\n\n`;
      md += `\`\`\`sql\n${s.sql.trim()}\n\`\`\`\n\n`;

      if (s.errorMessage) {
        md += `> ⚠️ **Error**: ${s.errorMessage}\n\n`;
      } else if (s.result && s.result.columns && s.result.columns.length > 0) {
        const res = s.result;
        md += `| ${res.columns.map(c => c.name).join(' | ')} |\n`;
        md += `| ${res.columns.map(() => '---').join(' | ')} |\n`;
        const limit = Math.min(res.rows.length, 50);
        for (let i = 0; i < limit; i++) {
          const row = res.rows[i];
          const vals = res.columns.map((_c, cIdx) => {
            const v = row[cIdx];
            return v === null || v === undefined ? '*NULL*' : String(v).replace(/\|/g, '\\|');
          });
          md += `| ${vals.join(' | ')} |\n`;
        }
        if (res.rows.length > 50) {
          md += `\n*Showing 50 of ${res.rows.length} rows*\n\n`;
        } else {
          md += `\n*Total rows: ${res.rows.length}* (${(s.executionDurationMs || res.executionTimeMs || 0).toFixed(1)} ms)\n\n`;
        }
      }
    }
    md += `---\n\n`;
  });

  return md;
}
