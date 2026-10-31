//! Versioned schema migrations, applied in the order of [`Migrator::migrations`].
//!
//! Each migration lives in its own `mYYYYMMDD_NNNNNN_<name>.rs` module and
//! implements both `up` and `down`.

use sea_orm_migration::prelude::*;

mod m20261031_000001_create_users;

/// Lists every migration of the server database.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20261031_000001_create_users::Migration)]
    }
}
