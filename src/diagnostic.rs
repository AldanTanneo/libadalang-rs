use libadalang_sys::ada_diagnostic;

use crate::text::Text;

pub use libadalang_sys::{
    ada_source_location as SourceLocation, ada_source_location_range as SourceLocationRange,
};

pub struct Diagnostic {
    pub sloc_range: SourceLocationRange,
    pub message: String,
}

impl Diagnostic {
    pub fn from_raw(raw: ada_diagnostic) -> Self {
        Self {
            sloc_range: raw.sloc_range,
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
        sloc_range: SourceLocationRange {
            start: SourceLocation {
                line: sloc_range[0].0,
                column: sloc_range[0].1,
            },
            end: SourceLocation {
                line: sloc_range[1].0,
                column: sloc_range[1].1,
            },
        },
        message: Text::new(msg).into_raw(),
    }
}
