#![allow(non_camel_case_types)]

use crate::runtime::{Error, Scope, Value};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, mem::ManuallyDrop, path::PathBuf};

pub type tl_ext_name = unsafe extern "C" fn() -> *const std::ffi::c_char;
pub type tl_ext_init = unsafe extern "C" fn(scope: *mut Scope) -> FFIResult<Value, Error>;

/// Creates a runtime extension
#[macro_export]
macro_rules! extension {
    (
        name: $name:expr,
        init: $init:expr$(,)?
    ) => {
        use std::ffi::c_char;
        use tl::runtime::{Error, Scope, Value, ValueKind, extension::FFIResult, types::builtin};

        #[unsafe(no_mangle)]
        pub extern "C" fn tl_ext_name() -> *const c_char {
            concat!($name, "\0").as_ptr() as *const c_char
        }

        /// # Safety
        /// The `scope` pointer needs to be a pointer to a `tl::runtime::Scope` object.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn tl_ext_init(
            scope: *mut tl::runtime::Scope,
        ) -> FFIResult<Value, Error> {
            let scope: &mut Scope = unsafe { &mut *scope };

            let init: fn(scope: &mut Scope) -> Result<Value, Error> = $init;

            FFIResult::from(init(scope))
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry(pub HashMap<String, RegistryEntry>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub path: PathBuf,
}

impl Registry {
    pub fn read() -> Option<Self> {
        let registry_path = PathBuf::from(std::env::var("TL_REGISTRY").ok()?);
        let file = File::open(registry_path).ok()?;

        serde_json::from_reader(file).ok()
    }
}

/// Value that can be converted to and from `Result` to send rust values over the C ABI
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FFIResult<T, E> {
    pub ok: *const ManuallyDrop<T>,
    pub err: *const ManuallyDrop<E>,
}

impl<T, E> From<Result<T, E>> for FFIResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => {
                let md = ManuallyDrop::new(v);
                FFIResult {
                    ok: Box::into_raw(Box::new(md)),
                    err: std::ptr::null(),
                }
            }
            Err(v) => {
                let md = ManuallyDrop::new(v);
                FFIResult {
                    ok: std::ptr::null(),
                    err: Box::into_raw(Box::new(md)),
                }
            }
        }
    }
}

impl<T, E> From<FFIResult<T, E>> for Result<T, E> {
    fn from(value: FFIResult<T, E>) -> Self {
        unsafe {
            if !value.ok.is_null() {
                let md = Box::from_raw(value.ok.cast_mut());
                Ok(ManuallyDrop::into_inner(*md))
            } else if !value.err.is_null() {
                let md = Box::from_raw(value.err.cast_mut());
                Err(ManuallyDrop::into_inner(*md))
            } else {
                unreachable!("Invalid FFIResult: both ok and err are null");
            }
        }
    }
}
