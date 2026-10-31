use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("users")
                    .col(uuid("id").primary_key())
                    .col(string("email"))
                    .col(string("password_hash"))
                    .col(string("name"))
                    .col(timestamp_with_time_zone_null("email_verified_at"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone_null("deleted_at"))
                    .to_owned(),
            )
            .await?;

        // One active account per email, whatever its case: a deleted account
        // frees its email so it can be registered again.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE UNIQUE INDEX users_email_active_key \
                 ON users (lower(email)) WHERE deleted_at IS NULL",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("users").to_owned())
            .await
    }
}
