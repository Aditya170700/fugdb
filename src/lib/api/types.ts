export type DriverType = 'postgres' | 'mysql' | 'sqlite' | 'mssql' | 'duckdb';

export type Environment = 'dev' | 'staging' | 'production';

export interface ConnectionConfig {
  id: string;
  name: string;
  driver: DriverType;
  environment: Environment;
  host?: string;
  port?: number;
  database?: string;
  username?: string;
  password?: string;
  filePath?: string; // For SQLite / DuckDB
  useSsh?: boolean;
  sshHost?: string;
  sshPort?: number;
  sshUser?: string;
  sshKeyPath?: string;
  sslMode?: 'disable' | 'prefer' | 'require';
}

export interface ColumnMetadata {
  name: string;
  dataType: string;
  isPrimaryKey: boolean;
  isForeignKey: boolean;
  nullable: boolean;
}

export interface QueryResult {
  columns: ColumnMetadata[];
  rows: any[][]; // 2D array of values
  affectedRows: number;
  executionTimeMs: number;
  totalRows?: number;
}

export interface TableItem {
  schema: string;
  name: string;
  tableType: 'table' | 'view' | 'materialized_view';
  rowCountEstimate?: number;
  columns?: ColumnMetadata[];
}

export interface SchemaTree {
  databases: string[];
  currentDatabase: string;
  tables: TableItem[];
  relations?: RelationEdge[];
}

export interface RelationEdge {
  id: string;
  fromTable: string;
  fromColumn: string;
  toTable: string;
  toColumn: string;
}

export type TransferFormat = 
  | { type: 'csv'; delimiter: string; hasHeader: boolean }
  | { type: 'json'; isNdjson: boolean }
  | { type: 'excel'; sheetName?: string }
  | { type: 'parquet' }
  | { type: 'sqlDump'; includeDdl: boolean; batchSize: number };

export interface TransferProgressEvent {
  jobId: string;
  rowsProcessed: number;
  bytesProcessed: number;
  rowsPerSecond: number;
  estimatedSecondsRemaining?: number;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  errorMessage?: string;
}
