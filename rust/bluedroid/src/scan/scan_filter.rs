use java_spaghetti::{Env, Local};
use java_spaghetti_result::JavaResult;
use uuid::Uuid;

use crate::{
    bindings::{
        android::{
            bluetooth::le::{ScanFilter as JavaScanFilter, ScanFilter_Builder},
            os::ParcelUuid,
        },
        java::lang::String as JString,
    },
    rust_java_uuid, rust_slice_to_java_byte_array,
};

#[derive(Debug, Default)]
pub struct ScanFilter {
    pub advertising_data_type: Option<AdvertistingDataType>,
    pub device_address: Option<String>,
    pub name: Option<String>,
    pub manufacturer_data: Option<ManufacturerData>,
    pub service_data: Option<ServiceData>,
    pub solicitation_uuid: Option<SolicitationUuid>,
    pub service_uuid: Option<ServiceUuid>,
}

#[derive(Debug)]
pub struct AdvertistingDataType {
    pub advertising_data_type: u32,
    pub data: Option<AdvertisingDataData>,
}

#[derive(Debug)]
pub struct AdvertisingDataData {
    pub data: Box<[u8]>,
    pub mask: Option<u128>,
}

#[derive(Debug)]
pub struct ManufacturerData {
    pub id: u32,
    pub data: Box<[u8]>,
    pub mask: Option<Box<[u8]>>,
}

#[derive(Debug)]
pub struct ServiceData {
    pub uuid: Uuid,
    pub data: Box<[u8]>,
    pub mask: Option<Box<[u8]>>,
}

#[derive(Debug)]
pub struct SolicitationUuid {
    pub uuid: Uuid,
    pub mask: Option<Uuid>,
}

#[derive(Debug)]
pub struct ServiceUuid {
    pub uuid: Uuid,
    pub mask: Option<Uuid>,
}

macro_rules! builder_property {
    ($self:ident, $builder:ident, $filter_property:ident, $if_some:block) => {
        let $builder = if let Some($filter_property) = $self.$filter_property.as_ref() {
            $if_some
        } else {
            $builder
        };
    };
}

impl ScanFilter {
    pub(super) fn java_representation<'a>(&self, env: Env<'a>) -> JavaResult<Local<'a, JavaScanFilter>> {
        let builder = ScanFilter_Builder::new(env)?;

        builder_property!(self, builder, advertising_data_type, {
            match advertising_data_type.data.as_ref() {
                Some(advertising_data_data) => {
                    let java_data = rust_slice_to_java_byte_array(env, &advertising_data_data.data);
                    let mask = advertising_data_data.mask.unwrap_or(u128::MAX);
                    let mask = mask.to_ne_bytes();
                    let java_mask = rust_slice_to_java_byte_array(env, &mask);
                    builder.setAdvertisingDataTypeWithData(
                        advertising_data_type.advertising_data_type.cast_signed(),
                        java_data,
                        java_mask,
                    )
                }
                None => builder.setAdvertisingDataType(advertising_data_type.advertising_data_type.cast_signed()),
            }?
            .unwrap()
        });
        builder_property!(self, builder, device_address, {
            let device_address = JString::from_env_str(env, device_address);
            builder.setDeviceAddress(device_address)?.unwrap()
        });
        builder_property!(self, builder, name, {
            let name = JString::from_env_str(env, name);
            builder.setDeviceName(name).unwrap().unwrap()
        });
        builder_property!(self, builder, manufacturer_data, {
            let manufacturer_id = manufacturer_data.id.cast_signed();
            let manufacturer_data_data = rust_slice_to_java_byte_array(env, &manufacturer_data.data);
            match manufacturer_data.mask.as_ref() {
                Some(mask) => {
                    let java_mask = rust_slice_to_java_byte_array(env, mask);
                    builder.setManufacturerData_int_byte_array_byte_array(
                        manufacturer_id,
                        manufacturer_data_data,
                        java_mask,
                    )
                }
                None => builder.setManufacturerData_int_byte_array(manufacturer_id, manufacturer_data_data),
            }?
            .unwrap()
        });

        builder_property!(self, builder, service_data, {
            let service_uuid = rust_java_uuid(service_data.uuid, env);
            let service_uuid = ParcelUuid::new(env, service_uuid).unwrap();
            let service_data_data = rust_slice_to_java_byte_array(env, &service_data.data);

            match service_data.mask.as_ref() {
                Some(mask) => {
                    let java_mask = rust_slice_to_java_byte_array(env, mask);
                    builder.setServiceData_ParcelUuid_byte_array_byte_array(service_uuid, service_data_data, java_mask)
                }
                None => builder.setServiceData_ParcelUuid_byte_array(service_uuid, service_data_data),
            }?
            .unwrap()
        });
        builder_property!(self, builder, solicitation_uuid, {
            let solicitation_uuid_full = rust_java_uuid(solicitation_uuid.uuid, env);
            let solicitation_uuid_full = ParcelUuid::new(env, solicitation_uuid_full).unwrap();

            match solicitation_uuid.mask.as_ref() {
                Some(mask) => {
                    let mask = rust_java_uuid(*mask, env);
                    let mask = ParcelUuid::new(env, mask).unwrap();

                    builder.setServiceSolicitationUuid_ParcelUuid_ParcelUuid(solicitation_uuid_full, mask)
                }
                None => builder.setServiceSolicitationUuid_ParcelUuid(solicitation_uuid_full),
            }?
            .unwrap()
        });

        builder_property!(self, builder, service_uuid, {
            let service_uuid_full = rust_java_uuid(service_uuid.uuid, env);
            let service_uuid_full = ParcelUuid::new(env, service_uuid_full).unwrap();

            match service_uuid.mask.as_ref() {
                Some(mask) => {
                    let mask = rust_java_uuid(*mask, env);
                    let mask = ParcelUuid::new(env, mask).unwrap();
                    builder.setServiceUuid_ParcelUuid_ParcelUuid(service_uuid_full, mask)
                }
                None => builder.setServiceUuid_ParcelUuid(service_uuid_full),
            }?
            .unwrap()
        });

        let scan_filter = builder.build()?.unwrap();
        // Fix annoying lifetime issues
        Ok(unsafe { Local::from_raw(env, scan_filter.into_raw()) })
    }
}
