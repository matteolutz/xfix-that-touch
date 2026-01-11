use std::{collections::HashMap, process::Command};

use clap::Args;

use crate::{
    commands::XFixCommandDelegate,
    config::{XFixConfig, XFixTouchscreen},
};

const TOUCHSCREEN_TYPE: &str = "ID_INPUT_MOUSE";

#[derive(Debug)]
struct XFixTouchscreenWithNode<'a> {
    screen: &'a XFixTouchscreen,
    node: Option<String>,
}

#[derive(Args)]
pub struct XFixCommandFix;

impl XFixCommandFix {
    fn find_touchscreen_nodes<'a>(
        &self,
        screens: &'a [XFixTouchscreen],
    ) -> Result<Vec<XFixTouchscreenWithNode<'a>>, Box<dyn std::error::Error>> {
        let all_touchscreen_nodes = glob::glob("/dev/input/event*")?
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|path| {
                let device_name = path.display().to_string();
                let udev_output = Command::new("udevadm")
                    .args(["info", "--query=property"])
                    .arg(format!("--name={}", device_name))
                    .output()
                    .ok()?;

                let udev_output_str = str::from_utf8(&udev_output.stdout).ok()?;

                let udev_properties = udev_output_str
                    .split("\n")
                    .into_iter()
                    .filter_map(|line| line.split_once("="))
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect::<HashMap<_, _>>();

                Some((device_name, udev_properties))
            })
            .filter(|(_, props)| props.get(TOUCHSCREEN_TYPE).is_some_and(|val| val == "1"))
            .collect::<Vec<_>>();

        let screens = screens
            .iter()
            .map(|s| {
                let node = all_touchscreen_nodes
                    .iter()
                    .find(|(_, props)| {
                        props
                            .get("ID_VENDOR")
                            .is_some_and(|vendor| vendor == &s.vendor)
                            && props.get("ID_PATH").is_some_and(|path| path == &s.id_path)
                    })
                    .map(|(node, _)| node.clone());

                XFixTouchscreenWithNode { screen: s, node }
            })
            .collect::<Vec<_>>();

        Ok(screens)
    }
}

impl XFixCommandDelegate for XFixCommandFix {
    fn run(&self, config: &XFixConfig) -> Result<(), Box<dyn std::error::Error>> {
        let screens = self.find_touchscreen_nodes(&config.touchscreens)?;

        println!("[xfix] Screens: {:?}", screens);

        Ok(())
    }
}
