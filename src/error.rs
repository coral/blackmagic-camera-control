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
