use std::{
    ffi::{CStr, c_char, c_int, c_void},
    mem::ManuallyDrop,
    ptr::NonNull,
};

use libadalang_sys::{
    ada_create_file_reader, ada_dec_ref_file_reader, ada_diagnostic, ada_file_reader,
    ada_file_reader__struct, ada_text,
};

use crate::{
    diagnostic::{self, Diagnostic},
    exception::Exception,
    text::Text,
};

/// A file reader wrapping a custom callback
pub struct FileReader(
    // use a non-null pointer for niche optimisations :)
    NonNull<ada_file_reader__struct>,
);

/// The file request sent to the callback
pub struct FileRequest<'a> {
    pub filename: &'a str,
    pub charset: &'a str,
    pub read_bom: bool,
}

/// SAFETY: the `data` pointer must point to a valid `F`,
/// allocated with a `Box<T>`.
unsafe extern "C-unwind" fn destroy_callback<F>(data: *mut c_void) {
    let ptr = data as *mut F;
    let boxed: Box<F> = unsafe { Box::from_raw(ptr) };
    drop(boxed)
}

/// SAFETY: the `data` pointer must point to a valid `F`.
///
/// `filename` and `charset` must point to valid null terminated strings.
///
/// The `buffer` and `diagnostic` pointers must point to valid memory region where the results
/// can be stored.
unsafe extern "C-unwind" fn read_callback<F>(
    data: *mut c_void,
    filename: *const c_char,
    charset: *const c_char,
    read_bom: c_int,
    buffer: *mut ada_text,
    diagnostic: *mut ada_diagnostic,
) where
    for<'a> F: FnMut(FileRequest<'a>) -> Result<String, Diagnostic> + 'static,
{
    // returns static CStr, but the string may not be static!
    // do not use outside of this function.
    //
    // we consider it "safe" because the resulting string is only used as an argument to the
    // callback, which cannot smuggle it out because it must be valid for any lifetime.
    unsafe fn to_cstr(ptr: *const c_char) -> &'static CStr {
        if ptr.is_null() {
            c""
        } else {
            unsafe { CStr::from_ptr(ptr) }
        }
    }

    let ptr = data as *mut F;
    let read_cb: &mut F = unsafe { &mut *ptr };

    let Ok(filename) = unsafe { to_cstr(filename) }.to_str() else {
        let diag = diagnostic::make_diag([(0, 0), (0, 0)], "invalid utf-8 in filename");
        unsafe { core::ptr::write(diagnostic, diag) };
        return;
    };
    let Ok(charset) = unsafe { to_cstr(charset) }.to_str() else {
        let diag = diagnostic::make_diag([(0, 0), (0, 0)], "invalid utf-8 in charset name");
        unsafe { core::ptr::write(diagnostic, diag) };
        return;
    };

    match read_cb(FileRequest {
        filename,
        charset,
        read_bom: read_bom != 0,
    }) {
        Ok(string) => {
            let text = Text::new(&string);
            unsafe { core::ptr::write(buffer, text.into_raw()) };
        }
        Err(diag) => {
            unsafe { core::ptr::write(diagnostic, diag.to_raw()) };
        }
    }
}

impl FileReader {
    /// Create a new file reader with a custom static fallback.
    ///
    /// To execute custom code on drop, capture a type with custom drop glue in the closure.
    pub fn new<F>(cb: F) -> Result<Self, Exception>
    where
        for<'a> F: FnMut(FileRequest<'a>) -> Result<String, Diagnostic> + 'static,
    {
        let boxed = Box::new(cb);
        let ptr = Box::into_raw(boxed);
        let res = unsafe {
            ada_create_file_reader(
                ptr as *mut c_void,
                Some(destroy_callback::<F>),
                Some(read_callback::<F>),
            )
        };

        let res = NonNull::new(Exception::wrap(res)?).unwrap();
        Ok(Self(res))
    }

    pub fn into_raw(self) -> ada_file_reader {
        let no_drop = ManuallyDrop::new(self);
        no_drop.0.as_ptr()
    }
}

impl Drop for FileReader {
    fn drop(&mut self) {
        unsafe { ada_dec_ref_file_reader(self.0.as_ptr()) };
        if let Some(err) = Exception::get_last() {
            eprintln!("{err}");
        }
    }
}
