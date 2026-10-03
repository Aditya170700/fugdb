export type DriverType = 'postgres' | 'mysql' | 'sqlite' | 'mssql' | 'duckdb' | 'redis';

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
  sshAuthType?: 'key' | 'password' | 'agent';
  sshPassword?: string;
  sshKeyPath?: string;
  sshKeyPassphrase?: string;
  sslMode?: 'disable' | 'prefer' | 'require';
  useKeyring?: boolean;
  savePasswordToKeyring?: boolean;
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
  | { type: 'csv'; delimiter?: string; hasHeader?: boolean; quoteChar?: string }
  | { type: 'tsv'; hasHeader?: boolean }
  | { type: 'json'; isNdjson?: boolean; pretty?: boolean }
  | { type: 'excel'; sheetName?: string }
  | { type: 'sqlDump'; includeDdl?: boolean; batchSize?: number };

export type ConflictStrategy = 
  | 'fail' 
  | 'ignore' 
  | { upsert: { matchColumns: string[] } };

export interface ExportJobRequest {
  connectionId: string;
  schema?: string;
  table?: string;
  query?: string;
  targetPath: string;
  format: TransferFormat;
}

export interface ImportJobRequest {
  connectionId: string;
  schema?: string;
  table: string;
  sourcePath: string;
  format: TransferFormat;
  conflictStrategy?: ConflictStrategy;
  createTableIfMissing?: boolean;
}

export interface DbToDbTransferRequest {
  sourceConnectionId: string;
  sourceSchema?: string;
  sourceTable?: string;
  sourceQuery?: string;
  targetConnectionId: string;
  targetSchema?: string;
  targetTable: string;
  conflictStrategy?: ConflictStrategy;
  createTableIfMissing?: boolean;
  truncateTargetFirst?: boolean;
  batchSize?: number;
}

export interface TransferProgressEvent {
  jobId: string;
  rowsProcessed: number;
  totalRowsEstimated?: number;
  bytesProcessed: number;
  rowsPerSecond: number;
  estimatedSecondsRemaining?: number;
  percentage?: number;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  message?: string;
  errorMessage?: string;
}

export interface FileInspectionResult {
  detectedFormat: string;
  delimiter?: string;
  hasHeader: boolean;
  columns: string[];
  sampleRows: any[][];
  totalBytes: number;
  sheetNames?: string[];
}

// QA Smart Mock Data Generator Types
export interface ColumnMockRule {
  columnName: string;
  dataType: string;
  isPrimaryKey: boolean;
  isForeignKey: boolean;
  nullable: boolean;
  include: boolean;
  generatorType: string;
  nullPercentage: number;
  customOptions?: string;
  fkTargetTable?: string;
  fkTargetColumn?: string;
  sampleFkValues?: any[];
}

export interface TableMockInspection {
  tableName: string;
  columns: ColumnMockRule[];
}

export interface MockBatchResult {
  insertedRows: number;
  executionTimeMs: number;
  chunksCount: number;
  tableName: string;
}

// AI Copilot & NL-to-SQL Types
export type AiProvider = 'ollama' | 'openai' | 'gemini' | 'anthropic' | 'deepseek' | 'custom';

export interface AiProviderConfig {
  provider: AiProvider;
  model: string;
  apiKey?: string;
  endpoint?: string;
  temperature?: number;
}

export interface AiSqlResponse {
  sql: string;
  explanation: string;
  tablesUsed: string[];
  dialect: string;
  modelUsed: string;
  executionTimeMs: number;
}

export interface OllamaModelInfo {
  name: string;
  size: number;
  modifiedAt: string;
}

export interface ExplainResult {
  rawPlan: string;
  jsonPlan?: any;
  queryResult: QueryResult;
  executionTimeMs?: number;
  planningTimeMs?: number;
  dialect: string;
  hasAnalyze: boolean;
}

// 5.1 Server Monitor & Health Stats Types
export interface ServerProcess {
  pid: string;
  user: string;
  database: string;
  clientAddr?: string;
  applicationName?: string;
  state: string;
  query?: string;
  durationSeconds: number;
  waitEvent?: string;
  blockedBy?: string;
  startedAt?: string;
}

export interface ServerHealthStats {
  activeConnections: number;
  idleConnections: number;
  totalConnections: number;
  maxConnections?: number;
  uptimeSeconds?: number;
  version: string;
  processes: ServerProcess[];
  summaryCounts: Record<string, number>;
}

// 5.2 Schema & Data Diff Sync Types
export type DiffAction = 'create' | 'drop' | 'alter' | 'identical';

export interface ColumnDiff {
  name: string;
  action: DiffAction;
  sourceType?: string;
  targetType?: string;
  sourceNullable?: boolean;
  targetNullable?: boolean;
  sourcePk?: boolean;
  targetPk?: boolean;
  diffReason?: string;
}

