use anyhow::{ensure, Result};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use std::io;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Tag chars
    #[clap(short, long, default_value = "dfqsvz0123456789")]
    chars: String,
    /// Tag length from 1 to 255
    #[clap(short, long, default_value = "6")]
    length: u8,
    /// Tags amount from 1 to 255
    #[clap(short, long, default_value = "1")]
    amount: u8,
    /// Outputs the completion file for given shell
    #[clap(short, long, arg_enum)]
    shell: Option<Shell>,
}

fn main() -> Result<()> {
    let args: Args = Parser::parse();
    if let Some(shell) = args.shell {
        let mut command = Args::command();
        let name = command.get_name().to_string();
        generate(shell, &mut command, name, &mut io::stdout());
        return Ok(());
    }
    ensure!(
        !args.chars.is_empty(),
        "at least one character must be provided"
    );
    ensure!(args.length > 0, "length must be at least 1");
    ensure!(args.amount > 0, "amount must be at least 1");
    let chars: Vec<char> = args.chars.chars().collect();
    let mut rng = OsRng {};
    for _ in 0..args.amount {
        let tag: String = (0..args.length)
            .map(|_| chars.choose(&mut rng).unwrap())
            .collect();
        println!("{}", tag);
    }
    Ok(())
}
