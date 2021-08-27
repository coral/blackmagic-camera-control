use crate::command::Command;
use fixed::types::I5F11;
use fixed::FixedI16;
use num_traits::cast::FromPrimitive;
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

#[derive(Debug, PartialEq)]
pub enum Operation {
    AssignValue,
    OffsetValue,
    Unknown,
}

impl Operation {
    pub fn from_u8(id: u8) -> Self {
        match id {
            0 => Operation::AssignValue,
            1 => Operation::OffsetValue,
            _ => Operation::Unknown,
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            Operation::AssignValue => 0,
            Operation::OffsetValue => 1,
            Operation::Unknown => 2,
        }
    }
}

pub trait Parameter {
    fn id(&self) -> u8;

    fn from_raw(cmd: RawCommand) -> Result<Self, CommandError>
    where
        Self: Sized;

    fn raw_type(&self) -> u8;

    fn to_bytes(&self) -> Vec<u8>;

    fn name(&self) -> String;
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

            data: data[8..8 + (data[1] - 4) as usize].to_vec(),
        })
    }

    pub fn to_raw(destination: u8, operation: Operation, cmd: &Command) -> Vec<u8> {
        let mut v = Vec::new();

        let mut data = cmd.to_bytes();

        //Destination
        v.push(destination);
        //Length
        v.push(data.len() as u8 + 4);
        //Command id
        v.push(0);
        //Reserved
        v.push(0);

        //Category
        v.push(cmd.id());
        //Paramter
        v.push(cmd.parameter_id());
        //Type
        v.push(cmd.raw_type());
        //Operation
        v.push(operation.id());

        //Data
        v.append(&mut data);

        v
    }
}

pub trait ParamType {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError>
    where
        Self: Sized;

    fn to_bytes(&self) -> Vec<u8>;
}

impl ParamType for String {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        Ok(String::from_utf8(data.to_vec())?)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl ParamType for u8 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.first()
            .map(|v| *v as u8)
            .ok_or(CommandError::NotEnoughBytes)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ParamType for i8 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.first()
            .map(|v| *v as i8)
            .ok_or(CommandError::NotEnoughBytes)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ParamType for i16 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(2)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| i16::from_le_bytes(x.try_into().unwrap()))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ParamType for i32 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(4)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| i32::from_le_bytes(x.try_into().unwrap()))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ParamType for i64 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(8)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| i64::from_le_bytes(x.try_into().unwrap()))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ParamType for f32 {
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        data.chunks_exact(8)
            .next()
            .ok_or(CommandError::NotEnoughBytes)
            .map(|x| f32::from(I5F11::from_le_bytes(x.try_into().unwrap())))
    }

    fn to_bytes(&self) -> Vec<u8> {
        I5F11::from_f32(*self).unwrap().to_le_bytes().to_vec()
    }
}

impl<T: ParamType> ParamType for Vec<T> {
    fn from_bytes(data: &[u8]) -> Result<Vec<T>, CommandError> {
        data.chunks_exact(std::mem::size_of::<T>())
            .map(<T as ParamType>::from_bytes)
            .collect()
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|x| x.to_bytes()).collect()
    }
}
