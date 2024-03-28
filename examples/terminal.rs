use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

use log_viewer::state::State;
use miette::IntoDiagnostic;

#[derive(Debug, Parser)]
struct Opt {
    path: PathBuf,
}

fn main() -> miette::Result<()> {
    pretty_env_logger::init();

    let opt = Opt::parse();

    let content = std::fs::read_to_string(opt.path).expect("read file");

    let state = State::new(&content).unwrap();

    let mut stdout = std::io::stdout().lock();

    for event in &state.events {
        match event.fields.message.as_str() {
            "enter" | "exit" | "new" | "close" => {}
            _ => {
                writeln!(stdout, "{}: {}", event.target, event.fields.message).into_diagnostic()?;
            }
        }
    }

    Ok(())
}
