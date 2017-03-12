use std::collections::HashMap;
use std::fmt::{Display, Formatter, Error};

pub mod parser;

#[derive(Debug)]
pub struct Message {
    pub command: String,
    pub tags: Option<HashMap<String, String>>,
    pub prefix: Option<String>,
    pub args: Option<Vec<String>>,
    pub suffix: Option<String>,
}

pub fn pass<S: Into<String>>(pass: S) -> Message {
    Message {
        tags: None,
        prefix: None,
        command: "PASS".into(),
        args: Some(vec![pass.into()]),
        suffix: None,
    }
}

pub fn nick<S: Into<String>>(nick: S) -> Message {
    Message {
        tags: None,
        prefix: None,
        command: "NICK".into(),
        args: Some(vec![nick.into()]),
        suffix: None,
    }
}

pub fn cap_req<S: Into<String>>(cap: S) -> Message {
    Message {
        tags: None,
        prefix: None,
        command: "CAP".into(),
        args: Some(vec!["REQ".into()]),
        suffix: Some(cap.into()),
    }
}

pub fn pong<S: Into<String>>(server: S) -> Message {
    Message {
        tags: None,
        prefix: None,
        command: "PONG".into(),
        args: None,
        suffix: Some(server.into()),
    }
}

pub fn join<S: Into<String>>(channel: S) -> Message {
    Message {
        tags: None,
        prefix: None,
        command: "JOIN".into(),
        args: Some(vec![channel.into()]),
        suffix: None,
    }
}

pub fn privmsg<S1: Into<String>, S2: Into<String>>(target: S1, message: S2) -> Message {
    Message {
        tags: None,
        prefix: None,
        command: "PRIVMSG".into(),
        args: Some(vec![target.into()]),
        suffix: Some(message.into()),
    }
}

impl Display for Message {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        if let Some(ref tags) = self.tags {
            let tags = tags.iter()
                .map(|(key, value)| format!("{}={}", key, value))
                .collect::<Vec<_>>()
                .join(";");

            write!(formatter, "@{} ", tags)?;
        }

        if let Some(ref prefix) = self.prefix {
            write!(formatter, ":{} ", prefix)?;
        }

        write!(formatter, "{}", self.command)?;

        if let Some(ref args) = self.args {
            write!(formatter, " {}", args.join(" "))?;
        }

        if let Some(ref suffix) = self.suffix {
            write!(formatter, " :{}", suffix)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatted_result_of_nick_should_match_expectation() {
        const EXPECTATION: &'static str = "NICK dankdroid";

        let nick = nick("dankdroid");
        let formatted_result = format!("{}", nick);

        assert_eq!(EXPECTATION, formatted_result);
    }

    #[test]
    fn formatted_result_of_pass_should_match_expectation() {
        const EXPECTATION: &'static str = "PASS abc123";

        let pass = pass("abc123");
        let formatted_result = format!("{}", pass);

        assert_eq!(EXPECTATION, formatted_result);
    }

    #[test]
    fn formatted_result_of_cap_req_should_match_expectation() {
        const EXPECTATION: &'static str = "CAP REQ :test.cap/tags";

        let cap_req = cap_req("test.cap/tags");
        let formatted_result = format!("{}", cap_req);

        assert_eq!(EXPECTATION, formatted_result);
    }

    #[test]
    fn formatted_result_of_pong_should_match_expectation() {
        const EXPECTATION: &'static str = "PONG :test.server";

        let pong = pong("test.server");
        let formatted_result = format!("{}", pong);

        assert_eq!(EXPECTATION, formatted_result);
    }

    #[test]
    fn formatted_result_of_join_should_match_expectation() {
        const EXPECTATION: &'static str = "JOIN #test";

        let join = join("#test");
        let formatted_result = format!("{}", join);

        assert_eq!(EXPECTATION, formatted_result);
    }

    #[test]
    fn formatted_result_of_privmsg_should_match_expectation() {
        const EXPECTATION: &'static str = "PRIVMSG #test :Memes for all!";

        let privmsg = privmsg("#test", "Memes for all!");
        let formatted_result = format!("{}", privmsg);

        assert_eq!(EXPECTATION, formatted_result);
    }
}
