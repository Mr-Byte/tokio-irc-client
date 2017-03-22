use std::ops::Range;
use std::slice::Iter;

/// An implementation of Iterator that iterates over the key/value pairs 
/// (in the form of a tuple) of the tags of a `Message`.
pub struct TagIter<'a> {
    source: &'a str,
    iter: Iter<'a, (Range<usize>, Option<Range<usize>>)>,
}

impl<'a> TagIter<'a> {
    // This is intended for internal usage and thus hidden.
    #[doc(hidden)]
    pub fn new(source: &'a str, iter: Iter<'a, (Range<usize>, Option<Range<usize>>)>) -> TagIter<'a> {
        TagIter {
            source: source,
            iter: iter
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = (&'a str, Option<&'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&(ref key, ref value)| {
            (&self.source[key.clone()], value.clone().map(|value| &self.source[value]))
        })
    }
}

pub trait Tag<'a> {
    fn name() -> &'static str;
    fn parse(tag: Option<&'a str>) -> Option<Self> where Self: Sized;

    fn try_match(tags: TagIter<'a>) -> Option<Self> where Self: Sized {
        for (key, value) in tags {
            if key == Self::name() {
                return Self::parse(value);
            }
        }

        None
    }
}