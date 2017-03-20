use std::ops::Range;
use std::slice::Iter;

use error::Result;
use commands::Command;
use tags::Tag;

mod parser;

#[derive(PartialEq)]
struct TagRange {
    key: Range<usize>,
    value: Range<usize>,
}

/// An implementation of Iterator that iterates over the arguments of a `Message`.
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

/// An implementation of Iterator that iterates over the key/value pairs 
/// (in the form of a tuple) of the tags of a `Message`.
pub struct TagIter<'a> {
    source: &'a str,
    iter: Iter<'a, TagRange>,
}

impl<'a> Iterator for TagIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|tag_range| {
            (&self.source[tag_range.key.clone()], &self.source[tag_range.value.clone()])
        })
    }
}

/// Representation of IRC messages that splits a message into its constituent
/// parts specified in RFC1459 and the IRCv3 spec.
pub struct Message {
    message: String,
    tags: Option<Vec<TagRange>>,
    prefix: Option<Range<usize>>,
    command: Range<usize>,
    arguments: Option<Vec<Range<usize>>>,
}

impl Message {
    /// Attempt to construct a new message from the given raw IRC message.
    pub fn try_from(value: String) -> Result<Message> {
        let result = parser::parse_message(value)?;

        Ok(result)
    }

    /// A strongly typed interface for determining the type of the command
    /// and retrieving the values of the command.
    pub fn command<'a, T: Command<'a>>(&'a self) -> Option<T> {
        <T as Command>::parse(self)
    }

    /// A strongly type way of accessing a specified tag associated with
    /// a message.
    pub fn tag<'a, T: Tag<'a>>(&'a self) -> Option<T> {
        for (key, value) in self.raw_tags() {
            if key == <T as Tag>::name() {
                return <T as Tag>::parse(value);
            }
        }

        None
    }

    /// Get an iterator to the raw key/value pairs of tags associated with
    /// this message.
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

    /// Attempt to get the raw prefix value associated with this message.
    pub fn raw_prefix(&self) -> Option<&str> {
        if let Some(ref prefix) = self.prefix {
            Some(&self.message[prefix.clone()])
        } else {
            None
        }
    }

    /// Retrieve the raw command associated with this message.
    pub fn raw_command(&self) -> &str {
        &self.message[self.command.clone()]
    }

    /// Get an iterator to the raw arguments associated with this message.
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

    /// Get the raw IRC command this message was constrcuted from.
    pub fn raw_message(&self) -> &str {
        &self.message
    }
}
