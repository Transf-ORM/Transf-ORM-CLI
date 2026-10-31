//! `users` table: accounts of the web application.
//!
//! An account is soft-deleted (`deleted_at` set) rather than removed, so its
//! email can be registered again while its history stays attached.

use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, Set};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// UUIDv7, generated on insert: hard to guess, but ordered by creation.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Always stored lowercase; unique among accounts that are not deleted.
    pub email: String,
    /// Password hash, never the password itself.
    pub password_hash: String,
    pub name: String,
    /// When the email was confirmed; `None` until then.
    pub email_verified_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    /// When the account was deleted; `None` for an active account.
    pub deleted_at: Option<DateTimeUtc>,
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Fills the generated columns and normalizes the email before every
    /// insert or update.
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = ChronoUtc::now();
        if insert {
            if self.id.is_not_set() {
                self.id = Set(Uuid::now_v7());
            }
            if self.created_at.is_not_set() {
                self.created_at = Set(now);
            }
        }
        self.updated_at = Set(now);
        if let ActiveValue::Set(email) = &self.email {
            self.email = Set(email.trim().to_lowercase());
        }
        Ok(self)
    }
}
