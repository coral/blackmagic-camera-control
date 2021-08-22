use crate::error::CameraControlError;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::error::Error;
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
    pub async fn new(name: String) -> Result<BluetoothCamera, Box<dyn Error>> {
        let bluetooth_manager = Manager::new().await?;

        let adapter = bluetooth_manager
            .adapters()
            .await
            .expect("Unable to fetch adapter list.")
            .into_iter()
            .nth(0)
            .expect("Unable to find adapters.");

        Ok(BluetoothCamera {
            name,
            bluetooth_manager,
            adapter,
            device: None,
            characteristics: Vec::new(),

            outgoing_camera_uuid: Uuid::parse_str("5DD3465F-1AEE-4299-8493-D2ECA2F8E1BB").unwrap(),
            incoming_camera_uuid: Uuid::parse_str("B864E140-76A0-416A-BF30-5876504537D9").unwrap(),
            timecode_uuid: Uuid::parse_str("6D8F2110-86F1-41BF-9AFB-451D87E976C8").unwrap(),
            camera_status_uuid: Uuid::parse_str("7FE8691D-95DC-4FC5-8ABD-CA74339B51B9").unwrap(),
            device_name_uuid: Uuid::parse_str("FFAC0C52-C9FB-41A0-B063-CC76282EB89C").unwrap(),
            protocol_version_uuid: Uuid::parse_str("8F1FD018-B508-456F-8F82-3D392BEE2706").unwrap(),
        })
    }

    pub async fn connect(&mut self, timeout: Duration) -> Result<(), Box<dyn Error>> {
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

                    //TODO: fix this hack once the underlying bluetooth library supports
                    // actually reporting when the connection is "established".
                    // Right now on you crash the library if you try to write on OSX
                    // without waiting and the is_connected() endpoint is hardcoded
                    // to return "false" on OSX. Oh well, SLEEP it is.

                    time::sleep(Duration::from_millis(500)).await;

                    let device = self.device.as_ref().unwrap();

                    //Seed the characteristics list.
                    match device.discover_characteristics().await {
                        Ok(v) => {
                            self.characteristics = v;
                        }
                        Err(e) => return Err(Box::new(e)),
                    }

                    //Subscribe to Incoming Camera Control
                    match self.get_characteristic(self.incoming_camera_uuid).await {
                        Some(characteristic) => {
                            device.subscribe(characteristic).await?;
                            let mut stream = device.notifications().await?;

                            tokio::spawn(async move {
                                while let Some(data) = stream.next().await {
                                    dbg!(data);
                                }
                            });
                        }
                        None => {}
                    }

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
