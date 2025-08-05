use java_spaghetti::{
    sys::{jbyteArray, jobject},
    Env, Global,
};
use uuid::{uuid, Uuid};

use crate::{
    bindings::android::bluetooth::BluetoothGattDescriptor as JavaDescriptor,
    characteristic::{
        rust_on_characteristic_read, rust_on_characteristic_write, CharacteristicReadReturnValue,
        CharacteristicWriteReturnValue,
    },
    device::DeviceWithGattLock,
    error::{BluetoothStatusCode, GattResult},
    java_debug_eq_hash, java_uuid_to_rust, rust_slice_to_java_byte_array, CallBackFuture, GattError,
};

#[derive(Clone)]
pub struct Descriptor {
    pub(crate) descriptor: Global<JavaDescriptor>,
    // doing GATT operations requires a reference the the device
    pub(crate) device: DeviceWithGattLock,
}
java_debug_eq_hash!(Descriptor, descriptor);

impl Descriptor {
    pub const CCC_DESCRIPTOR: Uuid = uuid!("00002902-0000-1000-8000-00805f9b34fb");

    pub fn uuid(&self) -> Uuid {
        self.descriptor.vm().with_env(|env| {
            let this = self.descriptor.as_ref(env);
            java_uuid_to_rust(this.getUuid().unwrap().unwrap())
        })
    }

    pub async fn read(&self) -> GattResult<Box<[u8]>> {
        let gatt_lock = self.device.gatt_lock.lock_arc().await;

        let finished = self.descriptor.vm().with_env(|env| {
            let (rust_obj, future) = CallBackFuture::<CharacteristicReadReturnValue>::new_locked(env, gatt_lock);

            let this = self.descriptor.as_ref(env);
            let device = self.device.device.as_ref(env);

            if !device.readDescriptor(this, rust_obj)? {
                return Err(GattError::NotExecuted);
            }
            Ok(future)
        })?;

        finished.await
    }

    pub async fn write(&self, value: &[u8]) -> GattResult<()> {
        if self.uuid() == Self::CCC_DESCRIPTOR {
            return Err(GattError::WritingToCCCDescriptor);
        }
        self.write_unchecked(value).await
    }

    pub(crate) async fn write_unchecked(&self, value: &[u8]) -> CharacteristicWriteReturnValue {
        let gatt_lock = self.device.gatt_lock.lock_arc().await;

        let finished = self.descriptor.vm().with_env(|env| {
            let (rust_obj, future) = CallBackFuture::<CharacteristicWriteReturnValue>::new_locked(env, gatt_lock);

            let this = self.descriptor.as_ref(env);
            let device = self.device.device.as_ref(env);

            let value = rust_slice_to_java_byte_array(env, value);

            BluetoothStatusCode::gatt_result(device.writeDescriptor(this, rust_obj, value)?)?;
            Ok::<_, GattError>(future)
        })?;

        finished.await
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnDescriptorRead(
    env: Env<'_>,
    this: jobject,
    rust_obj: jobject,
    descriptor_data: jbyteArray,
    status: i32,
) {
    rust_on_characteristic_read(env, this, rust_obj, descriptor_data, status);
}

#[unsafe(no_mangle)]
pub(crate) extern "system" fn Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnDescriptorWrite(
    env: Env<'_>,
    this: jobject,
    rust_obj: jobject,
    status: i32,
) {
    rust_on_characteristic_write(env, this, rust_obj, status);
}
