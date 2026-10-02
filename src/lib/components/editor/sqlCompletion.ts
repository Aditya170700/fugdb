import { 
  sql, 
  PostgreSQL, 
  MySQL, 
  MSSQL, 
  SQLite, 
  StandardSQL,
  type SQLDialect,
  type SQLNamespace 
} from '@codemirror/lang-sql';
import { 
  type Completion, 
  type CompletionContext, 
  type CompletionResult,
  snippetCompletion
} from '@codemirror/autocomplete';
import type { DriverType, SchemaTree } from '$lib/api/types';

export function getSqlDialect(driver?: DriverType): SQLDialect {
  switch (driver) {
    case 'postgres':
      return PostgreSQL;
    case 'mysql':
      return MySQL;
    case 'mssql':
      return MSSQL;
    case 'sqlite':
      return SQLite;
    default:
      return StandardSQL;
  }
}

export function buildSchemaMap(schemaTree?: SchemaTree): {
  schema: SQLNamespace;
  tables: Completion[];
} {
  if (!schemaTree || !schemaTree.tables || schemaTree.tables.length === 0) {
    return { schema: {}, tables: [] };
  }

  const schemaMap: Record<string, any> = {};
  const tableCompletions: Completion[] = [];

  for (const table of schemaTree.tables) {
    const cols = (table.columns || []).map(col => {
      const isPk = col.isPrimaryKey;
      const isFk = col.isForeignKey;
      const detail = `${col.dataType}${isPk ? ' • PK' : ''}${isFk ? ' • FK' : ''}${col.nullable ? ' (null)' : ''}`;
      
      return {
        label: col.name,
        type: isPk ? 'keyword' : isFk ? 'class' : 'property',
        detail,
        info: `Column: ${col.name}\nTable: ${table.name}\nType: ${col.dataType}\nPrimary Key: ${isPk ? 'Yes' : 'No'}\nForeign Key: ${isFk ? 'Yes' : 'No'}\nNullable: ${col.nullable ? 'Yes' : 'No'}`,
        boost: isPk ? 95 : isFk ? 85 : 60
      } satisfies Completion;
    });

    tableCompletions.push({
      label: table.name,
      type: table.tableType === 'view' ? 'interface' : 'type',
      detail: `${table.schema || 'public'} • ${table.tableType} (${(table.columns || []).length} cols)`,
      info: `Table: ${table.name}\nSchema: ${table.schema || 'default'}\nType: ${table.tableType}\nColumns: ${(table.columns || []).map(c => c.name).join(', ')}`,
      boost: 90
    });

    // Top-level table access: `users.id`
    schemaMap[table.name] = cols;

    // Schema-qualified table access: `public.users.id`
    if (table.schema) {
      if (!schemaMap[table.schema]) {
        schemaMap[table.schema] = {};
      }
      schemaMap[table.schema][table.name] = cols;
      schemaMap[`${table.schema}.${table.name}`] = cols;
    }
  }

  return {
    schema: schemaMap,
    tables: tableCompletions
  };
}

