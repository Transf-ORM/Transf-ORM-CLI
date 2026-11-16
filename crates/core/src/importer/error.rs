use std::fmt;

/// Category of an [`ImportError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportErrorKind {
    /// The source file could not be parsed (syntax error).
    Parse,
    /// The parsed structure is semantically invalid (e.g. out-of-range literal).
    Schema,
}

impl fmt::Display for ImportErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportErrorKind::Parse => f.write_str("parse"),
            ImportErrorKind::Schema => f.write_str("schema"),
        }
    }
}

/// Position of a diagnostic in the source, with the offending line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column, counted in characters.
    pub column: usize,
    /// Number of characters to underline, starting at `column` (at least 1).
    pub len: usize,
    /// The full source line containing the error, without its line terminator.
    pub snippet: String,
}

impl SourceLocation {
    /// Builds a location from a byte range of `input`.
    ///
    /// The underline is clamped to the line containing `start`. Offsets past the
    /// end of `input` or inside a UTF-8 sequence are clamped to the nearest valid
    /// boundary.
    pub fn from_byte_range(input: &str, start: usize, end: usize) -> Self {
        let start = floor_char_boundary(input, start);
        let end = floor_char_boundary(input, end.max(start));

        let line_start = input[..start].rfind('\n').map_or(0, |i| i + 1);
        let line_end = input[start..].find('\n').map_or(input.len(), |i| start + i);

        let line = input[..start].matches('\n').count() + 1;
        let column = input[line_start..start].chars().count() + 1;
        let len = input[start..end.min(line_end)].chars().count().max(1);
        let snippet = input[line_start..line_end]
            .trim_end_matches('\r')
            .to_string();

        Self {
            line,
            column,
            len,
            snippet,
        }
    }
}

fn floor_char_boundary(input: &str, mut idx: usize) -> usize {
    idx = idx.min(input.len());
    while !input.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Error returned by any importer when parsing or resolving a schema fails.
///
/// Its [`Display`](fmt::Display) output is a compiler-style diagnostic:
///
/// ```text
/// parse error: expected a field type, found `}`
///  --> schema.prisma:7:8
///   |
/// 7 |   email
///   |        ^
/// ```
///
/// followed by an optional `= help: ...` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError(Box<Inner>);

// Boxed so that `Result<_, ImportError>` stays pointer-sized on the error path.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Inner {
    kind: ImportErrorKind,
    message: String,
    location: Option<SourceLocation>,
    path: Option<String>,
    help: Option<String>,
}

impl ImportError {
    /// Creates a [`ImportErrorKind::Parse`] error without a location.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(ImportErrorKind::Parse, message)
    }

    /// Creates a [`ImportErrorKind::Schema`] error without a location.
    pub fn schema(message: impl Into<String>) -> Self {
        Self::new(ImportErrorKind::Schema, message)
    }

    fn new(kind: ImportErrorKind, message: impl Into<String>) -> Self {
        Self(Box::new(Inner {
            kind,
            message: message.into(),
            location: None,
            path: None,
            help: None,
        }))
    }

    /// Attaches the source position the error points at.
    #[must_use]
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.0.location = Some(location);
        self
    }

    /// Attaches the path of the file being imported, shown in the `-->` line.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.0.path = Some(path.into());
        self
    }

    /// Attaches a hint on how to fix the error, shown as a `= help:` line.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.0.help = Some(help.into());
        self
    }

    /// The category of the error.
    pub fn kind(&self) -> ImportErrorKind {
        self.0.kind
    }

    /// The short, single-line description of the error.
    pub fn message(&self) -> &str {
        &self.0.message
    }

    /// Where the error occurred, when known.
    pub fn location(&self) -> Option<&SourceLocation> {
        self.0.location.as_ref()
    }

    /// The path of the imported file, when attached with [`ImportError::with_path`].
    pub fn path(&self) -> Option<&str> {
        self.0.path.as_deref()
    }

    /// A hint on how to fix the error, when one is known.
    pub fn help(&self) -> Option<&str> {
        self.0.help.as_deref()
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error: {}", self.0.kind, self.0.message)?;

        let Some(loc) = &self.0.location else {
            if let Some(path) = &self.0.path {
                write!(f, "\n --> {path}")?;
            }
            if let Some(help) = &self.0.help {
                write!(f, "\n  = help: {help}")?;
            }
            return Ok(());
        };

        let gutter = loc.line.to_string().len();
        let pad = "";
        let origin = self.path().map(|p| format!("{p}:")).unwrap_or_default();

        // Reproduce tabs in the caret prefix so the underline stays aligned.
        let prefix: String = loc
            .snippet
            .chars()
            .take(loc.column - 1)
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();

        write!(f, "\n{pad:gutter$}--> {origin}{}:{}", loc.line, loc.column)?;
        write!(f, "\n{pad:gutter$} |")?;
        write!(f, "\n{} | {}", loc.line, loc.snippet)?;
        write!(f, "\n{pad:gutter$} | {prefix}{}", "^".repeat(loc.len))?;
        if let Some(help) = &self.0.help {
            write!(f, "\n{pad:gutter$} |\n{pad:gutter$} = help: {help}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ImportError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_from_byte_range() {
        let input = "line one\n  bad token\nlast";
        let start = input.find("bad").unwrap();
        let loc = SourceLocation::from_byte_range(input, start, start + 3);
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 3);
        assert_eq!(loc.len, 3);
        assert_eq!(loc.snippet, "  bad token");
    }

    #[test]
    fn location_at_end_of_input() {
        let input = "a\nb";
        let loc = SourceLocation::from_byte_range(input, input.len(), input.len());
        assert_eq!((loc.line, loc.column, loc.len), (2, 2, 1));
        assert_eq!(loc.snippet, "b");
    }

    #[test]
    fn location_counts_columns_in_chars() {
        let input = "é = x";
        let start = input.find('x').unwrap();
        let loc = SourceLocation::from_byte_range(input, start, start + 1);
        assert_eq!(loc.column, 5);
    }

    #[test]
    fn display_without_location() {
        let err = ImportError::schema("something is off");
        assert_eq!(err.to_string(), "schema error: something is off");
    }

    #[test]
    fn display_with_help() {
        let input = "x";
        let err = ImportError::parse("oops")
            .with_location(SourceLocation::from_byte_range(input, 0, 1))
            .with_help("try y");
        assert_eq!(
            err.to_string(),
            "parse error: oops\n --> 1:1\n  |\n1 | x\n  | ^\n  |\n  = help: try y"
        );
    }

    #[test]
    fn display_keeps_tabs_aligned() {
        let input = "\tid Int ?";
        let start = input.find('?').unwrap();
        let err = ImportError::parse("oops").with_location(SourceLocation::from_byte_range(
            input,
            start,
            start + 1,
        ));
        assert_eq!(
            err.to_string(),
            "parse error: oops\n --> 1:9\n  |\n1 | \tid Int ?\n  | \t       ^"
        );
    }
}
