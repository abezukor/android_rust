#[cfg(target_os = "linux")]
mod linux {
    use std::{pin::Pin, sync::Arc, time::Duration};

    use bluedroid_example_app::{
        self, DESCRIPTOR_INITIAL_VALUE, PAIRED_CHARACTERISTIC_INITIAL_VALUE, PSM,
    };
    use bluer::{
        Uuid,
        adv::Advertisement,
        agent::{Agent, DisplayPinCode},
        gatt::local::{
            Application, Characteristic, CharacteristicNotify, CharacteristicNotifyMethod,
            CharacteristicRead, CharacteristicWrite, CharacteristicWriteMethod, Descriptor,
            DescriptorRead, ReqResult, Service,
        },
        l2cap::{SocketAddr, StreamListener},
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    const SERVICE_UUID: Uuid = Uuid::from_u128_le(bluedroid_example_app::SERVICE_U128);
    const CHARACTERISTIC_UUID: Uuid =
        Uuid::from_u128_le(bluedroid_example_app::CHARACTERISTIC_U128);
    const DESCRIPTOR_UUID: Uuid = Uuid::from_u128_le(bluedroid_example_app::DESCRIPTOR_U128);
    const PAIRED_CHARACTERISTIC_UUID: Uuid =
        Uuid::from_u128_le(bluedroid_example_app::PAIRED_CHARACTERISTIC_U128);

    #[tokio::main]
    pub async fn main() -> bluer::Result<()> {
        env_logger::init();
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        println!(
            "Advertising on Bluetooth adapter {} with address {}",
            adapter.name(),
            adapter.address().await?
        );
        let le_advertisement = Advertisement {
            service_uuids: vec![SERVICE_UUID].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some("gatt_echo_server".to_string()),
            min_interval: Some(Duration::from_millis(200)),
            max_interval: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let _adv_handle = adapter.advertise(le_advertisement).await?;

        let (characteristic_data, _) = tokio::sync::watch::channel(
            bluedroid_example_app::CHARACTERISTIC_INITIAL_VALUE.to_vec(),
        );
        let characteristic_data = Arc::new(characteristic_data);

        println!(
            "Serving GATT echo service on Bluetooth adapter {}",
            adapter.name()
        );
        let app = Application {
            services: vec![Service {
                uuid: SERVICE_UUID,
                primary: true,
                characteristics: vec![
                    Characteristic {
                        uuid: CHARACTERISTIC_UUID,
                        write: Some(CharacteristicWrite {
                            write_without_response: true,
                            method: CharacteristicWriteMethod::Fun({
                                let characteristic_data = characteristic_data.clone();
                                Box::new(move |data, request| {
                                    log::debug!(
                                        "Got write request {:?}, new value {:?}",
                                        request,
                                        data
                                    );
                                    characteristic_data.send_replace(data);
                                    Box::pin(async { ReqResult::Ok(()) })
                                })
                            }),
                            ..Default::default()
                        }),
                        notify: Some(CharacteristicNotify {
                            notify: true,
                            method: CharacteristicNotifyMethod::Fun({
                                let recv = characteristic_data.subscribe();
                                Box::new(move |notification_request| {
                                    log::debug!("Got notification request");
                                    let mut recv = recv.clone();
                                    tokio::spawn(async move {
                                        let mut notification_request = notification_request;
                                        {
                                            let starting_value = recv.borrow_and_update().clone();
                                            if notification_request
                                                .notify(starting_value)
                                                .await
                                                .is_err()
                                            {
                                                return;
                                            };
                                        };
                                        loop {
                                            if recv.changed().await.is_err() {
                                                return;
                                            }
                                            let value = recv.borrow_and_update().clone();
                                            log::trace!(
                                                "Sending Notification with data {:?}",
                                                value
                                            );
                                            if notification_request.notify(value).await.is_err() {
                                                return;
                                            };
                                        }
                                    });
                                    Box::pin(async { () })
                                })
                            }),
                            ..Default::default()
                        }),
                        read: Some(CharacteristicRead {
                            read: true,
                            fun: {
                                let recv = characteristic_data.subscribe();
                                Box::new(move |request| {
                                    log::debug!("Got read request {:?}", request);
                                    let data = recv.borrow().clone();
                                    Box::pin(async { ReqResult::Ok(data) })
                                })
                            },
                            ..Default::default()
                        }),
                        descriptors: vec![Descriptor {
                            uuid: DESCRIPTOR_UUID,
                            read: Some(DescriptorRead {
                                read: true,
                                fun: Box::new(|descriptor_read_request| {
                                    log::debug!(
                                        "Got descriptor read request {:?}",
                                        descriptor_read_request
                                    );

                                    Box::pin(async { Ok(DESCRIPTOR_INITIAL_VALUE.to_vec()) })
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    Characteristic {
                        uuid: PAIRED_CHARACTERISTIC_UUID,
                        read: Some(CharacteristicRead {
                            read: true,
                            encrypt_read: true,
                            encrypt_authenticated_read: true,
                            secure_read: true,
                            fun: {
                                Box::new(move |_request| {
                                    Box::pin(async {
                                        ReqResult::Ok(PAIRED_CHARACTERISTIC_INITIAL_VALUE.to_vec())
                                    })
                                })
                            },
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        let _app_handle = adapter.serve_gatt_application(app).await?;

        let local_sa = SocketAddr::new(
            adapter.address().await.unwrap(),
            adapter.address_type().await.unwrap(),
            PSM,
        );
        let listener = StreamListener::bind(local_sa).await?;
        tokio::spawn(stream_handler(listener));

        let _agent_handle = session
            .register_agent(Agent {
                request_default: false,
                display_pin_code: Some(Box::new(display_pin_code)),
                ..Default::default()
            })
            .await
            .unwrap();

        std::future::pending::<()>().await;
        Ok(())
    }

    async fn stream_handler(listener: StreamListener) {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut stream = stream;
                loop {
                    let Ok(byte) = stream.read_u8().await else {
                        return;
                    };
                    if stream.write_u8(!byte).await.is_err() {
                        return;
                    }
                }
            });
        }
    }

    fn display_pin_code(
        code: DisplayPinCode,
    ) -> Pin<Box<dyn Future<Output = bluer::agent::ReqResult<()>> + Send>> {
        Box::pin(async move {
            println!("Pairing Code {}", code.pincode);
            Ok(())
        })
    }
}

#[cfg(target_os = "linux")]
use linux::main;

#[cfg(not(target_os = "linux"))]
fn main() {
    unimplemented!("Example does not work on non-linux os")
}
