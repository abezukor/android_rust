use std::sync::Arc;

use futures_channel::mpsc::UnboundedReceiver;
use futures_core::Stream;
use futures_lite::StreamExt;
use java_spaghetti::{
    sys::{jobject, jobjectArray},
    Env, Global, Local, ObjectArray, Ref,
};
use log::{error, trace};
use rust_android_utilities::{get_application_context, java_wrapped_object::to_java, JavaError, JavaResult};
use thiserror::Error;

use crate::{
    bindings::{
        android::{bluetooth::BluetoothDevice as RawAndroidBluetoothDevice, content::Context},
        com::maticrobots::{
            rust_android_utilities::RustArcBoxDynAny,
            rust_bluedroid::{Adapter as JavaAdapter, BluetoothDevice as JavaBluetoothDevice, Service as JavaService},
        },
        java::lang::Throwable,
    },
    callback_mpsc_channel_send,
    error::GattResult,
    java_debug_eq_hash, CallBackFuture, CallBackFutureData, Channel, ConnectionState, GattError, Service,
};

type ReadRemoteRssiReturnValue = Result<i32, GattError>;
type DiscoverServicesFutureValue = Result<Vec<Global<JavaService>>, GattError>;
type ConnectionStateChannelData = GattResult<ConnectionState>;

#[derive(Clone)]
pub struct Device {
    device: DeviceWithGattLock,
    adapter: Global<JavaAdapter>,
}
java_debug_eq_hash!(Device, device.device);

#[derive(Clone)]
pub(crate) struct DeviceWithGattLock {
    pub(crate) device: Global<JavaBluetoothDevice>,
    pub(crate) gatt_lock: Arc<async_lock::Mutex<()>>,
}

#[derive(Error, Debug)]
pub enum PairingError {
    #[error("An error occured while pairing (e.g) user did not enter a pairing code")]
    PairingError,
    #[error("Unknown Pairing Error {0:}")]
    Unknown(i32),
    #[error(transparent)]
    JavaError(#[from] JavaError),
    #[error("Device Must be connected to be paired")]
    NotConnected,
}

impl Device {
    pub(crate) fn new(device: Global<JavaBluetoothDevice>, adapter: Global<JavaAdapter>) -> Self {
        Self {
            device: DeviceWithGattLock { device, gatt_lock: Default::default() },
            adapter,
        }
    }

