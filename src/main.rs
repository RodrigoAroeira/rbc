use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::path::PathBuf;

mod complex;
mod parsing;
mod token;

use complex::Complex;

const OHM: char = '\u{03A9}';

fn eval(line: &str) -> Result<Complex> {
    let tokens = token::tokenize(line)?;
    parsing::parse_tokens(&tokens)
}

fn get_history_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "rbc").map(|dirs| dirs.cache_dir().join("history"))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|s| !s.trim().is_empty())
        .collect();
    if args.is_empty() {
        let mut rl = DefaultEditor::new()?;
        let history_path = get_history_path();
        if let Some(ref path) = history_path {
            _ = rl.load_history(path);
        }

        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    _ = rl.add_history_entry(trimmed);
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

        if let Some(ref path) = history_path {
            if let Some(parent) = path.parent() {
                _ = std::fs::create_dir_all(parent);
            }
            _ = rl.save_history(path);
        }
    } else {
        let buf = args.join(" ");
        println!("{} {OHM}", eval(&buf)?);
    }
    Ok(())
}
