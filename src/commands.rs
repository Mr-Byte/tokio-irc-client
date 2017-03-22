use error::Result;
use message::Message;

pub trait Command<'a> {
    fn parse(message: &'a Message) -> Option<Self> where Self: Sized;
}

pub struct Ping<'a>(pub &'a str);

impl<'a> Command<'a> for Ping<'a> {
    fn parse(message: &'a Message) -> Option<Ping<'a>> {
        if message.raw_command() == "PING" {
            message.raw_args().next().map(|suffix| Ping(suffix))
        } else {
            None
        }
    }
}

pub fn ping<H: Into<String>>(host: H) -> Result<Message> {
    Message::try_from(format!("PING :{}", host.into()))
}

pub struct Pong<'a>(pub &'a str);

impl<'a> Command<'a> for Pong<'a> {
    fn parse(message: &'a Message) -> Option<Pong<'a>> {
        if message.raw_command() == "PONG" {
            message.raw_args().next().map(|suffix| Pong(suffix))
        } else {
            None
        }
    }
}

pub fn pong<H: Into<String>>(host: H) -> Result<Message> {
    Message::try_from(format!("PONG {}", host.into()))
}

pub fn pass<P: Into<String>>(pass: P) -> Result<Message> {
    Message::try_from(format!("PASS {}", pass.into()))
}

pub fn nick<N: Into<String>>(nick: N) -> Result<Message> {
    Message::try_from(format!("NICK {}", nick.into()))
}

pub fn user<U: Into<String>, N: Into<String>>(username: U, real_name: N) -> Result<Message> {
    Message::try_from(format!("USER {} 0 * :{}", username.into(), real_name.into()))
}

pub fn cap_req<C: Into<String>>(cap: C) -> Result<Message> {
    Message::try_from(format!("CAP REQ :{}", cap.into()))
}

pub fn join<C: Into<String>>(channel: C) -> Result<Message> {
    Message::try_from(format!("JOIN {}", channel.into()))
}

pub struct Privmsg<'a>(pub &'a str, pub &'a str);

impl<'a> Command<'a> for Privmsg<'a> {
    fn parse(message: &'a Message) -> Option<Privmsg<'a>> {
        if message.raw_command() == "PRIVMSG" {
            let mut args = message.raw_args();
            args.next().and_then(|target| args.next().map(|suffix| Privmsg(target, suffix)))
        } else {
            None
        }
    }
}

pub fn privmsg<T: Into<String>, M: Into<String>>(targets: T, message: M) -> Result<Message> {
    Message::try_from(format!("PRIVMSG {} :{}", targets.into(), message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_command() {
        let message = ping("test.host.com").unwrap();
        let Ping(host) = message.command::<Ping>().unwrap();

        assert_eq!("test.host.com", host);
    }

    #[test]
    fn test_pong_command() {
        let message = pong("test.host.com").unwrap();
        let Pong(host) = message.command::<Pong>().unwrap();

        assert_eq!("test.host.com", host);
    }

    #[test]
    fn test_privmsg_command() {
        let message = privmsg("#channel", "This is a message!").unwrap();
        let Privmsg(target, message) = message.command::<Privmsg>().unwrap();

        assert_eq!("#channel", target);
        assert_eq!("This is a message!", message);
    }
}