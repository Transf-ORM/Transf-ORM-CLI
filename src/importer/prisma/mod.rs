use std::collections::{HashMap, HashSet};

use pest::Parser;

use crate::pivot::{DatabaseKind, Enum, Schema};

mod ast;
mod mappers;
mod parser;
mod resolver;
use ast::{RawModel, RawTypeAlias};
use parser::{PrismaParser, Rule};

// ── Public API ─────────────────────────────────────────────────────────────

/// Importer for Prisma Schema Language (`.prisma`) files.
pub struct PrismaImporter;

impl super::Importer for PrismaImporter {
    fn import(&self, input: &str) -> Result<crate::pivot::Schema, super::ImportError> {
        parse_schema(input)
    }
}

/// Parse a Prisma Schema Language string and return the pivot [`Schema`].
///
/// # Errors
///
/// Returns [`ImportError::Parse`] if the input is not valid PSL, or
/// [`ImportError::Schema`] if the schema is structurally invalid (e.g. a
/// field references a model that does not exist).
///
/// # Examples
///
/// ```
/// use transf_orm_cli::importer::prisma::parse_schema;
///
/// let input = r#"
///   datasource db { provider = "postgresql" url = env("DATABASE_URL") }
///   model User {
///     id    Int    @id @default(autoincrement())
///     email String @unique
///   }
/// "#;
///
/// let schema = parse_schema(input).unwrap();
/// assert_eq!(schema.tables.len(), 1);
/// assert_eq!(schema.tables[0].name, "User");
/// ```
pub fn parse_schema(input: &str) -> Result<Schema, super::ImportError> {
    let pairs = PrismaParser::parse(Rule::schema, input)?;
    let schema_pair = pairs.into_iter().next().expect("pest always produces at least one pair for a valid schema rule");

    let mut database = DatabaseKind::PostgreSql;
    let mut raw_models: Vec<RawModel> = Vec::new();
    let mut raw_views: Vec<RawModel> = Vec::new();
    let mut enums: Vec<Enum> = Vec::new();
    let mut type_aliases: HashMap<String, RawTypeAlias> = HashMap::new();

    for pair in schema_pair.into_inner() {
        match pair.as_rule() {
            Rule::block => {
                let inner = pair.into_inner().next().expect("a block pair always has an inner rule");
                match inner.as_rule() {
                    Rule::datasource_block => database = parser::extract_database_kind(inner),
                    Rule::model_block => raw_models.push(parser::parse_model_block(inner)?),
                    Rule::view_block => raw_views.push(parser::parse_model_block(inner)?),
                    Rule::enum_block => enums.push(parser::parse_enum_block(inner)?),
                    Rule::type_alias => {
                        let alias = parser::parse_type_alias(inner)?;
                        type_aliases.insert(alias.name.clone(), alias);
                    }
                    _ => {} // generator_block
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    let model_names: HashSet<String> = raw_models.iter().map(|m| m.name.clone()).collect();
    let enum_names: HashSet<String> = enums.iter().map(|e| e.name.clone()).collect();
    let tables = resolver::resolve_models(raw_models, &model_names, &enum_names, &type_aliases)?;
    let views = resolver::resolve_views(raw_views, &enum_names, &type_aliases)?;

    let mut schema = Schema::new(database);
    schema.tables = tables;
    schema.enums = enums;
    schema.views = views;
    Ok(schema)
}
// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pivot::metadata::OrmKind;
    use crate::pivot::relation::{ReferentialAction, RelationKind};
    use crate::pivot::table::{Behavior, IndexType, SortDirection};
    use crate::pivot::types::{DbHint, DefaultFn, DefaultValue, LiteralValue, ScalarType};

    const SCHEMA: &str = r#"
        datasource db {
          provider = "postgresql"
          url      = env("DATABASE_URL")
        }

        generator client {
          provider = "prisma-client-js"
        }

        model User {
          id        Int      @id @default(autoincrement())
          email     String   @unique @db.VarChar(255)
          name      String?
          role      Role     @default(USER)
          posts     Post[]
          updatedAt DateTime @updatedAt @map("updated_at")

          @@map("users")
        }

        model Post {
          id        Int     @id @default(autoincrement())
          title     String  @db.VarChar(255)
          content   String?
          published Boolean @default(false)
          authorId  Int
          author    User    @relation(fields: [authorId], references: [id], onDelete: Cascade)
          tags      Tag[]

          @@index([title])
          @@map("posts")
        }

        model Tag {
          id    Int    @id @default(autoincrement())
          name  String @unique
          posts Post[]

          @@map("tags")
        }

        enum Role {
          USER
          ADMIN @map("admin")
        }
    "#;

    #[test]
    fn detects_postgresql_database() {
        let schema = parse_schema(SCHEMA).unwrap();
        assert_eq!(schema.database, DatabaseKind::PostgreSql);
    }

    #[test]
    fn parses_three_tables() {
        let schema = parse_schema(SCHEMA).unwrap();
        assert_eq!(schema.tables.len(), 3);
        assert_eq!(schema.tables[0].name, "User");
        assert_eq!(schema.tables[1].name, "Post");
        assert_eq!(schema.tables[2].name, "Tag");
    }

    #[test]
    fn maps_table_db_names() {
        let schema = parse_schema(SCHEMA).unwrap();
        assert_eq!(schema.tables[0].db_name.as_deref(), Some("users"));
        assert_eq!(schema.tables[1].db_name.as_deref(), Some("posts"));
    }

    #[test]
    fn user_primary_key_and_default() {
        let schema = parse_schema(SCHEMA).unwrap();
        let user = &schema.tables[0];
        let pk = user.primary_key.as_ref().unwrap();
        assert_eq!(pk.columns, ["id"]);
        let id_col = user.columns.iter().find(|c| c.name == "id").unwrap();
        assert_eq!(
            id_col.default,
            Some(DefaultValue::Function(DefaultFn::AutoIncrement))
        );
        assert!(!id_col.nullable);
    }

    #[test]
    fn varchar_db_hint_preserved() {
        let schema = parse_schema(SCHEMA).unwrap();
        let user = &schema.tables[0];
        let email = user.columns.iter().find(|c| c.name == "email").unwrap();
        assert_eq!(
            email.col_type.db_hint,
            Some(DbHint::VarChar { length: 255 })
        );
    }

    #[test]
    fn nullable_and_unique() {
        let schema = parse_schema(SCHEMA).unwrap();
        let user = &schema.tables[0];
        let name_col = user.columns.iter().find(|c| c.name == "name").unwrap();
        assert!(name_col.nullable);
        assert!(user
            .unique_constraints
            .iter()
            .any(|u| u.columns == ["email"]));
    }

    #[test]
    fn enum_column_and_default() {
        let schema = parse_schema(SCHEMA).unwrap();
        let user = &schema.tables[0];
        let role = user.columns.iter().find(|c| c.name == "role").unwrap();
        assert_eq!(
            role.col_type.scalar,
            ScalarType::Enum {
                name: "Role".to_string()
            }
        );
        assert_eq!(
            role.default,
            Some(DefaultValue::Literal(LiteralValue::String(
                "USER".to_string()
            )))
        );
    }

    #[test]
    fn updated_at_behavior() {
        let schema = parse_schema(SCHEMA).unwrap();
        let user = &schema.tables[0];
        assert!(user
            .behaviors
            .iter()
            .any(|b| matches!(b, Behavior::UpdatedAt { column } if column == "updatedAt")));
    }

    #[test]
    fn post_has_foreign_key_and_index() {
        let schema = parse_schema(SCHEMA).unwrap();
        let post = &schema.tables[1];
        let fk = post
            .foreign_keys
            .iter()
            .find(|fk| fk.referenced_table == "User")
            .unwrap();
        assert_eq!(fk.columns, ["authorId"]);
        assert_eq!(fk.referenced_columns, ["id"]);
        assert_eq!(fk.on_delete, Some(ReferentialAction::Cascade));
        assert!(post
            .indexes
            .iter()
            .any(|i| i.columns.iter().any(|c| c.column == "title")));
    }

    #[test]
    fn implicit_m2m_post_tag() {
        let schema = parse_schema(SCHEMA).unwrap();
        let post = &schema.tables[1];
        let m2m = post
            .relations
            .iter()
            .find(|r| matches!(&r.kind, RelationKind::ManyToMany { implicit, .. } if *implicit));
        assert!(m2m.is_some(), "Post should have an implicit M2M relation");
    }

    #[test]
    fn enum_values_parsed() {
        let schema = parse_schema(SCHEMA).unwrap();
        assert_eq!(schema.enums.len(), 1);
        let role = &schema.enums[0];
        assert_eq!(role.name, "Role");
        assert_eq!(role.values[0].name, "USER");
        assert_eq!(role.values[1].name, "ADMIN");
        assert_eq!(role.values[1].db_name.as_deref(), Some("admin"));
    }

    #[test]
    fn source_orm_is_prisma() {
        let schema = parse_schema(SCHEMA).unwrap();
        assert_eq!(schema.tables[0].metadata.source_orm, Some(OrmKind::Prisma));
    }

    #[test]
    fn scalar_array_field_sets_array_flag() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Post {
              id   Int      @id
              tags String[]
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let post = &schema.tables[0];
        let tags = post.columns.iter().find(|c| c.name == "tags").unwrap();
        assert!(tags.col_type.array, "String[] should set array: true");
    }

    #[test]
    fn db_schema_attribute() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Analytics {
              id Int @id
              @@schema("analytics")
            }
        "#;
        let schema = parse_schema(input).unwrap();
        assert_eq!(schema.tables[0].db_schema.as_deref(), Some("analytics"));
    }

    #[test]
    fn model_ignore_stored_in_metadata() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Legacy {
              id Int @id
              @@ignore
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let table = &schema.tables[0];
        let feat = table
            .metadata
            .unknown_features
            .iter()
            .find(|f| f.name == "prisma.ignore");
        assert!(
            feat.is_some(),
            "@@ignore should produce unknown_feature 'prisma.ignore'"
        );
        assert_eq!(feat.unwrap().value, serde_json::json!(true));
    }

    #[test]
    fn field_ignore_stored_in_metadata() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model User {
              id     Int    @id
              secret String @ignore
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let table = &schema.tables[0];
        let feat = table
            .metadata
            .unknown_features
            .iter()
            .find(|f| f.name == "prisma.ignoredFields");
        assert!(feat.is_some());
        assert_eq!(feat.unwrap().value, serde_json::json!(["secret"]));
        // Column is still present (it exists in the DB)
        assert!(table.columns.iter().any(|c| c.name == "secret"));
    }

    #[test]
    fn db_decimal_two_args() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Order {
              id    Int     @id
              price Decimal @db.Decimal(10, 2)
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let col = schema.tables[0]
            .columns
            .iter()
            .find(|c| c.name == "price")
            .unwrap();
        assert_eq!(
            col.col_type.db_hint,
            Some(DbHint::Numeric {
                precision: 10,
                scale: 2
            })
        );
    }

    #[test]
    fn type_alias_inlined() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            type PostId = Int @id @default(autoincrement())
            model Post {
              id PostId
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let post = &schema.tables[0];
        let id_col = post.columns.iter().find(|c| c.name == "id").unwrap();
        assert_eq!(id_col.col_type.scalar, ScalarType::Int);
        assert_eq!(
            id_col.default,
            Some(DefaultValue::Function(DefaultFn::AutoIncrement))
        );
        assert!(post
            .primary_key
            .as_ref()
            .map(|pk| pk.columns.contains(&"id".to_string()))
            .unwrap_or(false));
    }

    #[test]
    fn view_block_parsed() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            view UserView {
              id    Int
              email String
              @@map("v_users")
              @@schema("reporting")
            }
        "#;
        let schema = parse_schema(input).unwrap();
        assert_eq!(schema.views.len(), 1);
        let view = &schema.views[0];
        assert_eq!(view.name, "UserView");
        assert_eq!(view.db_name.as_deref(), Some("v_users"));
        assert_eq!(view.db_schema.as_deref(), Some("reporting"));
        assert_eq!(view.columns.len(), 2);
        assert_eq!(view.columns[0].name, "id");
        assert!(!view.updatable);
    }

    #[test]
    fn index_with_sort_and_type() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Post {
              id    Int    @id
              title String
              body  String
              @@index([title(sort: Asc), body(sort: Desc)], type: Hash)
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let idx = &schema.tables[0].indexes[0];
        assert_eq!(idx.columns[0].sort, SortDirection::Asc);
        assert_eq!(idx.columns[1].sort, SortDirection::Desc);
        assert!(matches!(idx.index_type, Some(IndexType::Hash)));
    }

    #[test]
    fn index_constraint_names() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model User {
              id    Int    @id(map: "pk_user")
              email String
              name  String
              @@unique(fields: [email, name], map: "uq_user_email_name")
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let table = &schema.tables[0];
        assert_eq!(
            table.primary_key.as_ref().unwrap().name.as_deref(),
            Some("pk_user")
        );
        let uq = table
            .unique_constraints
            .iter()
            .find(|u| u.columns.contains(&"email".to_string()))
            .unwrap();
        assert_eq!(uq.name.as_deref(), Some("uq_user_email_name"));
    }

    #[test]
    fn fulltext_index() {
        let input = r#"
            datasource db { provider = "mysql" url = env("DATABASE_URL") }
            model Post {
              id    Int    @id
              title String
              body  String
              @@fulltext([title, body])
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let idx = schema.tables[0].indexes.iter().find(
            |i| matches!(&i.index_type, Some(IndexType::Custom { name }) if name == "fulltext"),
        );
        assert!(
            idx.is_some(),
            "@@fulltext should produce a 'fulltext' index"
        );
        assert_eq!(idx.unwrap().columns.len(), 2);
    }

    #[test]
    fn db_type_hints_uuid_and_date() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Event {
              id        String   @id @db.Uuid
              happenedOn DateTime @db.Date
              startTime DateTime @db.Time(3)
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let cols = &schema.tables[0].columns;
        let id = cols.iter().find(|c| c.name == "id").unwrap();
        assert_eq!(id.col_type.db_hint, Some(DbHint::Uuid));
        let date = cols.iter().find(|c| c.name == "happenedOn").unwrap();
        assert_eq!(date.col_type.db_hint, Some(DbHint::Date));
        let time = cols.iter().find(|c| c.name == "startTime").unwrap();
        assert_eq!(time.col_type.db_hint, Some(DbHint::Time { precision: 3 }));
    }

    #[test]
    fn parse_full_test_schema() {
        let input = include_str!("../../../test.prisma");
        let schema = parse_schema(input).expect("test.prisma should parse without error");

        // Database
        assert_eq!(schema.database, DatabaseKind::PostgreSql);

        // Tables
        let table_names: Vec<&str> = schema.tables.iter().map(|t| t.name.as_str()).collect();
        assert!(table_names.contains(&"User"), "User table missing");
        assert!(table_names.contains(&"Profile"), "Profile table missing");
        assert!(table_names.contains(&"Post"), "Post table missing");
        assert!(table_names.contains(&"Comment"), "Comment table missing");
        assert!(table_names.contains(&"Tag"), "Tag table missing");
        assert!(table_names.contains(&"Media"), "Media table missing");
        assert!(table_names.contains(&"Review"), "Review table missing");
        assert!(table_names.contains(&"Product"), "Product table missing");
        assert!(table_names.contains(&"Order"), "Order table missing");
        assert!(
            table_names.contains(&"OrderItem"),
            "OrderItem table missing"
        );
        assert!(table_names.contains(&"Event"), "Event table missing");

        // Enums
        assert_eq!(schema.enums.len(), 3);

        // View
        assert_eq!(schema.views.len(), 1);
        assert_eq!(schema.views[0].name, "UserStats");

        // Type alias inlining: User.id should be String/Uuid from UuidPk alias
        let user = schema.tables.iter().find(|t| t.name == "User").unwrap();
        let user_id = user.columns.iter().find(|c| c.name == "id").unwrap();
        assert_eq!(user_id.col_type.scalar, ScalarType::String);
        assert_eq!(user_id.col_type.db_hint, Some(DbHint::Uuid));
        assert!(user
            .primary_key
            .as_ref()
            .map(|pk| pk.columns.contains(&"id".to_string()))
            .unwrap_or(false));

        // @@schema
        assert_eq!(user.db_schema.as_deref(), Some("public"));
        let product = schema.tables.iter().find(|t| t.name == "Product").unwrap();
        assert_eq!(product.db_schema.as_deref(), Some("shop"));
        let event = schema.tables.iter().find(|t| t.name == "Event").unwrap();
        assert_eq!(event.db_schema.as_deref(), Some("analytics"));

        // @@ignore on Event
        assert!(event
            .metadata
            .unknown_features
            .iter()
            .any(|f| f.name == "prisma.ignore"));

        // @ignore on Post.internalNote — column still present
        let post = schema.tables.iter().find(|t| t.name == "Post").unwrap();
        assert!(post.columns.iter().any(|c| c.name == "internalNote"));
        assert!(post
            .metadata
            .unknown_features
            .iter()
            .any(|f| f.name == "prisma.ignoredFields"));

        // Post.tags String[] — scalar array
        let user_tags = user.columns.iter().find(|c| c.name == "tags").unwrap();
        assert!(user_tags.col_type.array);

        // Implicit M2M Post ↔ Tag
        assert!(post
            .relations
            .iter()
            .any(|r| matches!(&r.kind, RelationKind::ManyToMany { implicit, .. } if *implicit)));

        // Named relations Review → User (two distinct)
        let review = schema.tables.iter().find(|t| t.name == "Review").unwrap();
        assert_eq!(review.foreign_keys.len(), 2);

        // Composite PK on OrderItem
        let order_item = schema
            .tables
            .iter()
            .find(|t| t.name == "OrderItem")
            .unwrap();
        let pk = order_item.primary_key.as_ref().unwrap();
        assert!(pk.columns.contains(&"orderId".to_string()));
        assert!(pk.columns.contains(&"productId".to_string()));

        // @@fulltext on Post
        assert!(post.indexes.iter().any(
            |i| matches!(&i.index_type, Some(IndexType::Custom { name }) if name == "fulltext")
        ));

        // @@index with sort on User
        let user_idx = user
            .indexes
            .iter()
            .find(|i| i.columns.iter().any(|c| c.column == "createdAt"))
            .unwrap();
        assert_eq!(user_idx.columns[0].sort, SortDirection::Desc);

        // @db.Decimal(12, 2) on User.balance
        let balance = user.columns.iter().find(|c| c.name == "balance").unwrap();
        assert_eq!(
            balance.col_type.db_hint,
            Some(DbHint::Numeric {
                precision: 12,
                scale: 2
            })
        );

        // @db.Inet on User.ipAddress
        let ip = user.columns.iter().find(|c| c.name == "ipAddress").unwrap();
        assert_eq!(ip.col_type.db_hint, Some(DbHint::Inet));

        // Unsupported("hstore") on Profile.rawConfig
        let profile = schema.tables.iter().find(|t| t.name == "Profile").unwrap();
        let raw_config = profile
            .columns
            .iter()
            .find(|c| c.name == "rawConfig")
            .unwrap();
        assert!(
            matches!(&raw_config.col_type.scalar, ScalarType::Unsupported { type_name } if type_name == "hstore")
        );

        // Round-trip through JSON
        let json = schema.to_canonical_json().expect("serialization failed");
        let restored = crate::pivot::Schema::from_json(&json).expect("deserialization failed");
        assert_eq!(schema, restored);
    }

    #[test]
    fn named_relation_cardinality() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model User {
              id            Int    @id
              writtenPosts  Post[] @relation("WrittenPosts")
              favoritePosts Post[] @relation("FavoritePosts")
            }
            model Post {
              id              Int  @id
              authorId        Int
              favoritedById   Int?
              author          User  @relation("WrittenPosts",  fields: [authorId],      references: [id])
              favoritedBy     User? @relation("FavoritePosts", fields: [favoritedById], references: [id])
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let post = &schema.tables[1];

        // Both FKs present
        assert_eq!(post.foreign_keys.len(), 2);

        // Both relations present and named
        let written = post
            .relations
            .iter()
            .find(|r| r.name.as_deref() == Some("WrittenPosts"));
        let favorite = post
            .relations
            .iter()
            .find(|r| r.name.as_deref() == Some("FavoritePosts"));
        assert!(
            written.is_some(),
            "WrittenPosts relation must exist on Post"
        );
        assert!(
            favorite.is_some(),
            "FavoritePosts relation must exist on Post"
        );
    }

    #[test]
    fn unknown_block_attr_preserved_in_metadata() {
        // Any @@attr not explicitly handled must land in unknown_features, not be silently dropped.
        // This ensures future Prisma attributes (and custom generator plugins) survive round-trips.
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model User {
              id Int @id
              @@shardKey([id])
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let table = &schema.tables[0];
        let feat = table
            .metadata
            .unknown_features
            .iter()
            .find(|f| f.name == "prisma.shardKey");
        assert!(
            feat.is_some(),
            "@@shardKey should be preserved as unknown_feature 'prisma.shardKey'"
        );
    }

    #[test]
    fn unknown_field_attr_preserved_in_metadata() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model User {
              id    Int    @id
              email String @omit
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let table = &schema.tables[0];
        let feat = table
            .metadata
            .unknown_features
            .iter()
            .find(|f| f.name == "prisma.field.email.omit");
        assert!(
            feat.is_some(),
            "@omit should be preserved as unknown_feature 'prisma.field.email.omit'"
        );
        // Column must still be present in the IR (it exists in the DB)
        assert!(table.columns.iter().any(|c| c.name == "email"));
    }

    #[test]
    fn relation_map_populates_fk_name() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Post {
              id       Int  @id
              authorId Int
              author   User @relation(fields: [authorId], references: [id], map: "fk_posts_author")
            }
            model User { id Int @id posts Post[] }
        "#;
        let schema = parse_schema(input).unwrap();
        let post = schema.tables.iter().find(|t| t.name == "Post").unwrap();
        let fk = post
            .foreign_keys
            .iter()
            .find(|fk| fk.referenced_table == "User")
            .unwrap();
        assert_eq!(fk.name.as_deref(), Some("fk_posts_author"));
    }

    #[test]
    fn relation_without_map_has_no_fk_name() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Post {
              id       Int  @id
              authorId Int
              author   User @relation(fields: [authorId], references: [id])
            }
            model User { id Int @id posts Post[] }
        "#;
        let schema = parse_schema(input).unwrap();
        let post = schema.tables.iter().find(|t| t.name == "Post").unwrap();
        let fk = post
            .foreign_keys
            .iter()
            .find(|fk| fk.referenced_table == "User")
            .unwrap();
        assert_eq!(fk.name, None);
    }

    #[test]
    fn string_escape_sequences_in_map_and_default() {
        let input = r#"
            datasource db { provider = "postgresql" url = env("DATABASE_URL") }
            model Post {
              id    Int    @id
              label String @default("hello \"world\"") @map("post_label")
            }
        "#;
        let schema = parse_schema(input).unwrap();
        let col = schema.tables[0]
            .columns
            .iter()
            .find(|c| c.name == "label")
            .unwrap();
        assert!(
            col.default.is_some(),
            "escaped-quote default should parse without error"
        );
        assert_eq!(col.db_name.as_deref(), Some("post_label"));
    }
}
