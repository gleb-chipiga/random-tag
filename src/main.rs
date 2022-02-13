use anyhow::{ensure, Result};
use clap::{crate_description, crate_version, Parser};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;

#[derive(clap::Parser, Debug)]
#[clap(version = crate_version!(), about = crate_description!())]
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
}

fn main() -> Result<()> {
    let args: Args = Parser::parse();
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
        println!("{tag}");
    }
    Ok(())
}
