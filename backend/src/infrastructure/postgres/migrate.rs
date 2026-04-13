use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

use crate::domain::errors::DomainError;

pub async fn connect_and_migrate(database_url: &str) -> Result<DatabaseConnection, DomainError> {
    let db = Database::connect(database_url)
        .await
        .map_err(|_| DomainError::EmptyField("database"))?;
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations/001_init.sql"));
    for part in sql.split(';') {
        let stmt = part.trim();
        if stmt.is_empty() {
            continue;
        }
        let with_semi = format!("{stmt};");
        db.execute(Statement::from_string(DbBackend::Postgres, with_semi))
            .await
            .map_err(|_| DomainError::EmptyField("migration"))?;
    }
    Ok(db)
}
