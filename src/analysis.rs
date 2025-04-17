pub mod context;
pub mod unit;

pub use context::Context;
pub use unit::Unit;

use libadalang_sys::ada_grammar_rule;

/// Enumeration of the Ada grammar rules implemented in Libadalang.
///
/// The bindgen-generated documentation on the type is incorrect.
pub type GrammarRule = ada_grammar_rule;
