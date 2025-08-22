use std::mem::ManuallyDrop;

use futures_channel::mpsc::TrySendError;
use java_owned_rust_object::get_ref;
use java_spaghetti::{
    ByteArray, Env, Global, Local, ObjectArray, PrimitiveArray, Ref, ReferenceType, sys::jobject,
};
use log::warn;
use uuid::Uuid;

pub use device::Device;
pub mod device;

pub use adapter::Adapter;
mod adapter;

use bindings::com::maticrobots::rust_android_utilities::RustArcBoxDynAny;
#[rustfmt::skip]
#[allow(mismatched_lifetime_syntaxes)]
mod bindings;

pub mod scan;

use callback_future::{CallBackFuture, CallBackFutureData};
mod callback_future;

pub use service::Service;
pub mod service;

pub use characteristic::Characteristic;
pub mod characteristic;

pub use descriptor::Descriptor;

use crate::bindings::java::lang::Throwable;
mod descriptor;

pub use error::GattError;
pub use java_spaghetti_result::JavaError;
pub mod error;

pub use l2cap_channel::Channel;
pub mod l2cap_channel;

mod gatt_callback;

#[derive(Debug)]
pub enum ConnectionState {
    Disconnected,
    Disconnecting,
    Connecting,
    Connected,
}

impl ConnectionState {
    pub fn from_java(connection_state: i32) -> Option<Self> {
        use bindings::android::bluetooth::BluetoothAdapter;

        match connection_state {
            BluetoothAdapter::STATE_DISCONNECTED => Some(Self::Disconnected),
            BluetoothAdapter::STATE_DISCONNECTING => Some(Self::Disconnecting),
            BluetoothAdapter::STATE_CONNECTING => Some(Self::Connecting),
            BluetoothAdapter::STATE_CONNECTED => Some(Self::Connected),
            other => {
                warn!("Trying to get connection state from invalid {other:?}");
                None
            }
        }
    }
}

fn local_array_to_global_vec<T: ReferenceType>(
    local_array: Local<ObjectArray<T, Throwable>>,
) -> Vec<Global<T>> {
    local_array
        .iter()
        .filter_map(|item| item.as_ref().map(Local::as_global))
        .collect()
}

fn java_uuid_to_rust(java_uuid: Local<'_, bindings::java::util::UUID>) -> Uuid {
    let most_signifigent_bits: u64 = java_uuid.getMostSignificantBits().unwrap().cast_unsigned();
    let least_signifigent_bits: u64 = java_uuid.getLeastSignificantBits().unwrap().cast_unsigned();
    Uuid::from_u64_pair(most_signifigent_bits, least_signifigent_bits)
}

fn rust_java_uuid(uuid: Uuid, env: Env) -> Local<bindings::java::util::UUID> {
    let (uuid_msb, uuid_lsb) = uuid.as_u64_pair();
    let (uuid_msb, uuid_lsb) = (uuid_msb.cast_signed(), uuid_lsb.cast_signed());
    bindings::java::util::UUID::new(env, uuid_msb, uuid_lsb).unwrap()
}

fn callback_mpsc_channel_send<T: 'static>(
    env: Env<'_>,
    mpsc_sender: jobject,
    to_send: T,
) -> Result<(), TrySendError<T>> {
    use futures_channel::mpsc::UnboundedSender;

    let rust_obj: Ref<RustArcBoxDynAny> = unsafe { Ref::from_raw(env, mpsc_sender) };
    let rust_ptr = rust_obj.getRust_ptr().unwrap();
    let rust_obj = unsafe { get_ref(rust_ptr) };
    let this = rust_obj.downcast_ref::<UnboundedSender<T>>().unwrap();
    this.unbounded_send(to_send)
}

fn java_byte_array_to_rust_boxed_slice(java_array: Ref<java_spaghetti::ByteArray>) -> Box<[u8]> {
    let signed_vec = ManuallyDrop::new(java_array.as_vec());
    unsafe {
        Vec::from_raw_parts(
            signed_vec.as_ptr() as *mut u8,
            signed_vec.len(),
            signed_vec.capacity(),
        )
    }
    .into_boxed_slice()
}

fn rust_slice_to_java_byte_array<'a>(env: Env<'a>, slice: &[u8]) -> Local<'a, ByteArray> {
    ByteArray::new_from(env, {
        let len = slice.len();
        let data = slice.as_ptr() as *const i8;
        // safety: any bit pattern is valid for u8 and i8, so transmuting them is fine.
        unsafe { std::slice::from_raw_parts(data, len) }
    })
}

pub(crate) mod java_macros {
    #[macro_export]
    macro_rules! java_debug_eq_hash {
        ($object:ident,$($this_object:tt)+) => {
            impl std::fmt::Debug for $object {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "{}",
                        self.$($this_object)+.vm().with_env(|env| {
                            let adapter = self.$($this_object)+.as_ref(env);
                            adapter.toString().unwrap().unwrap().to_string().unwrap()
                        })
                    )
                }
            }
            impl PartialEq for $object {
                fn eq(&self, other: &Self) -> bool {
                    self.$($this_object)+.vm().with_env(|env| {
                        let this = self.$($this_object)+.as_ref(env);
                        let other = other.$($this_object)+.as_ref(env);
                        this.equals(other).unwrap()
                    })
                }
            }

            impl Eq for $object {}

            impl std::hash::Hash for $object {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hashcode = self.$($this_object)+.vm().with_env(|env| {
                        let this = self.$($this_object)+.as_ref(env);
                        this.hashCode().unwrap()
                    });
                    hashcode.hash(state);
                }
            }
        };
        ($object:ident) => {
            java_debug_eq_hash!($object, 0);
        };
    }
}

pub fn initialize() {
    java_owned_rust_object::initialize();

    const RUST_BLUEDROID_JAVA_BYTECODE: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));
    java_spaghetti_class_loader::load_bytecode(
        "com.maticrobots.rust_bluedroid",
        RUST_BLUEDROID_JAVA_BYTECODE,
    );

    java_spaghetti_class_loader::declare_native_class_methods(
        scan::LE_SCAN_CALLBACK_CLASS,
        scan::LE_SCAN_CALLBACK_NATIVE_METHODS,
    );

    java_spaghetti_class_loader::declare_native_class_methods(
        gatt_callback::GATT_CALLBACK_CLASS_NAME,
        gatt_callback::LE_SCAN_CALLBACK_NATIVE_METHODS,
    );
}
