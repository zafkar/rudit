use anyhow::{bail, Result};
use clap::{arg, command, Command};
use crossterm::{
    event::{self},
    terminal,
};
use rudit::{config::Config, editor::Editor};
use std::io::stdout;

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
            } else if let Some(home_path) = home::home_dir() {
                let config_path = home_path.join(".config/rudit.toml");
                if config_path.exists() {
                    editor.set_config(config_path)?;
                }
            }

            if let Some(path) = matches.get_one::<String>("FILE") {
                editor.set_document(path)?;
            }

            {
                editor.update_layout(terminal::size()?.into());
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
