pub mod rawcommand;

pub mod command {
    include!(concat!(env!("OUT_DIR"), "/command.rs"));
}

pub mod bluetooth_camera;
pub mod error;
// pub mod message;

// pub use bluetooth_camera::BluetoothCamera;
// pub use message::Message;
