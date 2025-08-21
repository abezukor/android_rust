use std::{
    mem::ManuallyDrop,
    sync::{Arc, LazyLock, OnceLock},
};

use bluedroid::{Channel, Device};
use futures::io::{AsyncReadExt, AsyncWriteExt};
use futures_lite::StreamExt;
use java_spaghetti::{sys::jobject, Env, Ref};
use log::{debug, info, trace, warn};
use java_owned_rust_object::{get_ref,to_java_arc,BoxedRustObj,  bindings::{com::maticrobots::java_rust_obj::RustArcBoxDynAny, java::lang::String as JString},};
use uuid::Uuid;

#[cfg(target_os = "android")]
use ctor::ctor;

static RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Builder::new_multi_thread().build().unwrap());

#[cfg(target_os = "android")]
#[ctor]
fn initialization() {
    use android_logger::Config;
    use log::{info, LevelFilter};

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    info!("Android Logger Started");

    std::panic::set_hook(Box::new(|panic_hook_info| {
        if let Some(s) = panic_hook_info.payload().downcast_ref::<&str>() {
            log::error!("Panic Payload: {s:?}");
        } else if let Some(s) = panic_hook_info.payload().downcast_ref::<String>() {
            log::error!("Panic Payload: {s:?}");
        }
        log::error!("Panic at {:?}", panic_hook_info.location());
        log::error!("Backtrace: {}", std::backtrace::Backtrace::force_capture());
    }));
}

#[unsafe(no_mangle)]
pub(crate) extern "system" fn Java_com_matician_bluerdroid_1example_1app_RustInitialization_rust_1bluetooth_1search(
    env: Env<'_>,
    _this: jobject,
    device_mac: jobject,
) -> jobject {
    let mac_address: Ref<JString> = unsafe { Ref::from_raw(env, device_mac) };
    let mac_address = mac_address.to_string().unwrap();
    let adapter = bluedroid::Adapter::default();

    let device_lock: Arc<BoxedRustObj> = Arc::new(Box::new(OnceLock::<Device>::new()));
    {
        let device_lock_owned = device_lock.clone();
        RUNTIME.spawn(async move {
            let mut stream = adapter.scan(Vec::new()).unwrap();
            let device_lock = device_lock_owned.downcast_ref::<OnceLock<Device>>().unwrap();
            while let Some(scan_result) = stream.next().await {
                let device: Device = scan_result.unwrap().device();
                let id = device.id();

                log::info!("Found device with id {id}");
                if id == mac_address {
                    device_lock.get_or_init(|| device);
                    return;
                }
            }
            panic!("Could not find device");
        });
    }

    ManuallyDrop::new(to_java_arc(env, device_lock).unwrap()).as_raw()
}

pub const SERVICE_U128: u128 = 70636206720504692524825634533565718410;
const SERVICE_UUID: Uuid = Uuid::from_u128_le(SERVICE_U128);

pub const CHARACTERISTIC_U128: u128 = 122390471841911414921581445682298165206;
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128_le(CHARACTERISTIC_U128);
pub const CHARACTERISTIC_INITIAL_VALUE: &[u8] = &[100];

pub const DESCRIPTOR_U128: u128 = 48453633131855706822698818651509372353;
const DESCRIPTOR_UUID: Uuid = Uuid::from_u128_le(DESCRIPTOR_U128);
pub const DESCRIPTOR_INITIAL_VALUE: &[u8] = &[101];

pub const PSM: u16 = 157;

pub const PAIRED_CHARACTERISTIC_U128: u128 = 272686533043362580079835821053973224242;
const PAIRED_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128_le(PAIRED_CHARACTERISTIC_U128);
pub const PAIRED_CHARACTERISTIC_INITIAL_VALUE: &[u8] = &[102];

