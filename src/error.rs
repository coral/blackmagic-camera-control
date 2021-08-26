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

    #[error("Cannot resolve characteristic from protocol")]
    NoCharacteristicFromProtocol,

    #[error(transparent)]
    BTLEError(#[from] btleplug::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    UUIDError(#[from] uuid::Error),
}
