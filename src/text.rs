//! Utilities to deal with text in libadalang

use std::{fmt::Display, mem::MaybeUninit};

use libadalang_sys::{ada_destroy_text, ada_text, ada_text_from_utf8};

use crate::exception::Exception;

/// A text buffer from libadalang.
///
/// Lifetime rules are hazy, so copy it into a Rust string when you must keep a reference to it.
/// The Rust owned wrapper will only hold owning copies, and as such should be safe to keep around.
#[repr(transparent)]
pub struct Text(ada_text);

impl Text {
    /// Allocate a new `Text` from a utf-8 string.
    pub fn new(str: &str) -> Self {
        let mut res = ada_text {
            chars: core::ptr::null_mut(),
            length: 0,
            is_allocated: 0,
        };

        unsafe { ada_text_from_utf8(str.as_ptr() as *const i8, str.len(), &raw mut res) };

        if let Some(err) = Exception::get_last() {
            panic!("{err}");
        }

        Text(res)
    }

    /// Unwrap a `Text` into its inner (owning) `ada_text`
    pub fn into_raw(self) -> ada_text {
        let maybe = MaybeUninit::new(self);
        let ptr = maybe.as_ptr() as *const ada_text;
        // SAFETY: the text will not be dropped in a MaybeUninit, and it is a transparent struct.
        // It is safe to read an `ada_text` from it.
        unsafe { core::ptr::read(ptr) }
    }

    /// Gets a non-owning copy of the inner `ada_text` wrapped by this `Text` value.
    pub fn as_raw_borrow(&self) -> ada_text {
        let ptr = &raw const self.0;
        let mut raw = unsafe { core::ptr::read(ptr) };
        raw.is_allocated = 0;
        raw
    }

    /// Create a `Text` from a raw `ada_text` value. Returns None is the `ada_text` is not owning,
    /// as we cannot track its lifetime from the Rust side.
    pub fn from_raw(value: ada_text) -> Option<Text> {
        if value.is_allocated == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Create a reference to a `Text` from a potentially non-owning `ada_text`.
    ///
    /// This value is tied to the lifetime of the `ada_text`, but it should not be kept around too
    /// long, as the real lifetime of an `ada_text` object is not encoded in the Rust type system.
    pub fn from_raw_borrow<'a>(value: &'a ada_text) -> &'a Text {
        // SAFETY: the `Text` struct is transparent
        unsafe { core::mem::transmute::<&'a ada_text, &'a Text>(value) }
    }
}

impl AsRef<[char]> for Text {
    fn as_ref(&self) -> &[char] {
        let (ptr, len) = if self.0.chars.is_null() {
            (core::ptr::dangling(), 0)
        } else {
            (self.0.chars as *const char, self.0.length)
        };

        // SAFETY: the chars pointers contains valid Unicode code points, and char is guaranteed to
        // have the same size, alignment, and function call ABI as u32 on all platforms.
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<&Text> for String {
    fn from(value: &Text) -> Self {
        String::from_iter(value.as_ref())
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        unsafe { ada_destroy_text(&raw mut self.0) };
    }
}
