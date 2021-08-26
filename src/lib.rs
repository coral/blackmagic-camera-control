pub mod rawcommand;

pub mod command {
    include!(concat!(env!("OUT_DIR"), "/command.rs"));
}

pub mod bluetooth_camera;
pub mod error;
pub use bluetooth_camera::BluetoothCamera;
