use clap::{Parser, Subcommand};

use crate::{
    commands::{XFixCommandDelegate, assign::XFixCommandAssign},
    config::XFixConfig,
};

mod commands;
mod config;

#[derive(Subcommand)]
enum XFixCommand {
    /// Assign touchscreens to outputs
    Assign(XFixCommandAssign),
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

    match args.command {
        XFixCommand::Assign(assign) => assign.run(&config),
    }?;

    Ok(())
}
