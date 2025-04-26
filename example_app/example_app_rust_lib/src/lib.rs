use log::{LevelFilter, info};
use std::sync::Arc;

use nsd_rs::{self, NsdServiceInfo, SharedRustObject, java_wrapped_object::to_java, jni_env};

use std::sync::OnceLock;

use jni::{objects::JObject, sys::jobject};
pub static VM: OnceLock<jni::JavaVM> = OnceLock::new();

// need call this function in java/kotlin first
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_maticrobots_nsd_1rs_1example_1app_RustNSDExample_java_1init(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    let vm = env.get_java_vm().unwrap();

    _ = VM.set(vm);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_maticrobots_nsd_1rs_1example_1app_RustNSDExample_init_1logging() {
    use android_logger::Config;

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));

    info!("Android Logger Started");
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_maticrobots_nsd_1rs_1example_1app_RustNSDExample_start_1discovery<
    'a,
>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass,
) -> JObject<'a> {
    let env = jni_env(env);
    let manager =
        nsd_rs::NSDManager::new(env, "_matic_hermes._tcp", Arc::new(()), Box::new(callback))
            .unwrap();

    let java_obj = to_java(env, manager).unwrap();
    unsafe { JObject::from_raw(java_obj.into_raw() as jobject) }
}

fn callback(service_inf: &NsdServiceInfo, _context: SharedRustObject) {
    info!("Library Callback {:?}", service_inf.toString().unwrap())
}
