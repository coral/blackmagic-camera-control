use thiserror::Error;

#[derive(Error, Debug)]
pub enum CameraControlError {
    #[error("ParseError")]
    ParseError,
    #[error("CategoryOutOfRange")]
    CategoryOutOfRange,
    #[error("ConnectionTimeout")]
    ConnectionTimeout,
}

#[derive(Error, Debug)]
pub enum BluetoothCameraError {
    #[error("No Bluetooth adapter detected.")]
    NoBluetooth,

    #[error(
        "Could not find the right characteristic. Make sure you connected to the right device."
    )]
    NoCharacteristic,

    #[error(transparent)]
    BTLEError(#[from] btleplug::Error),
}
