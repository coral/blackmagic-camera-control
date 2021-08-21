use crate::error::CameraControlError;
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

pub struct BluetoothCamera {
    name: String,

    bluetooth_manager: Manager,
    adapter: Adapter,

    device: Option<Peripheral>,
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
        })
    }

    pub async fn connect(&self, timeout: Duration) -> Result<bool, Box<dyn Error>> {
        let now = time::Instant::now();
        self.adapter.start_scan().await?;

        loop {
            if now.elapsed().as_millis() > timeout.as_millis() {
                break;
            }

            match self.find_camera().await {
                Some(v) => {
                    println!("Connected");
                    break;
                }
                None => {}
            };
        }

        Ok(true)
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
}
