use std::mem::ManuallyDrop;

use java_spaghetti::{Global, ReferenceType, VM};

/// Gets a global reference to the apps JVM.
pub fn get_vm() -> VM {
    let ctx = ndk_context::android_context();
    let vm = ctx.vm();
    assert!(!vm.is_null(), "VM is null");
    unsafe { VM::from_raw(vm.cast()) }
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
    let app_context = ndk_context::android_context();
    // Do not drop the stored global reference, instead return a clone that is safe to drop
    let app_context =
        ManuallyDrop::new(unsafe { Global::from_raw(get_vm(), app_context.context().cast()) });
    Global::clone(&app_context)
}
