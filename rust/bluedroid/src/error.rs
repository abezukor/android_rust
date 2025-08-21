use java_spaghetti::Local;
use java_spaghetti_result::JavaError;
use std::num::NonZeroI32;
use thiserror::Error;

use crate::bindings::{
    android::bluetooth::{BluetoothGatt, BluetoothStatusCodes},
    java::lang::Throwable,
};

pub type GattResult<T> = Result<T, GattError>;

#[derive(Error, Debug)]
pub enum GattError {
    #[error("GATT read operation is not permitted")]
    GattReadNotPermitted,
    #[error("GATT write operation is not permitted")]
    GattWriteNotPermitted,
    #[error("Insufficient authentication for a given operation")]
    GattInsufficientAuthentication,
    #[error("The given request is not supported")]
    GattRequestNotSupported,
    #[error("Insufficient encryption for a given operation")]
    GattInsufficientEncryption,
    #[error("A read or write operation was requested with an invalid offset")]
    GattInvalidOffset,
    #[error("Insufficient authorization for a given operation")]
    GattInsufficientAuthorization,
    #[error("A write operation exceeds the maximum length of the attribute")]
    GattInvalidAttributeLength,
    #[error("A remote device connection is congested.")]
    GattConnectionCongested,
    #[error(
        "GATT connection timed out, likely due to the remote device being out of range or not advertising as connectable."
    )]
    GattConnectionTimeout,
    #[error("A GATT operation failed, errors other than the above")]
    GattFailure,
    #[error("There was an attempt to write to the characteristic control descriptor, that can cause race conditions when interafcting with enabling/disabling characteristic notifications and is thus unsupported")]
    WritingToCCCDescriptor,

    #[error(transparent)]
    BluetoothStatusCode(#[from] BluetoothStatusCode),

    #[error("Unknown Error with code {0}")]
    UnknownError(NonZeroI32),

    #[error("Java Exception thrown: {0:?}")]
    JavaError(#[from] JavaError),

    #[error("Java Gatt Function did not execute (returned false).")]
    NotExecuted,

    #[error("The device must be connected to execcute this operation.")]
    NotConnected,
}

impl GattError {
    pub(crate) fn status_error(status: i32) -> Result<(), Self> {
        match status {
            BluetoothGatt::GATT_SUCCESS => Ok(()),
            BluetoothGatt::GATT_READ_NOT_PERMITTED => Err(Self::GattReadNotPermitted),
            BluetoothGatt::GATT_WRITE_NOT_PERMITTED => Err(Self::GattWriteNotPermitted),
            BluetoothGatt::GATT_INSUFFICIENT_AUTHENTICATION => Err(Self::GattInsufficientAuthentication),
            BluetoothGatt::GATT_REQUEST_NOT_SUPPORTED => Err(Self::GattRequestNotSupported),
            BluetoothGatt::GATT_INSUFFICIENT_ENCRYPTION => Err(Self::GattInsufficientEncryption),
            BluetoothGatt::GATT_INVALID_OFFSET => Err(Self::GattInvalidOffset),
            BluetoothGatt::GATT_INSUFFICIENT_AUTHORIZATION => Err(Self::GattInsufficientAuthorization),
            BluetoothGatt::GATT_INVALID_ATTRIBUTE_LENGTH => Err(Self::GattInvalidAttributeLength),
            BluetoothGatt::GATT_CONNECTION_CONGESTED => Err(Self::GattConnectionCongested),
            BluetoothGatt::GATT_CONNECTION_TIMEOUT => Err(Self::GattConnectionTimeout),
            BluetoothGatt::GATT_FAILURE => Err(Self::GattFailure),
            other => Err(Self::UnknownError(NonZeroI32::new(other).unwrap())),
        }
    }
}

impl From<Local<'_, Throwable>> for GattError {
    fn from(err: Local<'_, Throwable>) -> Self {
        Self::JavaError(err.into())
    }
}

#[derive(Error, Debug)]
pub enum BluetoothStatusCode {
    #[error("Error code indicating that the API call was initiated by neither the system nor the active user.")]
    NotAllowed,
    #[error("Error code indicating that Bluetooth is not enabled.")]
    NotEnabled,
    #[error("Error code indicating that the Bluetooth Device specified is not bonded.")]
    NotBonded,
    #[error("A GATT writeCharacteristic request is not permitted on the remote device.")]
    GattWriteNotAllowed,
    #[error("GATT writeCharacteristic request is not permitted on the remote device.")]
    GattWriteBusy,
    #[error(
        "Error code indicating that the caller does not have the Manifest.permission.BLUETOOTH_CONNECT permission."
    )]
    MissingBluetoothConnectPermission,
    #[error("Error code indicating that the profile service is not bound. You can bind a profile service by calling BluetoothAdapter.getProfileProxy.")]
    ProfileServiceNotBound,
    #[error("Indicates that an unknown error has occurred.")]
    Unknown,
    #[error("Indicates that the feature status is not configured yet.")]
    FeatureNotConfigured,
    #[error("Indicates that the feature is not supported.")]
    FeatureNotSupported,
    #[error("Unknown Error with code {0}")]
    UnknownError(NonZeroI32),
}

impl BluetoothStatusCode {
    pub(crate) fn java_status_code(code: NonZeroI32) -> Self {
        let raw_code = code.get();
        match raw_code {
            BluetoothStatusCodes::ERROR_BLUETOOTH_NOT_ALLOWED => Self::NotAllowed,
            BluetoothStatusCodes::ERROR_BLUETOOTH_NOT_ENABLED => Self::NotEnabled,
            BluetoothStatusCodes::ERROR_DEVICE_NOT_BONDED => Self::NotBonded,
            BluetoothStatusCodes::ERROR_GATT_WRITE_NOT_ALLOWED => Self::GattWriteNotAllowed,
            BluetoothStatusCodes::ERROR_GATT_WRITE_REQUEST_BUSY => Self::GattWriteBusy,
            BluetoothStatusCodes::ERROR_MISSING_BLUETOOTH_CONNECT_PERMISSION => Self::MissingBluetoothConnectPermission,
            BluetoothStatusCodes::ERROR_PROFILE_SERVICE_NOT_BOUND => Self::ProfileServiceNotBound,
            BluetoothStatusCodes::ERROR_UNKNOWN => Self::Unknown,
            BluetoothStatusCodes::FEATURE_NOT_CONFIGURED => Self::FeatureNotConfigured,
            BluetoothStatusCodes::FEATURE_NOT_SUPPORTED => Self::FeatureNotSupported,
            _ => Self::UnknownError(code),
        }
    }

    #[inline]
    pub(crate) fn status_result(code: i32) -> Result<(), Self> {
        match NonZeroI32::new(code) {
            Some(code) => Err(Self::java_status_code(code)),
            None => Ok(()),
        }
    }

    #[inline]
    pub(crate) fn gatt_result(code: i32) -> Result<(), GattError> {
        Self::status_result(code).map_err(GattError::BluetoothStatusCode)
    }
}
