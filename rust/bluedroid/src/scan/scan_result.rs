use std::{collections::HashMap, sync::OnceLock, time::Duration};

use java_spaghetti::{ByteArray, Global, Local, PrimitiveArray};
use uuid::Uuid;

use crate::{
    bindings::{
        android::{
            bluetooth::le::{ScanRecord, ScanResult as JavaScanResult},
            content::Context,
            os::ParcelUuid,
        },
        com::maticrobots::rust_bluedroid::BluetoothDevice as JavaBluetoothDevice,
        java::util::Map_Entry,
    },
    java_byte_array_to_rust_boxed_slice, java_uuid_to_rust, rust_java_uuid, Adapter, Device,
};

pub struct ScanResult {
    result: Global<JavaScanResult>,
    record: OnceLock<Global<ScanRecord>>,
}

pub enum DataStatus {
    Complete,
    Truncated,
}

#[derive(Default, Clone, Copy)]
pub struct AdvertisingFlags(i32);

impl ScanResult {
    pub(crate) fn new(scan_result: Global<JavaScanResult>) -> Self {
        Self { result: scan_result, record: OnceLock::new() }
    }

    pub fn advertising_sid(&self) -> u32 {
        self.result
            .vm()
            .with_env(|env| {
                let sc = self.result.as_ref(env);
                sc.getAdvertisingSid().unwrap()
            })
            .cast_unsigned()
    }

    pub fn data_status(&self) -> DataStatus {
        let data_status = self.result.vm().with_env(|env| {
            let sc = self.result.as_ref(env);
            sc.getDataStatus().unwrap()
        });
        match data_status {
            JavaScanResult::DATA_COMPLETE => DataStatus::Complete,
            JavaScanResult::DATA_TRUNCATED => DataStatus::Truncated,
            other => unreachable!("Invalid data status {}", other),
        }
    }

    pub fn device(&self) -> Device {
        let application_context = unsafe { java_spaghetti_context::get_application_context::<Context>() };
        let device = self.result.vm().with_env(|env| {
            let context = application_context.as_ref(env);
            let scan_result = self.result.as_ref(env);
            let device = scan_result.getDevice().unwrap().unwrap();
            let device = JavaBluetoothDevice::new(env, device, context).unwrap();
            device.as_global()
        });
        Device::new(device, Adapter::default_java_adapter())
    }

    pub fn periodic_advertising_interval(&self) -> Option<Duration> {
        const ANDROID_PERIODIC_ADVERTISING_INTERVAL_DURATION: Duration = Duration::from_micros(1250);

        let periodic_advertising_interval = self.result.vm().with_env(|env| {
            let scan_result = self.result.as_ref(env);
            scan_result.getPeriodicAdvertisingInterval().unwrap()
        });
        if periodic_advertising_interval == JavaScanResult::PERIODIC_INTERVAL_NOT_PRESENT {
            return None;
        }
        let periodic_advertising_interval: u32 = periodic_advertising_interval
            .try_into()
            .expect("Periodic advertising interval should be non-negative");
        Some(ANDROID_PERIODIC_ADVERTISING_INTERVAL_DURATION * periodic_advertising_interval)
    }

    pub fn rssi(&self) -> i8 {
        self.result
            .vm()
            .with_env(|env| {
                let scan_result = self.result.as_ref(env);
                scan_result.getRssi().unwrap()
            })
            .try_into()
            .expect("rssi should be [-127, 126]")
    }

    pub fn advertising_flags(&self) -> Option<AdvertisingFlags> {
        let record = self.scan_record();
        let advertising_flags = record.vm().with_env(|env| {
            let record = record.as_ref(env);
            record.getAdvertiseFlags().unwrap()
        });

        (advertising_flags != -1).then_some(AdvertisingFlags(advertising_flags))
    }

    pub fn manufacturer_specific_data(&self) -> HashMap<u32, Box<[u8]>> {
        let record = self.scan_record();
        record.vm().with_env(|env| {
            let record = record.as_ref(env);
            let Some(manufacturer_data) = record.getManufacturerSpecificData().unwrap() else {
                return HashMap::new();
            };

            let data_size = manufacturer_data.size().unwrap();
            let mut manufacturer_data_map = HashMap::with_capacity(data_size.try_into().unwrap());
            for i in 0..data_size {
                let key = manufacturer_data.keyAt(i).unwrap();
                let value = manufacturer_data.valueAt(i).unwrap().expect("Reported size is wrong");
                let value: Local<ByteArray> = value.cast().unwrap();
                let value: Box<[u8]> = value.as_vec().into_iter().map(i8::cast_unsigned).collect();
                manufacturer_data_map.insert(key.cast_unsigned(), value);
            }
            manufacturer_data_map
        })
    }

    pub fn manufacturer_id_specific_data(&self, manufacturer_id: u32) -> Option<Box<[u8]>> {
        let record = self.scan_record();
        record.vm().with_env(|env| {
            let record = record.as_ref(env);
            let mfg_data = record
                .getManufacturerSpecificData_int(manufacturer_id.cast_signed())
                .unwrap()?;
            let mfg_data: Box<[u8]> = mfg_data.as_vec().into_iter().map(i8::cast_unsigned).collect();
            Some(mfg_data)
        })
    }

    pub fn get_service_data(&self, service_uuid: Uuid) -> Option<Box<[u8]>> {
        let record = self.scan_record();
        record.vm().with_env(|env| {
            let record = record.as_ref(env);
            let service_uuid = rust_java_uuid(service_uuid, env);
            let service_uuid = ParcelUuid::new(env, service_uuid).unwrap();
            let service_data = record.getServiceData_ParcelUuid(service_uuid).unwrap()?;
            Some(java_byte_array_to_rust_boxed_slice(service_data.as_ref()))
        })
    }

