use sf_core::{SfError, SfResult};

/// Quote a SQL identifier (table name) for SQLite / PostgreSQL.
///
/// Embedded double quotes are doubled so `orders" UNION SELECT ...` cannot
/// break out of the quoted identifier; statement separators and control
/// characters are rejected outright since no real table name contains them.
pub fn quote_ident(name: &str) -> SfResult<String> {
    if name.trim().is_empty() {
        return Err(SfError::Validation("Table name must not be empty".into()));
    }
    if name.contains(';') || name.chars().any(char::is_control) {
        return Err(SfError::Validation(format!(
            "Invalid table name {:?}: statement separators and control characters are not allowed",
            name
        )));
    }
    Ok(format!("\"{}\"", name.replace('"', "\"\"")))
}

/// A `filter` is a WHERE predicate written by the recipe author. It is SQL
/// by design, but it must be a single predicate: a second statement after
/// `;` is never legitimate here.
fn validate_filter(filter: &str) -> SfResult<()> {
    if filter.contains(';') || filter.contains('\0') {
        return Err(SfError::Validation(
            "Filter must be a single WHERE predicate (';' is not allowed)".into(),
        ));
    }
    Ok(())
}

/// Build a SELECT query from table/query/filter options.
///
/// Priority: query > table+filter > error
///
/// A raw `query` is passed through as-is; the loaders execute it on a
/// read-only connection / transaction so a recipe cannot modify the database.
pub fn build_select_query(
    table: Option<&str>,
    query: Option<&str>,
    filter: Option<&str>,
) -> SfResult<String> {
    if let Some(q) = query {
        if q.contains('\0') {
            return Err(SfError::Validation("Query contains a NUL byte".into()));
        }
        return Ok(q.to_string());
    }

    match table {
        Some(t) => {
            let mut sql = format!("SELECT * FROM {}", quote_ident(t)?);
            if let Some(f) = filter {
                validate_filter(f)?;
                sql.push_str(&format!(" WHERE {}", f));
            }
            Ok(sql)
        }
        None => Err(SfError::Validation(
            "Either 'table' or 'query' must be specified for database sources".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query_from_table() {
        let sql = build_select_query(Some("orders"), None, None).unwrap();
        assert_eq!(sql, "SELECT * FROM \"orders\"");
    }

    #[test]
    fn test_build_query_with_filter() {
        let sql = build_select_query(Some("orders"), None, Some("price > 100")).unwrap();
        assert_eq!(sql, "SELECT * FROM \"orders\" WHERE price > 100");
    }

    #[test]
    fn test_build_query_from_raw() {
        let sql = build_select_query(None, Some("SELECT id FROM t"), None).unwrap();
        assert_eq!(sql, "SELECT id FROM t");
    }

    #[test]
    fn test_build_query_no_table_no_query() {
        let result = build_select_query(None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_name_quotes_are_escaped() {
        let sql = build_select_query(Some("orders\" UNION SELECT 1 --"), None, None).unwrap();
        assert_eq!(sql, "SELECT * FROM \"orders\"\" UNION SELECT 1 --\"");
    }

    #[test]
    fn test_table_name_with_semicolon_rejected() {
        assert!(build_select_query(Some("orders; DROP TABLE x"), None, None).is_err());
    }

    #[test]
    fn test_filter_with_semicolon_rejected() {
        assert!(build_select_query(Some("orders"), None, Some("1=1; DELETE FROM orders")).is_err());
    }
}
