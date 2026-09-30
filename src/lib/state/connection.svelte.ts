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
      port: 5433,
      database: 'fugdb_test',
      username: 'fugdb_user',
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
    },
    {
      id: 'conn-prod-analytics',
      name: 'Production Warehouse',
      driver: 'postgres',
      environment: 'production',
      host: 'db.prod.company.com',
      port: 5432,
      database: 'analytics',
      username: 'readonly_analyst',
    }
  ]);

  activeConnectionId = $state<string>('conn-local-pg');
  schemas = $state<Record<string, SchemaTree>>({});
  isLoading = $state<boolean>(false);
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
    if (!this.schemas[id]) {
      await this.loadSchema(id);
    }
  }

  async loadSchema(connectionId: string) {
    this.isLoading = true;
    try {
      const tree = await api.fetchSchema(connectionId);
      this.schemas[connectionId] = tree;
    } catch (err) {
      console.error('Failed to fetch schema:', err);
    } finally {
      this.isLoading = false;
    }
  }

  addConnection(config: ConnectionConfig) {
    this.connections.push(config);
    this.selectConnection(config.id);
  }
}

export const connectionStore = new ConnectionStore();
