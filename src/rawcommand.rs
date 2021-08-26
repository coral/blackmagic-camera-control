use fixed::types::I5F11;
use std::convert::TryInto;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Message is too short")]
    MessageShort,

    #[error("Category not defined")]
    CategoryNotDefined,

    #[error("Parameter not defined")]
    ParameterNotDefined,

    #[error("Not Enough Bytes")]
    NotEnoughBytes,

    #[error(transparent)]
    UTF8Error(#[from] std::string::FromUtf8Error),
}

pub trait Parameter {
    fn id(self) -> u8;

    fn from_raw(cmd: RawCommand) -> Result<Self, CommandError>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct RawCommand {
    pub destination_device: u8,
    pub command_id: u8,
    pub category: u8,
    pub parameter: u8,
    pub data_type: u8,
    pub operation: u8,
    pub data: Vec<u8>,
}

impl RawCommand {
    pub fn from_raw(data: &[u8]) -> Result<Self, CommandError> {
        if data.len() < 8 {
            return Err(CommandError::MessageShort);
        }

        Ok(RawCommand {
            destination_device: data[0],
            command_id: data[2],
            category: data[4],
            parameter: data[5],
            data_type: data[6],
            operation: data[7],

            data: data[8..8 + data[1] as usize].to_vec(),
        })
    }
}

pub trait ParamType {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError>
    where
        Self: Sized;
}

impl ParamType for String {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        Ok(String::from_utf8(data.to_vec())?)
    }
}

impl ParamType for u8 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.first()
            .map(|v| *v as u8)
            .ok_or(CommandError::NotEnoughBytes)
    }
}

impl ParamType for i8 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.first()
            .map(|v| *v as i8)
            .ok_or(CommandError::NotEnoughBytes)
    }
}

impl ParamType for i16 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(2)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| i16::from_le_bytes(x.try_into().unwrap()))
    }
}

impl ParamType for i32 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(4)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| i32::from_le_bytes(x.try_into().unwrap()))
    }
}

impl ParamType for i64 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(8)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| i64::from_le_bytes(x.try_into().unwrap()))
    }
}

impl ParamType for f32 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(8)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| f32::from(I5F11::from_le_bytes(x.try_into().unwrap())))
    }
}

impl<T: ParamType> ParamType for Vec<T> {
    fn from_bytes(data: &[u8]) -> Result<Vec<T>, CommandError> {
        data.chunks_exact(std::mem::size_of::<T>())
            .map(<T as ParamType>::from_bytes)
            .collect()
    }
}
