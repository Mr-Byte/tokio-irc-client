use tokio_core::io::{Codec, EasyBuf};

use message::Message;
use message::parser;

use std::io;
use std::io::Write;
use std::str;

const DELIMETER_LENGTH: usize = 2;

pub struct IrcCodec;

impl Codec for IrcCodec {
    type In = Message;
    type Out = Message;

    fn decode(&mut self, buffer: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        if let Some(index) = buffer.as_slice().iter().position(|&b| b == b'\n') {
            let command = buffer.drain_to(index - 1);
            buffer.drain_to(DELIMETER_LENGTH);

            let parse_result = parser::parse_message(command.as_slice());

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

    fn encode(&mut self, message: Self::Out, buffer: &mut Vec<u8>) -> io::Result<()> {
        write!(buffer, "{}\r\n", message)?;

        Ok(())
    }
}
