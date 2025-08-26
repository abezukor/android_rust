//! A rust crate to use [Android NSD](https://developer.android.com/develop/connectivity/wifi/use-nsd) from rust.
//!  Eventually Intended to be a fully functional backend for [zeroconf](https://crates.io/crates/zeroconf).

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

pub fn initialize() {
    use java_spaghetti_class_loader::JNINativeMethod;

    java_owned_rust_object::initialize();

    const RUST_NSD_JAVA_BYTECODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));
    java_spaghetti_class_loader::load_bytecode("com.maticrobots.nsd_rs", RUST_NSD_JAVA_BYTECODE);

    const DISCOVERY_LISTENER_METHODS: &[JNINativeMethod] = unsafe {
        &[
        JNINativeMethod::new(
        c"rustOnStartDiscoveryFailed",
        c"(Ljava/lang/String;I)V",
        crate::nsd_discovery_listener::Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnStartDiscoveryFailed as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnStopDiscoveryFailed",
        c"(Ljava/lang/String;I)V",
        crate::nsd_discovery_listener::Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnStopDiscoveryFailed as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnDiscoveryStarted",
        c"(Ljava/lang/String;)V",
        crate::nsd_discovery_listener::Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnDiscoveryStarted as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnDiscoveryStopped",
        c"(Ljava/lang/String;)V",
        crate::nsd_discovery_listener::Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnDiscoveryStopped as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnServiceFound",
        c"(Landroid/net/nsd/NsdServiceInfo;J)V",
        crate::nsd_discovery_listener::Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnServiceFound as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnServiceLost",
        c"(Landroid/net/nsd/NsdServiceInfo;)V",
        crate::nsd_discovery_listener::Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnServiceLost as *mut _,
    ),]
    };

    java_spaghetti_class_loader::declare_native_class_methods(
        "com/maticrobots/nsd_rs/NSDDiscoveryListener",
        DISCOVERY_LISTENER_METHODS,
    );

    const RESOLVER_LISTENER_METHODS: &[JNINativeMethod] = unsafe {
        &[
            JNINativeMethod::new(
            c"rustOnResolveFailed",
            c"(JLandroid/net/nsd/NsdServiceInfo;I)V",
            crate::nsd_resolve_listener::Java_com_maticrobots_nsd_1rs_NSDServiceResolver_rustOnResolveFailed as *mut _,
        ),
        JNINativeMethod::new(
            c"rustOnServiceResolved",
            c"(JLandroid/net/nsd/NsdServiceInfo;)V",
            crate::nsd_resolve_listener::Java_com_maticrobots_nsd_1rs_NSDServiceResolver_rustOnServiceResolved as *mut _,
        ),
    ]
    };

    java_spaghetti_class_loader::declare_native_class_methods(
        "com/maticrobots/nsd_rs/NSDServiceResolver",
        RESOLVER_LISTENER_METHODS,
    );
}
