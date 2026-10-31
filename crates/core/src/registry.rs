use std::path::{Path, PathBuf};

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
    let orm = ORMS.iter().find(|o| o.extensions.contains(&ext));
    tracing::debug!(
        path,
        ext,
        orm = orm.map(|o| o.name),
        "detected ORM from extension"
    );
    orm
}

/// Directories never searched by [`detect_in_dir`]: dependencies and build
/// output. Hidden directories (`.git`, `.next`…) are skipped too.
const IGNORED_DIRS: &[&str] = &["node_modules", "target", "dist", "build"];

/// How deep [`detect_in_dir`] looks below the starting directory
/// (`prisma/schema.prisma` is at depth 1).
const MAX_DEPTH: usize = 4;

/// A schema file found by [`detect_in_dir`].
pub struct DetectedSchema {
    /// ORM the file belongs to.
    pub orm: &'static OrmDescriptor,
    /// Path of the file, relative to the searched directory.
    pub path: PathBuf,
}

/// Finds the schema files of known ORMs in `dir` and its subdirectories.
///
/// Only ORMs whose extension is unambiguous (`certain`) are detected: a
/// `.ts` file, for instance, is not necessarily a Drizzle schema. Hidden
/// directories, dependencies and build output are skipped, and the search
/// stops a few levels deep.
///
/// # Arguments
///
/// * `dir` - Directory to search, usually the current directory
///
/// # Returns
///
/// The files found, sorted by path; empty when no ORM is detected.
/// Unreadable directories are skipped.
///
/// # Examples
///
/// ```
/// let found = transf_orm_core::registry::detect_in_dir(std::path::Path::new("."));
/// for schema in found {
///     println!("{}: {}", schema.orm.display_name, schema.path.display());
/// }
/// ```
pub fn detect_in_dir(dir: &Path) -> Vec<DetectedSchema> {
    let mut found = Vec::new();
    collect_schemas(dir, Path::new(""), 0, &mut found);
    found.sort_by(|a, b| a.path.cmp(&b.path));
    tracing::debug!(dir = %dir.display(), count = found.len(), "detected schemas in directory");
    found
}

fn collect_schemas(root: &Path, relative: &Path, depth: usize, found: &mut Vec<DetectedSchema>) {
    let Ok(entries) = std::fs::read_dir(root.join(relative)) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let path = relative.join(&name);
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            let name = name.to_string_lossy();
            if depth < MAX_DEPTH && !name.starts_with('.') && !IGNORED_DIRS.contains(&name.as_ref())
            {
                collect_schemas(root, &path, depth + 1, found);
            }
        } else if let Some(orm) = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| {
                ORMS.iter()
                    .find(|o| o.certain && o.extensions.contains(&ext))
            })
        {
            found.push(DetectedSchema { orm, path });
        }
    }
}

/// Find an ORM descriptor by its CLI name (case-insensitive).
pub fn find_by_name(name: &str) -> Option<&'static OrmDescriptor> {
    let orm = ORMS.iter().find(|o| o.name.eq_ignore_ascii_case(name));
    tracing::debug!(name, found = orm.is_some(), "looked up ORM by name");
    orm
}

/// List CLI names of all known ORMs.
pub fn names() -> Vec<&'static str> {
    ORMS.iter().map(|o| o.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates an empty directory unique to `name`, removed first if it exists.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("transf-orm-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(dir: &Path, path: &str) {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    fn paths(found: &[DetectedSchema]) -> Vec<PathBuf> {
        found.iter().map(|s| s.path.clone()).collect()
    }

    #[test]
    fn detect_in_dir_finds_nested_schemas_sorted() {
        let dir = temp_dir("nested");
        touch(&dir, "prisma/schema.prisma");
        touch(&dir, "a.prisma");
        touch(&dir, "src/index.ts");

        let found = detect_in_dir(&dir);
        assert_eq!(
            paths(&found),
            [
                PathBuf::from("a.prisma"),
                Path::new("prisma").join("schema.prisma")
            ]
        );
        assert!(found.iter().all(|s| s.orm.name == "prisma"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn detect_in_dir_skips_ignored_hidden_and_deep_dirs() {
        let dir = temp_dir("ignored");
        touch(&dir, "node_modules/pkg/schema.prisma");
        touch(&dir, ".git/schema.prisma");
        touch(&dir, "a/b/c/d/e/schema.prisma");

        assert!(detect_in_dir(&dir).is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn detect_in_dir_on_missing_dir_is_empty() {
        assert!(detect_in_dir(Path::new("/nonexistent/transf-orm")).is_empty());
    }
}
