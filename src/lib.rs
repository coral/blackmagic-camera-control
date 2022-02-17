#[cfg(feature = "ble")]
pub mod blecamera;
#[cfg(feature = "ble")]
pub use blecamera::BluetoothCamera;

pub mod error;
pub mod rawcommand;

pub mod command {
    include!(concat!(env!("OUT_DIR"), "/command.rs"));
}

//Exports
pub use rawcommand::Operation;
