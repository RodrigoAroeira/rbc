use anyhow::Result;
use std::io::{self, Write};

mod parsing;
mod token;

const OHM: char = '\u{03A9}';

fn eval(line: &str) -> Result<f64> {
    let tokens = token::tokenize(line)?;
    parsing::parse_tokens(&tokens)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|s| !s.trim().is_empty())
        .collect();
    if args.is_empty() {
        let stdin = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut buf = String::new();
            if stdin.read_line(&mut buf)? == 0 {
                println!();
                break;
            }
            match eval(&buf) {
                Ok(ans) => println!("{ans} {OHM}"),
                Err(e) => println!("error: {e}"),
            }
        }
    } else {
        let buf = args.join(" ");
        println!("{} {OHM}", eval(&buf)?);
    }
    Ok(())
}
