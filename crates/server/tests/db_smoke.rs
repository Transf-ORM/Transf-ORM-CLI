//! Smoke test of the persistence layer against a real PostgreSQL.
//!
//! Uses `DATABASE_URL`, or the database of `docker-compose.yml` when unset.
//! When no database is reachable the test is skipped locally, but fails in CI
//! (`CI` set) so a missing service cannot go unnoticed.

use sea_orm::prelude::ChronoUtc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use sea_orm_migration::{MigratorTrait, SchemaManager};
use transf_orm_server::db::{self, entity::user, Migrator};

const DEFAULT_URL: &str = "postgres://transf_orm:transf_orm@localhost:5442/transf_orm";

/// Connects to the test database, or returns `None` to skip the test.
async fn connect() -> Option<DatabaseConnection> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_URL.to_owned());
    match db::connect(&url).await {
        Ok(conn) => Some(conn),
        Err(err) if std::env::var_os("CI").is_none() => {
            eprintln!(
                "skipping db_smoke: no database at {url} ({err}); run `docker compose up -d`"
            );
            None
        }
        Err(err) => panic!("no database at {url}: {err}"),
    }
}

fn new_user(email: &str, name: &str) -> user::ActiveModel {
    user::ActiveModel {
        email: Set(email.to_owned()),
        password_hash: Set("hash".to_owned()),
        name: Set(name.to_owned()),
        ..Default::default()
    }
}

#[tokio::test]
async fn db_smoke_migrate_insert_read() {
    let Some(conn) = connect().await else {
        return;
    };

    // Start from an empty schema so the test does not depend on previous runs.
    Migrator::fresh(&conn).await.unwrap();

    let inserted = new_user(" Ada@Example.com ", "Ada")
        .insert(&conn)
        .await
        .unwrap();
    assert_eq!(inserted.id.get_version_num(), 7);
    assert_eq!(inserted.email, "ada@example.com");
    assert_eq!(inserted.created_at, inserted.updated_at);
    assert_eq!(inserted.email_verified_at, None);
    assert_eq!(inserted.deleted_at, None);

    let found = user::Entity::find_by_id(inserted.id)
        .one(&conn)
        .await
        .unwrap()
        .expect("inserted user not found");
    assert_eq!(found, inserted);

    // The email is unique among active accounts, whatever its case...
    let duplicate = new_user("ADA@example.com", "Ada bis").insert(&conn).await;
    assert!(duplicate.is_err(), "duplicate email was accepted");

    // ...and a deleted account frees it.
    let mut deleted = found.into_active_model();
    deleted.deleted_at = Set(Some(ChronoUtc::now()));
    let deleted = deleted.update(&conn).await.unwrap();
    assert!(deleted.updated_at > deleted.created_at);
    new_user("ada@example.com", "Ada again")
        .insert(&conn)
        .await
        .unwrap();

    // Every migration can be rolled back, then applied again.
    let schema = SchemaManager::new(&conn);
    Migrator::down(&conn, None).await.unwrap();
    assert!(!schema.has_table("users").await.unwrap());
    db::migrate(&conn).await.unwrap();
    assert!(schema.has_table("users").await.unwrap());
    assert!(Migrator::get_pending_migrations(&conn)
        .await
        .unwrap()
        .is_empty());
}
