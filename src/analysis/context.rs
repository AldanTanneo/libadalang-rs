use std::{
    ffi::{CString, c_int},
    num::NonZeroU8,
};

use libadalang_sys::{
    ada_allocate_analysis_context, ada_analysis_context, ada_context_decref, ada_context_incref,
    ada_event_handler, ada_get_analysis_unit_from_file, ada_gpr_project_initialize_context,
};

use crate::{exception::Exception, gpr_project::GprProject};

use super::{GrammarRule, Unit};

/// Reference-counted analysis context.
#[repr(transparent)]
pub struct Context(ada_analysis_context);

impl Context {
    /// Build a new analysis context
    pub fn build(gpr_project: GprProject) -> ContextBuilder {
        ContextBuilder::new(gpr_project)
    }

    /// Create a new analysis context from a raw value
    ///
    /// # Safety
    /// The `raw` value must be a valid analysis context, or a null pointer.
    pub unsafe fn from_raw(raw: ada_analysis_context) -> Option<Self> {
        if raw.is_null() { None } else { Some(Self(raw)) }
    }

    pub fn get_unit_from_file(
        &self,
        filename: &str,
        charset: &str,
        reparse: bool,
        rule: GrammarRule,
    ) -> Unit {
        let filename = CString::new(filename).unwrap();
        let charset = CString::new(charset).unwrap();

        let unit = unsafe {
            ada_get_analysis_unit_from_file(
                self.0,
                filename.as_ptr(),
                charset.as_ptr(),
                reparse as c_int,
                rule,
            )
        };

        Exception::log_and_ignore();
        unsafe { Unit::from_raw(unit) }
    }
}

pub struct ContextBuilder {
    gpr_project: GprProject,
    subproject: Option<CString>,
    event_handler: Option<ada_event_handler>,
    with_trivia: bool,
    tab_stop: u8,
}

impl ContextBuilder {
    /// Creates a new context builder from a GPR project
    pub fn new(gpr_project: GprProject) -> Self {
        Self {
            gpr_project,
            subproject: None,
            event_handler: None,
            with_trivia: false,
            tab_stop: 3,
        }
    }

    /// Specify a subproject name to load.
    pub fn subproject(mut self, subproject: &str) -> Self {
        self.subproject = Some(CString::new(subproject).unwrap());
        self
    }

    /// TODO
    pub fn event_handler(mut self, event_handler: ()) -> Self {
        #![allow(unused)]
        todo!("Event handler")
    }

    pub fn with_trivia(mut self, with_trivia: bool) -> Self {
        self.with_trivia = with_trivia;
        self
    }

    /// Set the number of columns a tab represents
    pub fn tab_stop(mut self, tab_stop: NonZeroU8) -> Self {
        self.tab_stop = tab_stop.get();
        self
    }

    /// Consume the builder and build the context
    pub fn finish(self) -> Result<Context, Exception> {
        let ctx = unsafe { ada_allocate_analysis_context() };
        let ctx = Context(Exception::wrap(ctx)?);

        unsafe {
            ada_gpr_project_initialize_context(
                self.gpr_project.as_raw(),
                ctx.0,
                crate::ptr_or_null(&self.subproject),
                self.event_handler.unwrap_or(core::ptr::null_mut()),
                self.with_trivia as c_int,
                self.tab_stop as c_int,
            );
        }

        Exception::wrap(ctx)
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self(unsafe { ada_context_incref(self.0) })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ada_context_decref(self.0) };
        Exception::log_and_ignore();
    }
}
