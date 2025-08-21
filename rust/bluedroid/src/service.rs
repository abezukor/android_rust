use java_spaghetti::Global;
use uuid::Uuid;

use crate::{
    Characteristic,
    bindings::{
        android::bluetooth::{
            BluetoothGattCharacteristic as RawJavaCharacteristic,
            BluetoothGattService as RawJavaService,
        },
        com::maticrobots::rust_bluedroid::Service as JavaService,
    },
    device::DeviceWithGattLock,
    java_debug_eq_hash, java_uuid_to_rust, local_array_to_global_vec,
};

#[derive(Clone)]
pub struct Service {
    pub(crate) service: Global<JavaService>,
    // doing GATT operations requires a reference the the device
    pub(crate) device: DeviceWithGattLock,
}

java_debug_eq_hash!(Service, service);

pub enum ServiceType {
    Primary,
    Secondary,
}

impl Service {
    pub fn uuid(&self) -> Uuid {
        self.service.vm().with_env(|env| {
            let this = self.service.as_ref(env);
            java_uuid_to_rust(this.uuid().unwrap().unwrap())
        })
    }
    pub fn service_type(&self) -> ServiceType {
        let service_type = self.service.vm().with_env(|env| {
            let this = self.service.as_ref(env);
            this.r#type().unwrap()
        });
        match service_type {
            RawJavaService::SERVICE_TYPE_PRIMARY => ServiceType::Primary,
            RawJavaService::SERVICE_TYPE_SECONDARY => ServiceType::Secondary,
            other => unreachable!("Invalid Service type {:?}", other),
        }
    }

    pub fn included_services(&self) -> Vec<Service> {
        let services: Vec<Global<JavaService>> = self.service.vm().with_env(|env| {
            let this = self.service.as_ref(env);
            let services = this.includedServices().unwrap().unwrap();
            local_array_to_global_vec(services)
        });
        services
            .into_iter()
            .map(|service| Self {
                service,
                device: self.device.clone(),
            })
            .collect()
    }

    pub fn characteristics(&self) -> Vec<Characteristic> {
        let services: Vec<Global<RawJavaCharacteristic>> = self.service.vm().with_env(|env| {
            let this = self.service.as_ref(env);
            let characteristics = this.characteristics().unwrap().unwrap();
            local_array_to_global_vec(characteristics)
        });
        services
            .into_iter()
            .map(|characteristic| Characteristic {
                this: characteristic,
                device: self.device.clone(),
            })
            .collect()
    }
}
