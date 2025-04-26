use java_spaghetti::{Env, Global};
use log::{error, trace};

use crate::{
    JavaResult, NsdServiceInfo, SharedRustObject,
    bindings::{
        android::{content::Context, net::nsd::NsdManager},
        java::lang::{String as JString, Throwable},
    },
    nsd_discovery_listener::DiscoveryListener,
};

pub struct NSDManager {
    _manager: Global<NsdManager>,
    _listener: DiscoveryListener,
}

impl NSDManager {
    pub fn new(
        env: Env<'_>,
        service_type: &str,
        callback_context: SharedRustObject,
        callback: impl Fn(&NsdServiceInfo, SharedRustObject) + Send + Sync + 'static,
    ) -> JavaResult<Self> {
        trace!("Making NSD Manager");
        let nsd_manager_name = JString::from_env_str(env, Context::NSD_SERVICE);

        let app_context = crate::get_application_context(env);

        let nsd_manager = app_context
            .getSystemService_String(nsd_manager_name)?
            .ok_or_else(|| {
                error!("None in getSystemService_String");
                Throwable::new_String(
                    env,
                    JString::from_env_str(env, "Could not get NSDMANAGER from context"),
                )
                .unwrap()
            })?;
        let nsd_manager = nsd_manager.as_global();
        let nsd_manager: Global<NsdManager> =
            unsafe { Global::from_raw(env.vm(), nsd_manager.into_raw()) };

        let service_type = JString::from_env_str(env, service_type);

        let discovery_listener =
            DiscoveryListener::new(env, nsd_manager.clone(), callback_context, callback)?;

        {
            let nsd_manager = nsd_manager.as_local(env);
            nsd_manager.discoverServices_String_int_DiscoveryListener(
                service_type,
                NsdManager::PROTOCOL_DNS_SD,
                discovery_listener.as_manager_listener(),
            )?;
        }

        Ok(Self {
            _manager: nsd_manager,
            _listener: discovery_listener,
        })
    }
}
