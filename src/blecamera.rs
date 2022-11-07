use crate::command::Command;
use crate::error::BluetoothCameraError;
use crate::rawcommand::{Operation, RawCommand};
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, ValueNotification,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time;
use uuid::Uuid;

pub const CAMERA_SERVICE: Uuid = Uuid::from_u128(54650678423016196498641639054569411539);
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

    updates: Sender<Command>,
    cache: Arc<RwLock<HashMap<String, Command>>>,
}

impl BluetoothCamera {
    /// Takes the BLE name of the camera and returns a new BluetoothCamera instance
    ///
    /// # Arguments
    ///
    /// * `name` - &str representing the Bluetooth name of the camera such as "A:5CA7128B"
    pub async fn new(name: &str) -> Result<BluetoothCamera, BluetoothCameraError> {
        let bluetooth_manager = Manager::new().await?;

        let adapter = bluetooth_manager.adapters().await?;

        let adapter = adapter
            .into_iter()
            .nth(0)
            .ok_or(BluetoothCameraError::NoBluetooth)?;

        Ok(BluetoothCamera {
            name: name.to_string(),
            bluetooth_manager,
            adapter,
            device: None,

            write_char: None,
            read_char: None,

            updates: broadcast::channel(16).0,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Tries to connect to the camera, waiting as long as supplied timeout specifies
    ///
    /// # Arguments
    ///
    /// * `timeout` - std::Duration of how long to wait before giving up
    pub async fn connect(&mut self, timeout: Duration) -> Result<(), BluetoothCameraError> {
        let now = time::Instant::now();
        self.adapter
            .start_scan(ScanFilter {
                services: vec![CAMERA_SERVICE],
            })
            .await?;

        loop {
            // TODO Port this to TokioTimeout
            if now.elapsed().as_millis() > timeout.as_millis() {
                break;
            }

            match self.find_camera().await {
                Ok(v) => {
                    v.connect().await?;
                    self.device = Some(v);

                    // TODO: fix this hack once the underlying bluetooth library supports
                    // actually reporting when the connection is "established".
                    // Right now on you crash the library if you try to write on OSX
                    // without waiting and the is_connected() endpoint is hardcoded
                    // to return "false" on OSX. Oh well, SLEEP it is.

                    time::sleep(Duration::from_millis(500)).await;

                    let device = self
                        .device
                        .as_ref()
                        .ok_or(BluetoothCameraError::DevRefError)?;

                    // Seed the characteristics list.
                    device.discover_services().await?;

                    let char = device.characteristics();

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
                    device
                        .subscribe(
                            self.read_char
                                .as_ref()
                                .ok_or(BluetoothCameraError::DevRefError)?,
                        )
                        .await?;
                    let stream = device.notifications().await?;

                    let ble_cache = self.cache.clone();
                    let ble_updates = self.updates.clone();
                    tokio::spawn(async move {
                        BluetoothCamera::handle_incoming(ble_cache, ble_updates, stream).await;
                    });

                    return Ok(());
                }
                Err(_) => {}
            }

            time::sleep(Duration::from_millis(50)).await;
        }

        Err(BluetoothCameraError::ConnectError)
    }

    /// Disconnects from the camera
    ///
    /// NOTE: THIS ACTUALLY DOESN'T WORK ON OSX BECAUSE THE UNDERLYING LIBRARY IS PEPEGA
    pub async fn disconnect(&mut self) -> Result<(), BluetoothCameraError> {
        Ok(self
            .device
            .as_ref()
            .ok_or(BluetoothCameraError::DevRefError)?
            .disconnect()
            .await?)
    }

    pub async fn write(
        &mut self,
        destination: u8,
        operation: Operation,
        command: Command,
    ) -> Result<(), BluetoothCameraError> {
        let device = self
            .device
            .as_ref()
            .ok_or(BluetoothCameraError::SendError)?;

        device
            .write(
                self.write_char
                    .as_ref()
                    .ok_or(BluetoothCameraError::NoCharacteristic)?,
                &RawCommand::to_raw(destination, operation, &command),
                btleplug::api::WriteType::WithoutResponse,
            )
            .await?;

        Ok(())
    }

    async fn handle_incoming(
        cache: Arc<RwLock<HashMap<String, Command>>>,
        updates: Sender<Command>,
        mut stream: Pin<Box<dyn futures::Stream<Item = ValueNotification> + Send>>,
    ) {
        while let Some(data) = stream.next().await {
            let cmd = Command::from_raw(&data.value);
            match cmd {
                Ok(v) => {
                    let (cg, pr) = v.normalized_name();
                    cache
                        .write()
                        .await
                        .insert(format!("{}_{}", cg, pr), v.clone());
                    let _ = updates.send(v.clone());
                }
                Err(_) => {}
            }
        }
    }

    /// Gives you the latest cached version of the supplied Command
    /// If no cached version is found, returns the empty property
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command like this: Command::Metadata(Metadata::LensDistance("".to_string()))
    pub async fn get(&self, cmd: Command) -> Command {
        let (cg, pr) = cmd.normalized_name();
        match self
            .cache
            .clone()
            .read()
            .await
            .get(&format! {"{}_{}", &cg, &pr})
        {
            Some(c) => c.clone(),
            None => cmd,
        }
    }

    /// Gives you the latest cached version of the supplied normalized_name
    ///
    /// # Arguments
    ///
    /// * `normalized_name` - &str like this: metadata_lens_distance
    pub async fn get_normalized(&self, normalized_name: &str) -> Option<Command> {
        match self.cache.clone().read().await.get(normalized_name) {
            Some(c) => Some(c.clone()),
            None => None,
        }
    }

    /// Returns a channel which allows you to get updates from the camera
    pub async fn updates(&mut self) -> Receiver<Command> {
        self.updates.subscribe()
    }

    async fn find_camera(&self) -> Result<Peripheral, BluetoothCameraError> {
        for p in self.adapter.peripherals().await? {
            dbg!(&p);
            if p.properties()
                .await?
                .ok_or(BluetoothCameraError::DiscoveryError)?
                .local_name
                .iter()
                .any(|name| name.contains(&self.name))
            {
                return Ok(p);
            }
        }
        Err(BluetoothCameraError::CameraNotFound(self.name.to_string()))
    }
}
