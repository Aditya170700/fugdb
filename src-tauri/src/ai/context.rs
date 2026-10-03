use crate::models::schema::SchemaTree;

pub fn build_schema_context(
    tree: &SchemaTree,
    dialect: &str,
    selected_tables: Option<&[String]>,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("Database Engine Dialect: {}\n", dialect));
    out.push_str(&format!("Database Name: {}\n\n", tree.current_database));
    out.push_str("Schema Tables and Columns:\n");

    let tables_to_include = match selected_tables {
        Some(selected) if !selected.is_empty() => tree
            .tables
            .iter()
            .filter(|t| selected.iter().any(|s| s == &t.name || s == &format!("{}.{}", t.schema, t.name)))
            .collect::<Vec<_>>(),
        _ => tree.tables.iter().collect::<Vec<_>>(),
    };

    for table in &tables_to_include {
        out.push_str(&format!("CREATE TABLE \"{}\".\"{}\" (\n", table.schema, table.name));
        let mut col_defs = Vec::new();
        for col in &table.columns {
            let mut def = format!("  \"{}\" {}", col.name, col.data_type);
            if col.is_primary_key {
                def.push_str(" PRIMARY KEY");
            }
            if !col.nullable && !col.is_primary_key {
                def.push_str(" NOT NULL");
            }
            col_defs.push(def);
        }
        out.push_str(&col_defs.join(",\n"));
        out.push_str("\n);\n\n");
    }

    if !tree.relations.is_empty() {
        out.push_str("Foreign Key Relations:\n");
        for rel in &tree.relations {
            let matches_from = tables_to_include.iter().any(|t| t.name == rel.from_table || format!("{}.{}", t.schema, t.name) == rel.from_table);
            let matches_to = tables_to_include.iter().any(|t| t.name == rel.to_table || format!("{}.{}", t.schema, t.name) == rel.to_table);
            if matches_from || matches_to {
                out.push_str(&format!(
                    "- {}.{} -> {}.{}\n",
                    rel.from_table, rel.from_column, rel.to_table, rel.to_column
                ));
            }
        }
    }

    out
}

pub fn build_system_prompt(dialect: &str) -> String {
    format!(
        r#"You are FugDB Copilot, an expert AI SQL Database Assistant.
Your task is to convert the user's natural language instructions into a high-performance, accurate SQL query tailored specifically for the {} database dialect.

RULES:
1. Use ONLY the tables, columns, and relationships defined in the provided schema context. Do NOT hallucinate tables or columns.
2. Adhere strictly to {} SQL syntax and dialect features (e.g. casing, string matching, date/time functions, JSON operators, and pagination).
3. Always format your output with a SQL code block:
```sql
<YOUR_SQL_QUERY_HERE>
```
Followed by a concise, bulleted explanation of what the query does and why specific joins/filters were used.
4. Keep the query optimized, safe, and ready to execute directly."#,
        dialect, dialect
    )
}
