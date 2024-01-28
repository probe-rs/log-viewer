use std::path::PathBuf;

use clap::Parser;

use log_viewer::state::State;

#[derive(Debug, Parser)]
struct Opt {
    path: PathBuf,
}

fn main() {
    pretty_env_logger::init();

    let opt = Opt::parse();

    let content = std::fs::read_to_string(opt.path).expect("read file");

    let state = State::new(&content).unwrap();

    for event in &state.events {
        match event.fields.message.as_str() {
            "enter" | "exit" | "new" | "close" => {}
            _ => {
                println!("{}: {}", event.target, event.fields.message)
            }
        }
    }
}
