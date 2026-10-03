import { api } from '$lib/api/client';
import type { ExplainResult, AiSqlResponse } from '$lib/api/types';
import { connectionStore } from '$lib/state/connection.svelte';
import { tabsStore } from '$lib/state/tabs.svelte';
import { aiStore } from '$lib/state/ai.svelte';

export interface PlanNode {
  id: string;
  nodeType: string;
  relationName?: string;
  alias?: string;
  indexName?: string;
  totalCost?: number;
  startupCost?: number;
  actualTotalTime?: number;
  actualStartupTime?: number;
  actualRows?: number;
  planRows?: number;
  actualLoops?: number;
  filter?: string;
  indexCond?: string;
  hashCond?: string;
  joinType?: string;
  sharedHitBlocks?: number;
  sharedReadBlocks?: number;
  sharedWrittenBlocks?: number;
  costPercent?: number;
  isBottleneck?: boolean;
  warningMessage?: string;
  rawDetails?: Record<string, any>;
  children: PlanNode[];
}

export interface ParsedPlanTree {
  root: PlanNode;
  maxCost: number;
  totalExecutionTime?: number;
  totalPlanningTime?: number;
  nodeCount: number;
  bottlenecksCount: number;
  seqScansCount: number;
  indexScansCount: number;
}

class ExplainStore {
  isOpen = $state(false);
  isLoading = $state(false);
  isOptimizing = $state(false);
  isAnalyze = $state(true);
  
  tabId = $state('');
  sql = $state('');
  connectionId = $state('');
  driver = $state('postgres');
  
  explainResult = $state<ExplainResult | null>(null);
  parsedTree = $state<ParsedPlanTree | null>(null);
  selectedNode = $state<PlanNode | null>(null);
  
  optimizationResult = $state<AiSqlResponse | null>(null);
  activeTab = $state<'visual' | 'raw' | 'ai'>('visual');
  error = $state<string | null>(null);

  async open(tabId: string, sql: string, analyze: boolean = true) {
    this.tabId = tabId;
    this.sql = sql.trim();
    this.isAnalyze = analyze;
    this.connectionId = connectionStore.activeConnectionId || '';
    this.driver = connectionStore.activeConnection?.driver || 'postgres';
    this.explainResult = null;
    this.parsedTree = null;
    this.selectedNode = null;
    this.optimizationResult = null;
    this.error = null;
    this.activeTab = 'visual';
    this.isOpen = true;

    await this.runExplain(analyze);
  }

  async runExplain(analyze?: boolean) {
    if (analyze !== undefined) {
      this.isAnalyze = analyze;
    }
    if (!this.connectionId || !this.sql) return;

    this.isLoading = true;
    this.error = null;

    try {
      const result = await api.explainQuery(
        this.connectionId,
        this.sql,
        this.isAnalyze,
        this.driver
      );
      this.explainResult = result;
      this.parsedTree = this.parseExplainOutput(result);
      if (this.parsedTree) {
        this.selectedNode = this.parsedTree.root;
      }
    } catch (err: any) {
      this.error = err?.message || String(err);
    } finally {
      this.isLoading = false;
    }
  }

  async runAiOptimize() {
    if (!this.connectionId || !this.sql || !this.explainResult) return;
    this.isOptimizing = true;
    this.error = null;

    try {
      const result = await api.optimizeQueryExplain(
        this.connectionId,
        aiStore.currentConfig,
        this.sql,
        this.explainResult.rawPlan,
        this.driver
      );
      this.optimizationResult = result;
      this.activeTab = 'ai';
    } catch (err: any) {
      this.error = err?.message || String(err);
    } finally {
      this.isOptimizing = false;
    }
  }

  applyOptimizedSql(optimizedSql: string) {
    if (this.tabId) {
      const tab = tabsStore.tabs.find(t => t.id === this.tabId);
      if (tab) {
        tab.sql = optimizedSql;
      }
    }
    this.close();
  }

  close() {
    this.isOpen = false;
  }

  selectNode(node: PlanNode) {
    this.selectedNode = node;
  }

