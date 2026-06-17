// Raw intermediate types produced by the PEG parser (Phase 1).
// These are internal to the prisma importer — never exposed publicly.

#[derive(Debug)]
pub(super) struct RawAttr {
    pub(super) path: String,
    pub(super) args: Vec<RawArg>,
}

impl RawAttr {
    pub(super) fn named(&self, key: &str) -> Option<&RawValue> {
        self.args.iter().find_map(|a| match a {
            RawArg::Named(k, v) if k == key => Some(v),
            _ => None,
        })
    }

    pub(super) fn positional(&self, idx: usize) -> Option<&RawValue> {
        self.args
            .iter()
            .filter_map(|a| match a {
                RawArg::Positional(v) => Some(v),
                _ => None,
            })
            .nth(idx)
    }
}

#[derive(Debug, Clone)]
pub(super) enum RawArg {
    Named(String, RawValue),
    Positional(RawValue),
}

#[derive(Debug, Clone)]
pub(super) enum RawValue {
    Str(String),
    Int(i64),
    Float(f64),
    Ident(String),
    Array(Vec<RawValue>),
    /// A function call — args use [`RawArg`] so named args (`sort: Desc`) are preserved.
    Call(String, Vec<RawArg>),
}

impl RawValue {
    pub(super) fn as_str(&self) -> Option<&str> {
        match self {
            RawValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(super) fn as_int(&self) -> Option<i64> {
        match self {
            RawValue::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub(super) fn as_ident(&self) -> Option<&str> {
        match self {
            RawValue::Ident(s) => Some(s),
            _ => None,
        }
    }

    pub(super) fn as_string_array(&self) -> Vec<String> {
        match self {
            RawValue::Array(items) => items
                .iter()
                .filter_map(|v| v.as_ident().or_else(|| v.as_str()).map(String::from))
                .collect(),
            _ => vec![],
        }
    }
}

#[derive(Debug)]
pub(super) struct RawField {
    pub(super) name: String,
    /// Set when the field type is `Unsupported("...")`.
    pub(super) unsupported_type: Option<String>,
    pub(super) base_type: String,
    pub(super) is_optional: bool,
    pub(super) is_array: bool,
    pub(super) attrs: Vec<RawAttr>,
}

impl RawField {
    pub(super) fn attr(&self, path: &str) -> Option<&RawAttr> {
        self.attrs.iter().find(|a| a.path == path)
    }
}

#[derive(Debug)]
pub(super) struct RawModel {
    pub(super) name: String,
    pub(super) fields: Vec<RawField>,
    pub(super) block_attrs: Vec<RawAttr>,
}

impl RawModel {
    pub(super) fn block_attr(&self, path: &str) -> Option<&RawAttr> {
        self.block_attrs.iter().find(|a| a.path == path)
    }

    pub(super) fn block_attrs_all<'a>(
        &'a self,
        path: &'a str,
    ) -> impl Iterator<Item = &'a RawAttr> {
        self.block_attrs.iter().filter(move |a| a.path == path)
    }
}

/// A Prisma `type` alias — a scalar type with optional default attributes.
#[derive(Debug)]
pub(super) struct RawTypeAlias {
    pub(super) name: String,
    pub(super) base_type: String,
    pub(super) attrs: Vec<RawAttr>,
}
