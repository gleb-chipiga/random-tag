mod command;
mod tags;

use anyhow::Result;
use clap::Parser;

use crate::{
    command::{generate_completion, validate_length_and_chars, Args, SubcommandVariants},
    tags::{generate_tags, used_tags},
};

fn main() -> Result<()> {
    let args = Args::parse();
    match args.subcommand {
        Some(SubcommandVariants::Completions { shell }) => generate_completion(shell),
        Some(SubcommandVariants::UsedTags) => used_tags(),
        None => {
            validate_length_and_chars(&args);
            generate_tags(args.chars, args.length, args.amount)
        }
    }
}
