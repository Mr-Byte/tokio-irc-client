use error::{Error, Result};
use message::{Command, Message};

pub struct Ping<'a>(pub &'a str);

impl<'a> Ping<'a> {
    pub fn new(host: &str) -> Result<Message> {
        Message::try_from(format!("PING :{}", host))
    }
}

impl<'a> Command<'a> for Ping<'a> {
    fn parse(message: &'a Message) -> Option<Ping<'a>> {
        panic!("Not implemented!")
    }
}

pub struct Pong<'a>(pub &'a str);

impl<'a> Pong<'a> {
    pub fn new(host: &str) -> Result<Message> {
        Message::try_from(format!("PONG: {}", host))
    }
}

impl<'a> Command<'a> for Pong<'a> {
    fn parse(message: &'a Message) -> Option<Pong<'a>> {
        panic!("Not implemented!");
    }
}