  parseExplainOutput(result: ExplainResult): ParsedPlanTree | null {
    try {
      // 1. PostgreSQL JSON Plan
      if (result.jsonPlan) {
        let rootObj = result.jsonPlan;
        if (Array.isArray(rootObj) && rootObj.length > 0) {
          rootObj = rootObj[0];
        }
        if (rootObj?.Plan) {
          const rawRoot = rootObj.Plan;
          const maxCost = rawRoot['Total Cost'] || 100;
          let nodeCount = 0;
          let bottlenecksCount = 0;
          let seqScansCount = 0;
          let indexScansCount = 0;

          const parsePostgresNode = (raw: any, depth: number = 0): PlanNode => {
            nodeCount++;
            const nodeType = raw['Node Type'] || 'Node';
            const totalCost = raw['Total Cost'];
            const startupCost = raw['Startup Cost'];
            const actualTotalTime = raw['Actual Total Time'];
            const actualStartupTime = raw['Actual Startup Time'];
            const actualRows = raw['Actual Rows'];
            const planRows = raw['Plan Rows'];
            const actualLoops = raw['Actual Loops'] || 1;
            const relationName = raw['Relation Name'];
            const alias = raw['Alias'];
            const indexName = raw['Index Name'];
            const filter = raw['Filter'];
            const indexCond = raw['Index Cond'];
            const hashCond = raw['Hash Cond'];
            const joinType = raw['Join Type'];
            const sharedHitBlocks = raw['Shared Hit Blocks'];
            const sharedReadBlocks = raw['Shared Read Blocks'];
            const sharedWrittenBlocks = raw['Shared Written Blocks'];

            const costPercent = totalCost && maxCost ? Math.min(100, Math.round((totalCost / maxCost) * 100)) : 0;

            let isBottleneck = false;
            let warningMessage: string | undefined;

            if (nodeType.toLowerCase().includes('seq scan')) {
              seqScansCount++;
              if ((actualRows && actualRows > 100) || (planRows && planRows > 100) || costPercent > 30) {
                isBottleneck = true;
                bottlenecksCount++;
                warningMessage = `Sequential Scan on ${relationName || 'table'} (${actualRows ?? planRows ?? 0} rows). Missing index?`;
              }
            } else if (nodeType.toLowerCase().includes('index')) {
              indexScansCount++;
            }

            if (!isBottleneck && actualRows !== undefined && planRows !== undefined && planRows > 0) {
              const ratio = actualRows / planRows;
              if (ratio > 10 || ratio < 0.1) {
                isBottleneck = true;
                bottlenecksCount++;
                warningMessage = `Row estimate misprediction (${actualRows} actual vs ${planRows} estimated). Outdated statistics?`;
              }
            }

            const children: PlanNode[] = [];
            if (Array.isArray(raw.Plans)) {
              for (const childRaw of raw.Plans) {
                children.push(parsePostgresNode(childRaw, depth + 1));
              }
            }

            return {
              id: `pg-node-${nodeCount}-${depth}`,
              nodeType,
              relationName,
              alias,
              indexName,
              totalCost,
              startupCost,
              actualTotalTime,
              actualStartupTime,
              actualRows,
              planRows,
              actualLoops,
              filter,
              indexCond,
              hashCond,
              joinType,
              sharedHitBlocks,
              sharedReadBlocks,
              sharedWrittenBlocks,
              costPercent,
              isBottleneck,
              warningMessage,
              rawDetails: raw,
              children,
            };
          };

          const root = parsePostgresNode(rawRoot);
          return {
            root,
            maxCost,
            totalExecutionTime: result.executionTimeMs ?? rootObj['Execution Time'],
            totalPlanningTime: result.planningTimeMs ?? rootObj['Planning Time'],
            nodeCount,
            bottlenecksCount,
            seqScansCount,
            indexScansCount,
          };
        }

        // MySQL JSON Plan format
        if (rootObj?.query_block) {
          return this.parseMySqlJsonPlan(rootObj);
        }
      }

      // 2. SQLite EXPLAIN QUERY PLAN (Tabular id, parent, notused, detail)
      if (result.queryResult?.rows && result.dialect.toLowerCase().includes('sqlite')) {
        return this.parseSqlitePlan(result.queryResult.rows);
      }

      // 3. Fallback: Parse indented text lines
      return this.parseIndentedTextPlan(result.rawPlan, result.dialect);
    } catch (e) {
      console.error('Failed to parse explain tree:', e);
      return this.parseIndentedTextPlan(result.rawPlan, result.dialect);
    }
  }

  private parseMySqlJsonPlan(rawJson: any): ParsedPlanTree {
    let nodeCount = 0;
    let bottlenecksCount = 0;
    let seqScansCount = 0;
    let indexScansCount = 0;

    const block = rawJson.query_block || {};
    const cost = parseFloat(block.cost_info?.query_cost || '10');

    const children: PlanNode[] = [];

    if (block.table) {
      nodeCount++;
      const tbl = block.table;
      const isAll = tbl.access_type === 'ALL';
      if (isAll) {
        seqScansCount++;
        bottlenecksCount++;
      } else {
        indexScansCount++;
      }

      children.push({
        id: `mysql-node-${nodeCount}`,
        nodeType: isAll ? 'Table Scan (ALL)' : `Index Lookup (${tbl.access_type})`,
        relationName: tbl.table_name,
        indexName: tbl.key,
        planRows: tbl.rows_examined_per_scan,
        filter: tbl.attached_condition,
        costPercent: 100,
        isBottleneck: isAll,
        warningMessage: isAll ? `Full table scan on ${tbl.table_name}. Possible index: ${tbl.possible_keys?.join(', ') || 'None'}` : undefined,
        rawDetails: tbl,
        children: []
      });
    }

    if (Array.isArray(block.nested_loop)) {
      for (const item of block.nested_loop) {
        if (item.table) {
          nodeCount++;
          const tbl = item.table;
          const isAll = tbl.access_type === 'ALL';
          if (isAll) {
            seqScansCount++;
            bottlenecksCount++;
          } else {
            indexScansCount++;
          }
          children.push({
            id: `mysql-node-${nodeCount}`,
            nodeType: isAll ? 'Table Scan (ALL)' : `Index Lookup (${tbl.access_type})`,
            relationName: tbl.table_name,
            indexName: tbl.key,
            planRows: tbl.rows_examined_per_scan,
            filter: tbl.attached_condition,
            costPercent: 80,
            isBottleneck: isAll,
            warningMessage: isAll ? `Full table scan on ${tbl.table_name}` : undefined,
            rawDetails: tbl,
            children: []
          });
        }
      }
    }

    const root: PlanNode = {
      id: 'mysql-root',
      nodeType: 'Query Block',
      totalCost: cost,
      costPercent: 100,
      children,
    };

    return {
      root,
      maxCost: cost,
      nodeCount: nodeCount + 1,
      bottlenecksCount,
      seqScansCount,
      indexScansCount,
    };
  }

