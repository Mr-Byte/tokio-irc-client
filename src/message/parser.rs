use message::{Message, TagRange, PrefixRange};
use error;

use std::ops::Range;

type ParseResult<'input, T> = error::Result<(T, usize)>;

pub fn parse_message<M: Into<String>>(message: M) -> error::Result<Message> {
    let message = message.into();

    let (tags, prefix, command, args) = {
        let input = message.as_bytes();
        let (tags, position) = parse_tags(input)?;

        let tags_end = position;

        if tags_end >= 512 {
            return Err(error::ErrorKind::InputTooLong("The tags length exceeded 512 bytes.".to_owned()).into());
        }        

        let (prefix, position) = parse_prefix(input, position)?;
        let (command, position) = parse_command(input, position)?;
        let (args, position) = parse_args(input, position)?;

        if (position - tags_end) >= 512 {
            return Err(error::ErrorKind::InputTooLong("The message length exceeded 512 bytes.".to_owned()).into());
        }

        (tags, prefix, command, args)
    };

    Ok(Message {
        message: message,
        tags: tags,
        prefix: prefix,
        command: command,
        arguments: args,
    })
}

fn move_next(value: usize, bound: usize) -> error::Result<usize> {
    let value = value + 1;

    if value >= bound {
        Err(error::ErrorKind::UnexpectedEndOfInput.into())
    } else {
        Ok(value)
    }
}

fn parse_tags(input: &[u8]) -> ParseResult<Option<Vec<TagRange>>> {
    if input.is_empty() {
        return Err(error::ErrorKind::UnexpectedEndOfInput.into());
    }

    if input[0] == b'@' {
        let mut tags: Vec<TagRange> = Vec::new();
        let mut position = 1; // We can skip the @.
        let len = input.len();

        loop {
            let key_start = position;
            while input[position] != b'=' && input[position] != b';' {
                if input[position] == b' ' {
                    return Err(error::ErrorKind::UnexpectedEndOfInput.into());
                }

                position = move_next(position, len)?;
            }

            let key_range = key_start..position;
            if input[position] == b'=' {             
                position = move_next(position, len)?;
            }

            let value_start = position;
            while input[position] != b';' && input[position] != b' ' {
                position = move_next(position, len)?;
            }

            let value_range = if value_start == position { None } else { Some(value_start..position) };

            tags.push((key_range, value_range));

            if input[position] == b' ' {
                position = move_next(position, len)?;
                break;
            }

            position = move_next(position, len)?;
        }

        Ok((Some(tags), position))
    } else {
        Ok((None, 0))
    }
}

fn parse_prefix(input: &[u8], mut position: usize) -> ParseResult<Option<PrefixRange>> {
    let len = input.len();

    if position >= len {
        return Err(error::ErrorKind::UnexpectedEndOfInput.into());
    }

    if input[position] == b':' {
        position = move_next(position, len)?;
        let prefix_start = position;

        while input[position] != b' ' && input[position] != b'!' && input[position] != b'@' {
            position = move_next(position, len)?;
        }

        let prefix_range = prefix_start..position;

        let mut user_range = None;
        if input[position] == b'!' {
            position = move_next(position, len)?;
            let user_start = position;

            while input[position] != b' ' && input[position] != b'@' {
                position = move_next(position, len)?;
            }

            user_range = Some(user_start..position);
        }

        let mut host_range = None;
        if input[position] == b'@' {
            position = move_next(position, len)?;
            let host_start = position;

            while input[position] != b' ' {
                position = move_next(position, len)?;
            }

            host_range = Some(host_start..position);
        }        

        let prefix_range = PrefixRange {
            raw_prefix: prefix_start..position,
            prefix: prefix_range,
            user: user_range,
            host: host_range
        };

        position = move_next(position, len)?;

        Ok((Some(prefix_range), position))
    } else {
        Ok((None, position))
    }
}

fn parse_command(input: &[u8], mut position: usize) -> ParseResult<Range<usize>> {
    let len = input.len();
    if position >= len {
        return Err(error::ErrorKind::UnexpectedEndOfInput.into());
    }

    if input[0] == b' ' {
        position += 1
    }

    let command_start = position;

    while position < len && input[position] != b' ' {
        position += 1;
    }

    let command_range = command_start..position;

    if position < len && input[position] == b' ' {
        position = move_next(position, len)?;
    }

    Ok((command_range, position))
}

