use java_spaghetti::{Env, Local};
use log::error;

use crate::{
    JavaResult,
    bindings::{
        android::net::{
            Network,
            nsd::{
                DiscoveryRequest as JavaDiscoveryRequest,
                DiscoveryRequest_Builder as JavaDiscoveryRequestBuilder,
            },
        },
        java::lang::{String as JString, Throwable},
    },
};

pub struct DiscoveryRequest {
    pub service_type: String,
    pub subtype: Option<String>,
    pub network: Option<i64>,
}

impl DiscoveryRequest {
    pub fn new(service_type: String, subtype: Option<String>, network: Option<i64>) -> Self {
        Self {
            service_type,
            subtype,
            network,
        }
    }

    pub(crate) fn java_object(self, env: Env<'_>) -> JavaResult<Local<'_, JavaDiscoveryRequest>> {
        let service_type = JString::from_env_str(env, self.service_type);
        let builder = JavaDiscoveryRequestBuilder::new(env, service_type)?;

        if let Some(subtype) = self.subtype {
            let subtype = JString::from_env_str(env, subtype);
            builder.setSubtype(subtype)?.unwrap();
        }

        if let Some(network_handle) = self.network {
            let network = Network::fromNetworkHandle(env, network_handle)?.ok_or_else(|| {
                error!("None in fromNetworkHandle");
                Throwable::new_String(
                    env,
                    JString::from_env_str(env, format!("No Network at handle {}", network_handle)),
                )
                .unwrap()
            })?;

            builder.setNetwork(network)?.unwrap();
        }

        let discover_request = builder.build()?.unwrap();

        Ok(discover_request.as_global().as_local(env))
    }
}