  private parseSqlitePlan(rows: any[][]): ParsedPlanTree {
    const nodeMap = new Map<number, PlanNode>();
    let bottlenecksCount = 0;
    let seqScansCount = 0;
    let indexScansCount = 0;

    const root: PlanNode = {
      id: 'sqlite-root',
      nodeType: 'Query Execution',
      costPercent: 100,
      children: [],
    };

    nodeMap.set(0, root);

    // rows: [id, parent, notused, detail]
    for (const r of rows) {
      const id = Number(r[0]);
      const parentId = Number(r[1]);
      const detail = String(r[3] || r[2] || '');

      const isScan = detail.includes('SCAN TABLE') || detail.includes('SCAN ');
      const isSearch = detail.includes('SEARCH TABLE') || detail.includes('SEARCH ');
      const isIndex = isSearch || detail.includes('USING INDEX') || detail.includes('USING COVERING INDEX');

      if (isScan && !isIndex) {
        seqScansCount++;
        bottlenecksCount++;
      } else if (isIndex) {
        indexScansCount++;
      }

      // Extract table name if present: "SCAN TABLE users" or "SEARCH TABLE orders"
      let relationName: string | undefined;
      const matchTbl = detail.match(/(?:SCAN|SEARCH)\s+TABLE\s+([a-zA-Z0-9_]+)/i);
      if (matchTbl) {
        relationName = matchTbl[1];
      }

      // Extract index name if present: "USING INDEX idx_name"
      let indexName: string | undefined;
      const matchIdx = detail.match(/USING\s+(?:COVERING\s+)?INDEX\s+([a-zA-Z0-9_]+)/i);
      if (matchIdx) {
        indexName = matchIdx[1];
      }

      const node: PlanNode = {
        id: `sqlite-${id}`,
        nodeType: isScan && !isIndex ? 'SCAN TABLE' : isIndex ? 'INDEX SEARCH' : 'OPERATION',
        relationName,
        indexName,
        filter: detail,
        isBottleneck: isScan && !isIndex,
        warningMessage: isScan && !isIndex ? `Full scan on ${relationName || 'table'}. Consider creating an index.` : undefined,
        costPercent: 50,
        children: [],
      };

      nodeMap.set(id, node);

      const parentNode = nodeMap.get(parentId) || root;
      parentNode.children.push(node);
    }

    return {
      root,
      maxCost: 100,
      nodeCount: rows.length + 1,
      bottlenecksCount,
      seqScansCount,
      indexScansCount,
    };
  }

  private parseIndentedTextPlan(rawText: string, dialect: string): ParsedPlanTree {
    const lines = rawText.split('\n').filter(l => l.trim().length > 0);
    let bottlenecksCount = 0;
    let seqScansCount = 0;
    let indexScansCount = 0;

    const root: PlanNode = {
      id: 'text-root',
      nodeType: `${dialect} Query Plan`,
      costPercent: 100,
      children: [],
    };

    const stack: { indent: number; node: PlanNode }[] = [{ indent: -1, node: root }];

    lines.forEach((line, index) => {
      const leadingSpaces = line.search(/\S|$/);
      const trimmed = line.trim().replace(/^->\s*|^\|--\s*|^[|\-\s]+/, '');

      const isSeqScan = /seq scan|table scan|scan table|access_type:\s*all/i.test(trimmed);
      const isIndex = /index|search table/i.test(trimmed);

      if (isSeqScan) {
        seqScansCount++;
        bottlenecksCount++;
      } else if (isIndex) {
        indexScansCount++;
      }

      const node: PlanNode = {
        id: `txt-node-${index}`,
        nodeType: trimmed.length > 50 ? trimmed.slice(0, 50) + '...' : trimmed,
        filter: trimmed,
        isBottleneck: isSeqScan,
        warningMessage: isSeqScan ? 'Sequential full table scan detected' : undefined,
        children: [],
      };

      while (stack.length > 1 && stack[stack.length - 1].indent >= leadingSpaces) {
        stack.pop();
      }

      stack[stack.length - 1].node.children.push(node);
      stack.push({ indent: leadingSpaces, node });
    });

    return {
      root,
      maxCost: 100,
      nodeCount: lines.length + 1,
      bottlenecksCount,
      seqScansCount,
      indexScansCount,
    };
  }
}

export const explainStore = new ExplainStore();
