use std::{ffi::CStr, mem::ManuallyDrop};

use android_log_sys::c_int;
use ctor::ctor;
use java_spaghetti::{self, Global, ReferenceType, sys::jobject};

unsafe extern "C" {
    fn android_rust_initialization_vm() -> *mut java_spaghetti::sys::JavaVM;

    fn android_rust_initialization_application_context() -> jobject;

    fn android_rust_initialization_class_loader() -> jobject;
}

const TAG: &CStr = c"Rust Android Initialization";

#[ctor]
fn set_class_loader() {
    use android_log_sys::{
        __android_log_assert as android_log_assert, __android_log_write as android_log_write,
        LogPriority,
    };

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

/// Gets a global reference to the apps JVM.
pub fn get_vm() -> java_spaghetti::VM {
    let vm = unsafe { android_rust_initialization_vm() };
    assert!(!vm.is_null(), "VM is null");
    unsafe { java_spaghetti::VM::from_raw(vm) }
}

/// Returns a global reference to the application context.
/// # Safety
/// The generic parameter should have a type corresponding to `Context`. This can generally
/// be found at
/// ```text
/// [your crates bindings]::android::content::Context
/// ```
/// Any superclass of `Context` is also valid, however the only one is `java.lang.Object`.
pub unsafe fn get_application_context<T: ReferenceType>() -> Global<T> {
    let app_context = unsafe { android_rust_initialization_application_context() };
    assert!(!app_context.is_null(), "Application Context is null");
    // Do not drop the stored global reference, instead return a clone that is safe to drop
    let app_context = ManuallyDrop::new(unsafe { Global::from_raw(get_vm(), app_context) });
    Global::clone(&app_context)
}