#[unsafe(no_mangle)]
pub(crate) extern "system" fn Java_com_matician_bluerdroid_1example_1app_RustInitialization_test_1gatt(
    env: Env<'_>,
    _this: jobject,
    device: jobject,
) {
    let device: Ref<RustArcBoxDynAny> = unsafe { Ref::from_raw(env, device) };
    let device_ptr = device.getRust_ptr().unwrap();
    let device_owned = unsafe { get_ref(device_ptr) };
    RUNTIME.spawn(async move {
        let device_lock = device_owned.downcast_ref::<OnceLock<Device>>().unwrap();
        let Some(device) = device_lock.get() else {
            warn!("Device has not been discovered");
            return;
        };

        device.connect().await.unwrap();
        info!("Connected");

        let services = device.discover_services().await.unwrap();
        let service = services
            .iter()
            .find(|service| service.uuid() == SERVICE_UUID)
            .expect("Service should exist in the device");
        info!("Found service");

        let characteristic = service
            .characteristics()
            .into_iter()
            .find(|characteristic| characteristic.uuid() == CHARACTERISTIC_UUID)
            .expect("Characteristic should exist in the service");
        info!("Found characteristic with properties {:?}", characteristic.properties());

        let value = characteristic.read().await.unwrap();
        assert_eq!(
            CHARACTERISTIC_INITIAL_VALUE, &value as &[u8],
            "Characteristic does not have the expected initial value"
        );
        info!("Read characteristic");

        let mut notify = characteristic.notify().await.unwrap().skip(1);
        info!("Started notify");

        const NEW_VALUE: &[u8] = &[1, 2, 3, 4, 5];
        characteristic.write(Default::default(), NEW_VALUE).await.unwrap();
        info!("wrote new value");

        let new_value = notify.next().await.unwrap();
        assert_eq!(NEW_VALUE, &new_value as &[u8], "Characteristic set value is incorrect");
        info!("got notification for write");

        let descriptor = characteristic
            .descriptors()
            .into_iter()
            .find(|descriptor| descriptor.uuid() == DESCRIPTOR_UUID)
            .unwrap();
        let descriptor_value = descriptor.read().await.unwrap();
        info!("Read descriptor {descriptor_value:?}");
        assert_eq!(DESCRIPTOR_INITIAL_VALUE, &descriptor_value as &[u8]);

        let mut stream = device.open_l2cap_channel(PSM, false).unwrap();
        info!("Opened Stream");
        const SEND_DATA_1: &[u8] = &[100, 101, 102, 103, 104, 105];
        checked_stream_write(&mut stream, SEND_DATA_1).await;
        debug!("Send Data 1");
        const SEND_DATA_2: &[u8] = &[110, 111, 112, 113, 114, 115];
        checked_stream_write(&mut stream, SEND_DATA_2).await;
        debug!("Send Data 2");
        info!("Stream Data Checks completed");
        drop(stream);

        info!("Pairing");
        device.pair().await.unwrap();
        info!("Paired");

        let characteristic = service
            .characteristics()
            .into_iter()
            .find(|characteristic| characteristic.uuid() == PAIRED_CHARACTERISTIC_UUID)
            .expect("Characteristic should exist in the service");
        info!(
            "Found paired characteristic with properties {:?}",
            characteristic.properties()
        );
        let value = characteristic.read().await.unwrap();
        assert_eq!(
            PAIRED_CHARACTERISTIC_INITIAL_VALUE, &value as &[u8],
            "Characteristic does not have the expected initial value"
        );
        info!("Read paired characteristic done");
    });
}

async fn checked_stream_write(channel: &mut Channel, data: &[u8]) {
    trace!("Writing Data to stream");
    channel.write_all(data).await.unwrap();
    let mut recv_data = vec![0; data.len()];
    trace!("Reading data from stream");
    channel.read_exact(&mut recv_data).await.unwrap();
    assert_eq!(
        &recv_data,
        data.iter().map(|byte| !byte).collect::<Vec<u8>>().as_slice()
    )
}
