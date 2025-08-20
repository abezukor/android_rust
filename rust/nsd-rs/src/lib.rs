// Re-Export both of these as our API depends on their types
pub use java_spaghetti;
pub use jni;

pub use java_spaghetti_result::JavaError;

mod nsd_discovery_listener;

pub use nsd_manager::NSDManager;
mod nsd_manager;

// Regenerate with java-spaghetti-gen generate
#[rustfmt::skip]
#[allow(mismatched_lifetime_syntaxes)]
mod bindings;

pub use bindings::java::lang::{String as JString, Throwable};

pub use nsd_resolve_listener::NsdServiceInfo;
mod nsd_resolve_listener;

pub use discovery_request::DiscoveryRequest;
mod discovery_request;

pub type SharedRustObject = std::sync::Arc<dyn std::any::Any + Send + Sync + 'static>;

pub fn jni_env(env: jni::JNIEnv<'_>) -> java_spaghetti::Env<'_> {
    unsafe { java_spaghetti::Env::from_raw(env.get_raw().cast()) }
}
