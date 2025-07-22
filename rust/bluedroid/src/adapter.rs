use java_spaghetti::Global;
use rust_android_utilities::{get_application_context, get_vm, JavaError, JavaResult};

use crate::{
    bindings::{
        android::content::Context as AndroidContext,
        com::maticrobots::rust_bluedroid::{Adapter as JavaAdapter, BluetoothDevice as JavaBluetoothDevice},
        java::lang::{String as JString, Throwable},
    },
    java_debug_eq_hash, local_array_to_global_vec,
    scan::{scan_filter::ScanFilter, BluetoothScan},
    Device,
};

#[derive(Clone)]
pub struct Adapter(Global<JavaAdapter>);
java_debug_eq_hash!(Adapter);

impl Default for Adapter {
    fn default() -> Self {
        Self(Self::default_java_adapter())
    }
}

impl Adapter {
    pub fn open_device(&self, address: &str) -> JavaResult<crate::Device> {
        let device: JavaResult<Global<JavaBluetoothDevice>> = self.0.vm().with_env(|env| {
            let adapter = self.0.as_ref(env);
            let address = JString::from_env_str(env, address);
            // Throws exception when address is invalid
            let device = adapter.openDevice(address)?;
            Ok(device.unwrap().as_global())
        });
        Ok(Device::new(device?, self.0.clone()))
    }

    pub fn bonded_devices(&self) -> JavaResult<Vec<Device>> {
        let devices = self.0.vm().with_env(|env| {
            let adapter = self.0.as_ref(env);
            //Null on error
            let devices = adapter.getBondedDevices().unwrap().ok_or_else(|| {
                Throwable::new_String(env, JString::from_env_str(env, "Error getting bonded devices")).unwrap()
            })?;
            Ok::<_, JavaError>(local_array_to_global_vec(devices))
        })?;
        Ok(self.devices_list(devices.into_iter()))
    }

    pub fn connected_devices(&self) -> JavaResult<Vec<Device>> {
        let devices = self.0.vm().with_env(|env| {
            let adapter = self.0.as_ref(env);
            let devices = adapter.getConnectedDevices()?.unwrap();
            Ok::<_, JavaError>(local_array_to_global_vec(devices))
        })?;
        Ok(self.devices_list(devices.into_iter()))
    }

    pub fn scan(&self, filters: Vec<ScanFilter>) -> Result<BluetoothScan, crate::scan::ScanError> {
        BluetoothScan::new(&self.0, filters)
    }

    pub(crate) fn default_java_adapter() -> Global<JavaAdapter> {
        get_vm().with_env(|env| {
            let adapter = JavaAdapter::new(env, unsafe { get_application_context::<AndroidContext>() }).unwrap();
            adapter.as_global()
        })
    }

    fn devices_list(&self, devices: impl Iterator<Item = Global<JavaBluetoothDevice>>) -> Vec<Device> {
        devices.map(|device| Device::new(device, self.0.clone())).collect()
    }
}
