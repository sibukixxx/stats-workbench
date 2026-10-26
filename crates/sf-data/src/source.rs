use sf_core::{DatasetRef, SfError, SfResult, SourceSpec};

/// Loads a dataset from a SourceSpec.
pub struct SourceLoader;

impl SourceLoader {
    pub async fn load(spec: &SourceSpec) -> SfResult<DatasetRef> {
        match spec {
            SourceSpec::Parquet { path } => {
                let lf = crate::parquet::read_parquet(path)?;
                Ok(DatasetRef::Lazy(Box::new(lf)))
            }
            SourceSpec::Sqlite { path, table, query } => {
                let lf =
                    crate::sqlite::read_sqlite(path, table.as_deref(), query.as_deref()).await?;
                Ok(DatasetRef::Lazy(Box::new(lf)))
            }
            SourceSpec::Postgres {
                url,
                table,
                query,
                filter,
            } => {
                let lf = crate::postgres::read_postgres(
                    url,
                    table.as_deref(),
                    query.as_deref(),
                    filter.as_deref(),
                )
                .await?;
                Ok(DatasetRef::Lazy(Box::new(lf)))
            }
        }
    }

    /// Parse a CLI --source string into a SourceSpec.
    pub fn parse_cli_source(
        source: &str,
        table: Option<&str>,
        query: Option<&str>,
    ) -> SfResult<SourceSpec> {
        if source.ends_with(".parquet") {
            Ok(SourceSpec::Parquet {
                path: source.to_string(),
            })
        } else if source.starts_with("sqlite://") {
            let path = source.strip_prefix("sqlite://").unwrap().to_string();
            Ok(SourceSpec::Sqlite {
                path,
                table: table.map(|s| s.to_string()),
                query: query.map(|s| s.to_string()),
            })
        } else if source.starts_with("pg://")
            || source.starts_with("postgres://")
            || source.starts_with("postgresql://")
        {
            let url = if source.starts_with("pg://") {
                source.replacen("pg://", "postgresql://", 1)
            } else {
                source.to_string()
            };
            Ok(SourceSpec::Postgres {
                url,
                table: table.map(|s| s.to_string()),
                query: query.map(|s| s.to_string()),
                filter: None,
            })
        } else {
            Err(SfError::Source(format!(
                "Unsupported source: '{}'. Expected .parquet, sqlite://path, or pg://host/db",
                source
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sqlite_source_preserves_db_options() {
        let source = SourceLoader::parse_cli_source(
            "sqlite://tmp/stats-workbench.db",
            Some("orders"),
            Some("SELECT * FROM orders"),
        )
        .unwrap();

        match source {
            SourceSpec::Sqlite { path, table, query } => {
                assert_eq!(path, "tmp/stats-workbench.db");
                assert_eq!(table.as_deref(), Some("orders"));
                assert_eq!(query.as_deref(), Some("SELECT * FROM orders"));
            }
            other => panic!("expected sqlite source, got {:?}", other),
        }
    }

    #[test]
    fn parse_postgres_source_normalizes_pg_scheme() {
        let source =
            SourceLoader::parse_cli_source("pg://localhost:5432/app", Some("metrics"), None)
                .unwrap();

        match source {
            SourceSpec::Postgres {
                url, table, query, ..
            } => {
                assert_eq!(url, "postgresql://localhost:5432/app");
                assert_eq!(table.as_deref(), Some("metrics"));
                assert_eq!(query, None);
            }
            other => panic!("expected postgres source, got {:?}", other),
        }
    }
}