export function createJoinCompletionSource(schemaTree?: SchemaTree) {
  return (context: CompletionContext): CompletionResult | null => {
    if (!schemaTree || !schemaTree.tables || schemaTree.tables.length === 0) return null;

    const line = context.state.doc.lineAt(context.pos);
    const textBefore = line.text.slice(0, context.pos - line.from);

    const joinMatch = textBefore.match(/\b(join|inner\s+join|left\s+join|right\s+join|full\s+join|cross\s+join)\s+([a-zA-Z0-9_]*)$/i);
    if (!joinMatch) return null;

    const typedWord = joinMatch[2];
    const fromPos = context.pos - typedWord.length;

    // Extract table names mentioned earlier in the query
    const fullDoc = context.state.doc.toString();
    const tableRegex = /\b(?:from|join)\s+([a-zA-Z0-9_]+)(?:\s+(?:as\s+)?([a-zA-Z0-9_]+))?/gi;
    const referencedTables: Array<{ tableName: string; alias: string }> = [];
    let match: RegExpExecArray | null;

    while ((match = tableRegex.exec(fullDoc)) !== null) {
      if (match.index >= context.pos) break;
      const tbl = match[1];
      const alias = match[2] || tbl;
      if (tbl.toLowerCase() !== 'join' && tbl.toLowerCase() !== 'from') {
        referencedTables.push({ tableName: tbl, alias });
      }
    }

    const options: Completion[] = [];

    // 1. Suggest from known schemaTree relations
    if (schemaTree.relations && schemaTree.relations.length > 0) {
      for (const rel of schemaTree.relations) {
        for (const ref of referencedTables) {
          if (ref.tableName.toLowerCase() === rel.fromTable.toLowerCase()) {
            const joinSnippet = `${rel.toTable} ON ${rel.toTable}.${rel.toColumn} = ${ref.alias}.${rel.fromColumn}`;
            options.push(
              snippetCompletion(joinSnippet, {
                label: `${rel.toTable} ON ${rel.toTable}.${rel.toColumn} = ${ref.alias}.${rel.fromColumn}`,
                type: 'text',
                detail: `FK: ${rel.fromTable}.${rel.fromColumn} ➔ ${rel.toTable}.${rel.toColumn}`,
                boost: 99
              })
            );
          }
          if (ref.tableName.toLowerCase() === rel.toTable.toLowerCase()) {
            const joinSnippet = `${rel.fromTable} ON ${rel.fromTable}.${rel.fromColumn} = ${ref.alias}.${rel.toColumn}`;
            options.push(
              snippetCompletion(joinSnippet, {
                label: `${rel.fromTable} ON ${rel.fromTable}.${rel.fromColumn} = ${ref.alias}.${rel.toColumn}`,
                type: 'text',
                detail: `FK: ${rel.fromTable}.${rel.fromColumn} ➔ ${rel.toTable}.${rel.toColumn}`,
                boost: 98
              })
            );
          }
        }
      }
    }

    // 2. Suggest by conventional column names (e.g. `users.id` matching `orders.user_id`)
    for (const ref of referencedTables) {
      const refTableObj = schemaTree.tables.find(t => t.name.toLowerCase() === ref.tableName.toLowerCase());
      if (!refTableObj) continue;

      for (const otherTable of schemaTree.tables) {
        if (otherTable.name.toLowerCase() === ref.tableName.toLowerCase()) continue;

        const singularRef = ref.tableName.replace(/s$/i, '');
        const matchingCol = (otherTable.columns || []).find(c => 
          c.name.toLowerCase() === `${singularRef.toLowerCase()}_id` || 
          c.name.toLowerCase() === `${ref.tableName.toLowerCase()}_id`
        );

        if (matchingCol) {
          const joinSnippet = `${otherTable.name} ON ${otherTable.name}.${matchingCol.name} = ${ref.alias}.id`;
          if (!options.some(o => o.label === joinSnippet)) {
            options.push(
              snippetCompletion(joinSnippet, {
                label: joinSnippet,
                type: 'text',
                detail: `Smart Join: ${otherTable.name}.${matchingCol.name} = ${ref.alias}.id`,
                boost: 90
              })
            );
          }
        }
      }
    }

    // 3. Fallback: Suggest all tables
    for (const table of schemaTree.tables) {
      if (!options.some(o => o.label.startsWith(table.name))) {
        options.push({
          label: table.name,
          type: 'type',
          detail: `Table (${table.columns?.length || 0} cols)`,
          boost: 70
        });
      }
    }

    return {
      from: fromPos,
      options: options.filter(o => !typedWord || o.label.toLowerCase().includes(typedWord.toLowerCase())),
      validFor: /^[a-zA-Z0-9_]*$/
    };
  };
}

export function createSqlLanguageSupport(driver?: DriverType, schemaTree?: SchemaTree) {
  const dialect = getSqlDialect(driver);
  const { schema, tables } = buildSchemaMap(schemaTree);
  const joinSource = createJoinCompletionSource(schemaTree);

  const langSupport = sql({
    dialect,
    schema,
    tables,
    upperCaseKeywords: true
  });

  return [
    langSupport,
    dialect.language.data.of({
      autocomplete: joinSource
    })
  ];
}
