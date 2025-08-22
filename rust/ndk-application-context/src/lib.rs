//! This crate allows for a form of autoinitialization of the [ndk-context](https://crates.io/crates/ndk-context)
//! from the android Application context.
//!
//! Note that it will **not** update the context based on the current activity.
//! # Usage
//! To use this crate, any dependents apps **must** also depend on the corresponding java package.
//! That packages `Initializer` must also run before this library is loaded.
//! See [`RustNSDExample.Java`](../../java/nsd_example_app/src/main/java/com/maticrobots/nsd_rs_example_app/RustNSDExample.java) as an example.

pub use ctor::ctor;
use std::ffi::{CStr, c_int};
use std::{ffi::c_void, sync::LazyLock};

#[cfg(target_os = "android")]
use android_log_sys::{__android_log_write as android_log_write, LogPriority};

#[cfg(target_os = "android")]
unsafe extern "C" {
    fn android_rust_initialization_vm() -> *mut c_void;

    fn android_rust_initialization_application_context() -> *mut c_void;

    #[cfg(feature = "java-spaghetti")]
    pub fn android_rust_initialization_class_loader() -> java_spaghetti::sys::jobject;
}

#[cfg(not(target_os = "android"))]
fn android_rust_initialization_vm() -> *mut c_void {
    unimplemented!("Only Available on Android")
}

#[cfg(not(target_os = "android"))]
fn android_rust_initialization_application_context() -> *mut c_void {
    unimplemented!("Only Available on Android")
}

#[cfg(all(not(target_os = "android"), feature = "java-spaghetti"))]
fn android_rust_initialization_class_loader() -> java_spaghetti::sys::jobject {
    unimplemented!("Only Available on Android")
}

const TAG: &CStr = c"Rust Android AutoInitialization";

#[cfg(target_os = "android")]
pub static NDK_CONTEXT_INITIALIZED: LazyLock<()> = LazyLock::new(|| {
    // Initialize NDK context with the application context
    unsafe {
        ndk_context::initialize_android_context(
            android_rust_initialization_vm(),
            android_rust_initialization_application_context(),
        );
    }

    #[cfg(target_os = "android")]
    unsafe {
        android_log_write(
            (LogPriority::VERBOSE as isize) as c_int,
            TAG.to_bytes().as_ptr().cast(),
            c"Set Class loader from application context"
                .to_bytes()
                .as_ptr()
                .cast(),
        );
    }

    #[cfg(feature = "java-spaghetti")]
    {
        set_class_loader();
    }
});

#[cfg(not(target_os = "android"))]
pub static NDK_CONTEXT_INITIALIZED: LazyLock<()> = LazyLock::new(|| ());

#[unsafe(no_mangle)]
#[ctor]
pub fn rust_android_initialize_context_ctor() {
    #[cfg(target_os = "android")]
    unsafe {
        android_log_write(
            (LogPriority::VERBOSE as isize) as c_int,
            TAG.to_bytes().as_ptr().cast(),
            c"Autoinitialization Ctor Running"
                .to_bytes()
                .as_ptr()
                .cast(),
        );
    }
    let _ = *NDK_CONTEXT_INITIALIZED;
}

#[cfg(feature = "java-spaghetti")]
pub fn set_class_loader() {
    #[cfg(target_os = "android")]
    use android_log_sys::__android_log_assert as android_log_assert;

    let class_loader = unsafe { android_rust_initialization_class_loader() };
    if class_loader.is_null() {
        #[cfg(target_os = "android")]
        unsafe {
            android_log_assert(
                c"Class loader is null".to_bytes().as_ptr().cast(),
                TAG.to_bytes().as_ptr().cast(),
                c"Could not initialize rust code."
                    .to_bytes()
                    .as_ptr()
                    .cast(),
            )
        }
    }
    unsafe { java_spaghetti::Env::<'_>::set_class_loader(class_loader) }
    #[cfg(target_os = "android")]
    unsafe {
        android_log_write(
            (LogPriority::VERBOSE as isize) as c_int,
            TAG.to_bytes().as_ptr().cast(),
            c"Set Class loader from application context"
                .to_bytes()
                .as_ptr()
                .cast(),
        );
    }
}