fn parse_args(input: &[u8], mut position: usize) -> ParseResult<Option<Vec<Range<usize>>>> {
    let len = input.len();

    if position >= len {
        return Ok((None, position));
    }

    let mut args = Vec::new();
    let mut arg_start = position;

    loop {
        if input[position] == b':' {
            position += 1;
            args.push(position..len);
            break;
        }

        if input[position] == b' ' {
            args.push(arg_start..position);

            arg_start = position + 1;
        }

        position += 1;

        if position >= len {
            args.push(arg_start..position);
            break;
        }
    }

    Ok((Some(args), position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command() {
        let result = parse_message("TEST").unwrap();

        assert_eq!(None, result.prefix());
        assert_eq!("TEST", result.raw_command());
    }

    #[test]
    fn parse_command_with_prefix() {
        let result = parse_message(":test.server.com TEST").unwrap();

        assert_eq!("test.server.com", result.raw_prefix().unwrap());
        assert_eq!("TEST", result.raw_command());
    }

    #[test]
    fn parse_command_with_argument_following_colon() {
        let result = parse_message("TEST :test.server.com").unwrap();

        let expected_args = vec!["test.server.com"];
        let actual_args: Vec<_> = result.raw_args().collect();

        assert_eq!("TEST", result.raw_command());
        assert_eq!(expected_args, actual_args);
    }

    #[test]
    fn parse_command_with_prefix_and_argument_following_colon() {
        let result = parse_message(":other.server.com TEST :test.server.com").unwrap();

        let expected_args = vec!["test.server.com"];
        let actual_args: Vec<_> = result.raw_args().collect();

        assert_eq!("other.server.com", result.raw_prefix().unwrap());
        assert_eq!("TEST", result.raw_command());
        assert_eq!(expected_args, actual_args);
    }

    #[test]
    fn parse_command_with_multiple_arguments() {
        let result = parse_message("TEST a b c").unwrap();

        let expected_args = vec!["a", "b", "c"];
        let actual_args: Vec<_> = result.raw_args().collect();

        assert_eq!("TEST", result.raw_command());
        assert_eq!(expected_args, actual_args);
    }

    #[test]
    fn parse_command_with_multiple_arguments_and_argument_following_colon() {
        let result = parse_message("TEST a b c :Memes for all!").unwrap();
        let expected_args = vec!["a", "b", "c", "Memes for all!"];
        let actual_args: Vec<_> = result.raw_args().collect();

        assert_eq!("TEST", result.raw_command());
        assert_eq!(expected_args, actual_args);
    }

    #[test]
    fn parse_command_with_multiple_tags() {
        let result = parse_message("@a=1;b=2;d=;f;a\\b=3;c= TEST").unwrap();

        let expected_tags = vec![
            ("a", Some("1")),
            ("b", Some("2")),
            ("d", None),
            ("f", None),
            ("a\\b", Some("3")),
            ("c", None)
        ];
        
        let actual_tags: Vec<_> = result.raw_tags().collect();

        assert_eq!("TEST", result.raw_command());
        assert_eq!(expected_tags, actual_tags);
    }

    #[test]
    fn parse_command_with_multibyte_character_arguments() {
        let result = parse_message("TEST :💖 Love 💖 Memes 💖").unwrap();

        let expected_args = vec!["💖 Love 💖 Memes 💖"];
        let actual_args: Vec<_> = result.raw_args().collect();

        assert_eq!(expected_args, actual_args);
    }

    #[test]
    fn parse_command_with_basic_prefix() {
        let result = parse_message(":foo TEST").unwrap();

        let prefix = result.prefix();

        assert_eq!(Some(("foo", None, None)), prefix);
    }

    #[test]
    fn parse_command_with_user_prefix() {
        let result = parse_message(":foo!foobert TEST").unwrap();

        let prefix = result.prefix();

        assert_eq!(Some(("foo", Some("foobert"), None)), prefix);
    }

    #[test]
    fn parse_command_with_user_prefix_and_host() {
        let result = parse_message(":foo!foobert@host.test.com TEST").unwrap();

        let prefix = result.prefix();

        assert_eq!(Some(("foo", Some("foobert"), Some("host.test.com"))), prefix);
    }

    #[test]
    fn parse_command_with_preefix_and_host() {
        let result = parse_message(":foo@host.test.com TEST").unwrap();

        let prefix = result.prefix();

        assert_eq!(Some(("foo", None, Some("host.test.com"))), prefix);
    }
}