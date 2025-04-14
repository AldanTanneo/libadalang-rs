//! Analysis units

use std::{
    ffi::{CStr, c_void},
    mem::MaybeUninit,
};

use libadalang_sys::{
    ada_analysis_unit, ada_diagnostic, ada_unit_context, ada_unit_diagnostic,
    ada_unit_diagnostic_count, ada_unit_filename, ada_unit_token_count, ada_unit_trivia_count,
};

use crate::diagnostic::Diagnostic;

use super::Context;

/// A libadalang analysis unit
pub struct Unit(ada_analysis_unit);

impl Unit {
    /// Create a new analysis unit from its raw value
    ///
    /// # Safety
    /// The `raw` parameter must be a valid analysis unit.
    pub unsafe fn from_raw(raw: ada_analysis_unit) -> Self {
        Self(raw)
    }

    /// Return the context that owns this unit.
    pub fn context(&self) -> Option<Context> {
        unsafe { Context::from_raw(ada_unit_context(self.0)) }
    }

    /// Return the filename this unit is associated to.
    pub fn filename(&self) -> String {
        let ptr = unsafe { ada_unit_filename(self.0) };
        let res = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_owned();
        unsafe { libadalang_sys::ada_free(ptr as *mut c_void) };
        res
    }

    pub fn token_count(&self) -> usize {
        let cnt = unsafe { ada_unit_token_count(self.0) };
        usize::try_from(cnt).unwrap()
    }

    pub fn trivia_count(&self) -> usize {
        let cnt = unsafe { ada_unit_trivia_count(self.0) };
        usize::try_from(cnt).unwrap()
    }

    pub fn diagnostic_count(&self) -> u32 {
        let cnt = unsafe { ada_unit_diagnostic_count(self.0) };
        u32::try_from(cnt).unwrap()
    }

    pub fn get_diagnostic(&self, idx: u32) -> Option<Diagnostic> {
        let mut diag = MaybeUninit::<ada_diagnostic>::uninit();
        let found = unsafe { ada_unit_diagnostic(self.0, idx, diag.as_mut_ptr()) };
        if found == 0 {
            None
        } else {
            let res = Diagnostic::from_raw(unsafe { diag.assume_init() });
            Some(res)
        }
    }
}