    pub fn get_service_datas(&self) -> HashMap<Uuid, Box<[u8]>> {
        let record = self.scan_record();
        record.vm().with_env(|env| {
            let record = record.as_ref(env);
            let Some(service_data) = record.getServiceData().unwrap() else {
                return HashMap::new();
            };

            let entry_set = service_data.entrySet().unwrap().unwrap();
            let entries = entry_set.toArray().unwrap().unwrap();

            HashMap::from_iter(entries.iter().filter_map(|entry| {
                let entry = entry?;

                let entry: Local<Map_Entry> = entry.cast().unwrap();

                let key = entry.getKey().unwrap().unwrap();
                let key: Local<ParcelUuid> = key.cast().unwrap();
                let key = java_uuid_to_rust(key.getUuid().unwrap().unwrap());

                let value = entry.getValue().unwrap().unwrap();
                let value: Local<ByteArray> = value.cast().unwrap();
                let service_data: Box<[u8]> = value.as_vec().into_iter().map(i8::cast_unsigned).collect();

                Some((key, service_data))
            }))
        })
    }

    pub fn get_service_solicitation_uuids(&self) -> Box<[Uuid]> {
        let record = self.scan_record();
        record.vm().with_env(|env| {
            let record = record.as_ref(env);
            let Some(uuids) = record.getServiceSolicitationUuids().unwrap() else {
                return Vec::with_capacity(0).into_boxed_slice();
            };
            let uuids = uuids.toArray().unwrap().unwrap();
            uuids
                .iter()
                .filter_map(|uuid| {
                    let uuid = uuid?;
                    let uuid: Local<ParcelUuid> = uuid.cast().unwrap();
                    let uuid = uuid.getUuid().unwrap().unwrap();
                    Some(java_uuid_to_rust(uuid))
                })
                .collect()
        })
    }

    pub fn get_service_uuids(&self) -> Vec<Uuid> {
        let record = self.scan_record();
        record.vm().with_env(|env| {
            let record = record.as_ref(env);
            let Some(uuids) = record.getServiceUuids().unwrap() else {
                return Vec::new();
            };
            let uuids = uuids.toArray().unwrap().unwrap();
            uuids
                .iter()
                .filter_map(|uuid| {
                    let uuid = uuid?;
                    let uuid: Local<ParcelUuid> = uuid.cast().unwrap();
                    let uuid = uuid.getUuid().unwrap().unwrap();
                    Some(java_uuid_to_rust(uuid))
                })
                .collect()
        })
    }

    pub fn tx_power_level(&self) -> Option<i8> {
        let tx_power = self.result.vm().with_env(|env| {
            let scan_result = self.result.as_ref(env);
            scan_result.getTxPower().unwrap()
        });
        if tx_power == JavaScanResult::TX_POWER_NOT_PRESENT {
            None
        } else {
            Some(tx_power.try_into().expect("Tx power should be [-127, 126]"))
        }
    }

    pub fn is_connectable(&self) -> bool {
        self.result.vm().with_env(|env| {
            let scan_result = self.result.as_ref(env);
            scan_result.isConnectable().unwrap()
        })
    }

    pub fn local_name(&self) -> Option<String> {
        let record = self.scan_record();
        record.vm().with_env(|env| {
            let record = record.as_ref(env);
            let name = record.getDeviceName().unwrap();
            name.map(|name| name.to_string().unwrap())
        })
    }

    fn scan_record(&self) -> &Global<ScanRecord> {
        self.record.get_or_init(|| {
            self.result.vm().with_env(|env| {
                let result = self.result.as_ref(env);
                result.getScanRecord().unwrap().unwrap().as_global()
            })
        })
    }
}

macro_rules! advertising_flags {
    ($fn_name:ident, $bitmask:ident) => {
        #[inline]
        pub const fn $fn_name(&self) -> bool {
            (self.0 & Self::$bitmask) != 0
        }
    };
}

impl AdvertisingFlags {
    // See https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/CSS_v11/out/en/supplement-to-the-bluetooth-core-specification/data-types-specification.html
    const LE_DISCOVERABLE_MODE: i32 = 0b1;
    const GENERAL_DISCOVERABLE_MODE: i32 = 0b10;
    const BR_EDR_NOT_SUPPORTED: i32 = 0b100;
    const SIMULTANIOUS_LE_BR_EDR: i32 = 0b1000;

    advertising_flags!(le_discoverable, LE_DISCOVERABLE_MODE);
    advertising_flags!(general_discoverable, GENERAL_DISCOVERABLE_MODE);
    advertising_flags!(br_edr_not_supported, BR_EDR_NOT_SUPPORTED);
    advertising_flags!(simultaneous_le_br_edr, SIMULTANIOUS_LE_BR_EDR);
}

impl From<AdvertisingFlags> for u32 {
    fn from(value: AdvertisingFlags) -> Self {
        value.0.cast_unsigned()
    }
}

impl std::fmt::Debug for AdvertisingFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdvertisingFlags")
            .field("le_discoverable", &self.le_discoverable())
            .field("general_discoverable", &self.general_discoverable())
            .field("br_edr_not_supported", &self.br_edr_not_supported())
            .field("simultaneous_le_br_edr", &self.simultaneous_le_br_edr())
            .field("raw", &self.0)
            .finish()
    }
}
