use bytes::{BytesMut, BufMut};
use tokio_io::codec::{Encoder, Decoder};

use message::Message;

use std::io;
use std::io::Write;
use std::str;

use super::error::{Error, Result};

const DELIMETER_LENGTH: usize = 2;

pub struct IrcCodec;

impl Decoder for IrcCodec {
    type Item = Message;
    type Error = Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>> {
        if let Some(index) = buffer.iter().position(|&b| b == b'\n') {
            let command = buffer.split_to(index - 1);
            buffer.split_to(DELIMETER_LENGTH);

            Ok(Some(Message::try_from(String::from_utf8(command.to_vec())?)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for IrcCodec {    
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, message: Self::Item, buffer: &mut BytesMut) -> Result<()> {
        write!(buffer.writer(), "{}\r\n", message.raw_message())?;

        Ok(())
    }
}
