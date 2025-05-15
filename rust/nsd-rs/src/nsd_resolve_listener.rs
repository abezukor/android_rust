use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use java_spaghetti::sys::{jlong, jobject};
use java_spaghetti::{AsArg, Env, Global, Local, Ref};
use log::{error, trace};

use rust_android_utilities::get_vm;

use crate::{
    bindings::{
        android::net::nsd::{NsdManager, NsdManager_ResolveListener, NsdServiceInfo},
        com::maticrobots::nsd_rs::NSDServiceResolver as JavaNSDResolveListener,
        java::lang::Throwable,
    },
    java_wrapped_object::{get_ref, to_java_arc, BoxedRustObj},
    SharedRustObject,
};

pub(crate) struct JavaResolvers {
    resolvers: Mutex<Vec<Arc<JavaResolver>>>,
    shared_context: Arc<SharedContext>,
}

pub(crate) struct JavaResolver {
    resolver: Global<JavaNSDResolveListener>,
    in_use: Arc<AtomicBool>,
    manager: Global<NsdManager>,
}

#[allow(clippy::type_complexity)]
struct SharedContext {
    user_context: SharedRustObject,
    callback: Box<dyn Fn(&NsdServiceInfo, SharedRustObject) + Send + Sync + 'static>,
}

struct ResolverContext {
    in_use: Arc<AtomicBool>,
    shared_context: Arc<SharedContext>,
}

impl JavaResolvers {
    pub fn new(
        context: SharedRustObject,
        callback: impl Fn(&NsdServiceInfo, SharedRustObject) + Send + Sync + 'static,
    ) -> Self {
        Self {
            resolvers: Mutex::new(Vec::new()),
            shared_context: Arc::new(SharedContext { user_context: context, callback: Box::new(callback) }),
        }
    }

    pub fn get_available_resolver(&self, manager: Global<NsdManager>) -> Arc<JavaResolver> {
        let mut resolvers = self.resolvers.lock().unwrap();
        resolvers
            .iter()
            .find_map(|resolver| (!resolver.in_use.load(Ordering::Relaxed)).then_some(resolver.clone()))
            .unwrap_or_else(|| {
                trace!("Could not find an open resolver, making a new one");
                let in_use = Arc::new(AtomicBool::new(false));
                let context: Arc<BoxedRustObj> = Arc::new(Box::new(ResolverContext {
                    shared_context: self.shared_context.clone(),
                    in_use: in_use.clone(),
                }));
                let java_resolver = get_vm().with_env(|env| {
                    let java_resolver = JavaNSDResolveListener::new(env, to_java_arc(env, context).unwrap()).unwrap();
                    java_resolver.as_global()
                });
                let java_resolver =
                    Arc::new(JavaResolver { resolver: java_resolver, in_use, manager: manager.clone() });
                resolvers.push(java_resolver.clone());
                java_resolver
            })
    }
}

impl JavaResolver {
    pub fn resolve<'a>(
        &'a self,
        env: Env<'a>,
        manager: &'a NsdManager,
        info: impl AsArg<NsdServiceInfo>,
    ) -> Result<(), Local<'a, Throwable>> {
        let resolver = self.resolver.as_ref(env);
        let casted_resolver = resolver.cast::<NsdManager_ResolveListener>().unwrap();
        self.in_use.store(true, Ordering::Relaxed);

        // I dont want to write 2 implementations for this, one for
        // https://developer.android.com/reference/kotlin/android/net/nsd/NsdManager#registerserviceinfocallback
        // and the other for
        // https://developer.android.com/reference/kotlin/android/net/nsd/NsdManager#resolveservice
        #[allow(deprecated)]
        manager.resolveService_NsdServiceInfo_ResolveListener(info, casted_resolver)
    }
}

impl Drop for JavaResolver {
    fn drop(&mut self) {
        get_vm().with_env(|env| {
            let manager = self.manager.as_ref(env);
            let resolver = self.resolver.as_ref(env);
            let casted_resolver = resolver.cast::<NsdManager_ResolveListener>().unwrap();
            let stop_resolution = self.in_use.swap(false, Ordering::Relaxed);
            trace!(
                "Should Stop Resolution for {:?}, {}",
                casted_resolver.toString(),
                stop_resolution
            );
            if stop_resolution {
                if let Err(e) = manager.stopServiceResolution(casted_resolver) {
                    error!("error stopping resolution for {:?}", e.toString());
                }
            }
        });
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDServiceResolver_rustOnResolveFailed(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    _rust_ptr: jlong,
    service_info: jobject,
    error_code: i32,
) {
    let service_info: Local<'_, NsdServiceInfo> = unsafe { Local::from_raw(env, service_info) };
    error!("Got Error {} when resolving {:?}", error_code, service_info.toString());
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDServiceResolver_rustOnServiceResolved(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    rust_ptr: jlong,
    service_info: jobject,
) {
    let service_info: Ref<'_, NsdServiceInfo> = unsafe { Ref::from_raw(env, service_info) };
    trace!("Resolved {:?}", service_info.toString());
    let this = unsafe { get_ref(rust_ptr) };
    let this = this.downcast_ref::<ResolverContext>().unwrap();
    (this.shared_context.callback)(&service_info, this.shared_context.user_context.clone());
    this.in_use.store(false, Ordering::Relaxed);
}
