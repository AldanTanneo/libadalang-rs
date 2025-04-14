//! GPR project loading

use std::ffi::{CStr, CString};

use libadalang_sys::{
    ada_free_string_array, ada_gpr_project, ada_gpr_project_scenario_variable as ada_scenario_var,
    ada_string_array_ptr,
};

use crate::{Error, exception::Exception};

/// An opaque GPR project wrapper. Can be used to build a new analysis context.
pub struct GprProject {
    inner: ada_gpr_project,
    // the ada_gpr_project may still hold references to values in
    // the builder? this field keeps them alive with it.
    _builder: GprProjectBuilder,
}

/// Builder for the GPR project type.
pub struct GprProjectBuilder {
    project_file: Option<CString>,
    scenario_vars: Vec<ada_scenario_var>,
    target: Option<CString>,
    runtime: Option<CString>,
    config_file: Option<CString>,
    ada_only: bool,
}

impl GprProjectBuilder {
    /// Create a new project builder from an explicit project file.
    pub fn new(project_file: &str) -> Self {
        Self {
            project_file: Some(CString::new(project_file).unwrap()),
            scenario_vars: Vec::new(),
            target: None,
            runtime: None,
            config_file: None,
            ada_only: false,
        }
    }

    /// Create a new implicit project builder. Loads an implicit project in the current directory.
    pub fn new_implicit() -> Self {
        Self {
            project_file: None,
            scenario_vars: Vec::new(),
            target: None,
            runtime: None,
            config_file: None,
            ada_only: false,
        }
    }

    /// Set a scenario variable. Panics if no project file is set.
    pub fn scenario_var(mut self, name: &str, value: &str) -> Self {
        assert!(
            self.project_file.is_some(),
            "cannot set scenario vars in an implicit project"
        );

        self.scenario_vars.push(ada_scenario_var {
            name: CString::new(name).unwrap().into_raw(),
            value: CString::new(value).unwrap().into_raw(),
        });
        self
    }

    /// Set several scenario variables. Panics if no project file is set.
    pub fn scenario_vars<'a, I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        assert!(
            self.project_file.is_some(),
            "cannot set scenario vars in an implicit project"
        );

        for (name, value) in iter {
            self = self.scenario_var(name, value);
        }
        self
    }

    /// Set the project's target
    pub fn target(mut self, target: &str) -> Self {
        self.target = Some(CString::new(target).unwrap());
        self
    }

    /// Set the project's runtime
    pub fn runtime(mut self, runtime: &str) -> Self {
        self.runtime = Some(CString::new(runtime).unwrap());
        self
    }

    /// Set the project's config file
    pub fn config_file(mut self, config_file: &str) -> Self {
        self.config_file = Some(CString::new(config_file).unwrap());
        self
    }

    /// Set if the project only has Ada sources.
    pub fn ada_only(mut self, ada_only: bool) -> Self {
        assert!(
            self.project_file.is_some(),
            "cannot set `ada_only` in an implicit project"
        );

        self.ada_only = ada_only;
        self
    }

    /// Load the project
    pub fn load(mut self) -> Result<GprProject, crate::Error> {
        let mut project: ada_gpr_project = core::ptr::null_mut();
        let mut errors: ada_string_array_ptr = core::ptr::null_mut();

        if let Some(project_file) = self.project_file.as_ref() {
            if !self.scenario_vars.is_empty() {
                // append a zeroed scenario var for the library to detect the end of the array
                self.scenario_vars.push(ada_scenario_var {
                    name: core::ptr::null_mut(),
                    value: core::ptr::null_mut(),
                });
            }

            let scenario_vars_ptr = if self.scenario_vars.is_empty() {
                core::ptr::null()
            } else {
                self.scenario_vars.as_ptr()
            };

            unsafe {
                libadalang_sys::ada_gpr_project_load(
                    project_file.as_ptr(),
                    scenario_vars_ptr,
                    crate::ptr_or_null(&self.target),
                    crate::ptr_or_null(&self.runtime),
                    crate::ptr_or_null(&self.config_file),
                    self.ada_only as _,
                    &raw mut project,
                    &raw mut errors,
                )
            };
        } else {
            unsafe {
                libadalang_sys::ada_gpr_project_load_implicit(
                    crate::ptr_or_null(&self.target),
                    crate::ptr_or_null(&self.runtime),
                    crate::ptr_or_null(&self.config_file),
                    &raw mut project,
                    &raw mut errors,
                )
            };
        }

        if let Some(err) = Exception::get_last() {
            return Err(crate::Error::Exception(err));
        }

        if project.is_null() || (!errors.is_null() && unsafe { (*errors).length != 0 }) {
            let array = if errors.is_null() {
                &[]
            } else {
                unsafe { core::slice::from_raw_parts((*errors).c_ptr, (*errors).length as usize) }
            };

            let errs = array
                .iter()
                .map(|&ptr| unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_owned())
                .collect::<Vec<_>>()
                .join("\n");

            if errs.is_empty() {
                return Err(Error::custom("invalid project"));
            }

            unsafe { ada_free_string_array(errors) };

            Err(errs.into())
        } else {
            Ok(GprProject {
                inner: project,
                _builder: self,
            })
        }
    }
}

impl Drop for GprProjectBuilder {
    fn drop(&mut self) {
        for ada_scenario_var { name, value } in self.scenario_vars.drain(..) {
            drop(unsafe { CString::from_raw(name) });
            drop(unsafe { CString::from_raw(value) });
        }
    }
}

impl GprProject {
    /// Build a new GPR project
    pub fn build(project_file: &str) -> GprProjectBuilder {
        GprProjectBuilder::new(project_file)
    }

    /// Build a new implicit GPR project
    pub fn build_implicit() -> GprProjectBuilder {
        GprProjectBuilder::new_implicit()
    }

    /// Get the inner raw pointer to the GPR project
    pub fn as_raw(&self) -> ada_gpr_project {
        self.inner
    }
}

impl Drop for GprProject {
    fn drop(&mut self) {
        unsafe { libadalang_sys::ada_gpr_project_free(self.inner) };
    }
}
