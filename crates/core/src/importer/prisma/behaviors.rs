use super::ast::RawField;

use crate::pivot::table::Behavior;
use crate::pivot::types::{DefaultFn, DefaultValue};

/// Detects Prisma conventions that map to first-class ORM behaviors.
///
/// Prisma has no dedicated attributes for these behaviors, so detection is
/// based on the conventional field name, type and default value.
pub(super) fn detect_behaviors(
    field: &RawField,
    effective_base: &str,
    default: &Option<DefaultValue>,
) -> Vec<Behavior> {
    let mut behaviors = Vec::new();

    // createdAt DateTime @default(now())
    if field.name == "createdAt"
        && effective_base == "DateTime"
        && !field.is_optional
        && !field.is_array
        && matches!(default, Some(DefaultValue::Function(DefaultFn::Now)))
    {
        behaviors.push(Behavior::CreatedAt {
            column: field.name.clone(),
        });
    }

    // deletedAt DateTime?
    if field.name == "deletedAt"
        && effective_base == "DateTime"
        && field.is_optional
        && !field.is_array
    {
        behaviors.push(Behavior::SoftDelete {
            column: field.name.clone(),
        });
    }

    // version Int
    if field.name == "version" && effective_base == "Int" && !field.is_optional && !field.is_array {
        behaviors.push(Behavior::Versioning {
            column: field.name.clone(),
        });
    }

    behaviors
}
