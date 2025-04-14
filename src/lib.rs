//! High level Rust bindings for Libadalang

pub mod analysis;
pub mod diagnostic;
pub mod event_handler;
pub mod exception;
pub mod file_reader;
pub mod gpr_project;
pub mod text;

use std::{
    ffi::{CString, c_char},
    fmt::Display,
};

use exception::Exception;

pub(crate) fn ptr_or_null(opt: &Option<CString>) -> *const c_char {
    opt.as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or(core::ptr::null())
}

pub enum Error {
    Exception(Exception),
    Custom(String),
}

impl Error {
    pub fn custom(msg: impl Display) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl From<Exception> for Error {
    fn from(value: Exception) -> Self {
        Self::Exception(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}
