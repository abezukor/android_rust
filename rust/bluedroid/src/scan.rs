use std::task::{Context, Poll};

use futures_core::Stream;
use futures_lite::StreamExt;
use java_owned_rust_object::to_java;
use java_spaghetti::{
    Env, Global, Local, Ref,
    sys::{JNINativeMethod, jobject},
};
use thiserror::Error;

use crate::{
    bindings::{
        android::bluetooth::le::{ScanCallback, ScanResult as JavaScanResult},
        com::maticrobots::rust_bluedroid::{Adapter as JavaAdapter, Scan as JavaBluetoothScan},
        java::util::{ArrayList, List},
    },
    callback_mpsc_channel_send,
};

use scan_filter::ScanFilter;
pub mod scan_filter;
use scan_result::ScanResult;
pub mod scan_result;

type ScanChannelData = Result<ScanResult, ScanError>;

pub struct BluetoothScan {
    items: futures_channel::mpsc::UnboundedReceiver<ScanChannelData>,
    scan: Global<JavaBluetoothScan>,
}

#[derive(Error, Debug)]
pub enum ScanError {
    #[error(
        "Fails to start scan as BLE scan with the same settings is already started by the app."
    )]
    AlreadyStarted,
    #[error("Fails to start scan as app cannot be registered.")]
    ApplicationRegistration,
    #[error("Fails to start power optimized scan as this feature is not supported.")]
    FeatureUnsupported,
    #[error("Fails to start scan due an internal error")]
    InteralError,
    #[error("Fails to start scan as it is out of hardware resources.")]
    OutOfHardwareResources,
    #[error("Fails to start scan as application tries to scan too frequently.")]
    ScanningToFrequently,
    #[error(
        "Scanning is unavailable right now. This usually indicates that bluetooth is disabled on the device"
    )]
    Unavailable,
    #[error("Unknown Error with code {0}")]
    Unknown(i32),
}

impl BluetoothScan {
    pub(crate) fn new(
        adapter: &Global<JavaAdapter>,
        filters: Vec<ScanFilter>,
    ) -> Result<Self, ScanError> {
        let (scan_send, scan_recv) = futures_channel::mpsc::unbounded();

        let scan = adapter.vm().with_env(|env| {
            let scan_send: Local<
                '_,
                crate::bindings::com::maticrobots::rust_android_utilities::RustArcBoxDynAny,
            > = to_java(env, scan_send).unwrap().cast().unwrap();

            let adapter = adapter.as_local(env);
            let scan = match filters.is_empty() {
                true => adapter.leScan_RustArcBoxDynAny(scan_send),
                false => {
                    let java_filters: Local<List> = ArrayList::new(env).unwrap().cast().unwrap();
                    for filter in filters.into_iter() {
                        java_filters
                            .add_Object(filter.java_representation(env).unwrap())
                            .unwrap();
                    }
                    adapter.leScan_RustArcBoxDynAny_List(scan_send, java_filters)
                }
            }
            .unwrap()
            .ok_or(ScanError::Unavailable);

            scan.map(|scan| scan.as_global())
        })?;
        Ok(Self {
            items: scan_recv,
            scan,
        })
    }
}

impl Stream for BluetoothScan {
    type Item = Result<ScanResult, ScanError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.items.poll_next(cx)
    }
}

impl Drop for BluetoothScan {
    fn drop(&mut self) {
        self.scan.vm().with_env(|env| {
            let scan = self.scan.as_local(env);
            log::trace!("Stopping Bluetooth Scan");
            scan.close().unwrap();
        })
    }
}

pub(crate) const LE_SCAN_CALLBACK_CLASS: &str = "com.maticrobots.rust_bluedroid.LEScanCallback";
pub(crate) const LE_SCAN_CALLBACK_NATIVE_METHODS: &[JNINativeMethod] = &[
    JNINativeMethod {
        name: c"processScanResult".as_ptr().cast_mut(),
        signature:
            c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;Landroid/bluetooth/le/ScanResult;)V"
                .as_ptr()
                .cast_mut(),
        fnPtr: Java_com_maticrobots_rust_1bluedroid_LEScanCallback_processScanResult as *mut _,
    },
    JNINativeMethod {
        name: c"processScanError".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;I)V"
            .as_ptr()
            .cast_mut(),
        fnPtr: Java_com_maticrobots_rust_1bluedroid_LEScanCallback_processScanError as *mut _,
    },
];

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_LEScanCallback_processScanResult(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    scan_result: jobject,
) -> bool {
    let scan_result: Ref<JavaScanResult> = unsafe { Ref::from_raw(env, scan_result) };

    callback_mpsc_channel_send::<ScanChannelData>(
        env,
        rust_obj,
        Ok(ScanResult::new(scan_result.as_global())),
    )
    .is_ok()
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_LEScanCallback_processScanError(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    error_code: i32,
) -> bool {
    callback_mpsc_channel_send::<ScanChannelData>(
        env,
        rust_obj,
        Err(ScanError::from_java(error_code)
            .expect("This function should only be called with a non-zero error code")),
    )
    .is_ok()
}

impl ScanError {
    pub fn from_java(error_code: i32) -> Option<Self> {
        match error_code {
            0 => None,
            ScanCallback::SCAN_FAILED_ALREADY_STARTED => Some(Self::AlreadyStarted),
            ScanCallback::SCAN_FAILED_APPLICATION_REGISTRATION_FAILED => {
                Some(Self::ApplicationRegistration)
            }
            ScanCallback::SCAN_FAILED_FEATURE_UNSUPPORTED => Some(Self::FeatureUnsupported),
            ScanCallback::SCAN_FAILED_INTERNAL_ERROR => Some(Self::InteralError),
            ScanCallback::SCAN_FAILED_OUT_OF_HARDWARE_RESOURCES => Some(Self::InteralError),
            ScanCallback::SCAN_FAILED_SCANNING_TOO_FREQUENTLY => Some(Self::ScanningToFrequently),
            other => Some(Self::Unknown(other)),
        }
    }
}
