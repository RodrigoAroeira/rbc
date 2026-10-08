use crate::parsing::eval;

use anyhow::Result;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Editor, Helper};

use std::borrow::Cow;
use std::path::PathBuf;

pub const OHM: char = '\u{03A9}';

fn get_history_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "rbc").map(|dirs| dirs.cache_dir().join("history"))
}

#[derive(Default)]
struct RbcHelper;
struct PreviewHint(String);

impl Hint for PreviewHint {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        None
    }
}

impl Completer for RbcHelper {
    type Candidate = String;
}

impl Hinter for RbcHelper {
    type Hint = PreviewHint;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        eval(trimmed)
            .ok()
            .map(|ans| format!("\n{ans} {OHM}"))
            .map(PreviewHint)
    }
}

impl Highlighter for RbcHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{hint}\x1b[0m"))
    }
}

impl Validator for RbcHelper {}

impl Helper for RbcHelper {}

fn get_editor() -> Result<Editor<RbcHelper, DefaultHistory>> {
    let mut rl = Editor::<RbcHelper, DefaultHistory>::new()?;
    rl.set_helper(Some(RbcHelper));
    let history_path = get_history_path();
    if let Some(ref path) = history_path {
        _ = rl.load_history(path);
    }
    Ok(rl)
}

pub fn run() -> Result<()> {
    let mut rl = get_editor()?;
    loop {
        match rl.readline("> ") {
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

    if let Some(ref path) = get_history_path() {
        if let Some(parent) = path.parent() {
            _ = std::fs::create_dir_all(parent);
        }
        _ = rl.save_history(path);
    }
    Ok(())
}
