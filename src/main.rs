mod command;
mod tags;

use anyhow::Result;
use clap::Parser;
use clap_complete_nushell::Nushell;

use crate::{
    command::{generate_completion, generate_man_page, validate_length_and_chars, Args, SubcommandVariants},
    tags::{check_db, drop_db, load_tags},
    tags::{dump_tags, generate_tags},
};

fn main() -> Result<()> {
    let args = Args::parse();
    match args.subcommand {
        Some(SubcommandVariants::Completions { shell }) => {
            generate_completion(shell);
            Ok(())
        }
        Some(SubcommandVariants::NuCompletions) => {
            generate_completion(Nushell);
            Ok(())
        }
        Some(SubcommandVariants::DumpTags) => dump_tags(),
        Some(SubcommandVariants::LoadTags { path }) => load_tags(path),
        Some(SubcommandVariants::CheckDb) => check_db(),
        Some(SubcommandVariants::DropDb) => drop_db(),
        Some(SubcommandVariants::GenManPage { dir }) => generate_man_page(dir),
        None => {
            validate_length_and_chars(&args);
            generate_tags(args.chars, args.length, args.amount)
        }
    }
}
