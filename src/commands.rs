use error::Result;
use message::Message;

pub trait Command<'a> {
    fn parse(message: &'a Message) -> Option<Self> where Self: Sized;
}

pub struct Ping<'a>(pub &'a str);

impl<'a> Command<'a> for Ping<'a> {
    fn parse(message: &'a Message) -> Option<Ping<'a>> {
        message.raw_args().next().map(|suffix| Ping(suffix))
    }
}

pub fn ping(host: &str) -> Result<Message> {
    Message::try_from(format!("PING :{}", host))
}

pub struct Pong<'a>(pub &'a str);

impl<'a> Command<'a> for Pong<'a> {
    fn parse(message: &'a Message) -> Option<Pong<'a>> {
        message.raw_args().next().map(|suffix| Pong(suffix))
    }
}

pub fn pong(host: &str) -> Result<Message> {
    Message::try_from(format!("PONG :{}", host))
}

// // TODO: Perhaps better explain what each of the mssages do and their usage
// // within IRC.

// /// Constructs a PASS message with the given password as an argument.
// pub fn pass<S: Into<String>>(pass: S) -> Message {
//     Message {
//         tags: None,
//         prefix: None,
//         command: "PASS".into(),
//         args: Some(vec![pass.into()]),
//         suffix: None,
//     }
// }

// /// Constructs a NICK message with the given nickname as an argument.
// pub fn nick<S: Into<String>>(nick: S) -> Message {
//     Message {
//         tags: None,
//         prefix: None,
//         command: "NICK".into(),
//         args: Some(vec![nick.into()]),
//         suffix: None,
//     }
// }

// /// Constructs a USER message from a username and a real name.
// pub fn user<S: Into<String>>(username: S, real_name: S) -> Message {
//     Message {
//         tags: None,
//         prefix: None,
//         command: "USER".into(),
//         args: Some(vec![username.into(), "0".into(), "*".into()]),
//         suffix: Some(real_name.into())
//     }
// }

// /// Constructs an IRCv3 CAP REQ messages which requests the specified
// /// capability.
// pub fn cap_req<S: Into<String>>(cap: S) -> Message {
//     Message {
//         tags: None,
//         prefix: None,
//         command: "CAP".into(),
//         args: Some(vec!["REQ".into()]),
//         suffix: Some(cap.into()),
//     }
// }

// /// Constructs a PONG message with the specified host.
// pub fn pong<S: Into<String>>(host: S) -> Message {
//     Message {
//         tags: None,
//         prefix: None,
//         command: "PONG".into(),
//         args: None,
//         suffix: Some(host.into()),
//     }
// }

// /// Constructs a JOIN message with the specified channel.
// pub fn join<S: Into<String>>(channel: S) -> Message {
//     Message {
//         tags: None,
//         prefix: None,
//         command: "JOIN".into(),
//         args: Some(vec![channel.into()]),
//         suffix: None,
//     }
// }

// // Constructs a PRIVMSG message for sending a message to the specified target.
// // The target is either another user or a channel.
// pub fn privmsg<S1: Into<String>, S2: Into<String>>(target: S1, message: S2) -> Message {
//     Message {
//         tags: None,
//         prefix: None,
//         command: "PRIVMSG".into(),
//         args: Some(vec![target.into()]),
//         suffix: Some(message.into()),
//     }
// }