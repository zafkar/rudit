use anyhow::{bail, Result};
use clap::{arg, command, Command};
use config::Config;
use crossterm::{
    event::{self},
    terminal,
};
use editor::Editor;
use std::io::stdout;

mod buffer;
mod color;
mod config;
mod editor;
mod pos;

fn main() -> Result<()> {
    let matches = command!()
        .args([
            arg!([FILE] "The file to edit"),
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            .required(false),
        ])
        .subcommands([Command::new("config").about("Print the default config")])
        .get_matches();

    match matches.subcommand() {
        Some(("config", _)) => {
            println!("{}", toml::to_string(&Config::default())?)
        }
        None => {
            let mut editor = Editor::new();
            if let Some(path) = matches.get_one::<String>("config") {
                editor.set_config(path)?;
            }

            if let Some(path) = matches.get_one::<String>("FILE") {
                editor.set_document(path)?;
            }

            {
                let (width, height) = terminal::size()?;
                editor.set_size(width, height);
            }

            let mut stdout = stdout();
            terminal::enable_raw_mode()?;

            while !editor.is_done() {
                editor.process_event(event::read()?)?;
                editor.display(&mut stdout)?;
            }

            Editor::cleanup(&mut stdout)?;
            terminal::disable_raw_mode()?;
        }
        _ => bail!("Unknown subcommand"),
    }

    Ok(())
}
