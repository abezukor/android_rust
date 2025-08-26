//! These functions (along with the `RustArcBoxDynAny` java class) provide a basic mechanism
//! for Java to "own" a rust object,
#![warn(missing_docs)]

use std::{any::Any, mem::ManuallyDrop, ptr::with_exposed_provenance, sync::Arc};

use java_spaghetti::{
    Env, Local,
    sys::{jlong, jobject},
};
use java_spaghetti_result::JavaResult;

#[allow(mismatched_lifetime_syntaxes)]
#[allow(missing_docs)]
#[rustfmt::skip]
pub mod bindings;

use crate::bindings::com::maticrobots::java_rust_obj::RustArcBoxDynAny;

/// The Rust object that java can own.
/// It needs to be `Arc<Box<dyn Any>>` because we need to be able to use Arc::from_raw and Arc::into_raw,
/// and those need to return thin pointers so that they fit in a java long
///
/// `dyn Any` is used as opposed to concrete type so that downcasts can be type safe and not cause undefined behavior.
pub type BoxedRustObj = Box<dyn Any + Send + Sync + 'static>;

/// Create A Java Object that wraps a rust object
pub fn to_java(
    env: Env,
    rust_obj: impl Any + Send + Sync + 'static,
) -> JavaResult<Local<RustArcBoxDynAny>> {
    to_java_arc(env, Arc::new(Box::new(rust_obj)))
}

/// Create A Java Object that wraps a rust object from an existing arc.
pub fn to_java_arc(env: Env, rust_obj: Arc<BoxedRustObj>) -> JavaResult<Local<RustArcBoxDynAny>> {
    let raw = Arc::into_raw(rust_obj);
    // raw.expose_provenance() can be 32 bit

    // Java does not have a concept of unsigned integers so we have to reinterpret this as an i64.
    Ok(RustArcBoxDynAny::new(
        env,
        raw.expose_provenance() as jlong,
    )?)
}

/// Get a rust object from a pointer to a RustArcBoxDynAny java class.
/// # Safety
///
/// Raw PRT must be the rust ptr inside a from a `RustArcBoxDynAny` java class,
pub unsafe fn get_ref(raw_ptr: jlong) -> Arc<BoxedRustObj> {
    let rust_ptr: *const BoxedRustObj = with_exposed_provenance(raw_ptr as usize);
    // Make sure to return a cloned copy, so users of this fn CANNOT delete the arc owned by java.
    let java_owned = ManuallyDrop::new(unsafe { Arc::from_raw(rust_ptr) });
    ManuallyDrop::into_inner(java_owned.clone())
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1android_1utilities_RustArcBoxDynAny_rust_1object_1destruct(
    _env: Env<'_>,
    _class: jobject,
    rust_ptr: jlong,
) {
    let rust_ptr: *const BoxedRustObj = with_exposed_provenance(rust_ptr as usize);
    unsafe {
        Arc::decrement_strong_count(rust_ptr);
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1android_1utilities_RustArcBoxDynAny_rust_1object_1clone(
    rust_ptr: jlong,
) {
    let rust_ptr: *const BoxedRustObj = with_exposed_provenance(rust_ptr as usize);
    unsafe { Arc::increment_strong_count(rust_ptr) }
}

#[cfg(target_os = "android")]
/// Initialize this class ny loading it into the Java VM
pub fn initialize() {
    use java_spaghetti_class_loader::JNINativeMethod;

    const RUST_ARC_BOX_DYN_ANY_JAVA_BYTECODE: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));
    java_spaghetti_class_loader::load_bytecode(
        "com.maticrobots.java_rust_obj",
        RUST_ARC_BOX_DYN_ANY_JAVA_BYTECODE,
    );

    const RUST_ARC_BOX_DN_ANY_METHODS: &[JNINativeMethod] = unsafe {
        &[
        JNINativeMethod::new(
            c"rust_object_destruct",
            c"(J)V",
            Java_com_maticrobots_rust_1android_1utilities_RustArcBoxDynAny_rust_1object_1destruct
                as *mut _,
        ),
        JNINativeMethod::new(
            c"rust_object_clone",
            c"(J)J",
            Java_com_maticrobots_rust_1android_1utilities_RustArcBoxDynAny_rust_1object_1clone
                as *mut _,
        ),
    ]
    };

    java_spaghetti_class_loader::declare_native_class_methods(
        "com/maticrobots/java_rust_obj/RustArcBoxDynAny",
        RUST_ARC_BOX_DN_ANY_METHODS,
    );
}

#[cfg(not(target_os = "android"))]
/// Initialize this class ny loading it into the Java VM
pub fn initialize() {}
