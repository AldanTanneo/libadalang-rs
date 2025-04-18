//! Low level bindings for Libadalang.
//!
//! Generated with [`bindgen`](https://github.com/rust-lang/rust-bindgen).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(rustdoc::broken_intra_doc_links)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Default for ada_grammar_rule {
    fn default() -> Self {
        ada_grammar_rule::COMPILATION
    }
}
