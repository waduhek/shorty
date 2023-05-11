use lazy_static::lazy_static;
use regex::Regex;

const SHORTEN_COMMAND: &str = "shorten";
const LENGTHEN_COMMAND: &str = "lengthen";

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ShortyCommand {
    /// Command to lengthen the provided short ID. The variant stores the short
    /// ID provided by the user.
    Lengthen(String),
    /// Command to shorten the provided URL. The variant stores the full URL
    /// that the user wants to shorten.
    Shorten(String),
}

pub(super) struct ShortyArgs {
    /// The command to execute.
    pub command: ShortyCommand,
}

impl ShortyArgs {
    /// Checks if the provided test string is a valid URL.
    fn is_valid_url(test_string: &str) -> bool {
        lazy_static! {
            static ref URL_RE: Regex =
                Regex::new(r#"^https?://[^ "]+$"#).unwrap();
        };

        URL_RE.is_match(test_string)
    }

    pub fn build(
        mut arg_iter: impl Iterator<Item = String>,
    ) -> Result<Self, &'static str> {
        // Ignore the name of the executable.
        arg_iter.next();

        let command = match arg_iter.next() {
            Some(string) => string,
            None => return Err("command positional argument was not found"),
        };
        let command_arg = match arg_iter.next() {
            Some(string) => string,
            None => {
                return Err("command_arg positional argument was not found")
            }
        };

        match &command[..] {
            LENGTHEN_COMMAND => Ok(ShortyArgs {
                command: ShortyCommand::Lengthen(command_arg),
            }),
            SHORTEN_COMMAND => {
                if Self::is_valid_url(&command_arg) {
                    Ok(ShortyArgs {
                        command: ShortyCommand::Shorten(command_arg),
                    })
                } else {
                    Err("invalid URL format")
                }
            }
            _ => Err("invalid command"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SHORTY_EXEC: &str = "shorty";

    #[test]
    fn should_build_shorten_command() {
        let test_url = "https://example.com".to_string();
        let args = vec![
            SHORTY_EXEC.to_string(),
            SHORTEN_COMMAND.to_string(),
            test_url.clone(),
        ];
        let built_args = ShortyArgs::build(args.into_iter());

        assert!(built_args.is_ok());
        assert_eq!(
            built_args.unwrap().command,
            ShortyCommand::Shorten(test_url)
        );
    }

    #[test]
    fn should_build_lengthen_command() {
        let test_short_id = "abcdAbc123".to_string();
        let args = vec![
            SHORTY_EXEC.to_string(),
            LENGTHEN_COMMAND.to_string(),
            test_short_id.clone(),
        ];
        let built_args = ShortyArgs::build(args.into_iter());

        assert!(built_args.is_ok());
        assert_eq!(
            built_args.unwrap().command,
            ShortyCommand::Lengthen(test_short_id)
        );
    }

    #[test]
    fn should_require_an_argument_to_shorten() {
        let args = vec![SHORTY_EXEC.to_string(), SHORTEN_COMMAND.to_string()];
        let built_args = ShortyArgs::build(args.into_iter());

        assert!(built_args.is_err());
    }

    #[test]
    fn should_require_an_argument_to_lengthen() {
        let args = vec![SHORTY_EXEC.to_string(), LENGTHEN_COMMAND.to_string()];
        let built_args = ShortyArgs::build(args.into_iter());

        assert!(built_args.is_err());
    }

    #[test]
    fn should_not_shorten_invalid_url() {
        let args = vec![
            SHORTY_EXEC.to_string(),
            SHORTEN_COMMAND.to_string(),
            "https;//example.com".to_string(),
        ];
        let built_args = ShortyArgs::build(args.into_iter());

        assert!(built_args.is_err());
    }
}
