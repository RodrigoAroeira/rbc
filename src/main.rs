mod complex;
mod parsing;
mod repl;
mod token;

use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|s| !s.trim().is_empty())
        .collect();

    if !args.is_empty() {
        let buf = args.join(" ");
        println!("{} {}", parsing::eval(&buf)?, repl::OHM);
        return Ok(());
    }

    repl::run()
}
