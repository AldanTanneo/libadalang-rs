use std::{error::Error, ffi::CStr, fmt::Display, process::abort};

use libadalang_sys::{ada_exception_name, ada_get_last_exception};

pub type ExceptionKind = libadalang_sys::ada_exception_kind;

#[derive(Debug)]
pub struct Exception {
    kind: ExceptionKind,
    msg: Box<str>,
}

impl Exception {
    /// Get the last raised exception
    pub fn get_last() -> Option<Self> {
        let ptr = unsafe { ada_get_last_exception() };
        if ptr.is_null() {
            None
        } else {
            let ex = unsafe { core::ptr::read(ptr) };
            let res = Self {
                kind: ex.kind,
                msg: unsafe { CStr::from_ptr(ex.information) }
                    .to_string_lossy() // to avoid a panic. error messages should be unicode anyway.
                    .into(),
            };
            Some(res)
        }
    }

    /// Return `Err(...)` if the last operation raised an exception, or `Ok(val)` with the passed
    /// value.
    pub fn wrap<T>(val: T) -> Result<T, Self> {
        match Self::get_last() {
            None => Ok(val),
            Some(e) => Err(e),
        }
    }

    /// Log the last raised exception and continue.
    ///
    /// Use this in `Drop` implementations: we don't want a destructor to panic, as this could
    /// occur during unwinding and abort the process.
    pub fn log_and_ignore() {
        if let Some(e) = Self::get_last() {
            eprintln!("{e}");
        }
    }

    /// Abort if the last operation raised an exception.
    pub fn log_and_abort() {
        if let Some(e) = Self::get_last() {
            eprintln!("{e}");
            abort()
        }
    }

    /// Get the exception kind.
    pub fn kind(&self) -> ExceptionKind {
        self.kind
    }

    /// Get the exception message.
    pub fn message(&self) -> &str {
        &self.msg
    }
}

impl Display for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            unsafe { CStr::from_ptr(ada_exception_name(self.kind)).to_string_lossy() },
            self.msg
        )
    }
}

impl Error for Exception {}
