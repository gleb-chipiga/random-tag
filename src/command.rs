use std::io;

use clap::{
    builder::{NonEmptyStringValueParser, TypedValueParser},
    error::{ContextKind, ContextValue, ErrorKind},
    value_parser, CommandFactory, Parser, Subcommand,
};
use clap_complete::{generate, Shell};

#[derive(Clone)]
struct NumberParser;

impl TypedValueParser for NumberParser {
    type Value = usize;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> anyhow::Result<Self::Value, clap::Error> {
        let value = value_parser!(u64)
            .range(1..128)
            .parse_ref(cmd, arg, value)?;
        Ok(value as usize)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Chars {
    pub(crate) value: Vec<char>,
    pub(crate) has_digit: bool,
    pub(crate) has_alphabetic: bool,
}

#[derive(Clone)]
struct CharsParser;

impl TypedValueParser for CharsParser {
    type Value = Chars;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> anyhow::Result<Self::Value, clap::Error> {
        let value = NonEmptyStringValueParser::new().parse_ref(cmd, arg, value)?;
        let chars: Vec<char> = value.chars().collect();
        let has_digit = chars.iter().any(|char| char.is_ascii_digit());
        let has_alphabetic = chars.iter().any(|char| char.is_ascii_alphabetic());
        if !chars.iter().all(|char| char.is_ascii_alphanumeric()) {
            let mut error = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
            if let Some(arg) = arg {
                error.insert(
                    ContextKind::InvalidArg,
                    ContextValue::String(arg.to_string()),
                );
            }
            error.insert(ContextKind::InvalidValue, ContextValue::String(value));
            Err(error)
        } else {
            Ok(Chars {
                value: chars,
                has_digit,
                has_alphabetic,
            })
        }
    }
}

#[derive(Subcommand, Debug)]
pub(crate) enum SubcommandVariants {
    /// Outputs the completion file for given shell
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Dump used tags as CSV
    UsedTags,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Args {
    /// Tag chars
    #[arg(short, long, default_value = "dfqsvz0123456789", value_parser = CharsParser)]
    pub(crate) chars: Chars,
    /// Tag length from 1 to 255
    #[arg(short, long, default_value = "6", value_parser = NumberParser)]
    pub(crate) length: usize,
    /// Tags amount from 1 to 255
    #[arg(short, long, default_value = "10", value_parser = NumberParser)]
    pub(crate) amount: usize,
    #[command(subcommand)]
    pub(crate) subcommand: Option<SubcommandVariants>,
}

pub(crate) fn generate_completion(shell: Shell) -> anyhow::Result<()> {
    let mut command = Args::command();
    let name = command.get_name().to_string();
    generate(shell, &mut command, name, &mut io::stdout());
    Ok(())
}

pub(crate) fn validate_length_and_chars(args: &Args) {
    if args.length == 1 && args.chars.has_digit && args.chars.has_alphabetic {
        Args::command()
            .error(
                ErrorKind::ArgumentConflict,
                "Tag length too short for alphanumeric chars",
            )
            .exit()
    }
}
