use std::collections::HashMap;
use super::Message;
use super::Command;
use super::super::error;
use std::str;

type ParseResult<'input, T> = error::Result<(T, &'input [u8])>;

pub fn parse_message(input: &[u8]) -> ParseResult<Message> {
    let (tags, input) = parse_tags(input)?;
    let (prefix, input) = parse_prefix(input)?;
    let (command, input) = parse_command(input)?;
    let (args, input) = parse_args(input)?;
    let (suffix, input) = parse_suffix(input)?;

    Ok((Message {
            tags: tags,
            prefix: prefix,
            command: command,
            args: args,
            suffix: suffix,
        },
        input))
}

fn move_next(value: usize, bound: usize) -> error::Result<usize> {
    let value = value + 1;

    if value >= bound {
        Err(error::ErrorKind::UnexpectedEndOfInput.into())
    } else {
        Ok(value)
    }
}

fn parse_tags(input: &[u8]) -> ParseResult<Option<HashMap<String, String>>> {
    if input.is_empty() {
        return Err(error::ErrorKind::UnexpectedEndOfInput.into());
    }

    if input[0] == b'@' {
        let len = input.len();
        let mut tags = HashMap::new();
        let mut remainder = move_next(0, len)?;

        loop {
            let key_start = remainder;
            while input[remainder] != b'=' {
                remainder = move_next(remainder, len)?;
            }

            let key = String::from_utf8(input[key_start..remainder].to_vec())?;

            remainder = move_next(remainder, len)?;

            let value_start = remainder;
            while input[remainder] != b';' && input[remainder] != b' ' {
                remainder = move_next(remainder, len)?;
            }

            let value = String::from_utf8(input[value_start..remainder].to_vec())?;

            tags.insert(key, value);

            if input[remainder] == b' ' {
                remainder = move_next(remainder, len)?;
                break;
            }

            remainder = move_next(remainder, len)?;
        }

        Ok((Some(tags), &input[remainder..]))
    } else {
        Ok((None, input))
    }
}

fn parse_prefix(input: &[u8]) -> ParseResult<Option<String>> {
    if input.is_empty() {
        return Err(error::ErrorKind::UnexpectedEndOfInput.into());
    }

    if input[0] == b':' {
        let len = input.len();
        let mut remainder = move_next(0, len)?;

        while input[remainder] != b' ' {
            remainder = move_next(remainder, len)?;
        }

        let prefix = String::from_utf8(input[1..remainder].to_vec())?;

        remainder = move_next(remainder, len)?;

        Ok((Some(prefix), &input[remainder..]))
    } else {
        Ok((None, input))
    }
}

fn parse_command(mut input: &[u8]) -> ParseResult<Command> {
    if input.is_empty() {
        return Err(error::ErrorKind::UnexpectedEndOfInput.into());
    }

    if input[0] == b' ' {
        input = &input[1..]
    }

    let mut remainder = 0;
    let len = input.len();

    while remainder < len && input[remainder] != b' ' {
        remainder += 1;
    }

    let command = match &*str::from_utf8(&input[0..remainder]).unwrap_or("").to_lowercase() {
        "pass" => Command::Pass,
        "nick" => Command::Nick,
        "user" => Command::User,
        c => Command::Other(c.into())
    };

    if remainder < len && input[remainder] == b' ' {
        remainder = move_next(remainder, len)?;
    }

    Ok((command, &input[remainder..]))
}

fn parse_args(input: &[u8]) -> ParseResult<Option<Vec<String>>> {
    if input.is_empty() {
        return Ok((None, input));
    }

    if input[0] == b':' {
        return Ok((None, input));
    }

    let mut args = Vec::new();
    let mut remainder = 0;
    let mut arg_start = 0;
    let len = input.len();

    loop {
        if input[remainder] == b':' {
            break;
        }

        if input[remainder] == b' ' {
            let arg = String::from_utf8(input[arg_start..remainder].to_vec())?;
            args.push(arg);

            arg_start = remainder + 1;
        }

        remainder += 1;

        if remainder >= len {
            let arg = String::from_utf8(input[arg_start..remainder].to_vec())?;
            args.push(arg);
            break;
        }
    }

    Ok((Some(args), &input[remainder..]))
}

fn parse_suffix(mut input: &[u8]) -> ParseResult<Option<String>> {
    if input.is_empty() {
        return Ok((None, input));
    }

    let len = input.len();

    if input[0] == b' ' {
        input = &input[1..];
    }

    if len >= 2 && input[0] == b':' {
        let suffix = String::from_utf8(input[1..len].to_vec())?;

        Ok((Some(suffix), &input[len..]))
    } else {
        Ok((None, input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Test this more thoroughly

    #[test]
    fn parsing_an_irc_message_with_just_command_should_give_the_command() {
        let (result, _) = parse_message("TEST".as_bytes()).unwrap();

        assert_eq!("TEST", result.command);
    }

    #[test]
    fn parsing_an_irc_message_with_prefix_should_give_the_prefix_and_command() {
        let (result, _) = parse_message(":test.server.com TEST".as_bytes()).unwrap();

        assert_eq!("test.server.com", result.prefix.unwrap());
        assert_eq!("TEST", result.command);
    }

    #[test]
    fn parsing_an_irc_message_with_suffix_should_give_the_suffix_and_command() {
        let (result, _) = parse_message("TEST :test.server.com".as_bytes()).unwrap();

        assert_eq!("TEST", result.command);
        assert_eq!("test.server.com", result.suffix.unwrap());
    }

    #[test]
    fn parsing_an_irc_message_with_prefix_and_suffix_should_give_the_prefix_suffix_and_command() {
        let (result, _) = parse_message(":other.server.com TEST :test.server.com".as_bytes())
            .unwrap();

        assert_eq!("other.server.com", result.prefix.unwrap());
        assert_eq!("TEST", result.command);
        assert_eq!("test.server.com", result.suffix.unwrap());
    }

    #[test]
    fn parsing_an_irc_message_with_arguments_should_give_the_command_and_arguments() {
        let (result, _) = parse_message("TEST a b c".as_bytes()).unwrap();

        assert_eq!("TEST", result.command);
        assert_eq!(vec!["a", "b", "c"], result.args.unwrap());
    }

    #[test]
    fn parsing_an_irc_message_with_arguments_and_suffix_should_give_the_command_suffix_and_arguments
        () {
        let (result, _) = parse_message("TEST a b c :Memes for all!".as_bytes()).unwrap();

        assert_eq!("TEST", result.command);
        assert_eq!(vec!["a", "b", "c"], result.args.unwrap());
        assert_eq!("Memes for all!", result.suffix.unwrap());
    }

    #[test]
    fn parsing_an_irc_message_with_tags_should_give_the_tags_and_command() {
        use std::collections::HashMap;

        let (result, _) = parse_message("@a=1;b=2;a\\b=3 TEST".as_bytes()).unwrap();

        let expected_tags: HashMap<String, String> = [("a", "1"), ("b", "2"), ("a\\b", "3")]
            .iter()
            .map(|&(key, value)| (key.into(), value.into()))
            .collect();

        assert_eq!("TEST", result.command);
        assert_eq!(expected_tags, result.tags.unwrap());
    }

    #[test]
    fn messages_containing_multibyte_characters_can_be_parsed() {
        let (result, _) = parse_message("TEST :💖".as_bytes()).unwrap();

        const EXPECTED_RESULT: &'static str = "💖";

        assert_eq!(EXPECTED_RESULT, result.suffix.unwrap());
    }
}
