use std::ops::Range;
use std::slice::Iter;

/// An implementation of Iterator that iterates over the arguments of a `Message`.
pub struct ArgumentIter<'a> {
    source: &'a str,
    iter: Iter<'a, Range<usize>>,
}

impl<'a> ArgumentIter<'a> {
    // This is intended for internal usage and thus hidden.
    #[doc(hidden)]
    pub fn new(source: &'a str, iter: Iter<'a, Range<usize>>) -> ArgumentIter<'a> {
        ArgumentIter {
            source: source,
            iter: iter
        }
    }
}

impl<'a> Iterator for ArgumentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|range| &self.source[range.clone()])
    }
}

pub trait Command<'a> {
    fn name() -> &'static str;
    fn parse(arguments: ArgumentIter<'a>) -> Option<Self> where Self: Sized;

    fn try_match(command: &str, arguments: ArgumentIter<'a>) -> Option<Self> where Self: Sized {
        if command == Self::name() {
            Self::parse(arguments)
        } else {
            None
        }
    }
}

pub struct Ping<'a>(pub &'a str);

impl<'a> Command<'a> for Ping<'a> {
    fn name() -> &'static str {
        "PING"
    }

    fn parse(mut arguments: ArgumentIter<'a>) -> Option<Ping<'a>> {
        arguments.next().map(|suffix| Ping(suffix))
    }
}

pub struct Pong<'a>(pub &'a str);

impl<'a> Command<'a> for Pong<'a> {
    fn name() -> &'static str {
        "PONG"
    }

    fn parse(mut arguments: ArgumentIter<'a>) -> Option<Pong<'a>> {
        arguments.next().map(|suffix| Pong(suffix))
    }
}

pub struct Privmsg<'a>(pub &'a str, pub &'a str);

impl<'a> Command<'a> for Privmsg<'a> {
    fn name() -> &'static str {
        "PRIVMSG"
    }

    fn parse(mut arguments: ArgumentIter<'a>) -> Option<Privmsg<'a>> {
        arguments.next().and_then(|target| arguments.next().map(|suffix| Privmsg(target, suffix)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use message::*;

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
