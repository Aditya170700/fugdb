import type { ConnectionConfig, SchemaTree } from '../api/types';
import { api } from '../api/client';

export class ConnectionStore {
  connections = $state<ConnectionConfig[]>([
    {
      id: 'conn-local-pg',
      name: 'Local PostgreSQL (Docker)',
      driver: 'postgres',
      environment: 'dev',
      host: 'localhost',
      port: 15432,
      database: 'fugdb_test',
      username: 'fugdb_user',
      password: 'fugdb_password',
    },
    {
      id: 'conn-local-mysql',
      name: 'Local MySQL (Docker)',
      driver: 'mysql',
      environment: 'dev',
      host: 'localhost',
      port: 3306,
      database: 'fugdb_test',
      username: 'fugdb_user',
      password: 'fugdb_password',
    },
  ]);

  activeConnectionId = $state<string>('conn-local-pg');
  schemas = $state<Record<string, SchemaTree>>({});
  isLoading = $state<boolean>(false);
  errorMessage = $state<string | null>(null);
  activeSchemaSearch = $state<string>('');

  activeConnection = $derived(
    this.connections.find(c => c.id === this.activeConnectionId)
  );

  activeSchemaTree = $derived(
    this.activeConnectionId ? this.schemas[this.activeConnectionId] : undefined
  );

  filteredTables = $derived.by(() => {
    const tree = this.activeSchemaTree;
    if (!tree) return [];
    if (!this.activeSchemaSearch.trim()) return tree.tables;
    const q = this.activeSchemaSearch.toLowerCase();
    return tree.tables.filter(t => t.name.toLowerCase().includes(q) || t.schema.toLowerCase().includes(q));
  });

  async selectConnection(id: string) {
    this.activeConnectionId = id;
    this.errorMessage = null;
    await this.loadSchema(id);
  }

  async loadSchema(connectionId: string) {
    const config = this.connections.find(c => c.id === connectionId);
    if (!config) return;

    this.isLoading = true;
    this.errorMessage = null;

    try {
      // 1. Ensure connected in backend
      await api.connect(config);
      // 2. Fetch schema tree
      const tree = await api.fetchSchema(connectionId);
      this.schemas[connectionId] = tree;
    } catch (err: any) {
      console.error('Failed to load schema:', err);
      this.errorMessage = typeof err === 'string' ? err : err?.message || JSON.stringify(err);
    } finally {
      this.isLoading = false;
    }
  }

  async switchDatabase(databaseName: string) {
    const config = this.connections.find(c => c.id === this.activeConnectionId);
    if (!config || config.database === databaseName) return;

    config.database = databaseName;
    await this.loadSchema(this.activeConnectionId);
  }

  async addConnection(config: ConnectionConfig) {
    // Avoid duplicate IDs
    const existingIdx = this.connections.findIndex(c => c.id === config.id);
    if (existingIdx >= 0) {
      this.connections[existingIdx] = config;
    } else {
      this.connections.push(config);
    }
    await this.selectConnection(config.id);
  }
}

export const connectionStore = new ConnectionStore();
