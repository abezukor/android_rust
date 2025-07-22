use log::{info, LevelFilter};
use std::sync::Arc;

use jni::{objects::JObject, sys::jobject};
use nsd_rs::{self, jni_env, DiscoveryRequest, NsdServiceInfo, SharedRustObject};
use rust_android_utilities::java_wrapped_object::to_java;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_maticrobots_nsd_1rs_1example_1app_RustNSDExample_init_1logging() {
    use android_logger::Config;

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));

    info!("Android Logger Started");

    std::panic::set_hook(Box::new(|panic_hook_info| {
        if let Some(s) = panic_hook_info.payload().downcast_ref::<&str>() {
            log::error!("Panic Payload: {s:?}");
        } else if let Some(s) = panic_hook_info.payload().downcast_ref::<String>() {
            log::error!("Panic Payload: {s:?}");
        }
        log::error!("Panic at {:?}", panic_hook_info.location());
        log::error!("Backtrace: {}", std::backtrace::Backtrace::force_capture());
    }))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_maticrobots_nsd_1rs_1example_1app_RustNSDExample_start_1discovery<'a>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass,
) -> JObject<'a> {
    let env = jni_env(env);
    let discovery_request = DiscoveryRequest::new("_matic_hermes._tcp".to_owned(), None, None);
    let manager = nsd_rs::NSDManager::new(discovery_request, Arc::new(()), Box::new(callback)).unwrap();

    let java_obj = to_java(env, manager).unwrap();
    unsafe { JObject::from_raw(java_obj.into_raw() as jobject) }
}

fn callback(service_info: &NsdServiceInfo, _context: SharedRustObject) {
    info!("Library Callback {:?}", service_info)
}