    pub fn id(&self) -> String {
        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            let id = device.id().unwrap().unwrap();
            id.to_string().unwrap()
        })
    }

    pub fn name(&self) -> Option<String> {
        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            let name = device.name().unwrap()?;
            Some(name.to_string().unwrap())
        })
    }

    pub async fn connect(&self) -> GattResult<()> {
        let id = self.id();
        trace!("Connecting to {id:?}");
        if self.check_connected().is_ok() {
            return Ok(());
        }
        let mut connection_state = self.connection_events();
        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            if !device.connect()? {
                return Err(GattError::NotExecuted);
            }
            Ok::<_, GattError>(())
        })?;
        match connection_state
            .find(|cs| {
                trace!("Connect got state {cs:?} for {id}");
                matches!(cs, Ok(ConnectionState::Connected) | Err(_))
            })
            .await
        {
            Some(connection_result) => {
                trace!("Connection result {connection_result:?} for {id}");
                connection_result.map(|_| ())
            }
            None => Err(GattError::NotConnected),
        }
    }

    pub async fn disconnect(&self) -> GattResult<()> {
        let mut connection_state = self.connection_events();
        if matches!(
            self.client_connection_state(),
            ConnectionState::Disconnected | ConnectionState::Disconnecting
        ) {
            return Ok(());
        }

        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);

            device.disconnect()?;
            Ok::<_, GattError>(())
        })?;

        match connection_state
            .find(|cs| matches!(cs, Ok(ConnectionState::Disconnected) | Err(_)))
            .await
        {
            Some(connection_result) => connection_result.map(|_| ()),
            None => Err(GattError::NotExecuted),
        }
    }

    pub fn connection_events(&self) -> impl Stream<Item = GattResult<ConnectionState>> {
        let (state_send, state_recv) = futures_channel::mpsc::unbounded();
        let state_recv: UnboundedReceiver<ConnectionStateChannelData> = state_recv; // Enforce channel type
        state_send.unbounded_send(Ok(self.client_connection_state())).unwrap(); // Start the channel off with the current client connection state
        self.device.device.vm().with_env(|env| {
            let rust_obj: Local<RustArcBoxDynAny> = to_java(env, state_send).unwrap().cast().unwrap();

            let device = self.device.device.as_ref(env);

            device.connectionStateChange(rust_obj).unwrap();
        });
        state_recv
    }

    pub fn paired(&self) -> bool {
        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            device.isPaired().unwrap()
        })
    }

    pub async fn pair(&self) -> Result<(), PairingError> {
        if self.paired() {
            trace!("Device is already paired");
            return Ok(());
        }

        if self.check_connected().is_err() {
            return Err(PairingError::NotConnected);
        }

        let app_context = unsafe { get_application_context::<Context>() };
        let (event_send, mut event_recv) = futures_channel::mpsc::unbounded::<i32>();
        let _broadcast_receiver = self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            let context = app_context.as_ref(env);
            let rust_obj: Local<'_, RustArcBoxDynAny> = to_java(env, event_send)?.cast().unwrap();
            Ok::<_, JavaError>(device.pair(context, rust_obj)?.unwrap().as_global())
        })?;
        while let Some(event) = event_recv.next().await {
            match event {
                RawAndroidBluetoothDevice::BOND_BONDED => {
                    return Ok(());
                }
                RawAndroidBluetoothDevice::BOND_BONDING => {
                    trace!("Pairing in progress");
                }
                RawAndroidBluetoothDevice::BOND_NONE => return Err(PairingError::PairingError),
                code => return Err(PairingError::Unknown(code)),
            }
        }
        unreachable!("Stopped receiving event updates from pair, sender must have been dropped");
    }

    pub async fn discover_services(&self) -> Result<Vec<Service>, GattError> {
        self.check_connected()?;

        let finished = self.device.device.vm().with_env(|env| {
            let (rust_obj, future) = CallBackFuture::<DiscoverServicesFutureValue>::new(env);

            let device = self.device.device.as_ref(env);
            if !device.discoverServices(rust_obj)? {
                return Err(GattError::NotExecuted);
            }
            Ok(future)
        })?;
        let services = finished.await?;
        Ok(services
            .into_iter()
            .map(|service| Service { service, device: self.device.clone() })
            .collect())
    }

    /// Returns a list of GATT services offered by the remote device.
    ///
    /// This function requires that service discovery has been completed for the given device.
    pub fn cached_services(&self) -> JavaResult<Vec<Service>> {
        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            // Exception if gatt is uninitialized
            let services = device.cachedServices()?.unwrap();
            Ok(services
                .iter()
                .filter_map(|service| {
                    service
                        .as_ref()
                        .map(|service| Service { service: service.as_global(), device: self.device.clone() })
                })
                .collect())
        })
    }

    pub async fn rssi(&self) -> ReadRemoteRssiReturnValue {
        self.check_connected()?;

        let finished = self.device.device.vm().with_env(|env| {
            let (rust_obj, future) = CallBackFuture::<ReadRemoteRssiReturnValue>::new(env);

            let device = self.device.device.as_ref(env);
            if !device.rssi(rust_obj)? {
                return Err(GattError::NotExecuted);
            }
            Ok(future)
        })?;
        finished.await
    }

    pub fn open_l2cap_channel(&self, psm: u16, secure: bool) -> JavaResult<Channel> {
        let device = self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            Ok::<_, JavaError>(device.getDevice()?.unwrap().as_global())
        })?;
        Channel::open_l2cap_channel(device, psm, secure)
    }

    pub fn services_changed(&self) -> impl Stream<Item = ()> {
        let (services_changed_send, services_changed_recv) = futures_channel::mpsc::unbounded();
        self.device.device.vm().with_env(|env| {
            let rust_obj: Local<RustArcBoxDynAny> = to_java(env, services_changed_send).unwrap().cast().unwrap();

            let device = self.device.device.as_ref(env);

            device.services_changed(rust_obj).unwrap();
        });
        services_changed_recv
    }

    /// Global connection state. This indicates if the android device is connected to a client.
    /// For that client to be usable, you must also connect your gatt client.
    pub fn device_connection_state(&self) -> GattResult<ConnectionState> {
        let connection_state = self.adapter.vm().with_env(|env| {
            let adapter = self.adapter.as_ref(env);
            let device = self.device.device.as_ref(env);

            Ok::<_, JavaError>(adapter.connectionState(device)?)
        })?;

        Ok(ConnectionState::from_java(connection_state).expect("Value should be a connection state"))
    }

    /// GATT client (app) connection state.
    /// `self.device_connection_state() == true` is required but not sufficiant to imply that this GATT client is connected
    pub fn client_connection_state(&self) -> ConnectionState {
        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            ConnectionState::from_java(device.clientConnectionState().unwrap())
                .expect("Value should be a connection state")
        })
    }

    #[inline]
    fn check_connected(&self) -> GattResult<()> {
        matches!(self.client_connection_state(), ConnectionState::Connected)
            .then_some(())
            .ok_or(GattError::NotConnected)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.device.device.vm().with_env(|env| {
            let device = self.device.device.as_ref(env);
            device.close().unwrap();
        })
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnConnectionChangeState(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    status: i32,
    connection_state: i32,
) -> bool {
    let connection_state = match GattError::status_error(status) {
        Ok(()) => Ok(ConnectionState::from_java(connection_state).expect("Value should be a connection state")),
        Err(err) => Err(err),
    };

    trace!("Got connection state {connection_state:?}");

    callback_mpsc_channel_send::<ConnectionStateChannelData>(env, rust_obj, connection_state).is_ok()
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_PairingReceiver_rustPairingEvent(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    bond_state: i32,
) -> bool {
    match callback_mpsc_channel_send(env, rust_obj, bond_state) {
        Ok(()) => true,
        Err(e) if e.is_disconnected() => {
            trace!("Pairing receiver dropped, stopping");
            false
        }
        Err(e) => {
            unreachable!(
                "This is an unbounded Channel. The only valid error type is Disconnected, but i got {:?}",
                e
            );
        }
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnServicesDiscoveredCallback(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    services: jobjectArray,
    status: i32,
) {
    let return_object = GattError::status_error(status).map(|()| {
        let services: Ref<ObjectArray<JavaService, Throwable>> = unsafe { Ref::from_raw(env, services) };
        services
            .iter()
            .filter_map(|service| service.map(|service| service.as_global()))
            .collect()
    });
    unsafe {
        CallBackFutureData::<DiscoverServicesFutureValue>::wake(env, rust_obj, return_object);
    }
}

#[unsafe(no_mangle)]
extern "system" fn rustOnReadRemoteRssiCallback(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
    rssi: i32,
    status: i32,
) {
    unsafe {
        CallBackFutureData::<ReadRemoteRssiReturnValue>::wake(
            env,
            rust_obj,
            GattError::status_error(status).map(|()| rssi),
        );
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnServiceChangedCallback(
    env: Env<'_>,
    _this: jobject,
    rust_obj: jobject,
) -> bool {
    callback_mpsc_channel_send(env, rust_obj, ()).is_ok()
}
