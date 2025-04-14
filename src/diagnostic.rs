use libadalang_sys::{ada_diagnostic, ada_source_location, ada_source_location_range};

use crate::text::Text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u16,
}

impl From<&ada_source_location> for SourceLocation {
    fn from(value: &ada_source_location) -> Self {
        Self {
            line: value.line,
            column: value.column,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocationRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl From<&ada_source_location_range> for SourceLocationRange {
    fn from(value: &ada_source_location_range) -> Self {
        Self {
            start: (&value.start).into(),
            end: (&value.end).into(),
        }
    }
}

pub struct Diagnostic {
    pub sloc_range: SourceLocationRange,
    pub message: String,
}

impl Diagnostic {
    pub fn from_raw(raw: ada_diagnostic) -> Self {
        Self {
            sloc_range: (&raw.sloc_range).into(),
            message: Text::from_raw_borrow(&raw.message).to_string(),
        }
    }

    pub fn to_raw(&self) -> ada_diagnostic {
        make_diag(
            [
                (self.sloc_range.start.line, self.sloc_range.start.column),
                (self.sloc_range.end.line, self.sloc_range.end.column),
            ],
            &self.message,
        )
    }
}

pub(crate) fn make_diag(sloc_range: [(u32, u16); 2], msg: &str) -> ada_diagnostic {
    ada_diagnostic {
        sloc_range: ada_source_location_range {
            start: ada_source_location {
                line: sloc_range[0].0,
                column: sloc_range[0].1,
            },
            end: ada_source_location {
                line: sloc_range[1].0,
                column: sloc_range[1].1,
            },
        },
        message: Text::new(msg).into_raw(),
    }
}
