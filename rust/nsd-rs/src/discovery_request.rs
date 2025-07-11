use java_spaghetti::{AsArg, Env, Local, Ref};
use log::error;

use crate::{
    bindings::{
        android::{
            content::Context as AndroidContext,
            net::{
                nsd::{
                    DiscoveryRequest as JavaDiscoveryRequest, DiscoveryRequest_Builder as JavaDiscoveryRequestBuilder,
                    NsdManager, NsdManager_DiscoveryListener,
                },
                Network,
            },
            os::Build_VERSION,
        },
        java::lang::{String as JString, Throwable},
    },
    JavaError, JavaResult,
};

pub struct DiscoveryRequest {
    pub service_type: String,
    pub subtype: Option<String>,
    pub network: Option<i64>,
}

impl DiscoveryRequest {
    pub fn new(service_type: String, subtype: Option<String>, network: Option<i64>) -> Self {
        Self { service_type, subtype, network }
    }

    fn java_network<'a>(&self, env: Env<'a>) -> JavaResult<Option<Local<'a, Network>>> {
        let Some(network_handle) = self.network else {
            return Ok(None);
        };

        Ok(Some(Network::fromNetworkHandle(env, network_handle)?.ok_or_else(
            || {
                error!("None in fromNetworkHandle");
                Throwable::new_String(
                    env,
                    JString::from_env_str(env, format!("No Network at handle {network_handle}")),
                )
                .unwrap()
            },
        )?))
    }

    fn java_object(self, env: Env<'_>) -> JavaResult<Local<'_, JavaDiscoveryRequest>> {
        let service_type = JString::from_env_str(env, &self.service_type);
        let builder = JavaDiscoveryRequestBuilder::new(env, service_type)?;

        if let Some(subtype) = self.subtype.as_ref() {
            let subtype = JString::from_env_str(env, subtype);
            builder.setSubtype(subtype)?.unwrap();
        }

        if let Some(network) = self.java_network(env)? {
            builder.setNetwork(network)?.unwrap();
        }

        let discover_request = builder.build()?.unwrap();

        Ok(discover_request.as_global().as_local(env))
    }

    pub(crate) fn discover(
        self,
        env: Env,
        manager: Local<NsdManager>,
        app_context: &Ref<AndroidContext>,
        discovery_listener: impl AsArg<NsdManager_DiscoveryListener>,
    ) -> JavaResult<()> {
        match is_discovery_request_available(env) {
            true => {
                let jo = self.java_object(env)?;
                manager.discoverServices_DiscoveryRequest_Executor_DiscoveryListener(
                    jo,
                    app_context.getMainExecutor()?,
                    discovery_listener,
                )?;
            }
            false => {
                if self.subtype.is_some() {
                    return Err(JavaError(
                        Throwable::new_String(
                            env,
                            JString::from_env_str(env, "Cannot use subtypes before android 33."),
                        )
                        .unwrap()
                        .as_global(),
                    ));
                }

                let service_type = JString::from_env_str(env, &self.service_type);

                match self.java_network(env)? {
                    Some(network) => manager.discoverServices_String_int_Network_Executor_DiscoveryListener(
                        service_type,
                        NsdManager::PROTOCOL_DNS_SD,
                        network,
                        app_context.getMainExecutor()?,
                        discovery_listener,
                    )?,
                    None => manager.discoverServices_String_int_DiscoveryListener(
                        service_type,
                        NsdManager::PROTOCOL_DNS_SD,
                        discovery_listener,
                    )?,
                }
            }
        }
        Ok(())
    }
}

fn is_discovery_request_available(env: Env) -> bool {
    Build_VERSION::SDK_INT(env) >= 35
}
