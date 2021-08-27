use crate::command::Command;
use crate::error::BluetoothCameraError;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ValueNotification};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;
use uuid::Uuid;

pub const CAMERA_MANUFACTURER: Uuid = Uuid::from_u128(855109558092022082745622393992443);
pub const CAMERA_MODEL: Uuid = Uuid::from_u128(854713417279450761057654674240763);
pub const OUTGOING_CAMERA_CONTROL: Uuid = Uuid::from_u128(124715205548830368390231916378743955899);
pub const INCOMING_CAMERA_CONTROL: Uuid = Uuid::from_u128(245101749559754194128926468485788547033);
pub const TIMECODE: Uuid = Uuid::from_u128(145629020620256484157652687441451644616);
pub const CAMERA_STATUS: Uuid = Uuid::from_u128(170018700332869099062316608707586904505);
pub const DEVICE_NAME: Uuid = Uuid::from_u128(339846463932956345205123112215954503836);
pub const PROTOCOL_VERSION: Uuid = Uuid::from_u128(190244785298557795456958317949635929862);

#[allow(dead_code)]
pub struct BluetoothCamera {
    name: String,

    bluetooth_manager: Manager,
    adapter: Adapter,

    device: Option<Peripheral>,

    write_char: Option<Characteristic>,
    read_char: Option<Characteristic>,
}

impl BluetoothCamera {
    pub async fn new(name: String) -> Result<BluetoothCamera, BluetoothCameraError> {
        let bluetooth_manager = Manager::new().await?;

        let adapter = bluetooth_manager.adapters().await?;

        let adapter = adapter
            .into_iter()
            .nth(0)
            .ok_or(BluetoothCameraError::NoBluetooth)?;

        Ok(BluetoothCamera {
            name,
            bluetooth_manager,
            adapter,
            device: None,

            write_char: None,
            read_char: None,
        })
    }

    pub async fn connect(&mut self, timeout: Duration) -> Result<(), BluetoothCameraError> {
        let now = time::Instant::now();
        self.adapter.start_scan().await?;

        loop {
            if now.elapsed().as_millis() > timeout.as_millis() {
                break;
            }

            match self.find_camera().await {
                Some(v) => {
                    v.connect().await?;
                    self.device = Some(v);

                    // TODO: fix this hack once the underlying bluetooth library supports
                    // actually reporting when the connection is "established".
                    // Right now on you crash the library if you try to write on OSX
                    // without waiting and the is_connected() endpoint is hardcoded
                    // to return "false" on OSX. Oh well, SLEEP it is.

                    time::sleep(Duration::from_millis(500)).await;

                    let device = self.device.as_ref().unwrap();

                    // Seed the characteristics list.
                    let char = device.discover_characteristics().await?;

                    let inc = char
                        .iter()
                        .find(|c| c.uuid == INCOMING_CAMERA_CONTROL)
                        .ok_or(BluetoothCameraError::NoCharacteristic)?;

                    self.read_char = Some(inc.to_owned());

                    let ouc = char
                        .iter()
                        .find(|c| c.uuid == OUTGOING_CAMERA_CONTROL)
                        .ok_or(BluetoothCameraError::NoCharacteristic)?;

                    self.write_char = Some(ouc.to_owned());

                    // Subscribe to Incoming Camera Control
                    device.subscribe(&self.read_char.as_ref().unwrap()).await?;
                    let mut stream = device.notifications().await?;

                    // let (tx, _) = broadcast::channel(16);

                    tokio::spawn(async move {
                        while let Some(data) = stream.next().await {
                            //dbg!(&data);
                            let cmd = Command::from_raw(&data.value);
                            match cmd {
                                Ok(v) => {
                                    println!("{:?}", v);
                                    //tx.send(v).unwrap();
                                }
                                Err(e) => {}
                            }
                        }
                    });

                    // tokio::spawn(async move {
                    //     let stream = stream;
                    //     BluetoothCamera::handle_incoming(stream).await;
                    // });

                    return Ok(());
                }
                None => {}
            };

            time::sleep(Duration::from_millis(50)).await;
        }

        Ok(())
    }

    async fn handle_incoming(mut stream: Pin<Box<dyn futures::Stream<Item = ValueNotification>>>) {
        while let Some(data) = stream.next().await {
            //dbg!(&data);
            let cmd = Command::from_raw(&data.value);
            match cmd {
                Ok(v) => println!("{:?}", v),
                Err(e) => {}
            }
        }
    }

    async fn find_camera(&self) -> Option<Peripheral> {
        for p in self.adapter.peripherals().await.unwrap() {
            if p.properties()
                .await
                .unwrap()
                .unwrap()
                .local_name
                .iter()
                .any(|name| name.contains(&self.name))
            {
                return Some(p);
            }
        }
        None
    }

    // async fn get_characteristic(&self, char: Uuid) -> Option<&Characteristic> {
    //     self.characteristics.iter().find(|c| c.uuid == char)
    // }
}
