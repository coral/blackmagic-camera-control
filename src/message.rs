use crate::data::{Category, Data, Operation};
use crate::error::CameraControlError;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use num::ToPrimitive;
use std::u8;

#[derive(Debug)]
pub struct Message {
    destination_device: u8,
    length: u8,
    id: u8,
    pub header: CommandHeader,
    pub data: Data,
}

#[derive(Debug)]
pub struct CommandHeader {
    pub category: Category,
    parameter: u8,
    cmd_type: u8,
    pub operation: u8,
}

impl Message {
    pub fn parse_message(message: Vec<u8>) -> Result<Message, CameraControlError> {
        let mut b = Bytes::from(message);
        let destination_device = b.get_u8();
        let length = b.get_u8();
        let id = b.get_u8();
        let _reserved = b.get_u8();

        if length < 4 {
            return Err(CameraControlError::ParseError);
        }

        let category = b.get_u8();
        let parameter = b.get_u8();
        let cmd_type = b.get_u8();
        let operation = b.get_u8();

        let header = CommandHeader {
            category: Category::lookup(category, parameter),
            parameter,
            cmd_type,
            operation,
        };

        let mut buf = b.take(length as usize);
        let mut data = vec![];

        data.put(&mut buf);

        let data = Data::decode(length, cmd_type, data);

        Ok(Message {
            destination_device,
            length,
            id,
            header,
            data,
        })
    }

    pub fn create_message(
        destination: u8,
        category: Category,
        operation: Operation,
        data: Data,
    ) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(64);
        let raw = data.to_bytes();

        //Destination
        buf.put_u8(destination);
        //Length
        buf.put_u8(4 + raw.len() as u8);
        //Command id
        buf.put_u8(0);
        //Reserved
        buf.put_u8(0);

        //Category
        buf.put_u8(category.get_category_id());
        //Parameter
        buf.put_u8(category.get_parameter_id());
        //Type
        buf.put_u8(data.get_type());
        //Operation
        buf.put_u8(operation.to_u8().unwrap());

        //Data
        buf.extend_from_slice(&raw);

        buf.to_vec()
    }
}
