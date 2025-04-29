use java_spaghetti::{Env, Global, Local, Ref, VM};
use std::{
    fmt::{Debug, Display},
    mem::ManuallyDrop,
};

// Re-Export both of these as our API depends on their types
pub use java_spaghetti;
use java_spaghetti::{Env, Global, Local};
pub use jni;

pub type JavaResult<T> = Result<T, JavaError>;

pub use nsd_discovery_listener::NsdServiceInfo;
mod nsd_discovery_listener;

pub use nsd_manager::NSDManager;
mod nsd_manager;

pub mod java_wrapped_object;

use bindings::com::maticrobots::nsd_rs::RustAppContentInitializer;
// Regenerate with java-spaghetti-gen generate
#[rustfmt::skip]
mod bindings;

mod nsd_resolve_listener;

pub use discovery_request::DiscoveryRequest;
mod discovery_request;

pub type SharedRustObject = std::sync::Arc<dyn std::any::Any + Send + Sync + 'static>;

/// Global Java Error that can be passed between threads.
pub struct JavaError(Global<bindings::java::lang::Throwable>)

impl std::fmt::Debug for JavaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.vm().with_env(|env| {
            let local_ref = self.0.as_local(env);
            write!(f, "{:?}", local_ref)
        })
    }
}

impl From<Local<'_, bindings::java::lang::Throwable>> for JavaError {
    fn from(value: Local<'_, bindings::java::lang::Throwable>) -> Self {
        Self(value.as_global())
    }
}

impl Display for JavaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.vm().with_env(|env| {
            let err = self.0.as_ref(env);
            match err.toString() {
                Ok(Some(err_str)) => err_str.to_string_lossy(),
                Ok(None) => "Error Has No String Representation".to_owned(),
                Err(e) => format!("{:?}", e),
            }
        }))
    }
}

fn get_application_context(env: Env<'_>) -> Local<'_, bindings::android::content::Context> {
    RustAppContentInitializer::applicationContext(env).unwrap()
}

pub fn jni_env(env: jni::JNIEnv<'_>) -> java_spaghetti::Env<'_> {
    unsafe { java_spaghetti::Env::from_raw(env.get_raw().cast()) }
}
