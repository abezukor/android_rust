//! These functions (along with the `RustArcBoxDynAny` java class) provide a basic mechanism
//! for Java to "own" a rust object,
use std::{any::Any, mem::ManuallyDrop, ptr::with_exposed_provenance, sync::Arc};

use java_spaghetti::{
    Env, Local,
    sys::{jlong, jobject},
};

use crate::{JavaResult, bindings::com::maticrobots::nsd_rs::RustArcBoxDynAny};

pub type BoxedRustObj = Box<dyn Any + Send + Sync + 'static>;

// It needs to be Arc<Box<dyn Any>> because we need to be able to use Arc::from_raw and Arc::into_raw,
// and those need to return thin pointers so that they fit in a java long
pub fn to_java(
    env: Env,
    rust_obj: impl Any + Send + Sync + 'static,
) -> JavaResult<Local<RustArcBoxDynAny>> {
    to_java_arc(env, Arc::new(Box::new(rust_obj)))
}

pub fn to_java_arc(env: Env, rust_obj: Arc<BoxedRustObj>) -> JavaResult<Local<RustArcBoxDynAny>> {
    let raw = Arc::into_raw(rust_obj);
    // raw.expose_provenance() can be 32 bit

    // Java does not have a concept of unsigned integers so we have to reinterpret this as an i64.
    Ok(RustArcBoxDynAny::new(
        env,
        raw.expose_provenance() as jlong,
    )?)
}

///# Safety
///
/// Raw PRT must be the rust ptr inside a from a `RustArcBoxDynAny` java class,
pub unsafe fn get_ref(raw_ptr: jlong) -> Arc<BoxedRustObj> {
    let rust_ptr: *const BoxedRustObj = with_exposed_provenance(raw_ptr as usize);
    // Make sure to return a cloned copy, so users of this fn CANNOT delete the arc owned by java.
    let java_owned = ManuallyDrop::new(unsafe { Arc::from_raw(rust_ptr) });
    ManuallyDrop::into_inner(java_owned.clone())
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_RustArcBoxDynAny_rust_1object_1destruct(
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
extern "system" fn Java_com_maticrobots_nsd_1rs_RustArcBoxDynAny_rust_1object_1clone(
    rust_ptr: jlong,
) {
    let rust_ptr: *const BoxedRustObj = with_exposed_provenance(rust_ptr as usize);
    unsafe { Arc::increment_strong_count(rust_ptr) }
}
