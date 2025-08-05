use futures_core::Stream;
use java_spaghetti::{
    sys::{jbyteArray, jobject},
    ByteArray, Env, Global, Local, Ref,
};
use rust_android_utilities::java_wrapped_object::to_java;
use uuid::Uuid;

use crate::{
    bindings::{
        android::bluetooth::{
            BluetoothGattCharacteristic as JavaCharacteristic, BluetoothGattDescriptor as JavaDescriptor,
        },
        com::maticrobots::rust_android_utilities::RustArcBoxDynAny,
    },
    callback_mpsc_channel_send,
    device::DeviceWithGattLock,
    error::{BluetoothStatusCode, GattResult},
    java_byte_array_to_rust_boxed_slice, java_debug_eq_hash, java_uuid_to_rust, rust_java_uuid,
    rust_slice_to_java_byte_array, CallBackFuture, CallBackFutureData, Descriptor, GattError,
};

#[derive(Clone)]
pub struct Characteristic {
    pub(crate) this: Global<JavaCharacteristic>,
    pub(crate) device: DeviceWithGattLock,
}
java_debug_eq_hash!(Characteristic, this);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CharacteristicProperties(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteType {
    Default,
    NoResponse,
    Signed,
}

pub(crate) type CharacteristicReadReturnValue = GattResult<Box<[u8]>>;
pub(crate) type CharacteristicWriteReturnValue = GattResult<()>;

impl Characteristic {
    pub fn uuid(&self) -> Uuid {
        self.this.vm().with_env(|env| {
            let this = self.this.as_ref(env);
            java_uuid_to_rust(this.getUuid().unwrap().unwrap())
        })
    }

    pub fn properties(&self) -> CharacteristicProperties {
        self.this.vm().with_env(|env| {
            let this = self.this.as_ref(env);
            CharacteristicProperties(this.getProperties().unwrap())
        })
    }

    pub fn descriptors(&self) -> Vec<Descriptor> {
        let descriptors: Vec<Global<JavaDescriptor>> = self.this.vm().with_env(|env| {
            let this = self.this.as_ref(env);
            let Some(descriptors) = this.getDescriptors().unwrap() else {
                return Vec::new();
            };
            let descriptors = descriptors.toArray().unwrap().unwrap();

            descriptors
                .iter()
                .filter_map(|item| {
                    item.as_ref()
                        .map(|item| item.cast::<JavaDescriptor>().unwrap().as_global())
                })
                .collect()
        });

        descriptors
            .into_iter()
            .map(|descriptor| Descriptor { descriptor, device: self.device.clone() })
            .collect()
    }

    pub fn get_descriptor(&self, uuid: Uuid) -> Option<Descriptor> {
        let descriptor = self.this.vm().with_env(|env| {
            let this = self.this.as_ref(env);

            let descriptor_uuid = rust_java_uuid(uuid, env);
            let descriptor = this.getDescriptor(descriptor_uuid).unwrap();
            descriptor.as_ref().map(Local::as_global)
        });
        descriptor.map(|descriptor| Descriptor { descriptor, device: self.device.clone() })
    }

    pub async fn read(&self) -> GattResult<Box<[u8]>> {
        let gatt_lock = self.device.gatt_lock.lock_arc().await;
        let finished = self.this.vm().with_env(|env| {
            let (rust_obj, future) = CallBackFuture::<CharacteristicReadReturnValue>::new_locked(env, gatt_lock);

            let this = self.this.as_ref(env);
            let device = self.device.device.as_ref(env);

            if !device.readCharacteristic(this, rust_obj)? {
                return Err(GattError::NotExecuted);
            }
            Ok(future)
        })?;

        finished.await
    }

    pub async fn write(&self, write_type: WriteType, value: &[u8]) -> GattResult<()> {
        let gatt_lock = self.device.gatt_lock.lock_arc().await;

        let finished = self.this.vm().with_env(|env| {
            let (rust_obj, future) = CallBackFuture::<CharacteristicWriteReturnValue>::new_locked(env, gatt_lock);

            let this = self.this.as_ref(env);
            let device = self.device.device.as_ref(env);

            let value = rust_slice_to_java_byte_array(env, value);

            BluetoothStatusCode::gatt_result(device.writeCharacteristic(
                this,
                rust_obj,
                value,
                write_type.java_representation(),
            )?)?;
            Ok::<_, GattError>(future)
        })?;

        finished.await
    }

    pub async fn notify(&self) -> GattResult<impl Stream<Item = Box<[u8]>>> {
        let (update_send, update_recv) = futures_channel::mpsc::unbounded();

        let adapter_notifications_enabled = self.this.vm().with_env(|env| {
            let this = self.this.as_ref(env);
            let device = self.device.device.as_ref(env);

            let scan_send: Local<RustArcBoxDynAny> = to_java(env, update_send)?.cast().unwrap();

            Ok::<_, GattError>(device.enableCharacteristicNotification(this, scan_send)?)
        })?;

        if !adapter_notifications_enabled {
            return Err(GattError::NotExecuted);
        }

        // CCC Descriptor is never null
        let ccc_descriptor = self.get_descriptor(Descriptor::CCC_DESCRIPTOR).unwrap();

        let enable_notification_value = self.this.vm().with_env(|env| {
            let enable_notification_value = JavaDescriptor::ENABLE_NOTIFICATION_VALUE(env).unwrap();
            java_byte_array_to_rust_boxed_slice(enable_notification_value.as_ref())
        });

        ccc_descriptor.write_unchecked(&enable_notification_value).await?;
        Ok(update_recv)
    }
}

#[unsafe(export_name = "Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnCharacteristicRead")]
pub(crate) extern "system" fn rust_on_characteristic_read(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    characteristic_data: jbyteArray,
    status: i32,
) {
    let val = GattError::status_error(status).map(|()| {
        let data: Ref<ByteArray> = unsafe { Ref::from_raw(env, characteristic_data) };
        java_byte_array_to_rust_boxed_slice(data)
    });

    //Using an explicit type annotation to ensure the type is correct
    unsafe {
        CallBackFutureData::<CharacteristicReadReturnValue>::wake(env, rust_obj, val);
    }
}

#[unsafe(export_name = "Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnCharacteristicWrite")]
pub(crate) extern "system" fn rust_on_characteristic_write(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    status: i32,
) {
    unsafe {
        CallBackFutureData::<CharacteristicWriteReturnValue>::wake(env, rust_obj, GattError::status_error(status));
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnCharacteristicChanged(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    characteristic_data: jbyteArray,
) -> bool {
    let characteristic_data: Ref<ByteArray> = unsafe { Ref::from_raw(env, characteristic_data) };
    let characteristic_data = java_byte_array_to_rust_boxed_slice(characteristic_data);

    callback_mpsc_channel_send(env, rust_obj, characteristic_data).is_ok()
}

macro_rules! characteristic_property {
    ($name:ident, $bitmask:path) => {
        #[inline]
        pub const fn $name(&self) -> bool {
            (self.0 & $bitmask) != 0
        }
    };
}
impl CharacteristicProperties {
    characteristic_property!(broadcast, JavaCharacteristic::PROPERTY_BROADCAST);
    characteristic_property!(extended_props, JavaCharacteristic::PROPERTY_EXTENDED_PROPS);
    characteristic_property!(indicate, JavaCharacteristic::PROPERTY_INDICATE);
    characteristic_property!(notify, JavaCharacteristic::PROPERTY_NOTIFY);
    characteristic_property!(read, JavaCharacteristic::PROPERTY_READ);
    characteristic_property!(signed_write, JavaCharacteristic::PROPERTY_SIGNED_WRITE);
    characteristic_property!(write_no_response, JavaCharacteristic::PROPERTY_WRITE_NO_RESPONSE);
    characteristic_property!(write, JavaCharacteristic::PROPERTY_WRITE);
}

impl std::fmt::Debug for CharacteristicProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharacteristicProperties")
            .field("broadcast", &self.broadcast())
            .field("extended_props", &self.extended_props())
            .field("indicate", &self.indicate())
            .field("notify", &self.notify())
            .field("read", &self.read())
            .field("signed_write", &self.signed_write())
            .field("write_no_response", &self.write_no_response())
            .field("write", &self.write())
            .finish()
    }
}

impl WriteType {
    fn java_representation(&self) -> i32 {
        match self {
            WriteType::Default => JavaCharacteristic::WRITE_TYPE_DEFAULT,
            WriteType::NoResponse => JavaCharacteristic::WRITE_TYPE_NO_RESPONSE,
            WriteType::Signed => JavaCharacteristic::WRITE_TYPE_SIGNED,
        }
    }
}

impl Default for WriteType {
    fn default() -> Self {
        Self::Default
    }
}
