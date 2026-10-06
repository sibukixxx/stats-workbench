use crate::{Result, Source};
use sf_data::SourceLoader;

/// Build a parquet source.
pub fn parquet(path: impl Into<String>) -> Source {
    Source::Parquet { path: path.into() }
}

/// Build a SQLite source.
pub fn sqlite(
    path: impl Into<String>,
    table: Option<impl Into<String>>,
    query: Option<impl Into<String>>,
) -> Source {
    Source::Sqlite {
        path: path.into(),
        table: table.map(Into::into),
        query: query.map(Into::into),
    }
}

/// Build a PostgreSQL source.
pub fn postgres(
    url: impl Into<String>,
    table: Option<impl Into<String>>,
    query: Option<impl Into<String>>,
    filter: Option<impl Into<String>>,
) -> Source {
    Source::Postgres {
        url: url.into(),
        table: table.map(Into::into),
        query: query.map(Into::into),
        filter: filter.map(Into::into),
    }
}

/// Parse a user-facing source reference such as a parquet path or database URL.
pub fn from_reference(reference: &str, table: Option<&str>, query: Option<&str>) -> Result<Source> {
    SourceLoader::parse_cli_source(reference, table, query)
}
