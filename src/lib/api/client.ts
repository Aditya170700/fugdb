import { invoke } from '@tauri-apps/api/core';
import type { 
  ConnectionConfig, 
  QueryResult, 
  SchemaTree, 
  RelationEdge, 
  TransferProgressEvent 
} from './types';

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const api = {
  // Connection commands
  async testConnection(config: ConnectionConfig): Promise<{ success: boolean; message: string; latencyMs: number }> {
    if (isTauri) {
      return await invoke('test_connection', { config });
    }
    // Browser Mock for instant dev preview
    await new Promise(r => setTimeout(r, 200));
    return { success: true, message: 'Connected successfully (Mock Mode)', latencyMs: 14 };
  },

  async connect(config: ConnectionConfig): Promise<void> {
    if (isTauri) {
      return await invoke('connect_database', { config });
    }
    console.log('[Mock API] Connected to:', config.name);
  },

  async disconnect(connectionId: string): Promise<void> {
    if (isTauri) {
      return await invoke('disconnect_database', { connectionId });
    }
    console.log('[Mock API] Disconnected:', connectionId);
  },

  // Schema commands
  async fetchSchema(connectionId: string): Promise<SchemaTree> {
    if (isTauri) {
      return await invoke('fetch_schema_tree', { connectionId });
    }
    return {
      databases: ['fugdb_test'],
      currentDatabase: 'fugdb_test',
      tables: [
        { schema: 'public', name: 'users', tableType: 'table', rowCountEstimate: 1200 },
        { schema: 'public', name: 'orders', tableType: 'table', rowCountEstimate: 45000 },
        { schema: 'public', name: 'order_items', tableType: 'table', rowCountEstimate: 120000 },
        { schema: 'public', name: 'products', tableType: 'table', rowCountEstimate: 340 },
        { schema: 'public', name: 'audit_logs', tableType: 'table', rowCountEstimate: 89000 },
      ]
    };
  },

  async generateErdMetadata(connectionId: string): Promise<RelationEdge[]> {
    if (isTauri) {
      return await invoke('generate_erd_metadata', { connectionId });
    }
    return [
      { id: 'e1', fromTable: 'orders', fromColumn: 'user_id', toTable: 'users', toColumn: 'id' },
      { id: 'e2', fromTable: 'order_items', fromColumn: 'order_id', toTable: 'orders', toColumn: 'id' },
      { id: 'e3', fromTable: 'order_items', fromColumn: 'product_id', toTable: 'products', toColumn: 'id' }
    ];
  },

  // Query commands
  async executeQuery(connectionId: string, sql: string, pageSize?: number, offset?: number): Promise<QueryResult> {
    if (isTauri) {
      return await invoke('execute_query', { connectionId, sql, pageSize, offset });
    }
    
    // Realistic browser mock result
    await new Promise(r => setTimeout(r, 60));
    return {
      columns: [
        { name: 'id', dataType: 'int4', isPrimaryKey: true, isForeignKey: false, nullable: false },
        { name: 'name', dataType: 'varchar', isPrimaryKey: false, isForeignKey: false, nullable: false },
        { name: 'email', dataType: 'varchar', isPrimaryKey: false, isForeignKey: false, nullable: false },
        { name: 'role', dataType: 'varchar', isPrimaryKey: false, isForeignKey: false, nullable: true },
        { name: 'is_active', dataType: 'bool', isPrimaryKey: false, isForeignKey: false, nullable: false },
        { name: 'created_at', dataType: 'timestamptz', isPrimaryKey: false, isForeignKey: false, nullable: false },
      ],
      rows: [
        [1, 'Aditya Pratama', 'aditya@fugdb.dev', 'admin', true, '2026-09-30T10:15:00Z'],
        [2, 'Jane Doe', 'jane@example.com', 'developer', true, '2026-09-30T10:20:00Z'],
        [3, 'John Smith', 'john@example.com', 'qa', false, '2026-09-30T10:25:00Z'],
        [4, 'Alice Wonder', 'alice@matrix.io', 'analyst', true, '2026-09-30T11:00:00Z'],
        [5, 'Bob Builder', 'bob@construct.dev', 'member', true, '2026-09-30T11:30:00Z'],
      ],
      affectedRows: 5,
      executionTimeMs: 4.2,
      totalRows: 5,
    };
  },

  // Mock data batch generator
  async generateMockBatch(connectionId: string, table: string, count: number): Promise<number> {
    if (isTauri) {
      return await invoke('generate_mock_batch', { connectionId, table, count });
    }
    await new Promise(r => setTimeout(r, 350));
    return count;
  }
};
