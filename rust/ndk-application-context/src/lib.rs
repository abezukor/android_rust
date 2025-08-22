pub use ctor::ctor;
use std::ffi::{CStr, c_int};
use std::{ffi::c_void, sync::LazyLock};

use android_log_sys::{__android_log_write as android_log_write, LogPriority};

unsafe extern "C" {
    fn android_rust_initialization_vm() -> *mut c_void;

    fn android_rust_initialization_application_context() -> *mut c_void;

    #[cfg(feature = "java-spaghetti")]
    pub fn android_rust_initialization_class_loader() -> java_spaghetti::sys::jobject;
}

const TAG: &CStr = c"Rust Android AutoInitialization";

pub static NDK_CONTEXT_INITIALIZED: LazyLock<()> = LazyLock::new(|| {
    // Initialize NDK context with the application context
    unsafe {
        ndk_context::initialize_android_context(
            android_rust_initialization_vm(),
            android_rust_initialization_application_context(),
        );
    }
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

#[unsafe(no_mangle)]
#[ctor]
pub fn rust_android_initialize_context_ctor() {
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
    use android_log_sys::__android_log_assert as android_log_assert;

    let class_loader = unsafe { android_rust_initialization_class_loader() };
    if class_loader.is_null() {
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
