use clap::{Parser, Subcommand};

use crate::{
    commands::{XFixCommandDelegate, assign::XFixCommandAssign, map::XFixCommandMap},
    config::XFixConfig,
};

mod commands;
mod config;
mod dev;

#[derive(Subcommand)]
enum XFixCommand {
    /// Assign touchscreens to outputs
    Assign(XFixCommandAssign),

    /// Map a new touchscreen to an output
    Map(XFixCommandMap),
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct TemptexArgs {
    #[clap(subcommand)]
    command: XFixCommand,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = TemptexArgs::parse();
    let config = XFixConfig::load()?;

    let updated_config = match args.command {
        XFixCommand::Assign(assign) => assign.run(&config),
        XFixCommand::Map(map) => map.run(&config),
    }?;

    if let Some(updated_config) = updated_config {
        updated_config.save()?;
    }

    Ok(())
}
