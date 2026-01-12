use clap::Args;

use crate::{commands::XFixCommandDelegate, config::XFixConfig};

#[derive(Args)]
pub struct XFixCommandMap;

impl XFixCommandDelegate for XFixCommandMap {
    fn run(&self, _config: &XFixConfig) -> Result<Option<XFixConfig>, Box<dyn std::error::Error>> {
        println!("mapping...");

        Ok(None)
    }
}
