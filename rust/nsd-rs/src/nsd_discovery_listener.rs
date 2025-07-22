use java_spaghetti::{
    sys::{jlong, jobject},
    Env, Global, Local, Ref,
};
use log::{error, trace};

use rust_android_utilities::{
    get_vm,
    java_wrapped_object::{get_ref, to_java},
    JavaResult,
};

use crate::{
    bindings::{
        android::net::nsd::{NsdManager, NsdManager_DiscoveryListener, NsdServiceInfo as JavaNsdServiceInfo},
        com::maticrobots::{nsd_rs::NSDDiscoveryListener, rust_android_utilities::RustArcBoxDynAny},
        java::lang::String as JString,
    },
    nsd_resolve_listener::JavaResolvers,
    NsdServiceInfo, SharedRustObject,
};

pub struct DiscoveryListener {
    java_listener: Global<NSDDiscoveryListener>,
    manager: Global<NsdManager>,
}

struct Context {
    manager: Global<NsdManager>,
    resolvers: JavaResolvers,
}

impl DiscoveryListener {
    pub fn new(
        manager: Global<NsdManager>,
        callback_context: SharedRustObject,
        callback: impl for<'a> Fn(&'a NsdServiceInfo<'a>, SharedRustObject) + Send + Sync + 'static,
    ) -> JavaResult<Self> {
        let resolvers = JavaResolvers::new(callback_context, callback);

        get_vm().with_env(|env| {
            let java_inner = to_java(env, Context { manager: manager.clone(), resolvers })?;
            let java_listener = NSDDiscoveryListener::new(env, java_inner.cast::<RustArcBoxDynAny>().unwrap())?;

            Ok(Self { java_listener: java_listener.as_global(), manager })
        })
    }

    pub(crate) fn as_manager_listener(&self) -> Global<NsdManager_DiscoveryListener> {
        get_vm().with_env(|env| {
            self.java_listener
                .as_ref(env)
                .cast::<NsdManager_DiscoveryListener>()
                .unwrap()
                .as_global()
        })
    }
}

impl Drop for DiscoveryListener {
    fn drop(&mut self) {
        trace!("Stopping Service Discovery");
        get_vm().with_env(|env| {
            let manager = self.manager.as_local(env);
            manager.stopServiceDiscovery(self.java_listener.clone()).unwrap()
        })
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnStartDiscoveryFailed(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    service_type: jobject,
    error_code: i32,
) {
    let service_type: Local<'_, JString> = unsafe { Local::from_raw(env, service_type) };
    error!(
        "Start Discovery on {:?} failed with {}",
        service_type.toString(),
        error_code
    );
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnStopDiscoveryFailed(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    service_type: jobject,
    error_code: i32,
) {
    let service_type: Local<'_, JString> = unsafe { Local::from_raw(env, service_type) };
    error!(
        "Stop Discovery on {:?} failed with {}",
        service_type.to_string(),
        error_code
    );
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnDiscoveryStarted(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    service_type: jobject,
) {
    let service_type: Ref<'_, JString> = unsafe { Ref::from_raw(env, service_type) };
    trace!("Discovery Started for {:?}", service_type.to_string());
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnDiscoveryStopped(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    service_type: jobject,
    error_code: i32,
) {
    let service_type: Ref<'_, JString> = unsafe { Ref::from_raw(env, service_type) };
    trace!(
        "Discovery stopped for {:?} with code {}",
        service_type.to_string(),
        error_code
    );
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnServiceFound(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    service_info: jobject,
    rust_ptr: jlong,
) {
    //trace!("Got Service");

    let this = unsafe { get_ref(rust_ptr) };
    let this = this.downcast_ref::<Context>().unwrap();
    let info: Ref<JavaNsdServiceInfo> = unsafe { Ref::from_raw(env, service_info) };
    trace!(
        "Got Service {:}",
        info.toString().unwrap().unwrap().to_string().unwrap()
    );
    let manager = this.manager.as_ref(env);
    let resolver = this.resolvers.get_available_resolver(this.manager.clone());

    if let Err(e) = resolver.resolve(env, &manager, info) {
        error!(
            "Error in resolveService_NsdServiceInfo_ResolveListener {:?}",
            e.toString()
        );
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDDiscoveryListener_rustOnServiceLost(
    _env: Env<'_>,
    _class: jobject, // self class, ignore,
    _service_info: jobject,
) {
}
