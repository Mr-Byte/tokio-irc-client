use std::collections::HashMap;
use std::ops::Range;
use std::slice::Iter;

use super::error::{Error, Result};

mod parser;
pub mod commands;

#[derive(PartialEq)]
struct TagRange {
    key: Range<usize>,
    value: Range<usize>,
}

pub struct Message {
    message: String,
    tags: Option<Vec<TagRange>>,
    prefix: Option<Range<usize>>,
    command: Range<usize>,
    arguments: Option<Vec<Range<usize>>>,
}

pub struct ArgumentIter<'a> {
    source: &'a str,
    iter: Iter<'a, Range<usize>>,
}

impl<'a> Iterator for ArgumentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|range| &self.source[range.clone()])
    }
}

pub struct TagIter<'a> {
    source: &'a str,
    iter: Iter<'a, TagRange>,
}

impl<'a> Iterator for TagIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|tag_range| {
                                 (&self.source[tag_range.key.clone()],
                                  &self.source[tag_range.value.clone()])
                             })
    }
}

pub trait Command<'a> {
    fn parse(message: &'a Message) -> Option<Self> where Self: Sized;
}

pub trait Tag<'a> {
    fn name() -> &'static str;
    fn parse(tag: &'a str) -> Option<Self> where Self: Sized;
}

impl Message {
    pub fn try_from(value: String) -> Result<Message> {
        let result = parser::parse_message(value)?;

        Ok(result)
    }

    pub fn command<'a, T: Command<'a>>(&'a self) -> Option<T> {
        <T as Command>::parse(self)
    }
    
    pub fn tag<'a, T: Tag<'a>>(&'a self) -> Option<T> {
        for (key, value) in self.raw_tags() {
            if key == <T as Tag>::name() {
                return <T as Tag>::parse(value);
            }
        }
        
        None
    }

    pub fn raw_tags(&self) -> TagIter {
        if let Some(ref tags) = self.tags {
            TagIter {
                source: &self.message,
                iter: tags.iter(),
            }
        } else {
            TagIter {
                source: &self.message,
                iter: [].iter(),
            }
        }
    }

    pub fn raw_prefix(&self) -> Option<&str> {
        if let Some(ref prefix) = self.prefix {
            Some(&self.message[prefix.clone()])
        } else {
            None
        }
    }

    pub fn raw_command(&self) -> &str {
        &self.message[self.command.clone()]
    }

    pub fn raw_args(&self) -> ArgumentIter {
        if let Some(ref arguments) = self.arguments {
            ArgumentIter {
                source: &self.message,
                iter: arguments.iter(),
            }
        } else {
            ArgumentIter {
                source: &self.message,
                iter: [].iter(),
            }
        }
    }

    pub fn raw_message(&self) -> &str {
        &self.message
    }
}

// /// Representation of IRC messages that splits a message into its constituent
// /// parts specified in RFC1459 and the IRCv3 spec.
// ///
// /// `command` represents the command being sent or received by the client.
// ///
// /// `tags` is an IRCv3 extension that provides a set of key/value pairs of
// /// metadata that can be associated with a message.
// ///
// /// `prefix` Is typically only populated by the server and indicates the origin
// /// of the message.
// ///
// /// `args` represents a collection of arguments associated with the command.
// ///
// /// `suffix` represents a suffix appended to a message, typically everything
// /// trailing the first `:` to follow the `command`. This is usually where
// /// information such as the message body of a PRIVMSG command is stored.
// #[derive(Debug)]
// pub struct Message {
//     pub command: String,
//     pub tags: Option<HashMap<String, String>>,
//     pub prefix: Option<String>,
//     pub args: Option<Vec<String>>,
//     pub suffix: Option<String>,
// }

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

// // Implementation of Display for `Message` that converts the `Message` into an
// // IRC command to be sent to the server. This does not include the required
// // `\r\n` that must be appended to the end of a message. The underlying IRC
// // codec will handle the conversion of messages and appending of the `\r\n`
// // so typically a consumer of this library will not need to call this directly.
// // Although it can be helpful for debugging purposes by allowing the consumer
// // to see what the IRC message being sent to the server looks like in string
// // format.
// impl Display for Message {
//     fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
//         if let Some(ref tags) = self.tags {
//             let tags = tags.iter()
//                 .map(|(key, value)| format!("{}={}", key, value))
//                 .collect::<Vec<_>>()
//                 .join(";");

//             write!(formatter, "@{} ", tags)?;
//         }

//         if let Some(ref prefix) = self.prefix {
//             write!(formatter, ":{} ", prefix)?;
//         }

//         write!(formatter, "{}", self.command)?;

//         if let Some(ref args) = self.args {
//             write!(formatter, " {}", args.join(" "))?;
//         }

//         if let Some(ref suffix) = self.suffix {
//             write!(formatter, " :{}", suffix)?;
//         }

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn formatted_result_of_nick_should_match_expectation() {
//         const EXPECTATION: &'static str = "NICK dankdroid";

//         let nick = nick("dankdroid");
//         let formatted_result = format!("{}", nick);

//         assert_eq!(EXPECTATION, formatted_result);
//     }

//     #[test]
//     fn formatted_result_of_pass_should_match_expectation() {
//         const EXPECTATION: &'static str = "PASS abc123";

//         let pass = pass("abc123");
//         let formatted_result = format!("{}", pass);

//         assert_eq!(EXPECTATION, formatted_result);
//     }

//     #[test]
//     fn formatted_result_of_cap_req_should_match_expectation() {
//         const EXPECTATION: &'static str = "CAP REQ :test.cap/tags";

//         let cap_req = cap_req("test.cap/tags");
//         let formatted_result = format!("{}", cap_req);

//         assert_eq!(EXPECTATION, formatted_result);
//     }

//     #[test]
//     fn formatted_result_of_pong_should_match_expectation() {
//         const EXPECTATION: &'static str = "PONG :test.server";

//         let pong = pong("test.server");
//         let formatted_result = format!("{}", pong);

//         assert_eq!(EXPECTATION, formatted_result);
//     }

//     #[test]
//     fn formatted_result_of_join_should_match_expectation() {
//         const EXPECTATION: &'static str = "JOIN #test";

//         let join = join("#test");
//         let formatted_result = format!("{}", join);

//         assert_eq!(EXPECTATION, formatted_result);
//     }

//     #[test]
//     fn formatted_result_of_privmsg_should_match_expectation() {
//         const EXPECTATION: &'static str = "PRIVMSG #test :Memes for all!";

//         let privmsg = privmsg("#test", "Memes for all!");
//         let formatted_result = format!("{}", privmsg);

//         assert_eq!(EXPECTATION, formatted_result);
//     }
// }
