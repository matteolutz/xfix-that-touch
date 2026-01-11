use std::{
    collections::{HashMap, hash_map::Keys},
    process::Command,
};

use clap::Args;
use regex::Regex;

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

#[derive(Debug)]
struct XFixTouchscreenWithXinputId<'a> {
    screen: XFixTouchscreenWithNode<'a>,
    id: Option<u32>,
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

                let udev_output_str = String::from_utf8(udev_output.stdout).ok()?;

                let udev_properties = udev_output_str
                    .lines()
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

    fn find_xinput_id<'a>(
        &self,
        screens: &[XFixTouchscreenWithNode<'a>],
    ) -> Result<Vec<XFixTouchscreenWithXinputId<'a>>, Box<dyn std::error::Error>> {
        let xinput_output = Command::new("xinput").args(["list", "--short"]).output()?;
        let xinput_output_str = str::from_utf8(&xinput_output.stdout)?;

        let pointer_devices_regex =
            Regex::new("Virtual core pointer(.*)\n(?<devices>(.*\n)*)(.*)Virtual core keyboard")?;
        let devices_str = &pointer_devices_regex.captures(xinput_output_str).unwrap()["devices"];

        let device_id_regex = Regex::new(".*id=(?<id>[0-9]+).*")?;

        let devices = devices_str
            .lines()
            .into_iter()
            .map(|line| {
                let device_id = device_id_regex.captures(line).unwrap()["id"]
                    .parse::<u32>()
                    .unwrap();
                device_id
            })
            .collect::<Vec<_>>();

        println!("[xfix] Devices: {:?}", devices);

        Ok(vec![])
    }
}

impl XFixCommandDelegate for XFixCommandFix {
    fn run(&self, config: &XFixConfig) -> Result<(), Box<dyn std::error::Error>> {
        let screens_with_node = self.find_touchscreen_nodes(&config.touchscreens)?;

        println!("[xfix] Screens: {:?}", screens_with_node);

        let screens_with_xinput_id = self.find_xinput_id(&screens_with_node)?;

        println!(
            "[xfix] Screens with XInput ID: {:?}",
            screens_with_xinput_id
        );

        Ok(())
    }
}
