use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

mod complex;
mod parsing;
mod token;

use complex::Complex;

const OHM: char = '\u{03A9}';

fn eval(line: &str) -> Result<Complex> {
    let tokens = token::tokenize(line)?;
    parsing::parse_tokens(&tokens)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|s| !s.trim().is_empty())
        .collect();
    if args.is_empty() {
        let mut rl = DefaultEditor::new()?;
        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match eval(trimmed) {
                        Ok(ans) => println!("{ans} {OHM}"),
                        Err(e) => println!("error: {e}"),
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("error: {err}");
                    break;
                }
            }
        }
    } else {
        let buf = args.join(" ");
        println!("{} {OHM}", eval(&buf)?);
    }
    Ok(())
}
