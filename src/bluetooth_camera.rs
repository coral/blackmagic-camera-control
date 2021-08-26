use crate::error::BluetoothCameraError;
use crate::protocol::BlackmagicCameraProtocol;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

#[allow(dead_code)]
pub struct BluetoothCamera {
    name: String,

    bluetooth_manager: Manager,
    adapter: Adapter,

    device: Option<Peripheral>,
    characteristics: Vec<Characteristic>,

    outgoing_camera_uuid: Uuid,
    incoming_camera_uuid: Uuid,
    timecode_uuid: Uuid,
    camera_status_uuid: Uuid,
    device_name_uuid: Uuid,
    protocol_version_uuid: Uuid,
}

impl BluetoothCamera {
    pub async fn new(name: String) -> Result<BluetoothCamera, BluetoothCameraError> {
        let protocol = BlackmagicCameraProtocol::new()?;

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
            characteristics: Vec::new(),

            outgoing_camera_uuid: protocol
                .pluck_characteristic("outgoing_camera_control")
                .ok_or(BluetoothCameraError::NoCharacteristicFromProtocol)?
                .uuid
                .clone(),
            incoming_camera_uuid: protocol
                .pluck_characteristic("incoming_camera_control")
                .ok_or(BluetoothCameraError::NoCharacteristicFromProtocol)?
                .uuid
                .clone(),
            timecode_uuid: protocol
                .pluck_characteristic("timecode")
                .ok_or(BluetoothCameraError::NoCharacteristicFromProtocol)?
                .uuid
                .clone(),
            camera_status_uuid: protocol
                .pluck_characteristic("camera_status")
                .ok_or(BluetoothCameraError::NoCharacteristicFromProtocol)?
                .uuid
                .clone(),
            device_name_uuid: protocol
                .pluck_characteristic("device_name")
                .ok_or(BluetoothCameraError::NoCharacteristicFromProtocol)?
                .uuid
                .clone(),
            protocol_version_uuid: protocol
                .pluck_characteristic("protocol_version")
                .ok_or(BluetoothCameraError::NoCharacteristicFromProtocol)?
                .uuid
                .clone(),
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
                    self.characteristics = device.discover_characteristics().await?;

                    // Subscribe to Incoming Camera Control
                    let characteristic = self
                        .get_characteristic(self.incoming_camera_uuid)
                        .await
                        .ok_or(BluetoothCameraError::NoCharacteristic)?;

                    device.subscribe(characteristic).await?;
                    let mut stream = device.notifications().await?;

                    tokio::spawn(async move {
                        while let Some(data) = stream.next().await {
                            dbg!(data);
                        }
                    });

                    return Ok(());
                }
                None => {}
            };

            time::sleep(Duration::from_millis(50)).await;
        }

        Ok(())
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

    async fn get_characteristic(&self, char: Uuid) -> Option<&Characteristic> {
        self.characteristics.iter().find(|c| c.uuid == char)
    }
}
