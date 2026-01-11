use clap::{Parser, Subcommand};

use crate::{
    commands::{XFixCommandDelegate, fix::XFixCommandFix},
    config::XFixConfig,
};

mod commands;
mod config;

#[derive(Subcommand)]
enum XFixCommand {
    Fix(XFixCommandFix),
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
        XFixCommand::Fix(fix) => fix.run(&config),
    }?;

    Ok(())
}
