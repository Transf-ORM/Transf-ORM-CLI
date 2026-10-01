use std::collections::{HashMap, HashSet};

use super::ast::{RawAttr, RawModel, RawTypeAlias, RawValue};
use super::mappers;
use crate::pivot::metadata::{OrmKind, OrmMetadata, UnknownFeature};
use crate::pivot::relation::{
    CascadeOptions, ForeignKey, JunctionTable, ReferentialAction, Relation, RelationKind,
};
use crate::pivot::table::{
    Behavior, Column, Index, IndexType, PrimaryKey, Table, UniqueConstraint,
};
use crate::pivot::types::{ColumnType, ScalarType};
use crate::pivot::view::{View, ViewColumn};

// ── Structs ────────────────────────────────────────────────────────────────

struct RelInfo {
    owner: String,
    target: String,
    owner_cols: Vec<String>,
    target_refs: Vec<String>,
    on_delete: Option<ReferentialAction>,
    on_update: Option<ReferentialAction>,
    rel_name: Option<String>,
    fk_name: Option<String>,
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn push_unknown_attrs(
    metadata: &mut OrmMetadata,
    attrs: &[RawAttr],
    known: &[&str],
    name_prefix: &str,
) {
    for attr in attrs {
        if !known.contains(&attr.path.as_str()) {
            metadata.unknown_features.push(UnknownFeature {
                name: format!("{}{}", name_prefix, attr.path),
                value: mappers::raw_args_to_json(&attr.args),
                recoverable: true,
            });
        }
    }
}

// ── Resolvers ──────────────────────────────────────────────────────────────

pub(super) fn resolve_views(
    raw_views: Vec<RawModel>,
    enum_names: &HashSet<String>,
    type_aliases: &HashMap<String, RawTypeAlias>,
) -> Result<Vec<View>, super::super::ImportError> {
    raw_views
        .into_iter()
        .map(|raw| {
            let (db_name, db_schema) = mappers::extract_db_naming(&raw);

            let columns = raw
                .fields
                .iter()
                .map(|f| {
                    let db_name = f
                        .attr("map")
                        .and_then(|a| a.positional(0))
                        .and_then(|v| v.as_str().map(String::from));
                    ViewColumn {
                        name: f.name.clone(),
                        db_name,
                    }
                })
                .collect();

            let mut metadata = OrmMetadata {
                source_orm: Some(OrmKind::Prisma),
                ..Default::default()
            };
            if raw.block_attr("ignore").is_some() {
                metadata.unknown_features.push(UnknownFeature {
                    name: "prisma.ignore".to_string(),
                    value: serde_json::json!(true),
                    recoverable: true,
                });
            }

            let ignored: Vec<String> = raw
                .fields
                .iter()
                .filter(|f| f.attr("ignore").is_some())
                .map(|f| f.name.clone())
                .collect();
            if !ignored.is_empty() {
                metadata.unknown_features.push(UnknownFeature {
                    name: "prisma.ignoredFields".to_string(),
                    value: serde_json::json!(ignored),
                    recoverable: true,
                });
            }

            const KNOWN_VIEW_BLOCK_ATTRS: &[&str] = &["map", "schema", "ignore"];
            push_unknown_attrs(
                &mut metadata,
                &raw.block_attrs,
                KNOWN_VIEW_BLOCK_ATTRS,
                "prisma.",
            );

            let _ = (enum_names, type_aliases); // view columns not resolved into IR scalars yet

            Ok(View {
                name: raw.name.clone(),
                db_name,
                db_schema,
                definition: String::new(),
                columns,
                updatable: false,
                metadata,
            })
        })
        .collect()
}

pub(super) fn resolve_models(
    raw_models: Vec<RawModel>,
    model_names: &HashSet<String>,
    enum_names: &HashSet<String>,
    type_aliases: &HashMap<String, RawTypeAlias>,
) -> Result<Vec<Table>, super::super::ImportError> {
    let mut rel_infos: Vec<RelInfo> = Vec::new();
    let mut implicit_m2m: Vec<(String, String, Option<String>)> = Vec::new();

    for raw in &raw_models {
        for field in &raw.fields {
            if !model_names.contains(&field.base_type) {
                continue;
            }

            if let Some(rel_attr) = field.attr("relation") {
                let owner_cols = rel_attr
                    .named("fields")
                    .map(RawValue::as_string_array)
                    .unwrap_or_default();
                if owner_cols.is_empty() {
                    continue;
                }

                let rel_name = rel_attr
                    .named("name")
                    .and_then(|v| v.as_str().map(String::from))
                    .or_else(|| {
                        rel_attr
                            .positional(0)
                            .and_then(|v| v.as_str().map(String::from))
                    });

                rel_infos.push(RelInfo {
                    owner: raw.name.clone(),
                    target: field.base_type.clone(),
                    owner_cols,
                    target_refs: rel_attr
                        .named("references")
                        .map(RawValue::as_string_array)
                        .unwrap_or_default(),
                    on_delete: rel_attr
                        .named("onDelete")
                        .and_then(|v| v.as_ident())
                        .and_then(mappers::parse_referential_action),
                    on_update: rel_attr
                        .named("onUpdate")
                        .and_then(|v| v.as_ident())
                        .and_then(mappers::parse_referential_action),
                    rel_name,
                    fk_name: rel_attr
                        .named("map")
                        .and_then(|v| v.as_str().map(String::from)),
                });
            } else if field.is_array {
                let other = raw_models.iter().find(|m| m.name == field.base_type);
                let other_is_array = other
                    .map(|m| {
                        m.fields
                            .iter()
                            .any(|f| f.base_type == raw.name && f.is_array)
                    })
                    .unwrap_or(false);
                let other_has_fk = other
                    .map(|m| {
                        m.fields.iter().any(|f| {
                            f.base_type == raw.name
                                && f.attr("relation")
                                    .map(|a| a.named("fields").is_some())
                                    .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);

                if other_is_array && !other_has_fk {
                    let mut pair = [raw.name.clone(), field.base_type.clone()];
                    pair.sort();
                    let rel_name = field
                        .attr("relation")
                        .and_then(|a| a.named("name").or_else(|| a.positional(0)))
                        .and_then(|v| v.as_str().map(String::from));
                    let key = (pair[0].clone(), pair[1].clone(), rel_name);
                    if !implicit_m2m.contains(&key) {
                        implicit_m2m.push(key);
                    }
                }
            }
        }
    }

    raw_models
        .iter()
        .map(|raw| {
            let mut table = build_scalar_table(raw, model_names, enum_names, type_aliases)?;

            for ri in rel_infos.iter().filter(|r| r.owner == raw.name) {
                table.foreign_keys.push(ForeignKey {
                    name: ri.fk_name.clone(),
                    columns: ri.owner_cols.clone(),
                    referenced_table: ri.target.clone(),
                    referenced_columns: ri.target_refs.clone(),
                    on_delete: ri.on_delete.clone(),
                    on_update: ri.on_update.clone(),
                });
            }

            for ri in rel_infos
                .iter()
                .filter(|r| r.owner == raw.name || r.target == raw.name)
            {
                let is_owner = ri.owner == raw.name;
                let other_name = if is_owner { &ri.target } else { &ri.owner };

                let back_field = raw_models
                    .iter()
                    .find(|m| &m.name == other_name)
                    .and_then(|m| {
                        m.fields.iter().find(|f| {
                            if f.base_type != raw.name {
                                return false;
                            }
                            let f_rel_name = f
                                .attr("relation")
                                .and_then(|a| a.named("name").or_else(|| a.positional(0)))
                                .and_then(|v| v.as_str());
                            match (ri.rel_name.as_deref(), f_rel_name) {
                                (Some(rn), Some(fn_)) => rn == fn_,
                                (None, None) => true,
                                _ => false,
                            }
                        })
                    });

                let back_is_array = back_field.map(|f| f.is_array).unwrap_or(true);
                let kind = if back_is_array {
                    RelationKind::OneToMany
                } else {
                    RelationKind::OneToOne {
                        owner_table: ri.owner.clone(),
                    }
                };

                table.relations.push(Relation {
                    name: ri.rel_name.clone(),
                    kind,
                    from_table: ri.target.clone(),
                    from_fields: ri.target_refs.clone(),
                    to_table: ri.owner.clone(),
                    to_fields: ri.owner_cols.clone(),
                    cascade: CascadeOptions {
                        on_delete: ri.on_delete.clone(),
                        on_update: ri.on_update.clone(),
                    },
                });
            }

            for (a, b, rel_name) in &implicit_m2m {
                let (our_side, other_side) = if a == &raw.name {
                    (a, b)
                } else if b == &raw.name {
                    (b, a)
                } else {
                    continue;
                };
                table.relations.push(Relation {
                    name: rel_name.clone(),
                    kind: RelationKind::ManyToMany {
                        junction: JunctionTable {
                            table: format!("_{}To{}", a, b),
                            from_column: "A".to_string(),
                            to_column: "B".to_string(),
                        },
                        implicit: true,
                    },
                    from_table: our_side.clone(),
                    from_fields: Vec::new(),
                    to_table: other_side.clone(),
                    to_fields: Vec::new(),
                    cascade: CascadeOptions {
                        on_delete: None,
                        on_update: None,
                    },
                });
            }

            Ok(table)
        })
        .collect()
}

fn build_scalar_table(
    raw: &RawModel,
    model_names: &HashSet<String>,
    enum_names: &HashSet<String>,
    type_aliases: &HashMap<String, RawTypeAlias>,
) -> Result<Table, super::super::ImportError> {
    let (db_name, db_schema) = mappers::extract_db_naming(raw);

    let primary_key = raw.block_attr("id").map(|a| {
        let cols = mappers::extract_columns_from_attr(a);
        let name = a.named("map").and_then(|v| v.as_str().map(String::from));
        PrimaryKey {
            name,
            columns: cols,
        }
    });

    let unique_constraints = raw
        .block_attrs_all("unique")
        .map(|a| {
            let cols = mappers::extract_columns_from_attr(a);
            let name = a.named("map").and_then(|v| v.as_str().map(String::from));
            UniqueConstraint {
                name,
                columns: cols,
            }
        })
        .collect();

    let mut indexes: Vec<Index> = raw
        .block_attrs_all("index")
        .filter_map(|a| {
            let cols_val = a.named("fields").or_else(|| a.positional(0))?;
            let index_type = a
                .named("type")
                .and_then(|v| v.as_ident())
                .map(mappers::map_index_type);
            Some(Index {
                name: a.named("name").and_then(|n| n.as_str().map(String::from)),
                columns: mappers::extract_index_columns(cols_val),
                unique: false,
                partial: None,
                index_type,
                include: Vec::new(),
            })
        })
        .collect();

    for a in raw.block_attrs_all("fulltext") {
        let cols_val = a.named("fields").or_else(|| a.positional(0));
        indexes.push(Index {
            name: a.named("name").and_then(|n| n.as_str().map(String::from)),
            columns: cols_val
                .map(mappers::extract_index_columns)
                .unwrap_or_default(),
            unique: false,
            partial: None,
            index_type: Some(IndexType::Custom {
                name: "fulltext".to_string(),
            }),
            include: Vec::new(),
        });
    }

    let mut metadata = OrmMetadata {
        source_orm: Some(OrmKind::Prisma),
        ..Default::default()
    };

    if raw.block_attr("ignore").is_some() {
        metadata.unknown_features.push(UnknownFeature {
            name: "prisma.ignore".to_string(),
            value: serde_json::json!(true),
            recoverable: true,
        });
    }

    const KNOWN_BLOCK_ATTRS: &[&str] = &[
        "map", "schema", "id", "unique", "index", "fulltext", "ignore",
    ];
    push_unknown_attrs(
        &mut metadata,
        &raw.block_attrs,
        KNOWN_BLOCK_ATTRS,
        "prisma.",
    );

    let mut table = Table {
        name: raw.name.clone(),
        db_name,
        db_schema,
        columns: Vec::new(),
        primary_key,
        indexes,
        unique_constraints,
        check_constraints: Vec::new(),
        foreign_keys: Vec::new(),
        relations: Vec::new(),
        inheritance: None,
        behaviors: Vec::new(),
        metadata,
    };

    let mut ignored_fields: Vec<String> = Vec::new();

    for field in &raw.fields {
        if model_names.contains(&field.base_type) {
            continue;
        }

        let alias = type_aliases.get(&field.base_type);
        let effective_base: &str = alias
            .map(|a| a.base_type.as_str())
            .unwrap_or(&field.base_type);
        let alias_attrs = alias.map(|a| a.attrs.as_slice()).unwrap_or(&[]);

        let scalar = if let Some(raw_type) = &field.unsupported_type {
            ScalarType::Unsupported {
                type_name: raw_type.clone(),
            }
        } else if enum_names.contains(effective_base) {
            ScalarType::Enum {
                name: effective_base.to_string(),
            }
        } else if let Some(s) = mappers::map_scalar_type(effective_base) {
            s
        } else {
            ScalarType::Unsupported {
                type_name: effective_base.to_string(),
            }
        };

        let db_hint = field
            .attrs
            .iter()
            .chain(alias_attrs.iter())
            .find(|a| a.path.starts_with("db."))
            .and_then(|a| mappers::map_db_hint(&a.path, &a.args));

        let default = field
            .attr("default")
            .or_else(|| alias_attrs.iter().find(|a| a.path == "default"))
            .and_then(|a| a.positional(0))
            .and_then(mappers::map_default_value);

        let db_name = field
            .attr("map")
            .and_then(|a| a.positional(0))
            .and_then(|v| v.as_str().map(String::from));

        table.columns.push(Column {
            name: field.name.clone(),
            db_name,
            col_type: ColumnType {
                scalar,
                db_hint,
                array: field.is_array,
            },
            nullable: field.is_optional,
            default,
        });

        let id_attr = field
            .attr("id")
            .or_else(|| alias_attrs.iter().find(|a| a.path == "id"));
        if id_attr.is_some() && table.primary_key.is_none() {
            let pk_name = id_attr
                .and_then(|a| a.named("map"))
                .and_then(|v| v.as_str().map(String::from));
            table.primary_key = Some(PrimaryKey {
                name: pk_name,
                columns: vec![field.name.clone()],
            });
        }

        let has_unique =
            field.attr("unique").is_some() || alias_attrs.iter().any(|a| a.path == "unique");
        if has_unique {
            table.unique_constraints.push(UniqueConstraint {
                name: None,
                columns: vec![field.name.clone()],
            });
        }

        let has_updated_at =
            field.attr("updatedAt").is_some() || alias_attrs.iter().any(|a| a.path == "updatedAt");
        if has_updated_at {
            table.behaviors.push(Behavior::UpdatedAt {
                column: field.name.clone(),
            });
        }

        if field.attr("ignore").is_some() {
            ignored_fields.push(field.name.clone());
        }

        const KNOWN_FIELD_ATTRS: &[&str] = &[
            "id",
            "default",
            "unique",
            "map",
            "updatedAt",
            "ignore",
            "relation",
        ];
        for attr in &field.attrs {
            if !KNOWN_FIELD_ATTRS.contains(&attr.path.as_str()) && !attr.path.starts_with("db.") {
                table.metadata.unknown_features.push(UnknownFeature {
                    name: format!("prisma.field.{}.{}", field.name, attr.path),
                    value: mappers::raw_args_to_json(&attr.args),
                    recoverable: true,
                });
            }
        }
    }

    if !ignored_fields.is_empty() {
        table.metadata.unknown_features.push(UnknownFeature {
            name: "prisma.ignoredFields".to_string(),
            value: serde_json::json!(ignored_fields),
            recoverable: true,
        });
    }

    Ok(table)
}
