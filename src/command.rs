use std::{io, path::PathBuf};

use anyhow::Context;
use clap::{builder::TypedValueParser, error::ErrorKind, value_parser, CommandFactory};

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

fn parse_chars(proxy_str: &str) -> Result<Chars, String> {
    let value: Vec<char> = proxy_str.chars().collect();
    let has_digit = value.iter().any(|char| char.is_ascii_digit());
    let has_alphabetic = value.iter().any(|char| char.is_ascii_alphabetic());
    if !value.is_empty() && value.iter().all(|char| char.is_ascii_alphanumeric()) {
        let chars = Chars {
            value,
            has_digit,
            has_alphabetic,
        };
        Ok(chars)
    } else {
        Err("not all chars is ascii alphanumeric".to_string())
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
    Nushell,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum SubcommandVariants {
    /// Outputs the completion file for shell
    Completions {
        /// Shell type
        shell: CompletionShell,
    },
    /// Generate man pages
    ManPages {
        /// Directory to save man pages
        dir: PathBuf,
    },
    /// Dump used tags as CSV to stdout
    DumpTags,
    /// Load used tags from stdin or file in CSV format
    LoadTags {
        /// Path to CSV file with tags
        path: Option<PathBuf>,
    },
    /// Check used tags database
    CheckDb,
    /// Drop used tags database
    DropDb,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Args {
    /// Tag chars
    #[arg(short, long, default_value = "dfqsvz0123456789", value_parser = parse_chars)]
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

pub(crate) fn generate_completion(shell: CompletionShell) {
    let mut command = Args::command();
    let name = command.get_name().to_string();
    macro_rules! generate_completion {
        ($shell_type:expr) => {
            clap_complete::generate($shell_type, &mut command, name, &mut io::stdout())
        };
    }
    match shell {
        CompletionShell::Bash => generate_completion!(clap_complete::Shell::Bash),
        CompletionShell::Elvish => generate_completion!(clap_complete::Shell::Elvish),
        CompletionShell::Fish => generate_completion!(clap_complete::Shell::Fish),
        CompletionShell::PowerShell => generate_completion!(clap_complete::Shell::PowerShell),
        CompletionShell::Zsh => generate_completion!(clap_complete::Shell::Zsh),
        CompletionShell::Nushell => generate_completion!(clap_complete_nushell::Nushell),
    }
}

pub(crate) fn generate_man_page(dir: PathBuf) -> anyhow::Result<()> {
    let cmd = Args::command();
    clap_mangen::generate_to(cmd, dir).context("failed to generate man pages")?;
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
