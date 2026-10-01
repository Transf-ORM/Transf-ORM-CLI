use crate::exporter::Exporter;
use crate::importer::prisma::PrismaImporter;
use crate::importer::Importer;

/// Describes an ORM known to the tool — its CLI name, file extensions,
/// and optional importer/exporter factories.
pub struct OrmDescriptor {
    /// Lowercase CLI name used with `--from` / `--to`.
    pub name: &'static str,
    /// Human-readable name for display messages.
    pub display_name: &'static str,
    /// File extensions that identify this ORM's schema files.
    pub extensions: &'static [&'static str],
    /// `true` when the extension alone is unambiguous (no confirmation needed).
    pub certain: bool,
    /// Factory for an importer, if this ORM can be used as a source.
    pub make_importer: Option<fn() -> Box<dyn Importer>>,
    /// Factory for an exporter, if this ORM can be used as a target.
    pub make_exporter: Option<fn() -> Box<dyn Exporter>>,
}

/// All ORMs known to the tool.
///
/// Adding a new ORM = adding one entry here. Nothing else changes.
pub static ORMS: &[OrmDescriptor] = &[OrmDescriptor {
    name: "prisma",
    display_name: "Prisma",
    extensions: &["prisma"],
    certain: true,
    make_importer: Some(|| Box::new(PrismaImporter)),
    make_exporter: None,
}];

/// Detect the ORM from a file path's extension.
pub fn detect(path: &str) -> Option<&'static OrmDescriptor> {
    let ext = std::path::Path::new(path).extension()?.to_str()?;
    ORMS.iter().find(|o| o.extensions.contains(&ext))
}

/// Find an ORM descriptor by its CLI name (case-insensitive).
pub fn find_by_name(name: &str) -> Option<&'static OrmDescriptor> {
    ORMS.iter().find(|o| o.name.eq_ignore_ascii_case(name))
}

/// List CLI names of all known ORMs.
pub fn names() -> Vec<&'static str> {
    ORMS.iter().map(|o| o.name).collect()
}
