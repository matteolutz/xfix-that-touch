use clap::Args;
use dialoguer::{
    Select,
    theme::{ColorfulTheme, SimpleTheme, Theme},
};

use crate::{
    commands::{XFixCommandDelegate, assign::XFixCommandAssign},
    config::XFixConfig,
    dev::{find_all_touchscreens_nodes, find_connected_video_outputs},
};

#[derive(Args)]
pub struct XFixCommandMap {
    /// Disable color output
    #[arg(short, long)]
    no_color: bool,
}

impl XFixCommandDelegate for XFixCommandMap {
    fn run(&self, config: &XFixConfig) -> Result<Option<XFixConfig>, Box<dyn std::error::Error>> {
        println!("[xfix] Searching for touchscreens...");

        let theme: Box<dyn Theme> = if self.no_color {
            Box::new(SimpleTheme)
        } else {
            Box::new(ColorfulTheme::default())
        };

        let screens = find_all_touchscreens_nodes()?;
        let selected_screen = Select::with_theme(theme.as_ref())
            .with_prompt("[xfix] Select a touchscreen")
            .default(0)
            .items(screens.iter().map(|screen| {
                if let Some(mapping) =
                    config.get_mapping(screen.vendor().unwrap(), screen.id_path().unwrap())
                {
                    format!("{} (-> {})", screen, mapping)
                } else {
                    format!("{}", screen)
                }
            }))
            .interact()?;

        println!("[xfix] Searching for video outputs...");

        let outputs = find_connected_video_outputs()?;
        let selected_output = Select::with_theme(theme.as_ref())
            .with_prompt("[xfix] Select a video output")
            .default(0)
            .items(&outputs)
            .interact()?;

        let selected_screen = &screens[selected_screen];
        let selected_output = &outputs[selected_output];

        println!(
            "[xfix] Mapping screen {:?} to output {:?}",
            selected_screen.event_path(),
            selected_output
        );

        let touchscreen = selected_screen.to_touchscreen(Some(selected_output.to_string()))?;

        let mut new_config = config.clone();
        new_config.add_touchscreen(touchscreen);

        println!("[xfix] Running assign command...");
        XFixCommandAssign.run(&new_config)?;

        Ok(Some(new_config))
    }
}
