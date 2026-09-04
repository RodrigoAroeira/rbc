use anyhow::Result;
use std::io::{self, Write};

mod parsing;
mod token;

const OHM: char = '\u{03A9}';

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|s| !s.trim().is_empty())
        .collect();
    let mut buf = String::new();
    if args.is_empty() {
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;
    } else {
        buf = args.iter().as_ref().join(" ");
    }
    let tokens = token::tokenize(&buf)?;
    let ans = parsing::parse_tokens(&tokens)?;
    println!("{ans} {OHM}");
    Ok(())
}
