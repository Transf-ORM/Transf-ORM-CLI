use transf_orm_core::api::{self, DatabaseKind, Pivot};

const PRISMA_SCHEMA: &str = r#"
    datasource db {
        provider = "postgresql"
        url = env("DATABASE_URL")
    }

    model User {
        id Int @id
    }
"#;

#[test]
fn public_facade_builds_and_converts_a_pivot() {
    let pivot = Pivot::new(DatabaseKind::PostgreSql);
    let serialized = api::export(&pivot, "pivot").expect("pivot serialization failed");
    let restored = Pivot::from_json(&serialized).expect("pivot deserialization failed");
    assert_eq!(restored, pivot);

    let parsed = api::import(PRISMA_SCHEMA, "prisma").expect("Prisma import failed");
    assert_eq!(parsed.tables[0].name, "User");

    let converted =
        api::transform(PRISMA_SCHEMA, "prisma", "pivot").expect("Prisma-to-pivot transform failed");
    let expected = api::export(&parsed, "pivot").expect("pivot export failed");
    assert_eq!(converted.content, expected);
}
