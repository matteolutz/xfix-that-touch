use clap::Args;

use crate::{
    commands::XFixCommandDelegate,
    config::XFixConfig,
    dev::{assign_screens_to_outputs, find_touchscreen_nodes, find_xinput_id},
};

#[derive(Args)]
pub struct XFixCommandAssign;

impl XFixCommandDelegate for XFixCommandAssign {
    fn run(&self, config: &XFixConfig) -> Result<Option<XFixConfig>, Box<dyn std::error::Error>> {
        let screens = find_touchscreen_nodes(&config.touchscreens)?;

        #[cfg(debug_assertions)]
        println!("[xfix] Screens with nodes: {:?}", screens);

        let screens = find_xinput_id(screens)?;

        #[cfg(debug_assertions)]
        println!("[xfix] Screens with xinput id: {:?}", screens);

        println!("[xfix] Assigning {} touchscreens", screens.len());

        assign_screens_to_outputs(screens)?;

        println!("[xfix] Done.");

        Ok(None)
    }
}
