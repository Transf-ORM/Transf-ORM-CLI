//! # Transf-ORM Core
//!
//! Conversion of ORM schemas through a pivot intermediate representation.
//!
//! [`api`] is the **only supported public entry point** for consumers such as
//! `transf-orm-cli` and `transf-orm-server`. Parsers, exporters, the registry,
//! and the pivot implementation are internal details.
//!
//! # Example
//!
//! ```
//! use transf_orm_core::api::{self, DatabaseKind, Pivot};
//!
//! let pivot = Pivot::new(DatabaseKind::PostgreSql);
//! let json = api::export(&pivot, "pivot").unwrap();
//! assert!(json.contains("pivotVersion"));
//! ```

pub mod api;

pub(crate) mod config;
pub(crate) mod exporter;
pub(crate) mod importer;
pub(crate) mod pivot;
pub(crate) mod registry;
