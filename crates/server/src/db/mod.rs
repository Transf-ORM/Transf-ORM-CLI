//! Persistence layer of the server: SeaORM entities and migrations
//! (see ADR-0006 in the wiki).

use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

pub mod entity;
pub mod migration;

pub use migration::Migrator;

/// Opens a connection pool to the database at `url`.
///
/// # Arguments
///
/// * `url` - PostgreSQL connection URL, e.g.
///   `postgres://transf_orm:transf_orm@localhost:5442/transf_orm`
///
/// # Errors
///
/// Returns an error when the URL is invalid or the database is unreachable.
pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(url).await
}

/// Applies every pending migration.
///
/// # Errors
///
/// Returns an error when a migration fails; migrations applied before it
/// are kept.
pub async fn migrate(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}
