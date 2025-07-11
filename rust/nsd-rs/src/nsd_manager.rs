use java_spaghetti::Global;
use log::{error, trace};
use rust_android_utilities::get_application_context;

use crate::{
    bindings::{
        android::{content::Context, net::nsd::NsdManager},
        java::lang::{String as JString, Throwable},
    },
    nsd_discovery_listener::DiscoveryListener,
    DiscoveryRequest, JavaResult, NsdServiceInfo, SharedRustObject,
};

pub struct NSDManager {
    _manager: Global<NsdManager>,
    _listener: DiscoveryListener,
}

impl NSDManager {
    pub fn new(
        discovery_request: DiscoveryRequest,
        callback_context: SharedRustObject,
        callback: impl Fn(&NsdServiceInfo, SharedRustObject) + Send + Sync + 'static,
    ) -> JavaResult<Self> {
        trace!("Making NSD Manager");
        let vm = rust_android_utilities::get_vm();

        vm.with_env(|env| {
            let nsd_manager_name = JString::from_env_str(env, Context::NSD_SERVICE);

            let app_context = unsafe { get_application_context::<Context>() };
            let app_context_ref = app_context.as_ref(env);

            let nsd_manager = app_context_ref
                .getSystemService_String(nsd_manager_name)?
                .ok_or_else(|| {
                    error!("None in getSystemService_String");
                    Throwable::new_String(env, JString::from_env_str(env, "Could not get NSDMANAGER from context"))
                        .unwrap()
                })?;
            let nsd_manager = nsd_manager.as_global();
            let nsd_manager: Global<NsdManager> = unsafe { Global::from_raw(env.vm(), nsd_manager.into_raw()) };

            let discovery_listener = DiscoveryListener::new(nsd_manager.clone(), callback_context, callback)?;

            {
                let nsd_manager = nsd_manager.as_local(env);
                discovery_request.discover(
                    env,
                    nsd_manager,
                    &app_context_ref,
                    discovery_listener.as_manager_listener(),
                )?;
            }

            Ok(Self { _manager: nsd_manager, _listener: discovery_listener })
        })
    }
}
