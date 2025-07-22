use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use java_spaghetti::sys::{jlong, jobject};
use java_spaghetti::{AsArg, ByteArray, Env, Global, Local, PrimitiveArray, Ref};
use log::{error, trace};

use rust_android_utilities::{
    get_vm,
    java_wrapped_object::{get_ref, to_java_arc, BoxedRustObj},
};

use crate::bindings::android::os::Build_VERSION;
use crate::bindings::java::util::Map_Entry;
use crate::{
    bindings::{
        android::net::nsd::{NsdManager, NsdManager_ResolveListener, NsdServiceInfo as JavaNsdServiceInfo},
        com::maticrobots::{
            nsd_rs::NSDServiceResolver as JavaNSDResolveListener, rust_android_utilities::RustArcBoxDynAny,
        },
        java::lang::{String as JString, Throwable},
    },
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

pub struct NsdServiceInfo<'a>(Ref<'a, JavaNsdServiceInfo>);

#[allow(clippy::type_complexity)]
struct SharedContext {
    user_context: SharedRustObject,
    callback: Box<dyn for<'a> Fn(&'a NsdServiceInfo<'a>, SharedRustObject) + Send + Sync + 'static>,
}

struct ResolverContext {
    in_use: Arc<AtomicBool>,
    shared_context: Arc<SharedContext>,
}

impl JavaResolvers {
    pub fn new(
        context: SharedRustObject,
        callback: impl for<'a> Fn(&'a NsdServiceInfo<'a>, SharedRustObject) + Send + Sync + 'static,
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
                    let java_resolver = JavaNSDResolveListener::new(
                        env,
                        to_java_arc(env, context).unwrap().cast::<RustArcBoxDynAny>().unwrap(),
                    )
                    .unwrap();
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
        info: impl AsArg<JavaNsdServiceInfo>,
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

impl<'a> NsdServiceInfo<'a> {
    pub fn host_name(&self) -> Option<String> {
        #[allow(deprecated)]
        let host = self.0.getHost().unwrap()?;
        let hostname = host.getHostName().unwrap()?;
        Some(hostname.to_string_lossy())
    }

    pub fn host_address(&self) -> Option<String> {
        #[allow(deprecated)]
        let host = self.0.getHost().unwrap()?;
        let host_address = host.getHostAddress().unwrap()?;
        Some(host_address.to_string_lossy())
    }

    pub fn get_service_name(&self) -> Option<String> {
        let service_type = self.0.getServiceName().unwrap()?;
        Some(service_type.to_string_lossy())
    }

    pub fn get_service_type(&self) -> Option<String> {
        let service_type = self.0.getServiceType().unwrap()?;
        Some(service_type.to_string_lossy())
    }

    pub fn get_port(&self) -> u16 {
        u16::try_from(self.0.getPort().unwrap()).unwrap()
    }

    pub fn get_subtypes(&self) -> Option<Vec<String>> {
        // Service types not available before android api 35
        log::trace!("Build version is {:?}", Build_VERSION::SDK_INT(self.0.env()));
        if Build_VERSION::SDK_INT(self.0.env()) < 35 {
            return None;
        };

        let subtypes = self.0.getSubtypes().unwrap()?;
        let sub_type_arr = subtypes.toArray().unwrap().unwrap();
        let mut sub_types = Vec::with_capacity(sub_type_arr.len());
        for sub_type in sub_type_arr.iter() {
            let Some(sub_type) = sub_type else {
                continue;
            };
            let sub_type = sub_type.cast::<JString>().unwrap();
            sub_types.push(sub_type.to_string_lossy());
        }
        Some(sub_types)
    }

    pub fn get_attributes(&self) -> HashMap<String, Box<[u8]>> {
        let Some(txt) = self.0.getAttributes().unwrap() else {
            return HashMap::new();
        };

        let entries = txt.entrySet().unwrap().unwrap();
        let num_entries = entries.size().unwrap();
        if num_entries == 0 {
            return HashMap::new();
        }
        let entries = entries.iterator().unwrap().unwrap();

        let mut entries_map = HashMap::with_capacity(usize::try_from(num_entries).unwrap());

        while entries.hasNext().unwrap() {
            let entry: Local<Map_Entry> = entries.next().unwrap().unwrap().cast().unwrap();
            let key: Local<JString> = entry.getKey().unwrap().unwrap().cast().unwrap();
            let value: Local<ByteArray> = entry.getValue().unwrap().unwrap().cast().unwrap();
            let value: Box<[u8]> = value.as_vec().into_iter().map(i8::cast_unsigned).collect();
            entries_map.insert(key.to_string().unwrap(), value);
        }

        entries_map
    }
}

impl<'a> Debug for NsdServiceInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0.toString().unwrap().unwrap().to_string().unwrap())
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
    let service_info: Local<'_, JavaNsdServiceInfo> = unsafe { Local::from_raw(env, service_info) };
    error!("Got Error {} when resolving {:?}", error_code, service_info.toString());
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_nsd_1rs_NSDServiceResolver_rustOnServiceResolved(
    env: Env<'_>,
    _class: jobject, // self class, ignore,
    rust_ptr: jlong,
    service_info: jobject,
) {
    let service_info: Ref<'_, JavaNsdServiceInfo> = unsafe { Ref::from_raw(env, service_info) };
    trace!("Resolved {:?}", service_info.toString());
    let this = unsafe { get_ref(rust_ptr) };
    let this = this.downcast_ref::<ResolverContext>().unwrap();
    (this.shared_context.callback)(&NsdServiceInfo(service_info), this.shared_context.user_context.clone());
    this.in_use.store(false, Ordering::Relaxed);
}
