use bytes::{BytesMut, BufMut};
use tokio_io::codec::{Encoder, Decoder};

use message::Message;
use message::parser;

use std::io;
use std::io::Write;
use std::str;

const DELIMETER_LENGTH: usize = 2;

pub struct IrcCodec;

impl Decoder for IrcCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if let Some(index) = buffer.iter().position(|&b| b == b'\n') {
            let command = buffer.split_to(index - 1);
            buffer.split_to(DELIMETER_LENGTH);

            let parse_result = parser::parse_message(&command);

            match parse_result {
                Ok((message, _)) => {
                    Ok(Some(message))
                }
                Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.description())),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for IrcCodec {    
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, message: Self::Item, buffer: &mut BytesMut) -> io::Result<()> {
        write!(buffer.writer(), "{}\r\n", message)?;

        Ok(())
    }
}
