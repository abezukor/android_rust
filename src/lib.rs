use bindings::{android::content::Context, com::maticrobots::nsd_rs::RustAppContentInitializer};
use java_spaghetti::{Env, Global, Local, VM};
use std::fmt::Debug;

pub type JavaResult<T> = Result<T, JavaError>;

pub use nsd_discovery_listener::NsdServiceInfo;
mod nsd_discovery_listener;

pub use nsd_manager::NSDManager;
mod nsd_manager;

pub mod java_wrapped_object;

// Regenerate with java-spaghetti-gen generate
#[rustfmt::skip]
mod bindings;

mod nsd_resolve_listener;

pub type SharedRustObject = std::sync::Arc<dyn std::any::Any + Send + Sync + 'static>;

/// Global Java Error that can be passed between threads.
pub struct JavaError {
    err: Global<bindings::java::lang::Throwable>,
    vm: VM,
}

impl Debug for JavaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vm.with_env(|env| {
            let local_ref = self.err.as_local(env);
            write!(f, "{:?}", local_ref)
        })
    }
}

impl From<Local<'_, bindings::java::lang::Throwable>> for JavaError {
    fn from(value: Local<'_, bindings::java::lang::Throwable>) -> Self {
        Self {
            vm: value.env().vm(),
            err: value.as_global(),
        }
    }
}

fn get_application_context(env: Env<'_>) -> Local<'_, Context> {
    RustAppContentInitializer::applicationContext(env).unwrap()
}

pub fn jni_env(env: jni::JNIEnv<'_>) -> java_spaghetti::Env<'_> {
    unsafe { java_spaghetti::Env::from_raw(env.get_raw().cast()) }
}
