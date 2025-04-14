pub mod context;
pub mod unit;

pub use context::Context;
use libadalang_sys::ada_grammar_rule;
pub use unit::Unit;

/// Enumeration of the Ada grammar rules implemented in Libadalang.
///
/// The bindgen-generated documentation on the type is incorrect.
pub type GrammarRule = ada_grammar_rule;
