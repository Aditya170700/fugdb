export interface SqlRiskAssessment {
  isDangerous: boolean;
  riskLevel: 'critical' | 'high' | 'medium' | 'none';
  reasons: string[];
  destructiveCommands: string[];
  cleanSql: string;
}

export function splitSqlStatements(sql: string): string[] {
  const stmts: string[] = [];
  let current = '';
  let inString = false;
  let quoteChar = '';

  for (let i = 0; i < sql.length; i++) {
    const char = sql[i];
    const prevChar = i > 0 ? sql[i - 1] : '';

    if ((char === "'" || char === '"') && prevChar !== '\\') {
      if (!inString) {
        inString = true;
        quoteChar = char;
      } else if (quoteChar === char) {
        inString = false;
      }
    }

    if (char === ';' && !inString) {
      if (current.trim()) {
        stmts.push(current.trim());
      }
      current = '';
    } else {
      current += char;
    }
  }

  if (current.trim()) {
    stmts.push(current.trim());
  }

  return stmts;
}

export function assessSqlRisk(sql: string): SqlRiskAssessment {
  if (!sql || !sql.trim()) {
    return { isDangerous: false, riskLevel: 'none', reasons: [], destructiveCommands: [], cleanSql: '' };
  }

  // 1. Strip comments (-- single line and /* multiline */)
  const cleanSql = sql
    .replace(/--.*$/gm, ' ')
    .replace(/\/\*[\s\S]*?\*\//g, ' ')
    .trim();

  // 2. Split statements by semicolon
  const statements = splitSqlStatements(cleanSql);
  const reasons: string[] = [];
  const destructiveCommands: string[] = [];
  let riskLevel: 'critical' | 'high' | 'medium' | 'none' = 'none';

  for (const stmt of statements) {
    const trimmed = stmt.trim();
    if (!trimmed) continue;

    // Strip string literals for keyword regex testing
    const noStrings = trimmed.replace(/'(?:''|[^'])*'/g, "''").replace(/"(?:""|[^"])*"/g, '""');

    // A. DROP DATABASE / SCHEMA / TABLE / VIEW / FUNCTION / TRIGGER
    const dropMatch = noStrings.match(/\bDROP\s+(DATABASE|SCHEMA|TABLE|VIEW|PROCEDURE|FUNCTION|TRIGGER|INDEX)\s+([a-zA-Z0-9_."`\[\]]+)/i);
    if (dropMatch) {
      const type = dropMatch[1].toUpperCase();
      const target = dropMatch[2];
      reasons.push(`Destructive ${type} deletion: DROP ${type} ${target}`);
      destructiveCommands.push(`DROP ${type} ${target}`);
      riskLevel = 'critical';
    }

    // B. TRUNCATE TABLE
    const truncateMatch = noStrings.match(/\bTRUNCATE\s+(?:TABLE\s+)?([a-zA-Z0-9_."`\[\]]+)/i);
    if (truncateMatch) {
      const target = truncateMatch[1];
      reasons.push(`Instant table data purge: TRUNCATE ${target}`);
      destructiveCommands.push(`TRUNCATE ${target}`);
      riskLevel = 'critical';
    }

    // C. DELETE without WHERE clause
    const isDelete = /\bDELETE\s+FROM\s+/i.test(noStrings) || /^\s*DELETE\s+[a-zA-Z0-9_."`\[\]]+/i.test(noStrings);
    if (isDelete) {
      const hasWhere = /\bWHERE\b/i.test(noStrings);
      if (!hasWhere) {
        reasons.push(`Unconstrained DELETE: Entire table data will be wiped (missing WHERE clause)`);
        destructiveCommands.push('DELETE (NO WHERE)');
        riskLevel = 'critical';
      } else {
        reasons.push(`DELETE operation targeting matching records`);
        destructiveCommands.push('DELETE (WITH WHERE)');
        if (riskLevel !== 'critical') riskLevel = 'high';
      }
    }

    // D. UPDATE without WHERE clause
    const isUpdate = /\bUPDATE\s+([a-zA-Z0-9_."`\[\]]+)\s+SET\b/i.test(noStrings);
    if (isUpdate) {
      const hasWhere = /\bWHERE\b/i.test(noStrings);
      if (!hasWhere) {
        reasons.push(`Unconstrained UPDATE: Every single row in the table will be modified (missing WHERE clause)`);
        destructiveCommands.push('UPDATE (NO WHERE)');
        riskLevel = 'critical';
      }
    }

    // E. ALTER TABLE DROP COLUMN / CONSTRAINT
    const alterDropMatch = noStrings.match(/\bALTER\s+TABLE\s+([a-zA-Z0-9_."`\[\]]+)\s+DROP\s+(?:COLUMN|CONSTRAINT|KEY)\s+([a-zA-Z0-9_."`\[\]]+)/i);
    if (alterDropMatch) {
      reasons.push(`Schema structure modification: Dropping column/constraint from ${alterDropMatch[1]}`);
      destructiveCommands.push(`ALTER TABLE DROP`);
      if (riskLevel !== 'critical') riskLevel = 'high';
    }
  }

  return {
    isDangerous: reasons.length > 0,
    riskLevel,
    reasons,
    destructiveCommands,
    cleanSql
  };
}