export interface TableDiff {
  tableName: string;
  action: DiffAction;
  sourceRowCount?: number;
  targetRowCount?: number;
  columns: ColumnDiff[];
  syncSql: string;
}

export interface SchemaDiffResult {
  sourceConnectionId: string;
  targetConnectionId: string;
  sourceDriver: string;
  targetDriver: string;
  totalSourceTables: number;
  totalTargetTables: number;
  tablesToCreate: number;
  tablesToDrop: number;
  tablesToAlter: number;
  tablesIdentical: number;
  tableDiffs: TableDiff[];
  fullMigrationSql: string;
  executionTimeMs: number;
}

// 5.3 Interactive SQL Scratchpad Notebook (.fugpad) Types
export type NotebookCellType = 'markdown' | 'sql';

export interface NotebookChartConfig {
  type: 'bar' | 'line' | 'area' | 'pie' | 'doughnut';
  xAxisColumn: string;
  yAxisColumns: string[];
  title?: string;
  aggregation?: 'none' | 'sum' | 'avg' | 'count';
}

export interface MarkdownNotebookCell {
  id: string;
  type: 'markdown';
  content: string;
  isEditing?: boolean;
}

export interface SqlNotebookCell {
  id: string;
  type: 'sql';
  sql: string;
  connectionId?: string;
  isExecuting?: boolean;
  result?: QueryResult;
  errorMessage?: string;
  displayMode: 'grid' | 'chart' | 'json';
  chartConfig?: NotebookChartConfig;
  collapsed?: boolean;
  executionDurationMs?: number;
}

export type NotebookCell = MarkdownNotebookCell | SqlNotebookCell;

export interface FugpadDocument {
  version: 1;
  id: string;
  title: string;
  description?: string;
  defaultConnectionId?: string;
  createdAt: number;
  updatedAt: number;
  cells: NotebookCell[];
}

// 5.4 Redis & Key-Value Polyglot Inspector Types
export type RedisKeyType = 'string' | 'hash' | 'list' | 'set' | 'zset' | 'stream' | 'none';

export interface RedisKeyItem {
  key: string;
  keyType: string;
  ttl: number; // -1 = persist, -2 = does not exist, >0 = seconds
  size?: number;
  memoryBytes?: number;
}

export interface RedisScanResult {
  cursor: number;
  keys: RedisKeyItem[];
  totalKeys: number;
  dbIndex: number;
}

export interface RedisZSetMember {
  member: string;
  score: number;
}

export interface RedisStreamEntry {
  id: string;
  fields: Record<string, string>;
}

export interface RedisKeyDetail {
  key: string;
  keyType: string;
  ttl: number;
  memoryUsageBytes?: number;
  valueString?: string | null;
  valueHash?: Record<string, string> | null;
  valueList?: string[] | null;
  valueSet?: string[] | null;
  valueZset?: RedisZSetMember[] | null;
  valueStream?: RedisStreamEntry[] | null;
}

export interface RedisServerInfo {
  version: string;
  os: string;
  uptimeSeconds: number;
  connectedClients: number;
  usedMemoryHuman: string;
  usedMemoryPeakHuman: string;
  totalKeys: number;
  rawInfo: Record<string, string>;
}

export interface RedisCliResponse {
  command: string;
  response: string;
  responseType: string;
  durationMs: number;
}

// 5.5 Scheduled Automations & Backups Types
export type ScheduleJobType = 'query_export' | 'database_backup';
export type ScheduleRunStatus = 'pending' | 'running' | 'success' | 'failed';

export interface ScheduledQueryConfig {
  sql: string;
  format: string; // "csv" | "json" | "tsv" | "excel" | "ndjson"
  target_dir: string;
  filename_pattern: string;
}

export interface ScheduledBackupConfig {
  tables: string[];
  include_schema: boolean;
  include_data: boolean;
  target_dir: string;
  filename_pattern: string;
}

export interface ScheduleJob {
  id: string;
  name: string;
  description?: string;
  enabled: boolean;
  job_type: ScheduleJobType;
  connection_id: string;
  database?: string;
  cron_expression: string;
  frequency_display: string;
  query_config?: ScheduledQueryConfig;
  backup_config?: ScheduledBackupConfig;
  created_at: number;
  updated_at: number;
  last_run_at?: number;
  last_run_status?: ScheduleRunStatus;
  last_run_duration_ms?: number;
  last_run_error?: string;
  last_run_file?: string;
  last_run_rows?: number;
  last_run_bytes?: number;
  next_run_at?: number;
}

export interface ScheduleLog {
  id: string;
  job_id: string;
  job_name: string;
  started_at: number;
  completed_at?: number;
  duration_ms?: number;
  status: ScheduleRunStatus;
  file_path?: string;
  rows_processed?: number;
  bytes_written?: number;
  error_message?: string;
}
